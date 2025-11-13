use axum::extract::ws::{Message, WebSocket};
/// Unified WebSocket handler for both orderbook and OHLCV updates
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use super::messages::MarketDataMessage;
use crate::indexer::candle_aggregator::CandleUpdate;
use crate::indexer::orderbook_reducer::{OrderbookSnapshot, OrderbookState};

pub type UnifiedState = (
    Arc<Mutex<OrderbookState>>,
    broadcast::Sender<OrderbookSnapshot>,
    broadcast::Sender<CandleUpdate>,
);

#[derive(Debug, Deserialize)]
pub struct SubscriptionQuery {
    /// Subscribe to orderbook updates (default: true)
    pub orderbook: Option<bool>,
    /// Subscribe to OHLCV updates (default: true)
    pub ohlcv: Option<bool>,
    /// Symbol filter (default: "ETH/USDC")
    pub symbol: Option<String>,
    /// OHLCV timeframes filter, comma-separated (e.g., "1m,5m")
    pub timeframes: Option<String>,
}

pub async fn ws_unified_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<SubscriptionQuery>,
    State((orderbook, ob_broadcast, candle_broadcast)): State<UnifiedState>,
) -> impl IntoResponse {
    let subscribe_orderbook = params.orderbook.unwrap_or(true);
    let subscribe_ohlcv = params.ohlcv.unwrap_or(true);
    let symbol_filter = params.symbol.unwrap_or_else(|| "ETH/USDC".to_string());
    let timeframe_filter: Option<Vec<String>> = params
        .timeframes
        .map(|tf| tf.split(',').map(|s| s.trim().to_string()).collect());

    ws.on_upgrade(move |socket| {
        handle_unified_socket(
            socket,
            orderbook,
            ob_broadcast,
            candle_broadcast,
            subscribe_orderbook,
            subscribe_ohlcv,
            symbol_filter,
            timeframe_filter,
        )
    })
}

async fn handle_unified_socket(
    socket: WebSocket,
    orderbook: Arc<Mutex<OrderbookState>>,
    ob_broadcast: broadcast::Sender<OrderbookSnapshot>,
    candle_broadcast: broadcast::Sender<CandleUpdate>,
    subscribe_orderbook: bool,
    subscribe_ohlcv: bool,
    symbol_filter: String,
    timeframe_filter: Option<Vec<String>>,
) {
    let (mut sender, mut receiver) = socket.split();

    info!(
        "📡 New unified WebSocket connection: ob={}, ohlcv={}, symbol={}",
        subscribe_orderbook, subscribe_ohlcv, symbol_filter
    );

    // Send initial orderbook snapshot if subscribed
    if subscribe_orderbook {
        let ob = orderbook.lock().await;
        let snapshot = ob.get_snapshot();
        drop(ob); // Release lock immediately

        let message = MarketDataMessage::orderbook_from_snapshot(symbol_filter.clone(), snapshot);
        if let Ok(json) = serde_json::to_string(&message) {
            if sender.send(Message::Text(json.into())).await.is_err() {
                error!("Failed to send initial orderbook snapshot");
                return;
            }
        }
    }

    // Subscribe to update channels
    let mut ob_rx = if subscribe_orderbook {
        Some(ob_broadcast.subscribe())
    } else {
        None
    };

    let mut candle_rx = if subscribe_ohlcv {
        Some(candle_broadcast.subscribe())
    } else {
        None
    };

    // Main event loop
    loop {
        tokio::select! {
            // Orderbook updates
            Some(ob_result) = async {
                if let Some(ref mut rx) = ob_rx {
                    Some(rx.recv().await)
                } else {
                    None
                }
            } => {
                match ob_result {
                    Ok(snapshot) => {
                        // Received orderbook snapshot from broadcast channel
                        debug!("Received orderbook snapshot: {:?}", snapshot);

                        let message = MarketDataMessage::orderbook_from_snapshot(symbol_filter.clone(), snapshot);
                        debug!("Sending orderbook update: {:?}", message);
                        if let Ok(json) = serde_json::to_string(&message) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                error!("Failed to send orderbook update");
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Orderbook: Client lagged, skipped {} updates", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Orderbook broadcast channel closed");
                        break;
                    }
                }
            }

            // OHLCV updates
            Some(candle_result) = async {
                if let Some(ref mut rx) = candle_rx {
                    Some(rx.recv().await)
                } else {
                    None
                }
            } => {
                match candle_result {
                    Ok(update) => {
                        // Filter by symbol
                        if update.s != symbol_filter {
                            continue;
                        }

                        // Filter by timeframe if specified
                        if let Some(ref timeframes) = timeframe_filter {
                            if !timeframes.contains(&update.i) {
                                continue;
                            }
                        }

                        // Send candle update
                        let message = MarketDataMessage::candle(update);

                        if let Ok(json) = serde_json::to_string(&message) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                error!("Failed to send candle update");
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("OHLCV: Client lagged, skipped {} updates", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("OHLCV broadcast channel closed");
                        break;
                    }
                }
            }

            // Handle client messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        info!("Client closed unified connection");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Text(_text))) => {
                        // Could implement subscription changes here
                        // For now, just acknowledge
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {:?}", e);
                        break;
                    }
                    None => {
                        info!("Connection lost");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("Unified WebSocket connection closed");
}
