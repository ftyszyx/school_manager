use salvo::test::TestClient;

mod helpers;

#[tokio::test]
async fn ws_handler_rejects_zero_id() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = helpers::unique_name("ws");
    let register = helpers::register_user(&app, &username, "testpass123").await;
    let token = register["data"]["token"].as_str().unwrap().to_string();

    let response = TestClient::get(helpers::get_url("/api/admin/ws/school/0"))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;

    assert_eq!(response.status_code, Some(salvo::http::StatusCode::BAD_REQUEST));
}

