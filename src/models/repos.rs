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
        if !insert && self.updated_at.is_unchanged() {
            self.updated_at = Set(Utc::now().into());
        }

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

// Optional: keep empty impl blocks for read/write extensions
impl Model {}
impl ActiveModel {}
