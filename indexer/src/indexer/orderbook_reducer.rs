use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::BTreeMap;
use tokio::sync::broadcast;
use tracing::info;

/// Price level in orderbook snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: u128,
    pub total_quantity: u128,
    pub order_count: usize,
}

/// Spread information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spread {
    pub best_bid: u128,
    pub best_ask: u128,
    pub spread: u128,
}

/// Summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSummary {
    pub total_bid_levels: usize,
    pub total_ask_levels: usize,
    pub total_orders: usize,
    pub total_bid_volume: u128,
    pub total_ask_volume: u128,
}

/// Complete orderbook snapshot - sent over broadcast channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSnapshot {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub spread: Option<Spread>,
    pub summary: OrderbookSummary,
}

#[derive(Debug)]
pub struct OrderbookState {
    pub bids: BTreeMap<u128, Vec<u64>>,
    pub asks: BTreeMap<u128, Vec<u64>>,
    pub orders: BTreeMap<u64, OrderInfo>,
    /// Optional broadcast channel for push-based snapshot updates
    broadcast_tx: Option<broadcast::Sender<OrderbookSnapshot>>,
}

#[derive(Debug)]
pub struct OrderInfo {
    pub order_id: u64,
    //pub trade: String,
    pub side: String,
    pub price: u128,
    pub quantity: u128,
    pub filled_quantity: u128,
    pub status: String,
}

impl OrderbookState {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: BTreeMap::new(),
            broadcast_tx: None,
        }
    }

    /// Create a new OrderbookState with broadcast channel for push-based updates
    pub fn with_broadcast(broadcast_tx: broadcast::Sender<OrderbookSnapshot>) -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: BTreeMap::new(),
            broadcast_tx: Some(broadcast_tx),
        }
    }

    /// Notify subscribers of orderbook change by sending full snapshot
    fn notify(&self) {
        if let Some(ref tx) = self.broadcast_tx {
            let snapshot = self.get_snapshot();
            tracing::debug!(
                "Broadcasting orderbook snapshot: {} bid levels, {} ask levels, {} orders",
                snapshot.summary.total_bid_levels,
                snapshot.summary.total_ask_levels,
                snapshot.summary.total_orders
            );
            if tx.send(snapshot).is_err() {
                tracing::debug!("No subscribers for orderbook updates");
            }
        }
    }

    /// Generate orderbook snapshot from current state
    pub fn get_snapshot(&self) -> OrderbookSnapshot {
        let bids: Vec<PriceLevel> = self
            .bids
            .iter()
            .rev()
            .map(|(price, orders)| {
                let total_quantity: u128 = orders
                    .iter()
                    .filter_map(|id| self.orders.get(id).map(|o| o.quantity - o.filled_quantity))
                    .sum();

                PriceLevel {
                    price: *price,
                    total_quantity,
                    order_count: orders.len(),
                }
            })
            .collect();

        let asks: Vec<PriceLevel> = self
            .asks
            .iter()
            .map(|(price, orders)| {
                let total_quantity: u128 = orders
                    .iter()
                    .filter_map(|id| self.orders.get(id).map(|o| o.quantity - o.filled_quantity))
                    .sum();

                PriceLevel {
                    price: *price,
                    total_quantity,
                    order_count: orders.len(),
                }
            })
            .collect();

        let (total_bid_volume, total_ask_volume): (u128, u128) =
            self.orders.values().fold((0, 0), |(bids, asks), order| {
                let remaining = order.quantity.saturating_sub(order.filled_quantity);
                if order.side == "Buy" {
                    (bids.saturating_add(remaining), asks)
                } else {
                    (bids, asks.saturating_add(remaining))
                }
            });

        let spread = self.get_spread().map(|(best_bid, best_ask)| Spread {
            best_bid,
            best_ask,
            spread: best_ask.saturating_sub(best_bid),
        });

        OrderbookSnapshot {
            bids,
            asks,
            spread,
            summary: OrderbookSummary {
                total_bid_levels: self.bids.len(),
                total_ask_levels: self.asks.len(),
                total_orders: self.orders.len(),
                total_bid_volume,
                total_ask_volume,
            },
        }
    }

    pub fn add_order(&mut self, order: OrderInfo) {
        let order_id = order.order_id;
        let price = order.price;
        let side = order.side.as_str();

        match side {
            "Buy" => {
                self.bids
                    .entry(price)
                    .or_default()
                    .push(order_id);
            }
            "Sell" => {
                self.asks
                    .entry(price)
                    .or_default()
                    .push(order_id);
            }
            _ => {}
        }

        self.orders.insert(order_id, order);

        info!("Added order with order_id {}", order_id);
        self.notify();
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
            return Err(anyhow::anyhow!("Order #{} not found", order_id)); // ← Error!
        };

        if status == "Filled" {
            self.remove_order_from_level(order_id, &side, price);
        }

        self.notify();
        Ok(())
    }

    pub fn remove_order_from_level(&mut self, order_id: u64, side: &str, price: u128) {
        match side {
            "Buy" => {
                if let Some(orders) = self.bids.get_mut(&price) {
                    orders.retain(|id| id != &order_id);
                    if orders.is_empty() {
                        self.bids.remove(&price);
                    }
                }
            }
            "Sell" => {
                if let Some(orders) = self.asks.get_mut(&price) {
                    orders.retain(|id| id != &order_id);
                    if orders.is_empty() {
                        self.asks.remove(&price);
                    }
                }
            }
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
        self.notify();

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
        let best_bid = self.bids.keys().next_back()?;
        let best_ask = self.asks.keys().next()?;
        Some((*best_bid, *best_ask))
    }
}

pub async fn process_order_filled(state: &mut OrderbookState, order_id: u64) -> Result<()> {
    if let Some(order) = state.orders.get(&order_id) {
        state.update_order(order_id, order.filled_quantity, "Filled")?;
        info!("OrderFilled: {}", order_id);
    }
    Ok(())
}

pub async fn process_order_cancelled(state: &mut OrderbookState, order_id: u64) -> Result<()> {
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
    quantity: u128,
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

