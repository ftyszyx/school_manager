use school_manager_server::core::constants::APP_USER_ALREADY_EXISTS;

mod helpers;

#[tokio::test]
async fn register_and_login_succeeds() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = helpers::unique_name("auth");

    let register = helpers::register_user(&app, &username, "testpass123").await;
    assert!(register["success"].as_bool().unwrap());

    let login = helpers::login_user(&app, &username, "testpass123", "login").await;
    assert!(login["success"].as_bool().unwrap());

    let duplicate = helpers::register_user(&app, &username, "testpass123").await;
    assert_eq!(duplicate["code"].as_u64().unwrap(), APP_USER_ALREADY_EXISTS as u64);
}

#[tokio::test]
async fn login_with_wrong_password_fails() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = helpers::unique_name("auth_fail");

    helpers::register_user(&app, &username, "testpass123").await;

    let login = helpers::login_user(&app, &username, "wrongpass", "login_fail").await;
    assert!(!login["success"].as_bool().unwrap());
    assert_eq!(login["code"].as_u64().unwrap(), school_manager_server::core::constants::APP_AUTH_FAILED as u64);
}

