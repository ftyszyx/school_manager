use salvo::prelude::*;
use anyhow::{Context };

pub mod apis;
pub mod core;
pub mod utils;

#[tokio::main]
async fn main() {
    // env
    dotenvy::dotenv().ok();
    let _guard = core::app::init_log().unwrap();

    let app_state = core::app::init_app().await.context("init app failed").unwrap();
    
    // Spawn the database listener as a background task
    let db_config_clone = app_state.config.database.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = core::db_listener::listen_for_notifications(&db_config_clone).await {
                tracing::error!("DB listener task failed: {}. Restarting after 5 seconds...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });

    let host = app_state.config.server.host.clone();
    let port = app_state.config.server.port;
    let app_service = core::router::create_router(app_state);

    // 启动服务器
    let addr=format!("{}:{}",host,port);
    tracing::info!("Server starting on {}", addr);
    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(app_service).await;
}

