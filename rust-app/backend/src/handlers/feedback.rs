use crate::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use shared::Feedback;
use std::sync::Arc;

pub async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Feedback>,
) -> impl IntoResponse {
    // Validate stars (Bug 1)
    if payload.stars < 1 || payload.stars > 5 {
        return (StatusCode::BAD_REQUEST, "Stars must be between 1 and 5").into_response();
    }

    let mut feedback_store = state.feedback.lock().unwrap();

    // Bound storage (Bug 2)
    if feedback_store.len() >= 1000 {
        feedback_store.remove(0);
    }

    feedback_store.push(payload);


    // TODO: Add rate limiting and authentication for production (Bug 3)
    (StatusCode::OK, "Feedback received").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::post;
    use axum::Router;
    use shared::RoomConfig;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_submit_feedback() {
        let (tx, _rx) = tokio::sync::broadcast::channel(10);
        let app_state = Arc::new(AppState {
            tx,
            participants: Arc::new(Mutex::new(HashMap::new())),
            knocking_participants: Arc::new(Mutex::new(HashMap::new())),
            room_config: Arc::new(Mutex::new(RoomConfig::default())),
            polls: Arc::new(Mutex::new(HashMap::new())),
            whiteboard: Arc::new(Mutex::new(Vec::new())),
            chat_history: Arc::new(Mutex::new(Vec::new())),
            breakout_rooms: Arc::new(Mutex::new(HashMap::new())),
            participant_locations: Arc::new(Mutex::new(HashMap::new())),
            shared_video_url: Arc::new(Mutex::new(None)),
            speaking_start_times: Arc::new(Mutex::new(HashMap::new())),
            feedback: Arc::new(Mutex::new(Vec::new())),
            remote_control_sessions: Arc::new(Mutex::new(HashMap::new())),
            pending_remote_control_requests: Arc::new(Mutex::new(std::collections::HashSet::new())),
        });

        let app = Router::new()
            .route("/api/feedback", post(submit_feedback))
            .with_state(app_state.clone());

        let feedback = Feedback {
            stars: 5,
            comment: "Great app!".to_string(),
            user_id: Some("user123".to_string()),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/feedback")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&feedback).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let stored = app_state.feedback.lock().unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].stars, 5);
    }

    #[tokio::test]
    async fn test_submit_feedback_invalid_stars() {
        let (tx, _rx) = tokio::sync::broadcast::channel(10);
        let app_state = Arc::new(AppState {
            tx,
            participants: Arc::new(Mutex::new(HashMap::new())),
            knocking_participants: Arc::new(Mutex::new(HashMap::new())),
            room_config: Arc::new(Mutex::new(RoomConfig::default())),
            polls: Arc::new(Mutex::new(HashMap::new())),
            whiteboard: Arc::new(Mutex::new(Vec::new())),
            chat_history: Arc::new(Mutex::new(Vec::new())),
            breakout_rooms: Arc::new(Mutex::new(HashMap::new())),
            participant_locations: Arc::new(Mutex::new(HashMap::new())),
            shared_video_url: Arc::new(Mutex::new(None)),
            speaking_start_times: Arc::new(Mutex::new(HashMap::new())),
            feedback: Arc::new(Mutex::new(Vec::new())),
            remote_control_sessions: Arc::new(Mutex::new(HashMap::new())),
            pending_remote_control_requests: Arc::new(Mutex::new(std::collections::HashSet::new())),
        });

        let app = Router::new()
            .route("/api/feedback", post(submit_feedback))
            .with_state(app_state.clone());

        let feedback = Feedback {
            stars: 6,
            comment: "Too good!".to_string(),
            user_id: None,
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/feedback")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&feedback).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let stored = app_state.feedback.lock().unwrap();
        assert!(stored.is_empty());
    }
}
