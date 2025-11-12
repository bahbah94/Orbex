use anyhow::Result;
use sqlx::PgPool;
use tracing::info;
use crate::indexer::candle_aggregator::CandleAggregator;

// TODO: This should come from the event
// I'm using a placeholder for now
const SYMBOL: &str = "ETH/USDC";

/// Parse TradeExecuted event and insert into trades table
pub async fn process_trade(
    pool: &PgPool,
    candle_agg: &mut CandleAggregator,
    trade_id: u64,
    block_number: u32,
    buy_order_id: u64,
    sell_order_id: u64,
    buyer: &str,
    seller: &str,
    price: u128,
    quantity: u128,
) -> Result<()> {
    // Calculate trade value
    let value = price.saturating_mul(quantity);

    // Insert into trades table
    sqlx::query(
        "INSERT INTO trades
        (trade_id, block_number, buy_order_id, sell_order_id, buyer, seller, price, quantity, value, symbol)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (trade_id) DO NOTHING"
    )
    .bind(trade_id as i64)
    .bind(block_number as i64)
    .bind(buy_order_id as i64)
    .bind(sell_order_id as i64)
    .bind(buyer)
    .bind(seller)
    .bind(price.to_string())  // NUMERIC in SQL
    .bind(quantity.to_string())
    .bind(value.to_string())
    // TODO: Replace with actual symbol from event
    .bind(SYMBOL)
    .execute(pool)
    .await?;

    info!(
        "✅ Trade #{} inserted: {} → {} | Price: {} | Qty: {}",
        trade_id, buyer, seller, price, quantity
    );

    // Update candles and broadcast to websocket subscribers
    // Use current timestamp in milliseconds
    let timestamp_ms = chrono::Utc::now().timestamp_millis();
    candle_agg.process_trade(SYMBOL, price, quantity, timestamp_ms)?;

    Ok(())
}
