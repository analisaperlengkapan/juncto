mod api;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use shared::{ServerMessage, Participant};

// AppState to hold the broadcast channel and participants list
#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<ServerMessage>,
    pub participants: Arc<Mutex<HashMap<String, Participant>>>,
}

#[tokio::main]
async fn main() {
    // Initialize broadcast channel (capacity 100)
    let (tx, _rx) = broadcast::channel(100);
    // Initialize participants list
    let participants = Arc::new(Mutex::new(HashMap::new()));

    let app_state = Arc::new(AppState { tx, participants });

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
