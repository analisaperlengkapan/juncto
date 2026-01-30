use axum::{
    extract::{Json, State, ws::{WebSocketUpgrade, WebSocket, Message}},
    response::IntoResponse,
    http::StatusCode,
};
use serde_json::json;
use shared::{RoomConfig, ChatMessage, ServerMessage, Participant};
use std::sync::Arc;
use crate::AppState;
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn create_room(Json(payload): Json<RoomConfig>) -> impl IntoResponse {
    let room_id = format!("room-{}", uuid::Uuid::new_v4());

    let response = json!({
        "room_id": room_id,
        "config": payload,
        "status": "created"
    });

    (StatusCode::CREATED, Json(response))
}

pub async fn chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // 1. Register new participant
    let my_id = uuid::Uuid::new_v4().to_string();
    let me = Participant {
        id: my_id.clone(),
        name: format!("User {}", &my_id[0..4]),
    };

    {
        let mut participants = state.participants.lock().unwrap();
        participants.insert(my_id.clone(), me.clone());
    }

    // 2. Broadcast Join Message
    let _ = state.tx.send(ServerMessage::ParticipantJoined(me.clone()));

    // 3. Send current participant list to the new user
    let current_list: Vec<Participant> = {
        let participants = state.participants.lock().unwrap();
        participants.values().cloned().collect()
    };
    if let Ok(json) = serde_json::to_string(&ServerMessage::ParticipantList(current_list)) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Subscribe to the broadcast channel
    let mut rx = state.tx.subscribe();

    // Spawn a task to send broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Serialize as JSON string
            if let Ok(json_msg) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json_msg)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Spawn a task to receive messages from this client and broadcast them
    let tx = state.tx.clone();
    let my_id_clone = my_id.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Deserialize message (Client sends ChatMessage directly, but we want to wrap it?)
            // Or client sends ChatMessage, and we wrap it in ServerMessage::Chat?
            // For now, let's assume client sends ChatMessage JSON.
            if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                // Broadcast to all connected clients as ServerMessage
                // Enforce the correct user_id
                let verified_msg = ChatMessage {
                    user_id: my_id_clone.clone(), // Use server-assigned ID
                    ..chat_msg
                };
                let _ = tx.send(ServerMessage::Chat(verified_msg));
            } else {
                eprintln!("Failed to parse message: {}", text);
            }
        }
    });

    // Wait for either task to finish (connection closed)
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Cleanup: Remove participant and broadcast Leave message
    {
        let mut participants = state.participants.lock().unwrap();
        participants.remove(&my_id);
    }
    let _ = state.tx.send(ServerMessage::ParticipantLeft(my_id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use axum::Router;
    use axum::routing::get;

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new().route("/health", get(health_check));

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
