use std::sync::Arc;
use tokio::sync::Mutex;

use crate::indexer::candle_aggregator::CandleAggregator;
use crate::indexer::orderbook_reducer::{OrderInfo, OrderbookState};
use crate::indexer::runtime;
use crate::indexer::trade_mapper::{process_trade, TradeProcessingContext};
use anyhow::Result;
use rust_decimal::Decimal;
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
                    match evt.as_event::<runtime::OrderPlaced>() {
                        Ok(Some(place_order_event)) => {
                            // Convert u128 to Decimal by dividing by 10^6
                            let price =
                                Decimal::from(place_order_event.price) / Decimal::from(1_000_000);
                            let quantity = Decimal::from(place_order_event.quantity)
                                / Decimal::from(1_000_000);

                            info!(
                                "📦 OrderPlaced: id={}, side={}, price={}, qty={}",
                                place_order_event.order_id, place_order_event.side, price, quantity
                            );
                            let mut state = orderbook_state.lock().await;
                            let order = OrderInfo {
                                order_id: place_order_event.order_id,
                                side: place_order_event.side.to_string(),
                                price,
                                quantity,
                                filled_quantity: Decimal::ZERO,
                                status: "Open".to_string(),
                            };
                            state.add_order(order);
                            info!("✅ Order #{} added to state", place_order_event.order_id);
                        }
                        Ok(None) => debug!("❌ OrderPlaced event is None (filtered?)"),
                        Err(e) => debug!("❌ Failed to parse orderplaced: {}", e),
                    }
                }
                ("Orderbook", "OrderCancelled") => {
                    info!("❌ Order cancelled in block {}", block_number);
                    match evt.as_event::<runtime::OrderCancelled>() {
                        Ok(Some(data)) => {
                            println!(
                                "❌ OrderCancelled: id={}, trader={}",
                                data.order_id, data.trader
                            );

                            let mut state = orderbook_state.lock().await;
                            let _ = state.cancel_order(data.order_id);
                            info!("✅ Order #{} cancelled", data.order_id);
                        }
                        Ok(None) => debug!("❌ OrderCancelled event is None (filtered?)"),
                        Err(e) => debug!("❌ Failed to parse orderCancelled: {}", e),
                    }
                }
                ("Orderbook", "OrderFilled") => {
                    info!("✅ Order filled in block {}", block_number);
                    match evt.as_event::<runtime::OrderFilled>() {
                        Ok(Some(data)) => {
                            println!(
                                "✅ OrderFilled: id={}, trader={}",
                                data.order_id, data.trader
                            );
                            let mut state = orderbook_state.lock().await;
                            let quantity =
                                { state.orders.get(&data.order_id).map(|order| order.quantity) };

                            // Now use the mutable state, doing this to fix clash of mut and immut borrow from before
                            if let Some(qty) = quantity {
                                let _ = state.update_order(data.order_id, qty, "Filled");
                            }
                            info!("✅ Order #{} marked as filled", data.order_id);
                        }
                        Ok(None) => debug!("❌ OrderFilled event is None (filtered?)"),
                        Err(e) => debug!("❌ Failed to parse order filled: {}", e),
                    }
                }
                ("Orderbook", "OrderPartiallyFilled") => {
                    match evt.as_event::<runtime::OrderPartiallyFilled>() {
                        Ok(Some(data)) => {
                            // Convert u128 to Decimal by dividing by 10^6
                            let filled_quantity =
                                Decimal::from(data.filled_quantity) / Decimal::from(1_000_000);
                            let remaining_quantity =
                                Decimal::from(data.remaining_quantity) / Decimal::from(1_000_000);

                            println!(
                                "📊 OrderPartiallyFilled: id={}, filled={}, remaining={}",
                                data.order_id, filled_quantity, remaining_quantity
                            );

                            let mut state = orderbook_state.lock().await;
                            let _ = state.update_order(
                                data.order_id,
                                filled_quantity,
                                "PartiallyFilled",
                            );
                            info!(
                                "✅ Order #{} partially filled ({}/{})",
                                data.order_id,
                                filled_quantity,
                                filled_quantity + remaining_quantity
                            );
                        }
                        Ok(None) => debug!("❌ OrderPartiallyFilled event is None (filtered?)"),
                        Err(e) => debug!("❌ Failed: {}", e),
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
