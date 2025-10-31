
use salvo::test::TestClient;
mod helpers;
use serde_json::json;

#[tokio::test]
async fn test_register_success() {
    let app = helpers::create_test_app().await;
    let register_body = json!({
        "username": format!("testuser_{}", chrono::Utc::now().timestamp()),
        "password": "testpass123"
    });
    let response = TestClient::post(helpers::get_url("/api/register"))
        .add_header("content-type", "application/json", true)
        .json(&register_body)
        .send(&app)
        .await;
    let bodyjson = helpers::print_response_body_get_json(response, "register").await;
    assert!(bodyjson["success"].as_bool().unwrap());
    assert!(bodyjson["data"]["token"].is_string());
}