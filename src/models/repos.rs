pub use super::_entities::repos::{ActiveModel, Entity, Model};
use loco_rs::prelude::ActiveValue::NotSet;
use loco_rs::prelude::Set;
use octocrab::{models::Repository, Octocrab};
use sea_orm::entity::prelude::*;
pub type Repos = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
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
        token: &str,
        db: &DbConn,
    ) -> Result<Model, Box<dyn std::error::Error>> {
        let octocrab = Octocrab::builder()
            .personal_token(token.to_string())
            .build()?;

        let gh_repo: Repository = octocrab.repos(owner, repo_name).get().await?;

        let model = ActiveModel {
            name: Set(gh_repo.name),
            owner: Set(gh_repo.owner.unwrap().login),
            stars: Set(gh_repo.stargazers_count.unwrap_or(0) as i32),
            forks: Set(gh_repo.forks_count.unwrap_or(0) as i32),
            issues: Set(gh_repo.open_issues_count.unwrap_or(0) as i32),
            watchers: Set(gh_repo.watchers_count.unwrap_or(0) as i32),
            license: if let Some(license) = &gh_repo.license {
                Set(Some(license.name.clone()))
            } else {
                NotSet
            },

            last_fetch: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        let saved_model: Model = model.save(db).await?.try_into()?;
        Ok(saved_model)
    }
}
