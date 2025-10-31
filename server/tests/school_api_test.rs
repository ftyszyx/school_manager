use salvo::test::TestClient;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn school_crud_flow() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = helpers::unique_name("school");
    let register = helpers::register_user(&app, &username, "testpass123").await;
    let token = register["data"]["token"].as_str().unwrap().to_string();

    let response = TestClient::post(helpers::get_url("/api/admin/schools"))
        .add_header("Authorization", helpers::bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({"name": helpers::unique_name("school_name")}))
        .send(&app)
        .await;
    let created = helpers::print_response_body_get_json(response, "create_school").await;
    assert!(created["success"].as_bool().unwrap());
    let school_id = created["data"]["id"].as_i64().unwrap() as i32;

    let response = TestClient::put(helpers::get_url(&format!("/api/admin/schools/{}", school_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({"name": "updated"}))
        .send(&app)
        .await;
    let updated = helpers::print_response_body_get_json(response, "update_school").await;
    assert_eq!(updated["data"]["name"].as_str().unwrap(), "updated");

    let response = TestClient::delete(helpers::get_url(&format!("/api/admin/schools/{}", school_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;
    let deleted = helpers::print_response_body_get_json(response, "delete_school").await;
    assert!(deleted["success"].as_bool().unwrap());
}

