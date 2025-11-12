use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tracing::info;
use tokio::sync::broadcast;

mod db;
mod indexer;
mod api;

use std::sync::Arc;
use tokio::sync::Mutex;

use indexer::orderbook_reducer::OrderbookState;
use indexer::candle_aggregator::{CandleAggregator, CandleUpdate};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv().ok();

    let node_url = env::var("NODE_WS_URL").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string());
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("🚀 Starting Orderbook Indexer");
    info!("📡 Node URL: {}", node_url);
    info!("🗄️  Database: {}", db_url);

    // Initialize database
    info!("📊 Connecting to database...");
    let pool = db::init_pool(&db_url).await?;

    info!("📈 Initializing orderbook state...");
    let orderbook_state = Arc::new(Mutex::new(OrderbookState::new()));

    // Create broadcast channel for OHLCV updates
    // Buffer of 1000 allows clients to lag behind without dropping messages
    info!("📊 Initializing candle broadcast channel...");
    let (candle_tx, _) = broadcast::channel::<CandleUpdate>(1000);

    // Initialize candle aggregator
    let candle_aggregator = Arc::new(Mutex::new(CandleAggregator::new(candle_tx.clone())));

    // Clone for API server
    let orderbook_for_api = orderbook_state.clone();
    let pool_for_api = pool.clone();
    let candle_tx_for_api = candle_tx.clone();

    // Start API server in background
    info!("🌐 Starting API server...");
    tokio::spawn(async move {
        if let Err(e) = api::server::run_server(orderbook_for_api, pool_for_api, candle_tx_for_api).await {
            eprintln!("❌ API server error: {}", e);
        }
    });

    // Start event collector
    info!("🔌 Connecting to node at {}", node_url);
    indexer::event_collector::start(&node_url, pool, orderbook_state, candle_aggregator).await?;

    Ok(())
}
