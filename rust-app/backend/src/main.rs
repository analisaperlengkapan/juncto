mod api;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;
use tokio::sync::{broadcast, oneshot};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use shared::{ServerMessage, Participant, RoomConfig, Poll, DrawAction};

type KnockingMap = HashMap<String, (Participant, oneshot::Sender<bool>)>;

// AppState to hold the broadcast channel and participants list
#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<ServerMessage>,
    pub participants: Arc<Mutex<HashMap<String, Participant>>>,
    pub knocking_participants: Arc<Mutex<KnockingMap>>,
    pub room_config: Arc<Mutex<RoomConfig>>,
    pub polls: Arc<Mutex<HashMap<String, Poll>>>,
    pub whiteboard: Arc<Mutex<Vec<DrawAction>>>,
}

#[tokio::main]
async fn main() {
    // Initialize broadcast channel (capacity 100)
    let (tx, _rx) = broadcast::channel(100);
    // Initialize participants list
    let participants = Arc::new(Mutex::new(HashMap::new()));
    // Initialize knocking participants list
    let knocking_participants = Arc::new(Mutex::new(HashMap::new()));
    // Initialize room config
    let room_config = Arc::new(Mutex::new(RoomConfig::default()));
    // Initialize polls
    let polls = Arc::new(Mutex::new(HashMap::new()));
    // Initialize whiteboard
    let whiteboard = Arc::new(Mutex::new(Vec::new()));

    let app_state = Arc::new(AppState { tx, participants, knocking_participants, room_config, polls, whiteboard });

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
