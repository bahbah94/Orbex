use std::sync::Arc;
use tokio::sync::Mutex;

use crate::indexer::candle_aggregator::CandleAggregator;
use crate::indexer::order_extractor::*;
use crate::indexer::orderbook_reducer::{OrderInfo, OrderbookState};
use crate::indexer::runtime;
use crate::indexer::trade_mapper::{process_trade, TradeProcessingContext};
use anyhow::Result;
use sqlx::PgPool;
use subxt::{OnlineClient, PolkadotConfig};
use tracing::{debug, info};

pub async fn start(
    node_url: &str,
    pool: PgPool,
    orderbook_state: Arc<Mutex<OrderbookState>>,
    candle_aggregator: Arc<Mutex<CandleAggregator>>,
) -> Result<()> {
    let api = OnlineClient::<PolkadotConfig>::from_url(node_url).await?;

    info!("✅ Connected to chain: {:?}", api.runtime_version());

    let mut blocks = api.blocks().subscribe_finalized().await?;

    info!("📡 Listening for events...");

    while let Some(block) = blocks.next().await {
        let block = block?;
        let block_number = block.header().number;

        info!("📦 Processing block number: {}", block_number);

        // Get events directly from block
        let events = block.events().await?;

        debug!("   EVENTS:");
        for evt in events.iter() {
            let evt = evt?;
            let pallet_name = evt.pallet_name();
            let event_name = evt.variant_name();
            let event_values = evt.field_values()?;

            // Route to appropriate handler
            match (pallet_name, event_name) {
                ("Orderbook", "TradeExecuted") => {
                    println!("🎯 TradeExecuted event detected!");

                    // Decode event using generated types
                    match evt.as_event::<runtime::TradeExecuted>() {
                        Ok(Some(trade_event)) => {
                            // Create context and process trade
                            let mut candle_agg = candle_aggregator.lock().await;
                            let mut ctx = TradeProcessingContext {
                                pool: &pool,
                                candle_agg: &mut candle_agg,
                            };

                            match process_trade(&mut ctx, block_number, &trade_event).await {
                                Ok(_) => {
                                    println!("✅ Trade inserted successfully!");
                                    info!("✅ Trade executed in block {}", block_number);
                                }
                                Err(e) => {
                                    debug!("❌ Failed to process trade: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            debug!("❌ TradeExecuted event is None (filtered?)");
                        }
                        Err(e) => {
                            debug!("❌ Failed to decode trade event: {}", e);
                        }
                    }
                }
                ("Orderbook", "OrderPlaced") => {
                    info!("📦 Order placed in block {}", block_number);
                    //println!("Full event_values: {:#?}", event_values);
                    match extract_order_placed(&event_values) {
                        Ok(data) => {
                            println!(
                                "📦 OrderPlaced: id={}, side={}, price={}, qty={}",
                                data.order_id, data.side, data.price, data.quantity
                            );
                            let mut state = orderbook_state.lock().await;
                            let order = OrderInfo {
                                order_id: data.order_id as u64,
                                side: data.side,
                                price: data.price,
                                quantity: data.quantity,
                                filled_quantity: 0,
                                status: "Open".to_string(),
                            };
                            state.add_order(order);
                            println!("✅ Order #{} added to state", data.order_id);
                            println!("orderbook state: {:?}", *state);
                        }
                        Err(e) => println!("❌ Failed to parse orderplaced: {}", e),
                    }
                }
                ("Orderbook", "OrderCancelled") => {
                    info!("❌ Order cancelled in block {}", block_number);
                    match extract_order_cancelled(&event_values) {
                        Ok(data) => {
                            println!(
                                "❌ OrderCancelled: id={}, trader={}",
                                data.order_id, data.trader
                            );

                            let mut state = orderbook_state.lock().await;
                            let _ = state.cancel_order(data.order_id as u64);
                            info!("✅ Order #{} cancelled", data.order_id);
                        }
                        Err(e) => println!("❌ Failed to parse orderCancelled: {}", e),
                    }
                }
                ("Orderbook", "OrderFilled") => {
                    info!("✅ Order filled in block {}", block_number);
                    match extract_order_filled(&event_values) {
                        Ok(data) => {
                            println!(
                                "✅ OrderFilled: id={}, trader={}",
                                data.order_id, data.trader
                            );
                            let mut state = orderbook_state.lock().await;
                            let quantity = {
                                state
                                    .orders
                                    .get(&(data.order_id as u64))
                                    .map(|order| order.quantity)
                            };

                            // Now use the mutable state, doing this to fix clash of mut and immut borrow from before
                            if let Some(qty) = quantity {
                                let _ = state.update_order(data.order_id as u64, qty, "Filled");
                            }
                            info!("✅ Order #{} marked as filled", data.order_id);
                        }
                        Err(e) => println!("❌ Failed to parse order filled: {}", e),
                    }
                }
                ("Orderbook", "OrderPartiallyFilled") => {
                    info!("📊 Order partially filled in block {}", block_number);
                    match extract_order_partially_filled(&event_values) {
                        Ok(data) => {
                            println!(
                                "📊 OrderPartiallyFilled: id={}, filled={}, remaining={}",
                                data.order_id, data.filled_quantity, data.remaining_quantity
                            );

                            let mut state = orderbook_state.lock().await;
                            let _ = state.update_order(
                                data.order_id as u64,
                                data.filled_quantity,
                                "PartiallyFilled",
                            );
                            info!(
                                "✅ Order #{} partially filled ({}/{})",
                                data.order_id,
                                data.filled_quantity,
                                data.filled_quantity + data.remaining_quantity
                            );
                        }
                        Err(e) => println!("❌ Failed: {}", e),
                    }
                }
                _ => {
                    // Ignore events from other pallets
                }
            }
        }
    }

    Ok(())
}
