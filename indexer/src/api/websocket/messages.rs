use crate::indexer::candle_aggregator::CandleUpdate;
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
    Candle(CandleUpdate),
    /// Connection status messages
    Status(StatusMessage),
}

/// Price level for websocket message (Hyperliquid format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPriceLevel {
    /// Price level
    pub px: String,
    /// Aggregate size at this level
    pub sz: String,
    /// Number of orders at this level
    pub n: usize,
}

/// Orderbook update message (Hyperliquid L2 book format)
///
/// Example JSON output:
/// ```json
/// {
///   "type": "orderbook",
///   "symbol": "ETH/USDC",
///   "time": 1754450974231,
///   "levels": [
///     [
///       {"px": "2000.0", "sz": "10.5", "n": 3},
///       {"px": "1999.0", "sz": "5.2", "n": 2}
///     ],
///     [
///       {"px": "2001.0", "sz": "8.3", "n": 4},
///       {"px": "2002.0", "sz": "12.1", "n": 5}
///     ]
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookUpdate {
    /// Trading pair identifier
    pub symbol: String,
    /// Snapshot timestamp in milliseconds
    pub time: i64,
    /// Two-element array: [bids, asks]
    pub levels: [Vec<WsPriceLevel>; 2],
}

/// Status messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMessage {
    pub message: String,
}

impl MarketDataMessage {
    /// Create orderbook message from OrderbookSnapshot (Hyperliquid L2 book format)
    pub fn orderbook_from_snapshot(symbol: String, snapshot: OrderbookSnapshot) -> Self {
        let bids: Vec<WsPriceLevel> = snapshot
            .bids
            .into_iter()
            .map(|level| WsPriceLevel {
                px: level.price.to_string(),
                sz: level.total_quantity.to_string(),
                n: level.order_count,
            })
            .collect();

        let asks: Vec<WsPriceLevel> = snapshot
            .asks
            .into_iter()
            .map(|level| WsPriceLevel {
                px: level.price.to_string(),
                sz: level.total_quantity.to_string(),
                n: level.order_count,
            })
            .collect();

        MarketDataMessage::Orderbook(OrderbookUpdate {
            symbol,
            time: chrono::Utc::now().timestamp_millis(),
            levels: [bids, asks],
        })
    }

    pub fn candle(update: CandleUpdate) -> Self {
        MarketDataMessage::Candle(update)
    }

    pub fn status(message: impl Into<String>) -> Self {
        MarketDataMessage::Status(StatusMessage {
            message: message.into(),
        })
    }
}
