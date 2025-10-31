use salvo::test::ResponseExt;
use school_manager_server::core::app;
use school_manager_server::core::router;
use salvo::prelude::*;
use serde_json::json;
use std::env;
use salvo::test::TestClient;
use school_manager_server::core::constants::*;

pub async fn create_test_app() -> Service {
    dotenvy::from_filename(".env.test").unwrap();
    let _guard = app::init_log().unwrap();
    let app_state = app::init_app()
        .await
        .unwrap_or_else(|e| panic!("failed to initialize app:{}", e.to_string()));
    let pool = sqlx::PgPool::connect(&app_state.config.database.db_url).await.unwrap();
    println!("clean database");
    sqlx::migrate!("./migrations").undo(&pool, 0).await.unwrap();
    println!("init database");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    println!("init database success");
    let app = router::create_router(app_state);
    app
}


pub async fn print_response_body_get_json(mut response: Response, label: &str) -> serde_json::Value {
    let status = response.status_code;
    let json =response.take_json().await.unwrap();
    println!("{}: status={:?}, body={:?}\n", label, status, json);
    json
}


#[allow(dead_code)]
pub async fn create_test_user_and_login(app: &Service) -> String {
    // 注册用户
    let register_body = json!({
        "username": "testuser",
        "password": "testpass123"
    });
    let url=get_url("/api/register");
    let response = TestClient::post(url)
        .add_header("content-type", "application/json", true)
        .json(&register_body)
        .send(app)
        .await;

    println!("register_body: {:?}", register_body);
    let json = print_response_body_get_json(response, "register_response").await;
    let code = json["code"].as_u64().unwrap();
    assert!(code == 0 || code == APP_USER_ALREADY_EXISTS as u64);

    // 登录获取 token
    let login_body = json!({
        "username": "testuser",
        "password": "testpass123"
    });

    let url=get_url("/api/login");
    let response = TestClient::post(url)
        .add_header("content-type", "application/json", true)
        .json(&login_body)
        .send(app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "login_response").await;
    json["data"]["token"].as_str().unwrap().to_string()
}

pub fn get_url(path: &str) -> String {
    let host = env::var("LISTEN_HOST").expect("LISTEN_HOST not set");
    let port = env::var("LISTEN_PORT").expect("LISTEN_PORT not set");
    if path.starts_with("/") {
        format!("http://{}:{}{}", host, port, path)
    } else {
        format!("http://{}:{}/{}", host, port, path)
    }
}
