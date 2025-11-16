use crate::indexer::candle_aggregator::CandleUpdate;
use crate::indexer::orderbook_reducer::OrderbookSnapshot;
/// Unified WebSocket message types for orderbook and OHLCV updates
use rust_decimal::Decimal;
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
    /// with cumulative depth: bids accumulate as prices go down, asks accumulate as prices go up
    pub fn orderbook_from_snapshot(symbol: String, snapshot: OrderbookSnapshot) -> Self {
        // For bids: accumulate quantities as we go down in price (highest to lowest)
        // Bids are already sorted from highest to lowest
        let mut cumulative_bid_qty = Decimal::ZERO;
        let bids: Vec<WsPriceLevel> = snapshot
            .bids
            .into_iter()
            .map(|level| {
                cumulative_bid_qty += level.total_quantity;
                WsPriceLevel {
                    px: level.price.to_string(),
                    sz: cumulative_bid_qty.to_string(),
                    n: level.order_count,
                }
            })
            .collect();

        // For asks: accumulate quantities as we go up in price (lowest to highest)
        // Asks are already sorted from lowest to highest
        let mut cumulative_ask_qty = Decimal::ZERO;
        let asks: Vec<WsPriceLevel> = snapshot
            .asks
            .into_iter()
            .map(|level| {
                cumulative_ask_qty += level.total_quantity;
                WsPriceLevel {
                    px: level.price.to_string(),
                    sz: cumulative_ask_qty.to_string(),
                    n: level.order_count,
                }
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
}
