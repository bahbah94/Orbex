use crate::api::{handlers, websocket};
use crate::indexer::candle_aggregator::CandleUpdate;
use crate::indexer::orderbook_reducer::{OrderbookSnapshot, OrderbookState};
use axum::{routing::get, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::{Any, CorsLayer};

pub async fn run_server(
    orderbook: Arc<Mutex<OrderbookState>>,
    pool: PgPool,
    ob_broadcast: broadcast::Sender<OrderbookSnapshot>,
    candle_broadcast: broadcast::Sender<CandleUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = (orderbook.clone(), pool);

    // Create unified websocket router with its own state
    let unified_ws_state = (
        orderbook.clone(),
        ob_broadcast.clone(),
        candle_broadcast.clone(),
    );
    let unified_router = Router::new()
        .route("/ws/market", get(websocket::ws_unified::ws_unified_handler))
        .with_state(unified_ws_state);

    let app = Router::new()
        //REST API endpoints
        .route(
            "/api/orderbook",
            get(handlers::orderbook_hand::get_orderbook),
        )
        .route("/api/candles", get(handlers::ohlcv_hand::get_candles))
        // .route("/api/order/:id", get(handlers::orderbook_hand::get_order))
        //add the udf stuff
        .nest("/udf", handlers::udf::udf_routes().await)
        //health stuff
        .route("/health", get(|| async { "OK" }))
        .with_state(app_state)
        // Merge unified websocket router
        .merge(unified_router)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("🌐 API Server: http://0.0.0.0:3000");
    println!("🔥 WebSocket (orderbook + OHLCV): ws://0.0.0.0:3000/ws/market");
    println!("📖 REST API:");
    println!("   - Orderbook: http://0.0.0.0:3000/api/orderbook");
    println!("   - Candles: http://0.0.0.0:3000/api/candles");
    println!("   - UDF: http://0.0.0.0:3000/udf/");

    axum::serve(listener, app).await?;

    Ok(())
}
