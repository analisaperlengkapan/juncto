mod api;
mod db;

#[cfg(test)]
mod tests;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;
use tokio::sync::{broadcast, oneshot};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use shared::{ServerMessage, Participant, RoomConfig, Poll, DrawAction, BreakoutRoom};

type KnockingMap = HashMap<String, (Participant, Option<oneshot::Sender<bool>>)>;

// AppState to hold the broadcast channel and participants list
#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<ServerMessage>,
    pub participants: Arc<Mutex<HashMap<String, Participant>>>,
    pub knocking_participants: Arc<Mutex<KnockingMap>>,
    pub room_config: Arc<Mutex<RoomConfig>>,
    pub polls: Arc<Mutex<HashMap<String, Poll>>>,
    pub whiteboard: Arc<Mutex<Vec<DrawAction>>>,
    pub chat_history: Arc<Mutex<Vec<shared::ChatMessage>>>,
    pub breakout_rooms: Arc<Mutex<HashMap<String, BreakoutRoom>>>,
    // Track participants' current room: participant_id -> room_id (None = Main)
    pub participant_locations: Arc<Mutex<HashMap<String, Option<String>>>>,
    pub shared_video_url: Arc<Mutex<Option<String>>>,
    pub speaking_start_times: Arc<Mutex<HashMap<String, u64>>>,
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
    // Initialize chat history
    let chat_history = Arc::new(Mutex::new(Vec::new()));
    // Initialize breakout rooms
    let breakout_rooms = Arc::new(Mutex::new(HashMap::new()));
    let participant_locations = Arc::new(Mutex::new(HashMap::new()));
    let shared_video_url = Arc::new(Mutex::new(None));
    let speaking_start_times = Arc::new(Mutex::new(HashMap::new()));

    let app_state = Arc::new(AppState {
        tx,
        participants,
        knocking_participants,
        room_config,
        polls,
        whiteboard,
        chat_history,
        breakout_rooms,
        participant_locations,
        shared_video_url,
        speaking_start_times,
    });

    // Define the router
    let app = create_router(app_state);

    // Run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let serve_dir = ServeDir::new("frontend/pkg")
        .not_found_service(ServeFile::new("frontend/pkg/index.html"));

    Router::new()
        .route("/api/health", get(api::health_check))
        .route("/api/rooms", post(api::create_room))
        .route("/api/upload", post(api::upload_handler))
        .route("/api/feedback", post(api::feedback_handler))
        .route("/ws/chat", axum::routing::any(api::chat_handler))
        .nest_service("/uploads", tower_http::services::ServeDir::new("uploads"))
        .fallback_service(serve_dir)
        .with_state(app_state)
}
