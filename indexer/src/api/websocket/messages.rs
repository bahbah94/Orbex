use crate::indexer::candle_aggregator::TvBar;
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

/// Orderbook update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookUpdate {
    pub symbol: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: i64,
}

/// Price level in orderbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: String,
    pub quantity: String,
    pub order_count: usize,
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
    pub fn orderbook(symbol: String, bids: Vec<PriceLevel>, asks: Vec<PriceLevel>) -> Self {
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
