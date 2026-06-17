use crate::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use shared::RoomConfig;
use std::sync::Arc;

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RoomConfig>,
) -> impl IntoResponse {
    // Mirror the validation enforced by the WebSocket `SetSubject` handler
    // so the REST endpoint cannot be used to bypass the 256-character limit.
    // Use `chars().count()` (not byte length) so multi-byte UTF-8 content
    // (CJK, emoji) is treated consistently with what users see.
    if let Some(ref s) = payload.subject {
        if s.chars().count() > 256 {
            let err = json!({ "error": "Invalid subject: too long" });
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    }

    // Normalize empty subject to `None` for defensive consistency with the
    // WebSocket handler.
    let mut payload = payload;
    payload.subject = payload.subject.filter(|s| !s.is_empty());

    {
        let mut config = state.room_config.lock().unwrap();
        *config = payload.clone();
    }

    // Clear state for new room (since we have a single global room instance)
    {
        let mut p = state.participants.lock().unwrap();
        p.clear();
    }
    {
        let mut k = state.knocking_participants.lock().unwrap();
        k.clear();
    }
    {
        let mut polls = state.polls.lock().unwrap();
        polls.clear();
    }
    {
        let mut wb = state.whiteboard.lock().unwrap();
        wb.clear();
    }
    {
        let mut ch = state.chat_history.lock().unwrap();
        ch.clear();
    }
    {
        let mut br = state.breakout_rooms.lock().unwrap();
        br.clear();
    }
    {
        let mut pl = state.participant_locations.lock().unwrap();
        pl.clear();
    }
    {
        let mut v = state.shared_video_url.lock().unwrap();
        *v = None;
    }
    {
        let mut s = state.speaking_start_times.lock().unwrap();
        s.clear();
    }
    {
        let mut rc = state.remote_control_sessions.lock().unwrap();
        rc.clear();
    }
    {
        let mut pending = state.pending_remote_control_requests.lock().unwrap();
        pending.clear();
    }
    {
        let mut fb = state.feedback.lock().unwrap();
        fb.clear();
    }

    let room_id = format!("room-{}", uuid::Uuid::new_v4());

    let response = json!({
        "room_id": room_id,
        "config": payload,
        "status": "created"
    });

    (StatusCode::CREATED, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::collections::HashMap;
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::post;
    use axum::Router;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_room() {
        use shared::RoomConfig;
        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        let app_state = Arc::new(AppState {
            tx,
            participants: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            knocking_participants: Arc::new(
                std::sync::Mutex::new(std::collections::HashMap::new()),
            ),
            room_config: Arc::new(std::sync::Mutex::new(RoomConfig::default())),
            polls: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            whiteboard: Arc::new(std::sync::Mutex::new(Vec::new())),
            chat_history: Arc::new(std::sync::Mutex::new(Vec::new())),
            breakout_rooms: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            participant_locations: Arc::new(
                std::sync::Mutex::new(std::collections::HashMap::new()),
            ),
            shared_video_url: Arc::new(std::sync::Mutex::new(None)),
            speaking_start_times: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            feedback: Arc::new(std::sync::Mutex::new(Vec::new())),
            remote_control_sessions: Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            pending_remote_control_requests: Arc::new(std::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            unmute_permissions: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            camera_permissions: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            feedback_timestamps: Arc::new(Mutex::new(HashMap::new())),
        });

        let config = RoomConfig {
            e2ee_enabled: true,
            ..Default::default()
        };
        // Mock app with just this route for testing
        let app = Router::new()
            .route("/api/rooms", post(create_room))
            .with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/rooms")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&config).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
