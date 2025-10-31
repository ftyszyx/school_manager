use salvo::test::TestClient;

mod helpers;

#[tokio::test]
async fn current_user_returns_profile() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = helpers::unique_name("user_me");

    let register = helpers::register_user(&app, &username, "testpass123").await;
    let token = register["data"]["token"].as_str().unwrap();

    let response = TestClient::get(helpers::get_url("/api/admin/me"))
        .add_header("Authorization", helpers::bearer(token), true)
        .send(&app)
        .await;
    let body = helpers::print_response_body_get_json(response, "me").await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["username"].as_str().unwrap(), username);
}

