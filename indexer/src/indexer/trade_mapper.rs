use crate::indexer::candle_aggregator::CandleAggregator;
use crate::indexer::runtime::TradeExecuted;
use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

// TODO: This should come from the event
// I'm using a placeholder for now
const SYMBOL: &str = "ETH/USDC";

/// Context for processing trades - holds shared resources
pub struct TradeProcessingContext<'a> {
    pub pool: &'a PgPool,
    pub candle_agg: &'a mut CandleAggregator,
}

/// Parsed trade data from an event
pub struct TradeData {
    pub trade_id: u128,
    pub block_number: u32,
    pub buy_order_id: u128,
    pub sell_order_id: u128,
    pub buyer: String,
    pub seller: String,
    pub price: u128,
    pub quantity: u128,
}

impl TradeData {
    /// Parse trade data from a TradeExecuted event using generated types
    pub fn from_typed_event(event: &TradeExecuted, block_number: u32) -> Self {
        Self {
            trade_id: event.trade_id as u128,
            block_number,
            buy_order_id: event.buy_order_id as u128,
            sell_order_id: event.sell_order_id as u128,
            buyer: format!("0x{}", hex::encode(event.buyer.0)),
            seller: format!("0x{}", hex::encode(event.seller.0)),
            price: event.price,
            quantity: event.quantity,
        }
    }

    /// Calculate trade value (price * quantity)
    pub fn value(&self) -> u128 {
        self.price.saturating_mul(self.quantity)
    }
}

/// Parse TradeExecuted event and insert into database with candle updates
pub async fn process_trade(
    ctx: &mut TradeProcessingContext<'_>,
    block_number: u32,
    event: &TradeExecuted,
) -> Result<()> {
    let trade = TradeData::from_typed_event(event, block_number);

    info!(
        "🎯 TradeExecuted parsed: trade_id={}, buy={}, sell={}, price={}, qty={}, value={}",
        trade.trade_id,
        trade.buy_order_id,
        trade.sell_order_id,
        trade.price,
        trade.quantity,
        trade.value()
    );

    let value = trade.value();

    // Insert into trades table
    sqlx::query(
        "INSERT INTO trades
        (trade_id, block_number, buy_order_id, sell_order_id, buyer, seller, price, quantity, value, symbol)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (trade_id) DO NOTHING",
    )
    .bind(trade.trade_id as i64)
    .bind(trade.block_number as i64)
    .bind(trade.buy_order_id as i64)
    .bind(trade.sell_order_id as i64)
    .bind(&trade.buyer)
    .bind(&trade.seller)
    .bind(trade.price as i64)
    .bind(trade.quantity as i64)
    .bind(value as i64)
    .bind(SYMBOL)
    .execute(ctx.pool)
    .await?;

    info!("✅ Trade #{} inserted into database!", trade.trade_id);

    // Update candles and broadcast to websocket subscribers
    let timestamp_ms = chrono::Utc::now().timestamp_millis();
    ctx.candle_agg
        .process_trade(SYMBOL, trade.price, trade.quantity, timestamp_ms)?;

    Ok(())
}
