use std::result;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use sqlx::PgPool;
use subxt::{OnlineClient, PolkadotConfig};
use tracing::{info, debug};
use subxt::ext::scale_value::{Value, Composite,Primitive};
use crate::order_extractor::*;
use crate::orderbook_reducer::{OrderbookState, OrderInfo};

pub async fn start(
    node_url: &str, 
    pool: PgPool,
    orderbook_state: Arc<Mutex<OrderbookState>>) -> Result<()> {
    let api = OnlineClient::<PolkadotConfig>::from_url(node_url).await?;
    
    info!("✅ Connected to chain: {:?}", api.runtime_version());

    let mut blocks = api.blocks().subscribe_finalized().await?;
    
    info!("📡 Listening for events...");

    while let Some(block) = blocks.next().await {
        let block = block?;
        let block_number = block.header().number;

        debug!("Processing block number: {}", block_number);

        // Get events directly from block
        let events = block.events().await?;

        debug!("   EVENTS:");
        for evt in events.iter() {
            let evt = evt?;
            let pallet_name = evt.pallet_name();
            let event_name = evt.variant_name();
            let event_values = evt.field_values()?;

            //println!("        {}_{}", pallet_name, event_name);
            //println!("          {}", event_values);

            // Route to appropriate handler
            match (pallet_name, event_name) {
                ("Orderbook", "TradeExecuted") => {
                    println!("🎯 TradeExecuted event detected!");
                    //println!("Raw event_values: {:?}", event_values);

                    
                    // wrapping it for completeness and consistency
                    let value = Value {
                        value: subxt::ext::scale_value::ValueDef::Composite(event_values.clone()),
                        context: 0,
                    };
                    //println!("Event values: {:#?}", event_values);
                    match parse_and_insert_trade(&pool, block_number, &value).await {
                        Ok(_) => {
                            println!("✅ Trade inserted successfully!");
                            info!("✅ Trade executed in block {}", block_number);
                        },
                        Err(e) => {
                            println!("❌ FAILED TO INSERT TRADE: {}", e);  // ← THIS WILL SHOW THE ERROR
                            debug!("❌ Failed to parse trade: {}", e);
                        }
                    }
                },
                ("Orderbook", "OrderPlaced") => {
                    info!("📦 Order placed in block {}", block_number);
                    //println!("Full event_values: {:#?}", event_values);
                    match extract_order_placed(&event_values) {
                        Ok(data) => {
                            println!("📦 OrderPlaced: id={}, side={}, price={}, qty={}", 
                                data.order_id, data.side, data.price, data.quantity);
                                let mut state = orderbook_state.lock().unwrap();
                                let order = OrderInfo {
                                    order_id: data.order_id as u64,
                                    side: data.side,
                                    price: data.price,
                                    quantity: data.quantity,
                                    filled_quantity: 0,
                                    status: "Open".to_string(),
                                };
                                state.add_order(order);
                                info!("✅ Order #{} added to state", data.order_id);
                            }
                        Err(e) => println!("❌ Failed to parse orderplaced: {}", e),
                    }
                },
                ("Orderbook", "OrderCancelled") => {
                    info!("❌ Order cancelled in block {}", block_number);
                    match extract_order_cancelled(&event_values) {
                        Ok(data) => {
                            println!("❌ OrderCancelled: id={}, trader={}", data.order_id, data.trader);

                            let mut state = orderbook_state.lock().unwrap();
                            let _ = state.cancel_order(data.order_id as u64);
                            info!("✅ Order #{} cancelled", data.order_id);
                        }
                        Err(e) => println!("❌ Failed to parse orderCancelled: {}", e),
                    }
                },
                ("Orderbook", "OrderFilled") => {
                    info!("✅ Order filled in block {}", block_number);
                    match extract_order_filled(&event_values) {
                        Ok(data) => {
                            println!("✅ OrderFilled: id={}, trader={}", data.order_id, data.trader);
                            let mut state = orderbook_state.lock().unwrap();
                            let quantity = {
                                state.orders.get(&(data.order_id as u64))
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
                },
                ("Orderbook", "OrderPartiallyFilled") => {
                    info!("📊 Order partially filled in block {}", block_number);
                    match extract_order_partially_filled(&event_values) {
                        Ok(data) => {
                            println!("📊 OrderPartiallyFilled: id={}, filled={}, remaining={}", 
                                data.order_id, data.filled_quantity, data.remaining_quantity);
                            
                            let mut state = orderbook_state.lock().unwrap();
                            let _ = state.update_order(data.order_id as u64, data.filled_quantity, "PartiallyFilled");
                            info!("✅ Order #{} partially filled ({}/{})", 
                                data.order_id, data.filled_quantity, data.filled_quantity + data.remaining_quantity);
                        }
                        Err(e) => println!("❌ Failed: {}", e),
                    }
            
                },
                _ => {
                    // Ignore events from other pallets
                }
            }
        }
    }

    Ok(())
}

async fn parse_and_insert_trade(
    pool: &PgPool,
    block_number: u32,
    event_values: &Value<u32>,
) -> Result<()> {
    let trade_id = extract_u128_by_name(event_values, "trade_id")?;
    let buy_order_id = extract_u128_by_name(event_values, "buy_order_id")?;
    let sell_order_id = extract_u128_by_name(event_values, "sell_order_id")?;
    let buyer = extract_account_by_name(event_values, "buyer")?;
    let seller = extract_account_by_name(event_values, "seller")?;
    let price = extract_u128_by_name(event_values, "price")?;
    let quantity = extract_u128_by_name(event_values, "quantity")?;

    let value = price.saturating_mul(quantity);

    info!(
        "🎯 TradeExecuted parsed: trade_id={}, buy={}, sell={}, price={}, qty={}, value={}",
        trade_id, buy_order_id, sell_order_id, price, quantity, value
    );

    sqlx::query(
        "INSERT INTO trades
        (trade_id, block_number,buy_order_id,sell_order_id,buyer,seller,price,quantity,value)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)"
    )
    .bind(trade_id as i64)
    .bind(block_number as i64)
    .bind(buy_order_id as i64)
    .bind(sell_order_id as i64)
    .bind(buyer.clone())
    .bind(seller.clone())
    .bind(price as i64)
    .bind(quantity as i64)
    .bind(value as i64).execute(pool).await?;

    info!("✅ Trade #{} inserted into database!", trade_id);

    Ok(())
}

/// helper functions to extract events parsed
fn extract_u128_by_name(value: &Value<u32>, field_name: &str) -> Result<u128> {
    if let subxt::ext::scale_value::ValueDef::Composite(Composite::Named(fields)) = &value.value {
        for (name, field_value) in fields {
            if name == field_name {
                if let subxt::ext::scale_value::ValueDef::Primitive(Primitive::U128(val)) = &field_value.value
                {
                    return Ok(*val);
                }
            }
        }
    }
    Err(anyhow::anyhow!(
        "Field {} not found or not a u128",
        field_name
    ))
}

fn extract_account_by_name(value: &Value<u32>, field_name: &str) -> Result<String> {
    if let subxt::ext::scale_value::ValueDef::Composite(Composite::Named(fields)) = &value.value {
        for (name, field_value) in fields {
            if name == field_name {
                // Account is a composite of 32 u128 values
                if let subxt::ext::scale_value::ValueDef::Composite(Composite::Unnamed(bytes)) =
                    &field_value.value
                {
                    if let subxt::ext::scale_value::ValueDef::Composite(Composite::Unnamed(
                        inner_bytes,
                    )) = &bytes[0].value
                    {
                        let mut account_bytes = Vec::new();
                        for byte_val in inner_bytes {
                            if let subxt::ext::scale_value::ValueDef::Primitive(
                                subxt::ext::scale_value::Primitive::U128(b),
                            ) = &byte_val.value
                            {
                                account_bytes.push(*b as u8);
                            }
                        }
                        // Return as hex string for storage
                        return Ok(format!("0x{}", hex::encode(&account_bytes)));
                    }
                }
            }
        }
    }
    Err(anyhow::anyhow!("Field {} not found or invalid account", field_name))
}


// Please refer to event_type_ref for understanding these reference types