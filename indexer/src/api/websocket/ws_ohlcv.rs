use axum::{
    extract::{State, WebSocketUpgrade, Query},
    response::IntoResponse,
};
use axum::extract::ws::{WebSocket, Message};
use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use crate::indexer::candle_aggregator::CandleUpdate;

pub type OhlcvState = broadcast::Sender<CandleUpdate>;

/// WebSocket OHLCV Streaming Endpoint
///
/// Streams real-time OHLCV (candlestick) updates in TradingView-compatible format.
///
/// # Query Parameters
/// - `symbol`: Trading pair symbol (e.g., "ETH/USDC"). Default: "ETH/USDC"
/// - `timeframes`: Comma-separated list of timeframes (e.g., "1m,5m,1h"). Default: all timeframes
///
/// # Message Format
/// Each update is a JSON object with TradingView Bar format:
/// ```json
/// {
///   "symbol": "ETH/USDC",
///   "timeframe": "1m",
///   "bar": {
///     "time": 1699000000,     // Unix timestamp in SECONDS
///     "open": 2000.0,
///     "high": 2100.0,
///     "low": 1950.0,
///     "close": 2050.0,
///     "volume": 15000.0
///   },
///   "is_closed": false        // true when candle closes
/// }
/// ```
///
/// See: https://www.tradingview.com/charting-library-docs/latest/api/interfaces/Charting_Library.Bar
#[derive(Debug, Deserialize)]
pub struct OhlcvQuery {
    /// Symbol to subscribe to (e.g., "ETH/USDC")
    pub symbol: Option<String>,
    /// Timeframes to subscribe to, comma-separated (e.g., "1m,5m,1h")
    /// If not provided, subscribes to all timeframes
    pub timeframes: Option<String>,
}

pub async fn ws_ohlcv_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<OhlcvQuery>,
    State(broadcast_tx): State<OhlcvState>,
) -> impl IntoResponse {
    let symbol_filter = params.symbol.unwrap_or_else(|| "ETH/USDC".to_string());
    let timeframe_filter: Option<Vec<String>> = params.timeframes.map(|tf| {
        tf.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    });

    ws.on_upgrade(move |socket| handle_ohlcv_socket(socket, broadcast_tx, symbol_filter, timeframe_filter))
}

async fn handle_ohlcv_socket(
    socket: WebSocket,
    broadcast_tx: broadcast::Sender<CandleUpdate>,
    symbol_filter: String,
    timeframe_filter: Option<Vec<String>>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut candle_rx = broadcast_tx.subscribe();

    println!("📊 New OHLCV WebSocket connection for symbol: {}", symbol_filter);

    // TODO: Send initial snapshot of current candles
    // This would require access to CandleAggregator to get current state
    // For now, clients will start receiving updates immediately

    loop {
        tokio::select! {
            // Receive candle updates from broadcast channel
            candle_update = candle_rx.recv() => {
                match candle_update {
                    Ok(update) => {
                        // Filter by symbol
                        if update.symbol != symbol_filter {
                            continue;
                        }

                        // Filter by timeframe if specified
                        if let Some(ref timeframes) = timeframe_filter {
                            if !timeframes.contains(&update.timeframe) {
                                continue;
                            }
                        }

                        // Send TradingView-compatible update
                        // The update already contains the TvBar format
                        if let Ok(json) = serde_json::to_string(&update) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                println!("❌ Failed to send candle update to client");
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        println!("⚠️  Client lagged behind, skipped {} updates", skipped);
                        // Continue receiving, client will catch up
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        println!("📪 Broadcast channel closed");
                        break;
                    }
                }
            }

            // Handle messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        println!("👋 Client closed OHLCV connection");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Handle client commands (e.g., subscribe to different symbols/timeframes)
                        println!("📥 Received message from client: {}", text);
                        // TODO: Implement dynamic subscription changes
                    }
                    None => {
                        println!("🔌 Connection lost");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    println!("🔚 OHLCV WebSocket connection closed");
}
