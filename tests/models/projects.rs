use gooncityhub::app::App;
use loco_rs::testing::prelude::*;
use sea_orm::EntityTrait;
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
async fn test_calc_health() {
    configure_insta!();

    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    let repo = gooncityhub::models::repos::Entity::fetch_from_github(
        "XAMPPRocky",
        "octocrab",
        &boot.app_context.db,
    )
    .await
    .expect("Should fetch repo successfully");

    let project = gooncityhub::models::projects::Entity::find_by_id(repo.project_id)
        .one(&boot.app_context.db)
        .await
        .unwrap()
        .expect("Project should exist");

    let health = project
        .recalculate_health(&boot.app_context.db)
        .await
        .expect("Health calculation should succeed");

    println!("Repo Health: {:#?}", repo.health());
    println!("Project Health: {:#?}", health);
    assert!(health > 10.);
    assert_eq!(repo.health(), health);
}
