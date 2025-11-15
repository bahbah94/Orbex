use crate::api::{handlers, websocket};
use crate::indexer::candle_aggregator::CandleUpdate;
use crate::indexer::orderbook_reducer::{OrderbookSnapshot, OrderbookState};
use axum::{routing::get, Router};
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

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
        .nest(
            "/api/orderbook",
            handlers::orderbook_hand::orderbook_routes().await,
        )
        .route("/api/candles", get(handlers::ohlcv_hand::get_candles))
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

    let port = env::var("INDEXER_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;

    info!("🌐 API Server: http://0.0.0.0:{}", port);
    info!(
        "🔥 WebSocket (orderbook + OHLCV): ws://0.0.0.0:{}/ws/market",
        port
    );
    info!("📖 REST API:");
    info!("   - Orderbook: http://0.0.0.0:{}/api/orderbook", port);
    info!("   - Candles: http://0.0.0.0:{}/api/candles", port);
    info!("   - UDF: http://0.0.0.0:{}/udf/", port);

    axum::serve(listener, app).await?;

    Ok(())
}
