use axum::{Router, routing::get};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use sqlx::PgPool;
use crate::indexer::orderbook_reducer::OrderbookState;
use crate::indexer::candle_aggregator::CandleUpdate;
use crate::api::{handlers, websocket};

pub async fn run_server(
    orderbook: Arc<Mutex<OrderbookState>>,
    pool: PgPool,
    candle_broadcast: broadcast::Sender<CandleUpdate>,
) -> Result<(),Box<dyn std::error::Error>> {

    let app_state = (orderbook, pool);

    // Create OHLCV router with its own state
    let ohlcv_router = Router::new()
        .route("/ws/ohlcv", get(websocket::ws_ohlcv::ws_ohlcv_handler))
        .with_state(candle_broadcast);

    let app = Router::new()
            //REST API endpoints
            .route("/api/orderbook", get(handlers::orderbook_hand::get_orderbook))
            .route("/api/order/:id", get(handlers::orderbook_hand::get_order))

            //add the udf stuff
            .nest("/udf", handlers::udf::udf_routes().await)

            //websocket stuff
            .route("/ws/orderbook", get(websocket::ws_orderbook::ws_handler))

            //health stuff
            .route("/health", get(|| async { "OK" })) // should be modified not sure

            .with_state(app_state)
            // Merge OHLCV router with separate state
            .merge(ohlcv_router)

            .layer(
                CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
            );

            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

            println!("🌐 API Server: http://0.0.0.0:3000");
            println!("📊 WebSocket Orderbook: ws://0.0.0.0:3000/ws/orderbook");
            println!("📈 WebSocket OHLCV: ws://0.0.0.0:3000/ws/ohlcv");
            println!("📖 Orderbook API: http://0.0.0.0:3000/api/orderbook");

            axum::serve(listener, app).await?;

        Ok(())
}