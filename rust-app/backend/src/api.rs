use axum::{
    extract::{Json, State, ws::{WebSocketUpgrade, WebSocket, Message}},
    response::IntoResponse,
    http::StatusCode,
};
use serde_json::json;
use shared::{RoomConfig, ChatMessage, ServerMessage, Participant, ClientMessage};
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

    // Send initial room config immediately so Prejoin screen knows status
    let current_config: RoomConfig = {
        let config = state.room_config.lock().unwrap();
        config.clone()
    };
    if let Ok(json) = serde_json::to_string(&ServerMessage::RoomUpdated(current_config)) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();

    // Channel for internal messages to self
    let (internal_tx, mut internal_rx) = tokio::sync::mpsc::channel::<ServerMessage>(10);

    // Send loop
    let send_task = tokio::spawn(async move {
        loop {
            let msg = tokio::select! {
                Ok(msg) = rx.recv() => msg,
                Some(msg) = internal_rx.recv() => msg,
                else => break,
            };

            if let Ok(json_msg) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json_msg)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Receive loop
    let tx = state.tx.clone();
    let participants_mutex = state.participants.clone();
    let room_config_mutex = state.room_config.clone();

    // We don't have an ID yet
    let mut my_id: Option<String> = None;

    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
            match client_msg {
                ClientMessage::Join(name) => {
                    if my_id.is_some() { continue; } // Already joined

                    let id = uuid::Uuid::new_v4().to_string();
                    let me = Participant { id: id.clone(), name };

                    {
                        let mut participants = participants_mutex.lock().unwrap();
                        participants.insert(id.clone(), me.clone());
                    }
                    my_id = Some(id.clone());

                    // Broadcast Join
                    let _ = tx.send(ServerMessage::ParticipantJoined(me));

                    // Send current participants to self
                    let current_list: Vec<Participant> = {
                        let participants = participants_mutex.lock().unwrap();
                        participants.values().cloned().collect()
                    };
                    let _ = internal_tx.send(ServerMessage::ParticipantList(current_list)).await;
                },
                ClientMessage::Chat(content) => {
                    if let Some(uid) = &my_id {
                        let chat_msg = ChatMessage {
                            user_id: uid.clone(),
                            content,
                            timestamp: chrono::Utc::now().timestamp_millis() as u64,
                        };
                        let _ = tx.send(ServerMessage::Chat(chat_msg));
                    }
                },
                ClientMessage::ToggleRoomLock => {
                    if my_id.is_some() {
                        let new_config = {
                            let mut config = room_config_mutex.lock().unwrap();
                            config.is_locked = !config.is_locked;
                            config.clone()
                        };
                        let _ = tx.send(ServerMessage::RoomUpdated(new_config));
                    }
                },
                ClientMessage::ToggleRecording => {
                    if my_id.is_some() {
                        let new_config = {
                            let mut config = room_config_mutex.lock().unwrap();
                            config.is_recording = !config.is_recording;
                            config.clone()
                        };
                        let _ = tx.send(ServerMessage::RoomUpdated(new_config));
                    }
                },
                ClientMessage::Reaction(emoji) => {
                    if let Some(uid) = &my_id {
                        let _ = tx.send(ServerMessage::Reaction {
                            sender_id: uid.clone(),
                            emoji,
                        });
                    }
                },
                ClientMessage::UpdateProfile(new_name) => {
                    if let Some(uid) = &my_id {
                        let updated_participant = {
                            let mut participants = participants_mutex.lock().unwrap();
                            if let Some(p) = participants.get_mut(uid) {
                                p.name = new_name.clone();
                                Some(p.clone())
                            } else {
                                None
                            }
                        };

                        if let Some(p) = updated_participant {
                            let _ = tx.send(ServerMessage::ParticipantUpdated(p));
                        }
                    }
                }
            }
        }
    }

    send_task.abort();

    // Cleanup
    if let Some(id) = my_id {
        {
            let mut participants = participants_mutex.lock().unwrap();
            participants.remove(&id);
        }
        let _ = tx.send(ServerMessage::ParticipantLeft(id));
    }
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
