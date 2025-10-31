use salvo::test::TestClient;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn role_crud_flow() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = helpers::unique_name("role");
    let register = helpers::register_user(&app, &username, "testpass123").await;
    let token = register["data"]["token"].as_str().unwrap().to_string();

    let response = TestClient::post(helpers::get_url("/api/admin/roles"))
        .add_header("Authorization", helpers::bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({"name": helpers::unique_name("role"), "description": "desc"}))
        .send(&app)
        .await;
    let created = helpers::print_response_body_get_json(response, "create_role").await;
    assert!(created["success"].as_bool().unwrap());
    let role_id = created["data"]["id"].as_i64().unwrap() as i32;

    let response = TestClient::get(helpers::get_url(&format!("/api/admin/roles/{}", role_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;
    let detail = helpers::print_response_body_get_json(response, "get_role").await;
    assert_eq!(detail["data"]["id"].as_i64().unwrap() as i32, role_id);

    let response = TestClient::get(helpers::get_url("/api/admin/roles?page=1&page_size=5"))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;
    let list = helpers::print_response_body_get_json(response, "list_roles").await;
    assert!(list["data"]["list"].as_array().unwrap().iter().any(|item| item["id"].as_i64() == Some(role_id as i64)));

    let response = TestClient::delete(helpers::get_url(&format!("/api/admin/roles/{}", role_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;
    let deleted = helpers::print_response_body_get_json(response, "delete_role").await;
    assert!(deleted["success"].as_bool().unwrap());
}

