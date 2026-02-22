use gooncityhub::app::App;
use gooncityhub::models::repos::Entity;
use loco_rs::testing::prelude::*;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serial_test::serial;

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

    let repo = Entity::fetch_from_github("XAMPPRocky", "octocrab", &boot.app_context.db)
        .await
        .expect("Should fetch repo successfully");

    // Verify saved to DB
    Entity::find()
        .filter(gooncityhub::models::_entities::repos::Column::Owner.eq("XAMPPRocky"))
        .one(&boot.app_context.db)
        .await
        .unwrap()
        .unwrap();

    println!("{:#?}", repo);
    assert!(repo.stars > 0);
    assert!(repo.prs > 0);
    assert_eq!(repo.name, "octocrab");
    assert_eq!(repo.owner, "XAMPPRocky");
}
