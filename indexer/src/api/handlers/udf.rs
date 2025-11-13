use crate::indexer::orderbook_reducer::OrderbookState;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type AppState = (Arc<Mutex<OrderbookState>>, PgPool);

const EXCHANGE: &str = "Polkadex";
const TIMEZONE: &str = "UTC";
const SYMBOL: &str = "ETH/USDC"; // Your symbol
const SUPPORTED_RESOLUTIONS: &[&str] = &["1", "5", "15", "30", "60", "240", "1D", "1W", "1M"];

#[derive(Debug, Deserialize)]
pub struct QuoteQuery {
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct DepthQuery {
    pub symbol: String,
    pub levels: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SymbolQuery {
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub symbol: String,
    pub from: i64,
    pub to: i64,
    pub resolution: String,
}

pub async fn udf_config() -> impl IntoResponse {
    Json(json!({
        "supported_resolutions":SUPPORTED_RESOLUTIONS,
        "supports_group_request": true,
        "supports_marks": false,
        "supports_search": false,
        "supports_timescale_marks": false,
    }))
}

pub async fn udf_quotes(
    Query(_params): Query<QuoteQuery>,
    State((orderbook, _pool)): State<AppState>,
) -> impl IntoResponse {
    let ob = orderbook.lock().await;
    match ob.get_spread() {
        Some((best_bid, best_ask)) => {
            // Get order counts at best levels
            let bid_orders = ob
                .bids
                .get(&best_bid)
                .map(|orders| orders.len())
                .unwrap_or(0);

            let ask_orders = ob
                .asks
                .get(&best_ask)
                .map(|orders| orders.len())
                .unwrap_or(0);

            let spread = best_ask.saturating_sub(best_bid);
            let mid_price = (best_bid.saturating_add(best_ask)) / 2;

            Json(json!({
                "s": "ok",
                "Symbol": SYMBOL,
                "bid": best_bid,
                "ask": best_ask,
                "spread": spread,
                "mid_price": mid_price,
                "bid_orders": bid_orders,
                "ask_orders": ask_orders,
                "timestamp": chrono::Utc::now().timestamp_millis(),
            }))
        }
        None => Json(json!({
            "s": "error",
            "errmsg": "No liquidity available"
        })),
    }
}

// udf search
pub async fn udf_search() -> impl IntoResponse {
    Json(vec![json!({
        "symbol": SYMBOL,
        "full_name": "Ethereum / USDC",
        "description": "Ethereum",
        "exchange": EXCHANGE,
        "type": "crypto",
        "ticker": "ETHUSDC"
    })])
}

//time
pub async fn udf_time() -> impl IntoResponse {
    let timestamp = chrono::Utc::now().timestamp();
    Json(json!(timestamp))
}

// resolve , we need this due to config configurations
pub async fn udf_resolve() -> impl IntoResponse {
    Json(json!({
        "s": "ok",
        "symbol": SYMBOL,
        "description": "Ethereum / USDC",
        "type": "crypto",
        "exchange": EXCHANGE,
        "minmove": 1,
        "pricescale": 100,
        "timezone": TIMEZONE,
        "session": "24x7",
        "has_intraday": true,
        "has_daily": true,
        "has_weekly_and_monthly": true,
        "supported_resolutions": SUPPORTED_RESOLUTIONS,
    }))
}

/// TradingView UDF getBars implementation
///
/// Returns historical OHLCV data from TimescaleDB continuous aggregates.
/// https://www.tradingview.com/charting-library-docs/latest/connecting_data/datafeed-api/required-methods#getbars
///
/// # Query Parameters (HistoryQuery)
/// - `symbol`: Trading pair (e.g., "ETH/USDC")
/// - `from`: Start timestamp in SECONDS (Unix epoch)
/// - `to`: End timestamp in SECONDS (Unix epoch)
/// - `resolution`: Time interval (1, 5, 15, 30, 60, 240, 1D, 1W, 1M)
///
/// # Response Format
/// Success:
/// ```json
/// {
///   "s": "ok",
///   "t": [1699000000, 1699000060],  // timestamps in seconds
///   "o": [2000.0, 2010.0],          // open prices
///   "h": [2100.0, 2110.0],          // high prices
///   "l": [1950.0, 1960.0],          // low prices
///   "c": [2050.0, 2060.0],          // close prices
///   "v": [15000.0, 16000.0]         // volumes
/// }
/// ```
///
/// No data:
/// ```json
/// {
///   "s": "no_data",
///   "nextTime": 1699000000
/// }
/// ```
pub async fn udf_bars(
    Query(params): Query<HistoryQuery>,
    State((_orderbook, pool)): State<AppState>,
) -> impl IntoResponse {
    // Map TradingView resolution to our TimescaleDB view names
    let view_name = match params.resolution.as_str() {
        "1" => "one_minute_candles",
        "5" => "five_minutes_candles",
        "15" => "fifteen_minutes_candles",
        "30" => "thirty_minutes_candles",
        "60" => "one_hour_candles",
        "240" => "four_hours_candles",
        "1D" | "D" => "one_day_candles",
        "1W" | "W" => "one_week_candles",
        "1M" | "M" => "one_month_candles",
        _ => {
            return Json(json!({
                "s": "error",
                "errmsg": format!("Unsupported resolution: {}", params.resolution)
            }));
        }
    };

    // Limit bars to prevent abuse (TradingView typically requests 300-5000 bars)
    const MAX_BARS: i64 = 10000;

    // Query the TimescaleDB view
    // Note: bucket is a timestamp, open/high/low/close are NUMERIC, volume is NUMERIC
    // Using parameterized queries to prevent SQL injection (view_name is validated via match)
    let query = format!(
        "SELECT
            EXTRACT(EPOCH FROM bucket)::bigint as time,
            open::float8 as open,
            high::float8 as high,
            low::float8 as low,
            close::float8 as close,
            volume::float8 as volume
        FROM {}
        WHERE symbol = $1
            AND bucket >= to_timestamp($2)
            AND bucket < to_timestamp($3)
        ORDER BY bucket ASC
        LIMIT $4",
        view_name
    );

    match sqlx::query_as::<_, (i64, f64, f64, f64, f64, f64)>(&query)
        .bind(&params.symbol)
        .bind(params.from)
        .bind(params.to)
        .bind(MAX_BARS)
        .fetch_all(&pool)
        .await
    {
        Ok(rows) => {
            if rows.is_empty() {
                // No data available for this range
                Json(json!({
                    "s": "no_data",
                    "nextTime": params.from
                }))
            } else {
                // Convert to TradingView UDF format
                let mut times = Vec::new();
                let mut opens = Vec::new();
                let mut highs = Vec::new();
                let mut lows = Vec::new();
                let mut closes = Vec::new();
                let mut volumes = Vec::new();

                for (time, open, high, low, close, volume) in rows {
                    times.push(time);
                    opens.push(open);
                    highs.push(high);
                    lows.push(low);
                    closes.push(close);
                    volumes.push(volume);
                }

                Json(json!({
                    "s": "ok",
                    "t": times,
                    "o": opens,
                    "h": highs,
                    "l": lows,
                    "c": closes,
                    "v": volumes
                }))
            }
        }
        Err(e) => {
            eprintln!("❌ Database error in udf_bars: {}", e);
            Json(json!({
                "s": "error",
                "errmsg": format!("Database error: {}", e)
            }))
        }
    }
}

//finally the depth, i think this is not part of trading view but keeping it regardlesss
pub async fn udf_depth(
    Query(params): Query<DepthQuery>,
    State((orderbook, _pool)): State<AppState>,
) -> impl IntoResponse {
    let ob = orderbook.lock().await;
    let depth = params.levels.unwrap_or(20);

    let ask_levels = ob.get_ask_depth(depth);
    let bid_levels = ob.get_bid_depth(depth);

    let bids: Vec<Vec<Value>> = bid_levels
        .iter()
        .map(|(price, count)| {
            // Calculate total quantity at this price level
            let qty: u128 = ob
                .bids
                .get(price)
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|id| {
                    ob.orders
                        .get(id)
                        .map(|o| o.quantity.saturating_sub(o.filled_quantity))
                })
                .sum();

            vec![json!(price), json!(count), json!(qty)]
        })
        .collect();

    let asks: Vec<Vec<Value>> = ask_levels
        .iter()
        .map(|(price, count)| {
            // Calculate total quantity at this price level
            let qty: u128 = ob
                .bids
                .get(price)
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|id| {
                    ob.orders
                        .get(id)
                        .map(|o| o.quantity.saturating_sub(o.filled_quantity))
                })
                .sum();

            vec![json!(price), json!(count), json!(qty)]
        })
        .collect();

    Json(json!({
        "s": "ok",
        "symbol": SYMBOL,
        "bids": bids,
        "asks": asks,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }))
}

pub async fn udf_routes() -> Router<AppState> {
    Router::new()
        .route("/config", get(udf_config))
        .route("/quotes", get(udf_quotes))
        .route("/depth", get(udf_depth))
        .route("/search", get(udf_search))
        .route("/symbols", get(udf_resolve))
        .route("/time", get(udf_time))
        .route("/history", get(udf_bars))
}
