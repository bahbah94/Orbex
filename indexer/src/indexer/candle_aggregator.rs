use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

/// TradingView-compatible Bar format
/// https://www.tradingview.com/charting-library-docs/latest/api/interfaces/Charting_Library.Bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvBar {
    /// Unix timestamp in SECONDS (TradingView requirement)
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
}

/// Internal candle representation with metadata
#[derive(Debug, Clone)]
pub struct Candle {
    pub symbol: String,
    pub timeframe: String, // "1m", "5m", "15m", etc.
    pub open: u128,
    pub high: u128,
    pub low: u128,
    pub close: u128,
    pub volume: u128,
    pub open_time: i64, // Unix timestamp in milliseconds
    pub close_time: i64,
    pub trade_count: u64,
}

impl Candle {
    /// Convert to TradingView Bar format
    pub fn to_tv_bar(&self) -> TvBar {
        TvBar {
            time: self.open_time / 1000, // Convert milliseconds to seconds
            open: self.open as f64,
            high: self.high as f64,
            low: self.low as f64,
            close: self.close as f64,
            volume: Some(self.volume as f64),
        }
    }
}

impl Candle {
    pub fn new(
        symbol: String,
        timeframe: String,
        price: u128,
        quantity: u128,
        timestamp: i64,
    ) -> Self {
        Self {
            symbol,
            timeframe,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: quantity,
            open_time: timestamp,
            close_time: timestamp,
            trade_count: 1,
        }
    }

    /// Update candle with new trade data
    pub fn update(&mut self, price: u128, quantity: u128, timestamp: i64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume = self.volume.saturating_add(quantity);
        self.close_time = timestamp;
        self.trade_count += 1;
    }

    /// Check if this timestamp belongs to the current candle
    pub fn is_in_timeframe(&self, timestamp: i64, timeframe_ms: i64) -> bool {
        let candle_start = (self.open_time / timeframe_ms) * timeframe_ms;
        let candle_end = candle_start + timeframe_ms;
        timestamp >= candle_start && timestamp < candle_end
    }
}

/// Update message sent over websocket (Hyperliquid candle format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleUpdate {
    /// End time in milliseconds
    #[serde(rename = "T")]
    pub end_time: i64,
    /// Start time in milliseconds
    pub t: i64,
    /// Open price (as string to match Hyperliquid)
    pub o: String,
    /// High price
    pub h: String,
    /// Low price
    pub l: String,
    /// Close price
    pub c: String,
    /// Volume
    pub v: String,
    /// Interval/timeframe (e.g., "1m", "5m")
    pub i: String,
    /// Symbol
    pub s: String,
    /// Number of trades
    pub n: u64,
}

impl CandleUpdate {
    pub fn from_candle(candle: &Candle, _is_closed: bool) -> Self {
        Self {
            end_time: candle.close_time,
            t: candle.open_time,
            o: candle.open.to_string(),
            h: candle.high.to_string(),
            l: candle.low.to_string(),
            c: candle.close.to_string(),
            v: candle.volume.to_string(),
            i: candle.timeframe.clone(),
            s: candle.symbol.clone(),
            n: candle.trade_count,
        }
    }
}

pub struct CandleAggregator {
    // Map of (symbol, timeframe) -> current candle
    current_candles: HashMap<(String, String), Candle>,
    broadcast_tx: broadcast::Sender<CandleUpdate>,
    // Supported timeframes in milliseconds
    timeframes: Vec<(String, i64)>,
}

impl CandleAggregator {
    pub fn new(broadcast_tx: broadcast::Sender<CandleUpdate>) -> Self {
        // Define supported timeframes
        let timeframes = vec![
            ("1m".to_string(), 60_000),     // 1 minute
            ("5m".to_string(), 300_000),    // 5 minutes
            ("15m".to_string(), 900_000),   // 15 minutes
            ("30m".to_string(), 1_800_000), // 30 minutes
            ("1h".to_string(), 3_600_000),  // 1 hour
            ("4h".to_string(), 14_400_000), // 4 hours
            ("1d".to_string(), 86_400_000), // 1 day
        ];

        Self {
            current_candles: HashMap::new(),
            broadcast_tx,
            timeframes,
        }
    }

    /// Process a new trade and update all timeframe candles
    pub fn process_trade(
        &mut self,
        symbol: &str,
        price: u128,
        quantity: u128,
        timestamp_ms: i64,
    ) -> Result<()> {
        for (timeframe_name, timeframe_ms) in &self.timeframes {
            let key = (symbol.to_string(), timeframe_name.clone());

            let mut is_closed = false;

            match self.current_candles.get_mut(&key) {
                Some(candle) => {
                    // Check if trade belongs to current candle
                    if candle.is_in_timeframe(timestamp_ms, *timeframe_ms) {
                        candle.update(price, quantity, timestamp_ms);
                    } else {
                        // Candle closed, broadcast the closed candle first
                        let closed_candle = candle.clone();
                        let _ = self
                            .broadcast_tx
                            .send(CandleUpdate::from_candle(&closed_candle, true));

                        // Start new candle
                        *candle = Candle::new(
                            symbol.to_string(),
                            timeframe_name.clone(),
                            price,
                            quantity,
                            timestamp_ms,
                        );
                        is_closed = false; // This is the new candle, not closed
                    }
                }
                None => {
                    // First trade for this symbol/timeframe
                    let candle = Candle::new(
                        symbol.to_string(),
                        timeframe_name.clone(),
                        price,
                        quantity,
                        timestamp_ms,
                    );
                    self.current_candles.insert(key.clone(), candle);
                }
            }

            // Broadcast updated candle
            if let Some(candle) = self.current_candles.get(&key) {
                let _ = self
                    .broadcast_tx
                    .send(CandleUpdate::from_candle(candle, is_closed));
            }
        }

        Ok(())
    }

    /// Get current candle for a symbol/timeframe (useful for initial snapshot)
    pub fn get_current_candle(&self, symbol: &str, timeframe: &str) -> Option<&Candle> {
        self.current_candles
            .get(&(symbol.to_string(), timeframe.to_string()))
    }

    /// Get all current candles for a symbol
    pub fn get_symbol_candles(&self, symbol: &str) -> Vec<&Candle> {
        self.current_candles
            .iter()
            .filter(|((s, _), _)| s == symbol)
            .map(|(_, candle)| candle)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_creation() {
        let candle = Candle::new("ETH/USDC".to_string(), "1m".to_string(), 2000, 10, 1000);
        assert_eq!(candle.open, 2000);
        assert_eq!(candle.high, 2000);
        assert_eq!(candle.low, 2000);
        assert_eq!(candle.close, 2000);
        assert_eq!(candle.volume, 10);
        assert_eq!(candle.trade_count, 1);
    }

    #[test]
    fn test_candle_update() {
        let mut candle = Candle::new("ETH/USDC".to_string(), "1m".to_string(), 2000, 10, 1000);
        candle.update(2100, 20, 2000);

        assert_eq!(candle.open, 2000);
        assert_eq!(candle.high, 2100);
        assert_eq!(candle.low, 2000);
        assert_eq!(candle.close, 2100);
        assert_eq!(candle.volume, 30);
        assert_eq!(candle.trade_count, 2);

        candle.update(1900, 15, 3000);
        assert_eq!(candle.high, 2100);
        assert_eq!(candle.low, 1900);
        assert_eq!(candle.close, 1900);
    }

    #[test]
    fn test_candle_timeframe() {
        let candle = Candle::new("ETH/USDC".to_string(), "1m".to_string(), 2000, 10, 60_000);

        // Same minute
        assert!(candle.is_in_timeframe(60_000, 60_000));
        assert!(candle.is_in_timeframe(119_999, 60_000));

        // Next minute
        assert!(!candle.is_in_timeframe(120_000, 60_000));
    }
}
