use crate::apis::ws_api::broadcast_status_update;
use crate::core::config::DatabaseConfig;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgListener, PgPoolOptions};
use tracing::{error, info};

#[derive(Deserialize, Serialize)]
pub struct NotificationPayload {
    pub school_id: i32,
    pub grade: i32,
    pub class: i32,
    pub class_id: i32,
    pub new_status: i32,
}

pub async fn listen_for_notifications(db_config: &DatabaseConfig) -> anyhow::Result<()> {
    let pool = PgPoolOptions::new().connect(&db_config.db_url).await?;
    let mut listener: PgListener = PgListener::connect_with(&pool).await?;
    listener.listen_all(vec!["class_status_updates"]).await?;

    info!("Listening for class status updates from PostgreSQL...");

    loop {
        let notification = listener.recv().await?;
        let payload_str = notification.payload();
        match serde_json::from_str::<NotificationPayload>(payload_str) {
            Ok(payload) => {
                info!(
                    "Received status update for class {}: new status {}",
                    payload.class_id, payload.new_status
                );
                tokio::spawn(async move {
                    broadcast_status_update(payload).await;
                });
            }
            Err(e) => {
                error!("Failed to deserialize notification payload: {}", e);
            }
        }
    }
}
