pub use super::_entities::repos::{ActiveModel, Entity, Model};
use loco_rs::prelude::Set;
use octocrab::{models::Repository, Octocrab};
pub type Repos = Entity;

use crate::models::projects::{ActiveModel as ProjectActiveModel, Model as ProjectModel};
use chrono::Duration;
use chrono::Utc;
use loco_rs::prelude::ActiveValue::NotSet;
use octocrab::params::State;
use sea_orm::prelude::*;
use sea_orm::TryIntoModel;

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
            // ✅ try_as_ref() - reads WITHOUT consuming
            let owner = self.owner.try_as_ref().unwrap().clone();
            let name = self.name.try_as_ref().unwrap().clone();

            let project: ProjectModel = ProjectActiveModel {
                name: Set(format!("{} / {}", owner, name)),
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

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {
    pub async fn fetch_from_github(
        owner: &str,
        repo_name: &str,
        token: &str, // Required token
        db: &DbConn,
    ) -> Result<Model, Box<dyn std::error::Error>> {
        let octocrab = Octocrab::builder()
            .personal_token(token.to_string())
            .build()?;

        // Fetch main repo info
        let gh_repo: Repository = octocrab.repos(owner, repo_name).get().await?;

        let prs_count: i32 = octocrab
            .pulls(owner, repo_name)
            .list()
            .state(State::All)
            .send()
            .await?
            .total_count
            .unwrap_or(0) // handle None
            .try_into() // now it's a u64 → i32 conversion
            .expect("PR count fits i32");

        // Fetch contributors count
        let contributors_count = octocrab
            .repos(owner, repo_name)
            .list_contributors()
            .send()
            .await?
            .items
            .len() as i32;

        let commits = octocrab
            .repos(owner, repo_name)
            .list_commits()
            .since(Utc::now().checked_sub_signed(Duration::days(30)).unwrap())
            .per_page(100)
            .send()
            .await?;

        let model = ActiveModel {
            name: Set(gh_repo.name),
            owner: Set(gh_repo.owner.unwrap().login),
            stars: Set(gh_repo.stargazers_count.unwrap_or(0) as i32),
            forks: Set(gh_repo.forks_count.unwrap_or(0) as i32),
            issues: Set(gh_repo.open_issues_count.unwrap_or(0) as i32),
            watchers: Set(gh_repo.watchers_count.unwrap_or(0) as i32),
            prs: Set(prs_count),
            contributors: Set(contributors_count as i32),
            commits_last_30d: Set(commits.items.len() as i32),
            license: if let Some(license) = &gh_repo.license {
                Set(Some(license.name.clone()))
            } else {
                NotSet
            },
            last_fetch: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        let saved_model: Model = model.save(db).await?.try_into()?;
        Ok(saved_model)
    }
}
