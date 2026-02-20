use crate::models::_entities::repos::Entity;
use crate::models::repos::Model;
use gooncityhub::app::App;
use loco_rs::testing::prelude::*;
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
    // seed::<App>(&boot.app_context).await.unwrap(); // Skip seed for clean test

    // Set your GitHub token via env or config/test.yaml
    let token = std::env::var("GITHUB_TOKEN").unwrap_or_else(|_| "ghp_fake".to_string());

    // Test octocrab/octocrab (small public repo)
    let repo = Entity::fetch_from_github("XAMPPRocky", "octocrab", &token, &boot.app_context.db)
        .await
        .expect("Should fetch repo successfully");

    // Assert core fields mapped correctly
    assert_eq!(repo.owner, "octocrab");
    assert_eq!(repo.name, "octocrab");
    assert!(repo.stars > 0, "Should have stars > 0");
    assert!(repo.forks >= 0);
    assert_eq!(repo.project_id, None, "Should NOT auto-assign project");

    // Verify it saved to DB
    let from_db = Entity::find()
        .filter(crate::models::_entities::repos::Column::Owner.eq("octocrab"))
        .filter(crate::models::_entities::repos::Column::Name.eq("octocrab"))
        .one(&boot.app_context.db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(from_db.stars, repo.stars);
    assert!(from_db.last_fetch.timestamp() > 0);

    // Snapshot for visual inspection
    assert_debug_snapshot!(repo);
}
