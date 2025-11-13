use crate::indexer::orderbook_reducer::OrderbookState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

// Type alias for our shared state
pub type AppState = (Arc<Mutex<OrderbookState>>, PgPool);

pub async fn get_orderbook(State((orderbook, _pool)): State<AppState>) -> impl IntoResponse {
    let ob = orderbook.lock().await;
    let snapshot = ob.get_snapshot();

    Json(snapshot)
}

pub async fn get_order(
    State((orderbook, _pool)): State<AppState>,
    Path(order_id): Path<u64>,
) -> impl IntoResponse {
    let ob = orderbook.lock().await;

    match ob.orders.get(&order_id) {
        Some(order) => (
            StatusCode::OK,
            Json(json!({
                "order_id": order.order_id,
                "side": order.side,
                "price": order.price,
                "quantity": order.quantity,
                "filled_quantity": order.filled_quantity,
                "remaining_quantity": order.quantity - order.filled_quantity,
                "status": order.status,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "irder Not found",
                "order_id": order_id,
            })),
        )
            .into_response(),
    }
}
