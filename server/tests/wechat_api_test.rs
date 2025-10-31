use salvo::test::TestClient;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn wechat_login_with_invalid_code() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;

    let response = TestClient::post(helpers::get_url("/api/login/wechat"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"code": "bad_code"}))
        .send(&app)
        .await;
    let body = helpers::print_response_body_get_json(response, "wechat_login").await;
    assert!(!body["success"].as_bool().unwrap());
}

