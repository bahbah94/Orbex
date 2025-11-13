use crate::indexer::candle_aggregator::CandleUpdate;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::indexer::orderbook_reducer::OrderbookState;

pub type AppState = (Arc<Mutex<OrderbookState>>, PgPool);

#[derive(Debug, Deserialize)]
pub struct CandleQuery {
    /// Trading pair symbol (e.g., "ETH/USDC")
    pub symbol: String,
    /// Start time in seconds (Unix timestamp)
    pub start_time: i64,
    /// End time in seconds (Unix timestamp)
    pub end_time: i64,
    /// Interval/timeframe (e.g., "1m", "5m", "15m", "1h", etc.)
    pub interval: String,
}

/// Get historical OHLCV candles in Hyperliquid format
///
/// Query parameters:
/// - `symbol`: Trading pair (e.g., "ETH/USDC")
/// - `start_time`: Start timestamp in SECONDS (Unix epoch)
/// - `end_time`: End timestamp in SECONDS (Unix epoch)
/// - `interval`: Time interval ("1m", "5m", "15m", "30m", "1h", "4h", "1d", "1w", "1M")
///
/// Returns array of candles in Hyperliquid format:
/// ```json
/// [
///   {
///     "T": 1699000060000,
///     "t": 1699000000000,
///     "o": "2000.0",
///     "h": "2100.0",
///     "l": "1950.0",
///     "c": "2050.0",
///     "v": "15000.0",
///     "i": "1m",
///     "s": "ETH/USDC",
///     "n": 42
///   }
/// ]
/// ```
pub async fn get_candles(
    Query(params): Query<CandleQuery>,
    State((_orderbook, pool)): State<AppState>,
) -> impl IntoResponse {
    // Map interval to TimescaleDB view names
    let view_name = match params.interval.as_str() {
        "1m" => "one_minute_candles",
        "5m" => "five_minutes_candles",
        "15m" => "fifteen_minutes_candles",
        "30m" => "thirty_minutes_candles",
        "1h" => "one_hour_candles",
        "4h" => "four_hours_candles",
        "1d" => "one_day_candles",
        "1w" => "one_week_candles",
        "1M" => "one_month_candles",
        _ => {
            return Json(json!({
                "error": format!("Unsupported interval: {}", params.interval)
            }));
        }
    };

    // Limit candles to prevent abuse
    const MAX_CANDLES: i64 = 5000;

    // Query TimescaleDB for candles
    // Note: bucket is timestamp, open/high/low/close/volume are NUMERIC, trade_count is BIGINT
    let query = format!(
        "SELECT
            EXTRACT(EPOCH FROM bucket)::bigint as bucket_time,
            open::float8 as open,
            high::float8 as high,
            low::float8 as low,
            close::float8 as close,
            volume::float8 as volume,
            trade_count::bigint as trade_count
        FROM {}
        WHERE symbol = $1
            AND bucket >= to_timestamp($2)
            AND bucket < to_timestamp($3)
        ORDER BY bucket ASC
        LIMIT $4",
        view_name
    );

    match sqlx::query_as::<_, (i64, f64, f64, f64, f64, f64, i64)>(&query)
        .bind(&params.symbol)
        .bind(params.start_time)
        .bind(params.end_time)
        .bind(MAX_CANDLES)
        .fetch_all(&pool)
        .await
    {
        Ok(rows) => {
            let candles: Vec<CandleUpdate> = rows
                .into_iter()
                .map(|(bucket_time, open, high, low, close, volume, trade_count)| {
                    // Calculate interval duration in milliseconds
                    let interval_ms = match params.interval.as_str() {
                        "1m" => 60_000,
                        "5m" => 300_000,
                        "15m" => 900_000,
                        "30m" => 1_800_000,
                        "1h" => 3_600_000,
                        "4h" => 14_400_000,
                        "1d" => 86_400_000,
                        "1w" => 604_800_000,
                        "1M" => 2_592_000_000, // ~30 days
                        _ => 60_000,
                    };

                    let start_time_ms = bucket_time * 1000;
                    let end_time_ms = start_time_ms + interval_ms;

                    CandleUpdate {
                        end_time: end_time_ms,
                        t: start_time_ms,
                        o: open.to_string(),
                        h: high.to_string(),
                        l: low.to_string(),
                        c: close.to_string(),
                        v: volume.to_string(),
                        i: params.interval.clone(),
                        s: params.symbol.clone(),
                        n: trade_count as u64,
                    }
                })
                .collect();

            Json(json!(candles))
        }
        Err(e) => {
            eprintln!("❌ Database error in get_candles: {}", e);
            Json(json!({
                "error": format!("Database error: {}", e)
            }))
        }
    }
}
