mod api;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use shared::{BreakoutRoom, DrawAction, Participant, Poll, RoomConfig, ServerMessage};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, oneshot};
use tower_http::services::{ServeDir, ServeFile};

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
    pub feedback: Arc<Mutex<Vec<shared::Feedback>>>,
    /// Active remote-control sessions: maps controller (requester) id ->
    /// controlled (target) id. Used by the handler to authorize subsequent
    /// `RemoteControlAction` and `StopRemoteControl` messages so a malicious
    /// client cannot inject actions without having been granted access.
    pub remote_control_sessions: Arc<Mutex<HashMap<String, String>>>,
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
    let feedback = Arc::new(Mutex::new(Vec::new()));
    let remote_control_sessions = Arc::new(Mutex::new(HashMap::new()));

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
        feedback,
        remote_control_sessions,
    });

    // Define the router
    let serve_dir =
        ServeDir::new("frontend/pkg").not_found_service(ServeFile::new("frontend/pkg/index.html"));
    let serve_static = ServeDir::new("backend/static");

    let app = Router::new()
        .nest_service("/static", serve_static)
        .route("/api/rooms", post(api::create_room))
        .route("/api/feedback", post(api::submit_feedback))
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
