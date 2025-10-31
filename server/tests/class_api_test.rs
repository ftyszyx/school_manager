use salvo::test::TestClient;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn class_crud_flow() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = helpers::unique_name("class");
    let register = helpers::register_user(&app, &username, "testpass123").await;
    let token = register["data"]["token"].as_str().unwrap().to_string();

    let school_resp = TestClient::post(helpers::get_url("/api/admin/schools"))
        .add_header("Authorization", helpers::bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({"name": helpers::unique_name("class_school")}))
        .send(&app)
        .await;
    let school = helpers::print_response_body_get_json(school_resp, "create_school_for_class").await;
    let school_id = school["data"]["id"].as_i64().unwrap() as i32;

    let class_payload = json!({
        "name": helpers::unique_name("class_name"),
        "grade": 3,
        "class": 1,
        "school_id": school_id,
        "status": 0,
        "password": "pass123"
    });

    let response = TestClient::post(helpers::get_url("/api/admin/classes"))
        .add_header("Authorization", helpers::bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&class_payload)
        .send(&app)
        .await;
    let created = helpers::print_response_body_get_json(response, "create_class").await;
    assert!(created["success"].as_bool().unwrap());
    let class_id = created["data"]["id"].as_i64().unwrap() as i32;

    let response = TestClient::put(helpers::get_url(&format!("/api/admin/classes/{}", class_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({"status": 2}))
        .send(&app)
        .await;
    let updated = helpers::print_response_body_get_json(response, "update_class").await;
    assert_eq!(updated["data"]["status"].as_i64().unwrap(), 2);

    let response = TestClient::get(helpers::get_url(&format!("/api/admin/classes?school_id={}", school_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;
    let list = helpers::print_response_body_get_json(response, "list_classes").await;
    assert!(list["data"]["list"].as_array().unwrap().iter().any(|item| item["id"].as_i64() == Some(class_id as i64)));

    let response = TestClient::delete(helpers::get_url(&format!("/api/admin/classes/{}", class_id)))
        .add_header("Authorization", helpers::bearer(&token), true)
        .send(&app)
        .await;
    let deleted = helpers::print_response_body_get_json(response, "delete_class").await;
    assert!(deleted["success"].as_bool().unwrap());
}

