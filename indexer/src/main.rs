use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tracing::info;

mod db;
mod event_collector;
mod trade_mapper;
mod orderbook_reducer;
mod order_extractor;
use std::sync::Arc;
use tokio::sync::Mutex;

use orderbook_reducer::OrderbookState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv().ok();

    let node_url = env::var("NODE_WS_URL")
        .unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string());
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    info!("🚀 Starting Orderbook Indexer");
    info!("📡 Node URL: {}", node_url);
    info!("🗄️  Database: {}", db_url);

    // Initialize database
    info!("📊 Connecting to database...");
    let pool = db::init_pool(&db_url).await?;
    db::run_migrations(&pool).await?;

    info!("📈 Initializing orderbook state...");
    let orderbook_state = Arc::new(Mutex::new(OrderbookState::new()));


    // Start event collector
    info!("🔌 Connecting to node at {}", node_url);
    event_collector::start(&node_url, pool, orderbook_state).await?;

    Ok(())
}