use crate::indexer::candle_aggregator::TvBar;
use crate::indexer::orderbook_reducer::OrderbookSnapshot;
/// Unified WebSocket message types for orderbook and OHLCV updates
use serde::{Deserialize, Serialize};

/// Unified message envelope for all websocket updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarketDataMessage {
    /// Orderbook snapshot or update
    Orderbook(OrderbookUpdate),
    /// OHLCV candle update
    Ohlcv(OhlcvUpdate),
    /// Connection status messages
    Status(StatusMessage),
}

/// Price level for websocket message (with string formatting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPriceLevel {
    pub price: String,
    pub quantity: String,
    pub order_count: usize,
}

/// Orderbook update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookUpdate {
    pub symbol: String,
    pub bids: Vec<WsPriceLevel>,
    pub asks: Vec<WsPriceLevel>,
    pub timestamp: i64,
}

/// OHLCV candle update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvUpdate {
    pub symbol: String,
    pub timeframe: String,
    pub bar: TvBar,
    pub is_closed: bool,
}

/// Status messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMessage {
    pub message: String,
}

impl MarketDataMessage {
    /// Create orderbook message from OrderbookSnapshot (converts to string format for JSON)
    pub fn orderbook_from_snapshot(symbol: String, snapshot: OrderbookSnapshot) -> Self {
        let bids: Vec<WsPriceLevel> = snapshot
            .bids
            .into_iter()
            .map(|level| WsPriceLevel {
                price: level.price.to_string(),
                quantity: level.total_quantity.to_string(),
                order_count: level.order_count,
            })
            .collect();

        let asks: Vec<WsPriceLevel> = snapshot
            .asks
            .into_iter()
            .map(|level| WsPriceLevel {
                price: level.price.to_string(),
                quantity: level.total_quantity.to_string(),
                order_count: level.order_count,
            })
            .collect();

        MarketDataMessage::Orderbook(OrderbookUpdate {
            symbol,
            bids,
            asks,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub fn ohlcv(symbol: String, timeframe: String, bar: TvBar, is_closed: bool) -> Self {
        MarketDataMessage::Ohlcv(OhlcvUpdate {
            symbol,
            timeframe,
            bar,
            is_closed,
        })
    }

    pub fn status(message: impl Into<String>) -> Self {
        MarketDataMessage::Status(StatusMessage {
            message: message.into(),
        })
    }
}
