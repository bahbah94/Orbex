use axum::{
    extract::{State, Query},
    response::{IntoResponse, Json},
    http::StatusCode,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::PgPool;
use crate::indexer::order_mapper::OrderbookState;

pub type AppState = (Arc<Mutex<OrderbookState>>, PgPool);

const EXCHANGE: &str = "Polkadex";
const TIMEZONE: &str = "UTC";
const SYMBOL: &str = "BTC/USD";  // Your symbol
const SUPPORTED_RESOLUTIONS: &[&str] = &["1", "5", "15", "30", "60", "1D", "1W", "1M"];

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
    match ob.get_spread(){
        Some ((best_bid, best_ask)) =>{
            // Get order counts at best levels
            let bid_orders = ob.bids.get(&best_bid).map(|orders| orders.len()).unwrap_or(0);

            let ask_orders = ob.asks.get(&best_ask).map(|orders| orders.len()).unwrap_or(0);

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
        None => {
            Json(json!({
                "s": "error",
                "errmsg": "No liquidity available"
            }))
        }  
    }
}


// udf search
pub async fn udf_search() -> impl IntoResponse {
    Json(vec![
        json!({
            "symbol": SYMBOL,
            "full_name": "Bitcoin / USD",
            "description": "Bitcoin",
            "exchange": EXCHANGE,
            "type": "crypto",
            "ticker": "BTCUSD"
        })
    ])
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
        "symbol": "BTC/USD",
        "description": "Bitcoin / USD",
        "type": "crypto",
        "exchange": "Polkadex/CLOB",
        "minmove": 1,
        "pricescale": 100,
        "timezone": "UTC",
        "session": "24x7",
        "has_intraday": true,
        "has_daily": true,
        "supported_resolutions": ["1", "5", "15", "30", "60", "1D"],
    }))
}

pub async fn udf_bars(
    Query(params): Query<HistoryQuery>,
    State((_orderbook, pool)): State<AppState>,
) -> impl IntoResponse {
    // Query your trades table from database
    // Aggregate into OHLCV bars
    // Return historical data
    //FAke for now Jo will handle this stuff
    
    Json(json!({
        "s": "ok",
        "t": [1699000000, 1699003600, 1699007200],
        "o": [43100, 43250, 43200],
        "h": [43300, 43500, 43400],
        "l": [43050, 43200, 43150],
        "c": [43250, 43400, 43300],
        "v": [1500, 1200, 1800]
    }))
}


//finally the depth, i think this is not part of trading view but keeping it regardlesss
pub async fn udf_depth(
    Query(_params): Query<DepthQuery>,
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
                    ob.orders.get(id).map(|o| {
                        o.quantity.saturating_sub(o.filled_quantity)
                    })
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
                    ob.orders.get(id).map(|o| {
                        o.quantity.saturating_sub(o.filled_quantity)
                    })
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