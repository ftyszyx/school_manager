use salvo::prelude::Service;
use salvo::test::TestClient;
use school_manager_server::core::constants::*;
use serde_json::{json,Value};
use std::time::{SystemTime,UNIX_EPOCH};
mod helpers;

fn unique_id() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
}

fn unique_username(prefix: &str) -> String {
    format!("{}_{}", prefix, unique_id())
}

fn bearer(token: &str) -> String {
    format!("Bearer {}", token)
}

async fn register_user(app: &Service, username: &str, password: &str) -> Value {
    let body = json!({"username": username, "password": password});
    let response = TestClient::post(helpers::get_url("/api/register"))
        .add_header("content-type", "application/json", true)
        .json(&body)
        .send(app)
        .await;
    let label = format!("register_{}", username);
    helpers::print_response_body_get_json(response, &label).await
}

async fn login_user(app: &Service, username: &str, password: &str, label: &str) -> Value {
    let body = json!({"username": username, "password": password});
    let response = TestClient::post(helpers::get_url("/api/login"))
        .add_header("content-type", "application/json", true)
        .json(&body)
        .send(app)
        .await;
    helpers::print_response_body_get_json(response, label).await
}

#[tokio::test]
async fn test_auth_flow() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = unique_username("auth_user");
    let register_json = register_user(&app, &username, "testpass123").await;
    assert!(register_json["success"].as_bool().unwrap());
    assert!(register_json["data"]["token"].is_string());
    let duplicate_json = register_user(&app, &username, "another").await;
    assert!(!duplicate_json["success"].as_bool().unwrap());
    assert_eq!(duplicate_json["code"].as_u64().unwrap(), APP_USER_ALREADY_EXISTS as u64);
    let login_success = login_user(&app, &username, "testpass123", "login_success").await;
    assert!(login_success["success"].as_bool().unwrap());
    let login_fail = login_user(&app, &username, "wrongpass", "login_fail").await;
    assert!(!login_fail["success"].as_bool().unwrap());
    assert_eq!(login_fail["code"].as_u64().unwrap(), APP_AUTH_FAILED as u64);
}

#[tokio::test]
async fn test_bind_and_unbind_class_flow() {
    let _guard = helpers::db_lock().await;
    let app = helpers::create_test_app().await;
    let username = unique_username("binder");
    let register_json = register_user(&app, &username, "bindpass123").await;
    assert!(register_json["success"].as_bool().unwrap());
    let token = register_json["data"]["token"].as_str().unwrap().to_string();
    let school_payload = json!({"name": format!("Test School {}", unique_id())});
    let response = TestClient::post(helpers::get_url("/api/admin/schools"))
        .add_header("Authorization", bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&school_payload)
        .send(&app)
        .await;
    let school_json = helpers::print_response_body_get_json(response, "create_school").await;
    assert!(school_json["success"].as_bool().unwrap());
    let school_id = school_json["data"]["id"].as_i64().unwrap() as i32;
    let class_password = "classpwd";
    let class_payload = json!({"name": "Class 1", "grade": 3, "class": 2, "school_id": school_id, "status": 0, "password": class_password});
    let response = TestClient::post(helpers::get_url("/api/admin/classes"))
        .add_header("Authorization", bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&class_payload)
        .send(&app)
        .await;
    let class_json = helpers::print_response_body_get_json(response, "create_class").await;
    assert!(class_json["success"].as_bool().unwrap());
    let class_id = class_json["data"]["id"].as_i64().unwrap() as i32;
    let bind_payload = json!({"class_id": class_id, "password": class_password});
    let response = TestClient::post(helpers::get_url("/api/admin/bind/class"))
        .add_header("Authorization", bearer(&token), true)
        .add_header("content-type", "application/json", true)
        .json(&bind_payload)
        .send(&app)
        .await;
    let bind_json = helpers::print_response_body_get_json(response, "bind_class").await;
    assert!(bind_json["success"].as_bool().unwrap());
    let response = TestClient::get(helpers::get_url("/api/admin/me"))
        .add_header("Authorization", bearer(&token), true)
        .send(&app)
        .await;
    let me_json = helpers::print_response_body_get_json(response, "me_after_bind").await;
    assert!(me_json["success"].as_bool().unwrap());
    assert!(me_json["data"]["class_infos"].as_array().unwrap().iter().any(|info| info["class_id"].as_i64() == Some(class_id as i64)));
    let response = TestClient::delete(helpers::get_url(&format!("/api/admin/unbind/class/{}", class_id)))
        .add_header("Authorization", bearer(&token), true)
        .send(&app)
        .await;
    let unbind_json = helpers::print_response_body_get_json(response, "unbind_class").await;
    assert!(unbind_json["success"].as_bool().unwrap());
    let response = TestClient::get(helpers::get_url("/api/admin/me"))
        .add_header("Authorization", bearer(&token), true)
        .send(&app)
        .await;
    let me_after_unbind = helpers::print_response_body_get_json(response, "me_after_unbind").await;
    assert!(me_after_unbind["success"].as_bool().unwrap());
    assert!(me_after_unbind["data"]["class_infos"].as_array().unwrap().is_empty());
}