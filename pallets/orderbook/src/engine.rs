use frame_support::pallet_prelude::*;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

// Import our types
use crate::types::*;
use frame_system::Config;

// This will match with the cache structure
pub fn match_pending_internal<T: Config>(
    pending_bids: BTreeMap<Amount, Vec<OrderId>>,
    pending_asks: BTreeMap<Amount, Vec<OrderId>>,
    orders_map: &mut BTreeMap<OrderId, Order<T>>,
) -> Result<(Vec<Trade<T>>, Vec<OrderId>), DispatchError> {
    let mut bid_book = pending_bids;
    let mut ask_book = pending_asks;
    let mut trades = Vec::new();

    let mut all_pending_ids = Vec::new();

    for (_price, order_ids) in bid_book.iter() {
        all_pending_ids.extend(order_ids.clone());
    }

    for (_price, order_ids) in ask_book.iter() {
        all_pending_ids.extend(order_ids.clone());
    }

    all_pending_ids.sort();

    for order_id in all_pending_ids {
        let mut order = match orders_map.get(&order_id) {
            Some(o) => o.clone(),
            None => continue,
        };

        remove_from_orderbook(order_id, &order, &mut bid_book, &mut ask_book);

        let order_trades = match order.side {
            OrderSide::Buy => match_buy_order(&mut order, &mut ask_book, orders_map)?,
            OrderSide::Sell => match_sell_order(&mut order, &mut bid_book, orders_map)?,
        };

        trades.extend(order_trades);

        orders_map.insert(order_id, order.clone());

        if order.status != OrderStatus::Filled {
            add_order_to_book(&order, &mut bid_book, &mut ask_book);
        }
    }

    let mut unmatched = Vec::new();
    for (_price, ids) in bid_book.iter() {
        unmatched.extend(ids.clone());
    }
    for (_price, ids) in ask_book.iter() {
        unmatched.extend(ids.clone());
    }

    Ok((trades, unmatched))
}

pub fn match_persistent_storage<T: Config>(
    persistent_bids: &mut BTreeMap<Amount, Vec<OrderId>>,
    persistent_asks: &mut BTreeMap<Amount, Vec<OrderId>>,
    unmatched: Vec<OrderId>,
    orders_map: &mut BTreeMap<OrderId, Order<T>>,
) -> Result<Vec<Trade<T>>, DispatchError> {
    let mut trades = Vec::new();

    for order_id in unmatched.iter() {
        let mut order = match orders_map.get(order_id) {
            Some(o) => o.clone(),
            None => continue,
        };

        let order_trades = match order.side {
            OrderSide::Buy => match_buy_order(&mut order, persistent_asks, orders_map),
            OrderSide::Sell => match_sell_order(&mut order, persistent_bids, orders_map),
        };

        trades.extend(order_trades.unwrap());

        orders_map.insert(*order_id, order.clone());

        if order.status != OrderStatus::Filled {
            add_order_to_book(&order, persistent_bids, persistent_asks);
        }
    }
    Ok(trades)
}

fn remove_from_orderbook<T: Config>(
    order_id: OrderId,
    order: &Order<T>,
    bid_book: &mut BTreeMap<Amount, Vec<OrderId>>,
    ask_book: &mut BTreeMap<Amount, Vec<OrderId>>,
) {
    let book = match order.side {
        OrderSide::Buy => bid_book,
        OrderSide::Sell => ask_book,
    };

    if let Some(ids) = book.get_mut(&order.price) {
        ids.retain(|id| *id != order_id);
        if ids.is_empty() {
            book.remove(&order.price);
        }
    }
}

fn add_order_to_book<T: Config>(
    order: &Order<T>,
    bid_book: &mut BTreeMap<Amount, Vec<OrderId>>,
    ask_book: &mut BTreeMap<Amount, Vec<OrderId>>,
) {
    let book = match order.side {
        OrderSide::Buy => bid_book,
        OrderSide::Sell => ask_book,
    };

    book.entry(order.price)
        .or_insert_with(Vec::new)
        .push(order.order_id);
}

fn match_buy_order<T: Config>(
    buy_order: &mut Order<T>,
    ask_book: &mut BTreeMap<Amount, Vec<OrderId>>,
    orders_map: &mut BTreeMap<OrderId, Order<T>>,
) -> Result<Vec<Trade<T>>, DispatchError> {
    let mut trades = Vec::new();
    let mut prices_to_remove = Vec::new();

    // Get all ask prices sorted (lowest first)
    let ask_prices: Vec<Amount> = ask_book.keys().cloned().collect();

    for price in ask_prices.iter() {
        // Check if we can match at this price
        match buy_order.order_type {
            OrderType::Market => {
                // Market orders match at any price
            }
            OrderType::Limit => {
                if buy_order.price < *price {
                    break; // Too expensive, stop
                }
            }
        }

        // Check if buy order still needs filling
        if remaining_quantity(buy_order) == 0 {
            break;
        }

        // Get sell orders at this price level
        if let Some(sell_order_ids) = ask_book.get_mut(price) {
            let mut indices_to_remove = Vec::new();

            // Match with each sell order (FIFO - price-time priority)
            for (idx, sell_order_id) in sell_order_ids.iter().enumerate() {
                // Get the sell order
                let mut sell_order = match orders_map.get(sell_order_id) {
                    Some(o) => o.clone(),
                    None => continue,
                };

                // Execute trade at this price level (maker's price)
                let trade = execute_trade(buy_order, &mut sell_order, *price)?;
                trades.push(trade);

                // Update sell order in orders_map
                orders_map.insert(*sell_order_id, sell_order.clone());

                // If sell order is filled, mark for removal
                if sell_order.status == OrderStatus::Filled {
                    indices_to_remove.push(idx);
                }

                // If buy order is filled, stop matching
                if buy_order.status == OrderStatus::Filled {
                    break;
                }
            }

            // Remove filled orders (reverse to maintain indices)
            for idx in indices_to_remove.iter().rev() {
                sell_order_ids.remove(*idx);
            }

            // If no orders left at this price, mark for removal
            if sell_order_ids.is_empty() {
                prices_to_remove.push(*price);
            }
        }
    }

    // Clean up empty price levels
    for price in prices_to_remove {
        ask_book.remove(&price);
    }

    Ok(trades)
}

fn match_sell_order<T: Config>(
    sell_order: &mut Order<T>,
    bid_book: &mut BTreeMap<Amount, Vec<OrderId>>,
    orders_map: &mut BTreeMap<OrderId, Order<T>>,
) -> Result<Vec<Trade<T>>, DispatchError> {
    let mut trades = Vec::new();
    let mut prices_to_remove = Vec::new();

    // Get all bid prices sorted (highest first)
    let mut bid_prices: Vec<Amount> = bid_book.keys().cloned().collect();
    bid_prices.sort_by(|a, b| b.cmp(a)); // Reverse sort

    for price in bid_prices.iter() {
        // Check if we can match at this price
        match sell_order.order_type {
            OrderType::Market => {
                // Market orders match at any price
            }
            OrderType::Limit => {
                if sell_order.price > *price {
                    break; // Too cheap, stop
                }
            }
        }

        // Check if sell order still needs filling
        if remaining_quantity(sell_order) == 0 {
            break;
        }

        // Get buy orders at this price level
        if let Some(buy_order_ids) = bid_book.get_mut(price) {
            let mut indices_to_remove = Vec::new();

            // Match with each buy order (FIFO - price-time priority)
            for (idx, buy_order_id) in buy_order_ids.iter().enumerate() {
                // Get the buy order
                let mut buy_order = match orders_map.get(buy_order_id) {
                    Some(o) => o.clone(),
                    None => continue,
                };

                // Execute trade at this price level (maker's price)
                let trade = execute_trade(&mut buy_order, sell_order, *price)?;
                trades.push(trade);

                // Update buy order in orders_map
                orders_map.insert(*buy_order_id, buy_order.clone());

                // If buy order is filled, mark for removal
                if buy_order.status == OrderStatus::Filled {
                    indices_to_remove.push(idx);
                }

                // If sell order is filled, stop matching
                if sell_order.status == OrderStatus::Filled {
                    break;
                }
            }

            // Remove filled orders (reverse to maintain indices)
            for idx in indices_to_remove.iter().rev() {
                buy_order_ids.remove(*idx);
            }

            // If no orders left at this price, mark for removal
            if buy_order_ids.is_empty() {
                prices_to_remove.push(*price);
            }
        }
    }

    // Clean up empty price levels
    for price in prices_to_remove {
        bid_book.remove(&price);
    }

    Ok(trades)
}

fn remaining_quantity<T: Config>(order: &mut Order<T>) -> Amount {
    order.quantity.saturating_sub(order.filled_quantity)
}

fn execute_trade<T: Config>(
    buy_order: &mut Order<T>,
    sell_order: &mut Order<T>,
    match_price: Amount,
) -> Result<Trade<T>, DispatchError> {
    let buy_remaining = remaining_quantity(buy_order);
    let sell_remaining = remaining_quantity(sell_order);
    let trade_qty = buy_remaining.min(sell_remaining);

    // update buy order
    buy_order.filled_quantity = buy_order
        .filled_quantity
        .checked_add(trade_qty)
        .ok_or("ArithmeticOverFlow")?;

    if buy_order.filled_quantity == buy_order.quantity {
        buy_order.status = OrderStatus::Filled;
    } else {
        buy_order.status = OrderStatus::PartiallyFilled;
    }

    //update sell order
    sell_order.filled_quantity = sell_order
        .filled_quantity
        .checked_add(trade_qty)
        .ok_or("ArithmeticOverFlow")?;

    if sell_order.filled_quantity == sell_order.quantity {
        sell_order.status = OrderStatus::Filled;
    } else {
        sell_order.status = OrderStatus::PartiallyFilled;
    }

    //Everything updated, now to emit the trades
    Ok(Trade {
        trade_id: 0, // placeholder
        buyer: buy_order.trader.clone(),
        seller: sell_order.trader.clone(),
        buy_order_id: buy_order.order_id,
        sell_order_id: sell_order.order_id,
        price: match_price,
        quantity: trade_qty,
    })
}

// now for cancellation
pub fn process_cancellations<T: Config>(
    order_ids: Vec<OrderId>,
    bid_book: &mut BTreeMap<Amount, Vec<OrderId>>,
    ask_book: &mut BTreeMap<Amount, Vec<OrderId>>,
    orders_map: &mut BTreeMap<OrderId, Order<T>>,
) -> Result<(), DispatchError> {
    for order_id in order_ids {
        if let Some(mut order) = orders_map.get(&order_id).cloned() {
            order.status = OrderStatus::Cancelled;

            remove_from_orderbook(order_id, &order, bid_book, ask_book);

            orders_map.insert(order_id, order.clone());
        }
    }
    Ok(())
}
