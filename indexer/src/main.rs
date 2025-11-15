use dotenvy::dotenv;
use anyhow::Result;
use std::env;
use tokio::sync::broadcast;
use tracing::info;

mod api;
mod db;
mod indexer;

use std::sync::Arc;
use tokio::sync::Mutex;

use indexer::candle_aggregator::{CandleAggregator, CandleUpdate};
use indexer::orderbook_reducer::OrderbookState;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables first
    dotenv().ok();

    // Initialize logging with environment variable support
    // Supports RUST_LOG env var (e.g., RUST_LOG=debug, RUST_LOG=indexer=trace)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();

    let node_url = env::var("NODE_WS_URL").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string());
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string()),
        env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string()),
        env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string()),
        env::var("POSTGRES_DB").unwrap_or_else(|_| "orbex".to_string()),
    );

    info!("🚀 Starting Orderbook Indexer");
    info!("📡 Node URL: {}", node_url);
    info!("🗄️  Database: {}", db_url);

    // Initialize database
    info!("📊 Connecting to database...");
    let pool = db::init_pool(&db_url).await?;

    // Create broadcast channels for push-based updates
    info!("📊 Initializing broadcast channels...");

    // Orderbook update channel (broadcasts full snapshots)
    let (ob_tx, _) = broadcast::channel::<indexer::orderbook_reducer::OrderbookSnapshot>(1000);

    // OHLCV update channel
    let (candle_tx, _) = broadcast::channel::<CandleUpdate>(1000);

    info!("📈 Initializing orderbook state with broadcast...");
    let orderbook_state = Arc::new(Mutex::new(OrderbookState::with_broadcast(ob_tx.clone())));

    // Initialize candle aggregator
    let candle_aggregator = Arc::new(Mutex::new(CandleAggregator::new(candle_tx.clone())));

    // Clone for API server
    let orderbook_for_api = orderbook_state.clone();
    let pool_for_api = pool.clone();
    let ob_tx_for_api = ob_tx.clone();
    let candle_tx_for_api = candle_tx.clone();

    // Start API server in background
    info!("🌐 Starting API server...");
    tokio::spawn(async move {
        if let Err(e) = api::server::run_server(
            orderbook_for_api,
            pool_for_api,
            ob_tx_for_api,
            candle_tx_for_api,
        )
        .await
        {
            eprintln!("❌ API server error: {}", e);
        }
    });

    // Start event collector
    info!("🔌 Connecting to node at {}", node_url);
    indexer::event_collector::start(&node_url, pool, orderbook_state, candle_aggregator).await?;

    Ok(())
}
