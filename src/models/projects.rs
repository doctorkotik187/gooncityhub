pub use super::_entities::projects::{ActiveModel, Entity, Model};
use sea_orm::entity::prelude::*;
pub type Projects = Entity;

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
impl Model {
    /// # Errors
    ///
    /// DB Error.
    #[allow(clippy::cast_precision_loss)]
    pub async fn recalculate_health<C>(&self, db: &C) -> Result<f32, DbErr>
    where
        C: ConnectionTrait,
    {
        use sea_orm::EntityTrait;

        // Fetch all repos for this project
        let repos = crate::models::repos::Entity::find()
            .filter(crate::models::_entities::repos::Column::ProjectId.eq(self.id))
            .all(db)
            .await?;

        // Compute average health
        let health = if repos.is_empty() {
            100.0
        } else {
            let sum: f32 = repos
                .iter()
                .map(super::_entities::repos::Model::health)
                .sum();
            sum / repos.len() as f32
        };

        // Update self in DB
        crate::models::projects::ActiveModel {
            id: sea_orm::ActiveValue::Set(self.id),
            health: sea_orm::ActiveValue::Set(health),
            ..Default::default()
        }
        .update(db)
        .await?;

        Ok(health)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
