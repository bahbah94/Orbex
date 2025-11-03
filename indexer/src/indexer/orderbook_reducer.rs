use anyhow::Result;
use sqlx::PgPool;
use tracing_subscriber::filter::combinator::Or;
use std::collections::BTreeMap;
use tracing::info;

pub struct OrderbookState{
    pub bids: BTreeMap<u128,Vec<u64>>,
    pub asks: BTreeMap<u128, Vec<u64>>,
    pub orders: BTreeMap<u64, OrderInfo>
}

pub struct OrderInfo{
    pub order_id: u64,
    //pub trade: String,
    pub side: String,
    pub price: u128,
    pub quantity: u128,
    pub filled_quantity: u128,
    pub status: String,
}

impl OrderbookState{
    pub fn new() -> Self{
        Self { bids: BTreeMap::new(),
             asks: BTreeMap::new(),
            orders: BTreeMap::new(),
         }
    }

    pub fn add_order(&mut self, order:OrderInfo) {
        let order_id = order.order_id;
        let price = order.price;
        let side = order.side.as_str();

        match side {
            "Buy" => {
                self.bids.entry(price).or_insert_with(Vec::new).push(order_id);
            },
            "Sell" => {
                self.asks.entry(price).or_insert_with(Vec::new).push(order_id);
            }
            _ => {}
        }

        self.orders.insert(order_id, order);

        info!("Added order with order_id {}", order_id);
    }

    pub fn update_order(
        &mut self,
        order_id: u64,
        filled_quantity: u128,
        status: &str,
    ) -> Result<()> {
        let (side, price) = if let Some(order) = self.orders.get_mut(&order_id) {
            order.filled_quantity = filled_quantity;
            order.status = status.to_string();
            (order.side.clone(), order.price)
        } else {
            return Err(anyhow::anyhow!("Order #{} not found", order_id));  // ← Error!
        };
    
        if status == "Filled" {
            self.remove_order_from_level(order_id, &side, price);
        }

        Ok(())
    }

    pub fn remove_order_from_level(&mut self, order_id: u64, side: &str, price: u128){
        match side {
            "Buy" => {
                if let Some(orders) = self.bids.get_mut(&price){
                    orders.retain(|id| id != &order_id);
                    if orders.is_empty(){
                        self.bids.remove(&price);
                    }
                }
            },
            "Sell" => {
                if let Some(orders) = self.asks.get_mut(&price){
                    orders.retain(|id| id != &order_id);
                    if orders.is_empty(){
                        self.asks.remove(&price);
                    }
                }
            },
            _ => {}
        }

        
    }

    pub fn cancel_order(&mut self, order_id: u64) -> Result<()> {
        let (side, price) = if let Some(order) = self.orders.get_mut(&order_id) {
            order.status = "Cancelled".to_string();
            (order.side.clone(), order.price)
        } else {
            return Err(anyhow::anyhow!("Order #{} not found", order_id));
        };
    
        self.remove_order_from_level(order_id, &side, price);
        info!(" Order #{} cancelled", order_id);
    
        Ok(())
    }

    pub fn get_bid_depth(&self, depth: usize) -> Vec<(u128, usize)> {
        self.bids
            .iter()
            .rev() // Highest prices first for bids
            .take(depth)
            .map(|(price, orders)| (*price, orders.len()))
            .collect()
    }

    pub fn get_ask_depth(&self, depth: usize) -> Vec<(u128, usize)> {
        self.asks
            .iter()
            .take(depth) // Lowest prices first for asks
            .map(|(price, orders)| (*price, orders.len()))
            .collect()
    }

    /// Get best bid/ask spread
    pub fn get_spread(&self) -> Option<(u128, u128)> {
        let best_bid = self.bids.keys().rev().next()?;
        let best_ask = self.asks.keys().next()?;
        Some((*best_bid, *best_ask))
    }
}

pub async fn process_order_filled(
    state: &mut OrderbookState,
    order_id: u64
) -> Result<()> {
    if let Some(order) = state.orders.get(&order_id) {
        state.update_order(order_id, order.filled_quantity, "Filled")?;
        info!("OrderFilled: {}", order_id);
    }
    Ok(())
    
}

pub async fn process_order_cancelled(
    state: &mut OrderbookState,
    order_id: u64,
) -> Result<()> {
    state.cancel_order(order_id)?;
    info!("Order {} Cancelled", order_id);

    Ok(())
}

pub async fn process_order_partially_filled(
    state: &mut OrderbookState,
    _pool: &PgPool,
    order_id: u64,
    filled_quantity: u128,
) -> Result<()> {
    state.update_order(order_id, filled_quantity, "PartiallyFilled")?;
    info!(
        "📊 OrderPartiallyFilled: #{} (filled: {})",
        order_id, filled_quantity
    );

    Ok(())
}

pub async fn process_order_place(
    state: &mut OrderbookState,
    order_id: u64,
    side: &str,
    price: u128,
    quantity: u128
) -> Result<()> {
    let order = OrderInfo {
        order_id,
        side: side.to_string(),
        price,
        quantity,
        filled_quantity: 0,
        status: "Open".to_string(),
    };

    state.add_order(order);

    info!(
        " OrderPlaced: #{} {} {} @ {}",
        order_id, side, quantity, price
    );

    Ok(())
}

pub fn get_orderbook_snapshot(state: &OrderbookState) -> serde_json::Value {
    let bids: Vec<_> = state
        .bids
        .iter()
        .rev()
        .map(|(price, orders)| {
            let total_quantity: u128 = orders
                .iter()
                .filter_map(|id| state.orders.get(id).map(|o| o.quantity - o.filled_quantity))
                .sum();
            
            serde_json::json!({
                "price": price,
                "count": orders.len(),
                "total_quantity": total_quantity,
                "orders": orders.iter()
                    .filter_map(|id| state.orders.get(id))
                    .map(|o| serde_json::json!({
                        "order_id": o.order_id,
                        "quantity": o.quantity,
                        "filled_quantity": o.filled_quantity,
                        "remaining": o.quantity - o.filled_quantity,
                        "status": o.status,
                    }))
                    .collect::<Vec<_>>()
            })
        })
        .collect();

    let asks: Vec<_> = state
        .asks
        .iter()
        .map(|(price, orders)| {
            let total_quantity: u128 = orders
                .iter()
                .filter_map(|id| state.orders.get(id).map(|o| o.quantity - o.filled_quantity))
                .sum();
            
            serde_json::json!({
                "price": price,
                "count": orders.len(),
                "total_quantity": total_quantity,
                "orders": orders.iter()
                    .filter_map(|id| state.orders.get(id))
                    .map(|o| serde_json::json!({
                        "order_id": o.order_id,
                        "quantity": o.quantity,
                        "filled_quantity": o.filled_quantity,
                        "remaining": o.quantity - o.filled_quantity,
                        "status": o.status,
                    }))
                    .collect::<Vec<_>>()
            })
        })
        .collect();

    let (total_bid_volume, total_ask_volume): (u128, u128) = state
        .orders
        .values()
        .fold((0, 0), |(bids, asks), order| {
            let remaining = order.quantity.saturating_sub(order.filled_quantity);
            if order.side == "Buy" {
                (bids.saturating_add(remaining), asks)
            } else {
                (bids, asks.saturating_add(remaining))
            }
        });

    serde_json::json!({
        "bids": bids,
        "asks": asks,
        "spread": if let Some((bid, ask)) = state.get_spread() {
            serde_json::json!({ 
                "best_bid": bid, 
                "best_ask": ask,
                "spread": ask.saturating_sub(bid)
            })
        } else {
            serde_json::json!(null)
        },
        "summary": {
            "total_bid_levels": state.bids.len(),
            "total_ask_levels": state.asks.len(),
            "total_orders": state.orders.len(),
            "total_bid_volume": total_bid_volume,
            "total_ask_volume": total_ask_volume,
        }
    })
}