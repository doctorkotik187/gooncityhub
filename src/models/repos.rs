pub use super::_entities::repos::{ActiveModel, Entity, Model};
use chrono::Utc;
use loco_rs::prelude::Set;
use octocrab::{models::Repository, params::State, Octocrab};
use sea_orm::prelude::*;
use sea_orm::TryIntoModel;

pub type Repos = Entity;

use crate::models::projects::{ActiveModel as ProjectActiveModel, Model as ProjectModel};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // update timestamp
        if !insert && self.updated_at.is_unchanged() {
            self.updated_at = Set(Utc::now().into());
        }

        // create project if needed
        if insert && self.project_id.is_not_set() {
            let owner = self.owner.try_as_ref().unwrap().clone();
            let name = self.name.try_as_ref().unwrap().clone();

            let project: ProjectModel = ProjectActiveModel {
                name: Set(name),
                owner: Set(owner),
                health: Set(100.),
                last_fetch: Set(Utc::now().naive_utc()),
                ..Default::default()
            }
            .save(db)
            .await?
            .try_into_model()?;

            self.project_id = Set(project.id);
        }

        // recalculate project health if project_id exists
        if let sea_orm::ActiveValue::Set(project_id) | sea_orm::ActiveValue::Unchanged(project_id) =
            self.project_id
        {
            if let Some(project) = crate::models::projects::Entity::find_by_id(project_id)
                .one(db)
                .await?
            {
                let _ = project.recalculate_health(db).await;
            }
        }

        Ok(self)
    }
}

impl Entity {
    /// Fetch repository from GitHub and persist in DB
    /// # Errors
    ///
    /// Any errors in the fetch from Github API process.
    pub async fn fetch_from_github(
        owner: &str,
        repo_name: &str,
        db: &DbConn,
    ) -> Result<Model, Box<dyn std::error::Error>> {
        // Try loading GitHub token (optional)
        let token = std::env::var("GITHUB_TOKEN").ok();

        let octocrab = if let Some(token) = token {
            println!("Using GitHub API token.");
            Octocrab::builder().personal_token(token).build()?
        } else {
            println!(
                "No GITHUB_TOKEN found. Using unauthenticated GitHub API (rate limits apply)."
            );
            Octocrab::builder().build()?
        };

        // Fetch GitHub repository
        let gh_repo: Repository = octocrab.repos(owner, repo_name).get().await?;

        // Fetch counts
        let prs_count = i32::try_from(
            octocrab
                .pulls(owner, repo_name)
                .list()
                .state(State::Open)
                .send()
                .await?
                .items
                .len(),
        );

        let contributors_count = i32::try_from(
            octocrab
                .repos(owner, repo_name)
                .list_contributors()
                .send()
                .await?
                .items
                .len(),
        );

        let commits_last_30d = i32::try_from(
            octocrab
                .repos(owner, repo_name)
                .list_commits()
                .since(Utc::now() - chrono::Duration::days(30))
                .per_page(100)
                .send()
                .await?
                .items
                .len(),
        );

        // Build model
        let model =
            Self::build_active_model(gh_repo, prs_count?, contributors_count?, commits_last_30d?);

        // Persist and return
        Ok(model.save(db).await?.try_into()?)
    }

    /// Map GitHub repo + stats into `ActiveModel`
    fn build_active_model(
        gh_repo: Repository,
        prs: i32,
        contributors: i32,
        commits_last_30d: i32,
    ) -> ActiveModel {
        ActiveModel {
            name: Set(gh_repo.name),
            owner: Set(gh_repo.owner.map(|o| o.login).unwrap_or_default()),
            stars: Set(gh_repo.stargazers_count.unwrap_or(0).cast_signed()),
            forks: Set(gh_repo.forks_count.unwrap_or(0).cast_signed()),
            issues: Set(gh_repo.open_issues_count.unwrap_or(0).cast_signed()),
            watchers: Set(gh_repo.watchers_count.unwrap_or(0).cast_signed()),
            prs: Set(prs),
            contributors: Set(contributors),
            commits_last_30d: Set(commits_last_30d),
            license: gh_repo.license.map_or(Set(None), |l| Set(Some(l.name))),
            last_fetch: Set(Utc::now().naive_utc()),
            ..Default::default()
        }
    }
}

impl Model {
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn health(&self) -> f32 {
        // Convert all i32 fields to f64 once
        let commits = f64::from(self.commits_last_30d);
        let contributors = f64::from(self.contributors);
        let prs = f64::from(self.prs);
        let stars = f64::from(self.stars);
        let issues = f64::from(self.issues);

        // Compute normalized factors (clamped 0.0–1.0)
        let activity = (commits / 30.0).min(1.0);
        let community = (contributors / 10.0).min(1.0);
        let adoption = (stars / 100.0).min(1.0);
        let maintenance = (prs / 10.0 * (1.0 - (issues / 50.0).min(1.0))).min(1.0);

        // Weighted sum
        let score = activity * 0.35 + community * 0.25 + adoption * 0.15 + maintenance * 0.25;

        // Scale to 0–100 and cast to f32 at the very end
        (score * 100.0).clamp(0.0, 100.0) as f32
    }
}
impl ActiveModel {}
