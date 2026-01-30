mod api;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;
use tokio::sync::broadcast;
use std::sync::Arc;
use shared::ChatMessage;

// AppState to hold the broadcast channel
#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<ChatMessage>,
}

#[tokio::main]
async fn main() {
    // Initialize broadcast channel (capacity 100)
    let (tx, _rx) = broadcast::channel(100);
    let app_state = Arc::new(AppState { tx });

    // Define the router
    let serve_dir = ServeDir::new("frontend/pkg")
        .not_found_service(ServeFile::new("frontend/pkg/index.html"));

    let app = Router::new()
        .route("/api/rooms", post(api::create_room))
        .route("/health", get(api::health_check))
        .route("/ws/chat", get(api::chat_handler))
        .fallback_service(serve_dir)
        .with_state(app_state);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
