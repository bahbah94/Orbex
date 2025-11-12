use crate::indexer::orderbook_reducer::{get_orderbook_snapshot, OrderbookState};
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub type AppState = (Arc<Mutex<OrderbookState>>, PgPool);

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State((orderbook, _pool)): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, orderbook))
}

// on upgrade is here
pub async fn handle_socket(socket: WebSocket, orderbook: Arc<Mutex<OrderbookState>>) {
    let (mut sender, mut receiver) = socket.split();

    // now we send initial snapshot

    {
        let ob = orderbook.lock().await;
        let snapshot = get_orderbook_snapshot(&ob);

        if let Ok(json) = serde_json::to_string(&snapshot) {
            if sender.send(Message::Text(json.into())).await.is_err() {
                println!("Failed to send intiial snapshot. Get Fucked");
                return;
            }
            println!(" Sent initial snapshot");
        }
    }
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let ob = orderbook.lock().await;
                let snapshot = get_orderbook_snapshot(&ob);

                if let Ok(json) = serde_json::to_string(&snapshot) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        println!("Failed to send initial snapshot");
                        break;
                    }
                    println!(" Sent initial snapshot");
                }
            }

            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        println!(" Client closed connection");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    None => {
                        println!("Connection Lost");
                        break;
                    }
                    Some(Err(e)) => {
                        println!("WebSocket error: {:?}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    println!("WebSocket connection closed");
}
