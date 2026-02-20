use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "repos",
            &[
                ("id", ColType::PkAuto),
                ("name", ColType::String),
                ("owner", ColType::String),
                ("stars", ColType::Integer),
                ("forks", ColType::Integer),
                ("issues", ColType::Integer),
                ("prs", ColType::Integer),
                ("contributors", ColType::Integer),
                ("commits_last_30d", ColType::Integer),
                ("watchers", ColType::Integer),
                ("license", ColType::StringNull),
                ("last_fetch", ColType::DateTime),
            ],
            &[("project", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "repos").await
    }
}
