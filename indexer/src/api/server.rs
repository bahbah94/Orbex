use axum::{Router, routing::get};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::PgPool;
use crate::indexer::order_mapper::OrderbookState;


pub async fn run_server(
    orderbook: Arc<Mutex<OrderbookState>>,
    pool: PgPool,
) -> Result<(),Box<dyn std::error::Error>> {

    let app_state = (orderbook, pool);
    let app = Router::new()
            //REST API endpoints
            .route(todo!("orderbook"))
            .route(todo!("order/:id"))

            //websocket stuff
            .route(todo!("/ws/orderbook"))

            //health stuff
            .route("/health", get(|| async { "OK" }))

            .layer(
                CorsLayer::new()
                .allow_origin(any)
                .allow_methods(any)
                .allow_header(any)
            )

            .with_state(app_state);

            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    
            println!(" API Server: http://0.0.0.0:3000");
            println!("WebSocket: ws://0.0.0.0:3000/ws/orderbook");
            println!("Orderbook: http://0.0.0.0:3000/api/orderbook");
            
            axum::serve(listener, app).await?;
        
        Ok(())
}