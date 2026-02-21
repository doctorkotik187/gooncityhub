use gooncityhub::app::App;
use gooncityhub::models::repos::Entity;
use insta::assert_debug_snapshot;
use loco_rs::testing::prelude::*;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serial_test::serial;
use dotenvy::dotenv;

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn test_fetch_github_repo() {
    configure_insta!();
    let boot = boot_test::<App>().await.unwrap();

    dotenv().ok();

    // Now you can safely read env vars
    let github_token = std::env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN must be set in .env");

    println!("Loaded GITHUB_TOKEN (len={} chars)", github_token.len());


    let repo = Entity::fetch_from_github("XAMPPRocky", "octocrab", &github_token, &boot.app_context.db)
        .await
        .expect("Should fetch repo successfully");

    assert_eq!(repo.owner, "XAMPPRocky");
    assert!(repo.stars > 0);

    // Verify saved to DB
    Entity::find()
        .filter(gooncityhub::models::_entities::repos::Column::Owner.eq("XAMPPRocky"))
        .one(&boot.app_context.db)
        .await
        .unwrap()
        .unwrap();

    assert_debug_snapshot!(repo);
}
