use salvo::test::TestClient;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn permission_crud_flow() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = helpers::unique_name("perm");
    let register = helpers::register_user(&app, &username, "testpass123").await;
    let token = register["data"]["token"].as_str().unwrap().to_string();

    let payload = json!({
        "name": helpers::unique_name("perm"),
        "resource": "/api/demo",
        "action": "read",
        "description": "demo"
    });

    let response = TestClient::post(helpers::get_url("/api/admin/permissions"))
        .add_header("Authorization", helpers::bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&payload)
        .send(&app)
        .await;
    let created = helpers::print_response_body_get_json(response, "create_permission").await;
    assert!(created["success"].as_bool().unwrap());
    assert_eq!(created["data"]["action"].as_str().unwrap(), "READ");
    let permission_id = created["data"]["id"].as_i64().unwrap() as i32;

    let response = TestClient::put(helpers::get_url(&format!("/api/admin/permissions/{}", permission_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({"description": "updated"}))
        .send(&app)
        .await;
    let updated = helpers::print_response_body_get_json(response, "update_permission").await;
    assert_eq!(updated["data"]["description"].as_str().unwrap(), "updated");

    let response = TestClient::get(helpers::get_url("/api/admin/permissions?page=1&page_size=10"))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;
    let list = helpers::print_response_body_get_json(response, "list_permissions").await;
    assert!(list["data"]["list"].as_array().unwrap().iter().any(|item| item["id"].as_i64() == Some(permission_id as i64)));

    let response = TestClient::delete(helpers::get_url(&format!("/api/admin/permissions/{}", permission_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;
    let deleted = helpers::print_response_body_get_json(response, "delete_permission").await;
    assert!(deleted["success"].as_bool().unwrap());
}

