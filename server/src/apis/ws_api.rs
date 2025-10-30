use futures_util::{StreamExt, FutureExt};
use salvo::prelude::*;
use salvo::websocket::{WebSocket, Message, WebSocketUpgrade};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::core::db_listener::NotificationPayload;

type Connections = RwLock<HashMap<i32, Vec<mpsc::UnboundedSender<Result<Message, salvo::Error>>>>>;
//NEXT_CONN_ID 是一个全局的原子计数器 (AtomicUsize)，确保每个连接都有一个唯一的 ID。
static NEXT_CONN_ID: AtomicUsize = AtomicUsize::new(1);
//CONNECTIONS 是一个全局的读写锁 (RwLock)，用于管理所有连接的映射。
static CONNECTIONS: LazyLock<Connections> = LazyLock::new(Connections::default);

#[handler]
pub async fn school_ws_handler(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let school_id: i32 = req.param("id").unwrap_or_default();
    if school_id == 0 {
        return Err(StatusError::bad_request());
    }
    WebSocketUpgrade::new()
        .upgrade(req, res, move |ws| handle_socket(ws, school_id))
        .await
}

async fn handle_socket(ws: WebSocket, school_id: i32,) {
    let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);
    tracing::info!("New WebSocket connection: conn_id={}, school_id={}", conn_id, school_id);

    //将WebSocket连接拆分为发送和接收两部分。
    //user_ws_tx 用于发送消息给客户端。
    //user_ws_rx 用于接收客户端的消息。
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    //创建一个无界通道 (unbounded channel)，用于在不同的任务之间传递消息。
    let (tx, rx) = mpsc::unbounded_channel();
    //将接收到的消息转换为流，以便在不同的任务之间传递消息。
    let rx = UnboundedReceiverStream::new(rx);

    //创建了一个新的后台异步任务。这个任务独立于 handle_socket 函数运行。
    //它创建了一个 future，这个 future 会不断地从 mpsc 通道的接收端 rx 读取消息
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            tracing::error!(error = ?e, "WebSocket send error");
        }
    }));

    CONNECTIONS.write().await.entry(school_id).or_default().push(tx);

    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(_) => {
                // We don't process incoming messages in this scenario
            }
            Err(e) => {
                tracing::error!(error = ?e, "WebSocket receive error");
                break;
            }
        }
    }
    tracing::info!("WebSocket connection closed: conn_id={}", conn_id);
    // On disconnect, the logic to remove the sender from the CONNECTIONS map is complex
    // because we only have the receiver here. A more robust implementation might involve
    // a channel to signal disconnection back to a central management task.
    // For this example, we'll rely on the broadcast function to clean up dead connections.
}


pub async fn broadcast_status_update(payload: NotificationPayload) {
    let mut conns = CONNECTIONS.write().await;
    if let Some(school_conns) = conns.get_mut(&payload.school_id) {
        school_conns.retain(|tx| {
            tx.send(Ok(Message::text(serde_json::to_string(&payload).unwrap()))).is_ok()
        });
    }
}
