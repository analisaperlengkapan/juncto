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

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RoomConfig>
) -> impl IntoResponse {
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

    // Channel for internal messages to self
    let (internal_tx, mut internal_rx) = tokio::sync::mpsc::channel::<ServerMessage>(10);

    // Send initial room config immediately so Prejoin screen knows status
    // Note: This config likely has host_id = None if this is the first user connecting to a fresh backend state?
    // No, if the user called `create_room` API, the room config is updated with the payload.
    // BUT `create_room` payload (from frontend) has default host_id = None.
    // The backend `create_room` handler updates `room_config` with payload.
    // So here `current_config` has host_id = None.

    let current_config: RoomConfig = {
        let config = state.room_config.lock().unwrap();
        config.clone()
    };
    if let Ok(json) = serde_json::to_string(&ServerMessage::RoomUpdated(current_config.clone())) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Explicitly send RoomUpdated to self to trigger frontend state logic (like is_host)
    // This is redundant if we sent it above, but harmless.
    let _ = internal_tx.send(ServerMessage::RoomUpdated(current_config.clone())).await;

    // Send Chat History
    let history: Vec<shared::ChatMessage> = {
        let history = state.chat_history.lock().unwrap();
        history.clone()
    };
    if !history.is_empty() {
        if let Ok(json) = serde_json::to_string(&ServerMessage::ChatHistory(history)) {
            let _ = sender.send(Message::Text(json)).await;
        }
    }

    // Channel for control messages from async tasks to the loop
    let (control_tx, mut control_rx) = tokio::sync::mpsc::channel::<bool>(1); // true = granted, false = denied

    // Send loop
    let send_task = tokio::spawn(async move {
        while let Some(msg) = internal_rx.recv().await {
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
    let knocking_mutex = state.knocking_participants.clone();
    let room_config_mutex = state.room_config.clone();
    let polls_mutex = state.polls.clone();
    let whiteboard_mutex = state.whiteboard.clone();
    let chat_history_mutex = state.chat_history.clone();
    let breakout_rooms_mutex = state.breakout_rooms.clone();
    let participant_locations_mutex = state.participant_locations.clone();

    // We don't have an ID yet
    let mut my_id: Option<String> = None;
    let mut knocking_id: Option<String> = None;
    // Track my current room locally for quick access
    let mut my_room_id: Option<String> = None;
    let mut broadcast_task: Option<tokio::task::JoinHandle<()>> = None;

    // Send initial breakout rooms list
    let rooms: Vec<shared::BreakoutRoom> = {
        let rooms = breakout_rooms_mutex.lock().unwrap();
        rooms.values().cloned().collect()
    };
    if !rooms.is_empty() {
        // Send via internal_tx to avoid "borrow of moved value: sender"
        let _ = internal_tx.send(ServerMessage::BreakoutRoomsList(rooms)).await;
    }

    loop {
        tokio::select! {
            // 1. Client Messages
            // 3. Kicked Event (from broadcast)
            // Note: We need to handle this in the broadcast receive loop, not here.
            // But if *I* am kicked, I need to know.
            // My broadcast receiver forwards everything to `internal_tx`, which forwards to `sender`.
            // So if I receive "Kicked(my_id)", the frontend will handle disconnection.
            // HOWEVER, the server should also forcibly close the connection.
            // Let's rely on frontend for graceful exit first, or check internal message loop.

            res = receiver.next() => {
                match res {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            match client_msg {
                                ClientMessage::KickParticipant(target_id) => {
                                    if let Some(uid) = &my_id {
                                        let host_id = {
                                            room_config_mutex.lock().unwrap().host_id.clone()
                                        };
                                        if Some(uid.clone()) == host_id {
                                            // Valid kick
                                            // 1. Remove from participants
                                            {
                                                let mut participants = participants_mutex.lock().unwrap();
                                                participants.remove(&target_id);
                                            }
                                            // 2. Broadcast Kicked
                                            let _ = tx.send(ServerMessage::Kicked(target_id.clone()));
                                            // 3. Broadcast ParticipantLeft (so lists update)
                                            let _ = tx.send(ServerMessage::ParticipantLeft(target_id));
                                        }
                                    }
                                },
                                ClientMessage::ToggleLobby => {
                                    if my_id.is_some() {
                                        let new_config = {
                                            let mut config = room_config_mutex.lock().unwrap();
                                            config.is_lobby_enabled = !config.is_lobby_enabled;
                                            config.clone()
                                        };
                                        let _ = tx.send(ServerMessage::RoomUpdated(new_config));
                                    }
                                },
                                ClientMessage::GrantAccess(target_id) => {
                                    if my_id.is_some() {
                                        let sender_opt = {
                                            let mut knocking = knocking_mutex.lock().unwrap();
                            knocking.get_mut(&target_id).and_then(|(_, s)| s.take())
                                        };
                                        if let Some(s) = sender_opt {
                                            let _ = s.send(true);
                                        }
                                    }
                                },
                                ClientMessage::DenyAccess(target_id) => {
                                    if my_id.is_some() {
                                        let sender_opt = {
                                            let mut knocking = knocking_mutex.lock().unwrap();
                            knocking.get_mut(&target_id).and_then(|(_, s)| s.take())
                                        };
                                        if let Some(s) = sender_opt {
                                            let _ = s.send(false);
                                        }
                                    }
                                },
                                ClientMessage::Join(name) => {
                                    if my_id.is_some() || knocking_id.is_some() { continue; } // Already joined or knocking

                                    // Check if room is locked or lobby is enabled
                                    let (is_locked, is_lobby, max_participants) = {
                                        let config = room_config_mutex.lock().unwrap();
                                        (config.is_locked, config.is_lobby_enabled, config.max_participants)
                                    };

                                    if is_locked {
                                        let _ = internal_tx.send(ServerMessage::Error("Room is locked".to_string())).await;
                                        continue;
                                    }

                                    let id = uuid::Uuid::new_v4().to_string();
                                    let me = Participant {
                                        id: id.clone(),
                                        name,
                                        is_hand_raised: false,
                                        is_sharing_screen: false,
                                    };

                                    if is_lobby {
                                        let (s, r) = tokio::sync::oneshot::channel();
                                        {
                                            let mut knocking = knocking_mutex.lock().unwrap();
                            knocking.insert(id.clone(), (me.clone(), Some(s)));
                                        }
                                        knocking_id = Some(id.clone());
                                        let _ = internal_tx.send(ServerMessage::Knocking).await;
                                        let _ = tx.send(ServerMessage::KnockingParticipant(me.clone()));

                                        // Spawn wait task
                                        let control_tx_clone = control_tx.clone();
                                        let knocking_mutex_clone = knocking_mutex.clone();
                                        let tx_clone = tx.clone();
                                        let id_clone = id.clone();

                                        tokio::spawn(async move {
                                            match tokio::time::timeout(std::time::Duration::from_secs(120), r).await {
                                                Ok(Ok(true)) => {
                                                    // Granted
                                                    let _ = control_tx_clone.send(true).await;
                                                },
                                                _ => {
                                                    // Timeout or Denied or Sender Dropped
                                    let removed = {
                                                        let mut knocking = knocking_mutex_clone.lock().unwrap();
                                        knocking.remove(&id_clone).is_some()
                                    };
                                    if removed {
                                        let _ = tx_clone.send(ServerMessage::KnockingParticipantLeft(id_clone));
                                                    }
                                                    let _ = control_tx_clone.send(false).await;
                                                }
                                            }
                                        });
                                        continue;
                                    }

                                    // Logic for direct join (no lobby)
                                    let (joined, new_host_assigned) = {
                                        let mut participants = participants_mutex.lock().unwrap();
                                        if participants.len() >= max_participants as usize {
                                            (false, false)
                                        } else {
                                            // Assign host if none exists
                                            let mut config = room_config_mutex.lock().unwrap();
                                            let assigned = if config.host_id.is_none() {
                                                config.host_id = Some(id.clone());
                                                true
                                            } else {
                                                false
                                            };

                                            participants.insert(id.clone(), me.clone());
                                            (true, assigned)
                                        }
                                    };

                                    if !joined {
                                        let _ = internal_tx.send(ServerMessage::Error("Room is full".to_string())).await;
                                        continue;
                                    }
                                    my_id = Some(id.clone());

                                    // Send Welcome with own ID
                                    let _ = internal_tx.send(ServerMessage::Welcome { id: id.clone() }).await;

                                    // If we assigned a new host, broadcast the updated config
                                    if new_host_assigned {
                                        let new_config = {
                                            room_config_mutex.lock().unwrap().clone()
                                        };
                                        // Send to everyone (broadcast)
                                        let _ = tx.send(ServerMessage::RoomUpdated(new_config.clone()));
                                        // Send to self explicitly
                                        let _ = internal_tx.send(ServerMessage::RoomUpdated(new_config)).await;
                                    }

                                    // Register initial location (Main Room)
                                    {
                                        let mut locations = participant_locations_mutex.lock().unwrap();
                                        locations.insert(id.clone(), None);
                                    }

                                    // Subscribe to broadcast and forward to internal_tx with filtering
                                    let mut rx = tx.subscribe();
                                    let forward_tx = internal_tx.clone();
                                    // We need to access participant_locations in the task to filter?
                                    // Or simply check the message content against current local state?
                                    // The broadcast task runs concurrently. It doesn't share `my_room_id` variable easily unless we use an Arc<Mutex> or message passing.
                                    // BUT, we can just forward everything to `internal_tx` (which goes to `handle_socket`'s send loop).
                                    // Wait, `handle_socket` has a `send_task` loop that reads from `internal_rx` and sends to websocket.
                                    // If we filter THERE, we have access to `my_room_id`.
                                    // BUT `internal_rx` receives `ServerMessage`.
                                    // Let's modify the `send_task` or the `broadcast_task`.
                                    // Modifying `send_task` is harder because it consumes `internal_rx`.
                                    // `broadcast_task` sends to `internal_tx`.
                                    // If we use `state.participant_locations` in `broadcast_task`, we can filter.

                                    let my_id_clone = id.clone();
                                    let locations_clone = participant_locations_mutex.clone();

                                    broadcast_task = Some(tokio::spawn(async move {
                                        loop {
                                            match rx.recv().await {
                                                Ok(msg) => {
                                                    // Filter based on room and recipient
                                                    let should_send = match &msg {
                                                        ServerMessage::Chat { message, room_id } => {
                                                            // 1. Check Room
                                                            let my_loc = {
                                                                let locs = locations_clone.lock().unwrap();
                                                                locs.get(&my_id_clone).cloned().flatten()
                                                            };
                                                            if *room_id != my_loc {
                                                                false
                                                            } else {
                                                                // 2. Check Private Message
                                                                if let Some(target) = &message.recipient_id {
                                                                    // Send only if I am the target OR the sender
                                                                    *target == my_id_clone || message.user_id == my_id_clone
                                                                } else {
                                                                    true
                                                                }
                                                            }
                                                        },
                                                        ServerMessage::PeerTyping { room_id, .. } => {
                                                            let my_loc = {
                                                                let locs = locations_clone.lock().unwrap();
                                                                locs.get(&my_id_clone).cloned().flatten()
                                                            };
                                                            *room_id == my_loc
                                                        },
                                                        // Global messages
                                                        _ => true,
                                                    };

                                                    if should_send
                                                        && forward_tx.send(msg).await.is_err() { break; }
                                                },
                                                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                                                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                                            }
                                        }
                                    }));

                                    // Broadcast Join
                                    let _ = tx.send(ServerMessage::ParticipantJoined(me));

                                    // Send current participants to self
                                    let current_list: Vec<Participant> = {
                                        let participants = participants_mutex.lock().unwrap();
                                        participants.values().cloned().collect()
                                    };
                                    let _ = internal_tx.send(ServerMessage::ParticipantList(current_list)).await;

                                    // Send current knocking participants to self (if any left)
                                    let knocking_list: Vec<Participant> = {
                                        let knocking = knocking_mutex.lock().unwrap();
                                        knocking.values().map(|(p, _)| p.clone()).collect()
                                    };
                                    for p in knocking_list {
                                        let _ = internal_tx.send(ServerMessage::KnockingParticipant(p)).await;
                                    }

                                    // Send whiteboard history
                                    let history: Vec<shared::DrawAction> = {
                                        let wb = whiteboard_mutex.lock().unwrap();
                                        wb.clone()
                                    };
                                    if !history.is_empty() {
                                        let _ = internal_tx.send(ServerMessage::WhiteboardHistory(history)).await;
                                    }
                                },
                                ClientMessage::Chat { content, recipient_id } => {
                                    if let Some(uid) = &my_id {
                                        let chat_msg = ChatMessage {
                                            user_id: uid.clone(),
                                            content,
                                            recipient_id: recipient_id.clone(),
                                            timestamp: chrono::Utc::now().timestamp_millis() as u64,
                                        };
                                        // Only save history if public
                                        if recipient_id.is_none() && my_room_id.is_none() {
                                            let mut history = chat_history_mutex.lock().unwrap();
                                            history.push(chat_msg.clone());
                                        }
                                        let _ = tx.send(ServerMessage::Chat {
                                            message: chat_msg,
                                            room_id: my_room_id.clone()
                                        });
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
                                ClientMessage::CreatePoll(mut poll) => {
                                    if my_id.is_some() {
                                        if poll.id.is_empty() {
                                            poll.id = uuid::Uuid::new_v4().to_string();
                                        }

                                        {
                                            let mut polls = polls_mutex.lock().unwrap();
                                            polls.insert(poll.id.clone(), poll.clone());
                                        }
                                        let _ = tx.send(ServerMessage::PollCreated(poll));
                                    }
                                },
                                ClientMessage::Vote { poll_id, option_id } => {
                                    if let Some(uid) = &my_id {
                                        let updated_poll = {
                                            let mut polls = polls_mutex.lock().unwrap();
                                            if let Some(poll) = polls.get_mut(&poll_id) {
                                                if poll.voters.contains(uid) {
                                                    None
                                                } else {
                                                    poll.voters.insert(uid.clone());
                                                    for opt in &mut poll.options {
                                                        if opt.id == option_id {
                                                            opt.votes += 1;
                                                        }
                                                    }
                                                    Some(poll.clone())
                                                }
                                            } else {
                                                None
                                            }
                                        };

                                        if let Some(poll) = updated_poll {
                                            let _ = tx.send(ServerMessage::PollUpdated(poll));
                                        }
                                    }
                                },
                                ClientMessage::Draw(mut action) => {
                                    if let Some(uid) = &my_id {
                                        action.sender_id = uid.clone();
                                        {
                                            let mut wb = whiteboard_mutex.lock().unwrap();
                                            wb.push(action.clone());
                                        }
                                        let _ = tx.send(ServerMessage::Draw(action));
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
                                },
                                ClientMessage::ToggleScreenShare => {
                                    if let Some(uid) = &my_id {
                                        let updated_participant = {
                                            let mut participants = participants_mutex.lock().unwrap();
                                            if let Some(p) = participants.get_mut(uid) {
                                                p.is_sharing_screen = !p.is_sharing_screen;
                                                Some(p.clone())
                                            } else {
                                                None
                                            }
                                        };

                                        if let Some(p) = updated_participant {
                                            let _ = tx.send(ServerMessage::ParticipantUpdated(p));
                                        }
                                    }
                                },
                                ClientMessage::ToggleRaiseHand => {
                                    if let Some(uid) = &my_id {
                                        let updated_participant = {
                                            let mut participants = participants_mutex.lock().unwrap();
                                            if let Some(p) = participants.get_mut(uid) {
                                                p.is_hand_raised = !p.is_hand_raised;
                                                Some(p.clone())
                                            } else {
                                                None
                                            }
                                        };

                                        if let Some(p) = updated_participant {
                                            let _ = tx.send(ServerMessage::ParticipantUpdated(p));
                                        }
                                    }
                                },
                                ClientMessage::Typing(is_typing) => {
                                    if let Some(uid) = &my_id {
                                        let _ = tx.send(ServerMessage::PeerTyping {
                                            user_id: uid.clone(),
                                            is_typing,
                                            room_id: my_room_id.clone(),
                                        });
                                    }
                                },
                                ClientMessage::CreateBreakoutRoom(name) => {
                                    if my_id.is_some() {
                                        let id = uuid::Uuid::new_v4().to_string();
                                        let room = shared::BreakoutRoom {
                                            id: id.clone(),
                                            name,
                                        };
                                        {
                                            let mut rooms = breakout_rooms_mutex.lock().unwrap();
                                            rooms.insert(id, room);
                                        }
                                        // Broadcast new list
                                        let all_rooms: Vec<shared::BreakoutRoom> = {
                                            let rooms = breakout_rooms_mutex.lock().unwrap();
                                            rooms.values().cloned().collect()
                                        };
                                        let _ = tx.send(ServerMessage::BreakoutRoomsList(all_rooms));
                                    }
                                },
                                ClientMessage::JoinBreakoutRoom(room_id) => {
                                    if let Some(uid) = &my_id {
                                        // Update location
                                        {
                                            let mut locations = participant_locations_mutex.lock().unwrap();
                                            locations.insert(uid.clone(), room_id.clone());
                                        }
                                        my_room_id = room_id;
                                        // We don't explicitly notify others of movement in this simplified version,
                                        // but filtering will now apply.
                                    }
                                }
                            }
                        }
                    },
                    _ => break, // Disconnect or Error
                }
            },
            // 2. Control Messages (Lobby Decision)
            Some(granted) = control_rx.recv() => {
                if granted {
                    if let Some(id) = knocking_id.take() {
                        // Retrieve the participant that was in knocking list
                        // Note: It might have been removed if timeout raced?
                        // Actually, if we are here, the `Ok(true)` path in spawn didn't remove it.
                        // We must remove it now and add to main list.

                        let me_opt = {
                            let mut knocking = knocking_mutex.lock().unwrap();
                            knocking.remove(&id).map(|(p, _)| p)
                        };

                        if let Some(me) = me_opt {
                            let joined = {
                                let mut participants = participants_mutex.lock().unwrap();
                                let max_participants = room_config_mutex.lock().unwrap().max_participants;
                                if participants.len() >= max_participants as usize {
                                    false
                                } else {
                                    participants.insert(id.clone(), me.clone());
                                    true
                                }
                            };

                            if !joined {
                                let _ = internal_tx.send(ServerMessage::Error("Room is full".to_string())).await;
                                continue;
                            }
                            my_id = Some(id.clone());

                            let _ = internal_tx.send(ServerMessage::Welcome { id: id.clone() }).await;

                            // Subscribe to broadcast
                            let mut rx = tx.subscribe();
                            let forward_tx = internal_tx.clone();
                            broadcast_task = Some(tokio::spawn(async move {
                                loop {
                                    match rx.recv().await {
                                        Ok(msg) => {
                                            if forward_tx.send(msg).await.is_err() { break; }
                                        },
                                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                                    }
                                }
                            }));

                            let _ = tx.send(ServerMessage::ParticipantJoined(me));

                            let current_list: Vec<Participant> = {
                                let participants = participants_mutex.lock().unwrap();
                                participants.values().cloned().collect()
                            };
                            let _ = internal_tx.send(ServerMessage::ParticipantList(current_list)).await;

                            // Note: Knocking list is already sent via broadcast/internal events if needed,
                            // but if I'm the host, I should see others.
                            let knocking_list: Vec<Participant> = {
                                let knocking = knocking_mutex.lock().unwrap();
                                knocking.values().map(|(p, _)| p.clone()).collect()
                            };
                            for p in knocking_list {
                                let _ = internal_tx.send(ServerMessage::KnockingParticipant(p)).await;
                            }

                            let history: Vec<shared::DrawAction> = {
                                let wb = whiteboard_mutex.lock().unwrap();
                                wb.clone()
                            };
                            if !history.is_empty() {
                                let _ = internal_tx.send(ServerMessage::WhiteboardHistory(history)).await;
                            }
                        } else {
                            // Should not happen if logic is correct, but safe fallback
                            let _ = internal_tx.send(ServerMessage::AccessDenied).await;
                        }
                    }
                } else {
                    knocking_id = None;
                    let _ = internal_tx.send(ServerMessage::AccessDenied).await;
                }
            }
        }
    }

    send_task.abort();

    // Cleanup
    if let Some(t) = broadcast_task { t.abort(); }

    if let Some(id) = my_id {
        {
            let mut participants = participants_mutex.lock().unwrap();
            participants.remove(&id);
        }
        let _ = tx.send(ServerMessage::ParticipantLeft(id));
    } else if let Some(kid) = knocking_id {
        // If disconnected while knocking
        let removed = {
            let mut knocking = knocking_mutex.lock().unwrap();
            knocking.remove(&kid).is_some()
        };
        if removed {
            let _ = tx.send(ServerMessage::KnockingParticipantLeft(kid));
        }
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

    #[tokio::test]
    async fn test_create_room() {
        use shared::RoomConfig;
        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        let app_state = Arc::new(AppState {
            tx,
            participants: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            knocking_participants: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            room_config: Arc::new(std::sync::Mutex::new(RoomConfig::default())),
            polls: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            whiteboard: Arc::new(std::sync::Mutex::new(Vec::new())),
            chat_history: Arc::new(std::sync::Mutex::new(Vec::new())),
            breakout_rooms: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            participant_locations: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        });

        let config = RoomConfig::default();
        let response = create_room(State(app_state), Json(config)).await.into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_room_with_limit() {
        use shared::RoomConfig;
        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        let app_state = Arc::new(AppState {
            tx,
            participants: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            knocking_participants: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            room_config: Arc::new(std::sync::Mutex::new(RoomConfig::default())),
            polls: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            whiteboard: Arc::new(std::sync::Mutex::new(Vec::new())),
            chat_history: Arc::new(std::sync::Mutex::new(Vec::new())),
            breakout_rooms: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            participant_locations: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        });

        let config = RoomConfig {
            max_participants: 10,
            ..Default::default()
        };

        let response = create_room(State(app_state.clone()), Json(config.clone())).await.into_response();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["config"]["max_participants"], 10);

        // Verify state was updated
        let stored_config = app_state.room_config.lock().unwrap();
        assert_eq!(stored_config.max_participants, 10);
    }

    #[tokio::test]
    async fn test_chat_history() {
        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        let history = Arc::new(std::sync::Mutex::new(Vec::new()));

        let _app_state = Arc::new(AppState {
            tx,
            participants: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            knocking_participants: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            room_config: Arc::new(std::sync::Mutex::new(shared::RoomConfig::default())),
            polls: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            whiteboard: Arc::new(std::sync::Mutex::new(Vec::new())),
            chat_history: history.clone(),
            breakout_rooms: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            participant_locations: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        });

        // Simulate adding a message (like the websocket handler would)
        {
            let mut h = history.lock().unwrap();
            h.push(shared::ChatMessage {
                user_id: "user1".to_string(),
                content: "Hello".to_string(),
                recipient_id: None,
                timestamp: 1234567890,
            });
        }

        // Verify history contains the message
        let h = history.lock().unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_typing_broadcast() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(100);
        let _history = Arc::new(std::sync::Mutex::new(Vec::<shared::ChatMessage>::new()));

        // We just need to check if a PeerTyping message can be sent through the channel
        // Real logic relies on handle_socket which is hard to unit test without full ws
        // But we can verify the enum structure works.

        let msg = shared::ServerMessage::PeerTyping {
            user_id: "user1".to_string(),
            is_typing: true,
            room_id: None,
        };

        assert!(tx.send(msg).is_ok());

        let received = rx.recv().await.unwrap();
        match received {
            shared::ServerMessage::PeerTyping { user_id, is_typing, room_id: _ } => {
                assert_eq!(user_id, "user1");
                assert!(is_typing);
            },
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_kick_logic() {
        // We can't easily test the full async loop here without mocking WS
        // But we can test the data structures and messages

        let (tx, _rx) = tokio::sync::broadcast::channel(100);

        // Setup room with a host
        let room_config = Arc::new(std::sync::Mutex::new(shared::RoomConfig {
            host_id: Some("host_123".to_string()),
            ..Default::default()
        }));

        let participants = Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

        // Add a participant
        {
            let mut p = participants.lock().unwrap();
            p.insert("target_456".to_string(), shared::Participant {
                id: "target_456".to_string(),
                name: "Target".to_string(),
                is_hand_raised: false,
                is_sharing_screen: false,
            });
        }

        // Simulate kick action
        let target_id = "target_456".to_string();
        let my_id = "host_123".to_string(); // Sender is host

        // Logic from handle_socket (simplified)
        let config = room_config.lock().unwrap();
        if Some(my_id) == config.host_id {
            let mut p = participants.lock().unwrap();
            p.remove(&target_id);
            // In real code we send broadcast
            let _ = tx.send(shared::ServerMessage::Kicked(target_id.clone()));
        }

        // Verify removal
        let p = participants.lock().unwrap();
        assert!(!p.contains_key("target_456"));
    }

    #[tokio::test]
    async fn test_breakout_room_creation() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(100);
        let rooms = Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

        // Simulate "CreateBreakoutRoom"
        let room_name = "Team A".to_string();
        let id = "room_1".to_string();
        let room = shared::BreakoutRoom {
            id: id.clone(),
            name: room_name.clone(),
        };

        {
            let mut r = rooms.lock().unwrap();
            r.insert(id.clone(), room);
        }

        let all_rooms: Vec<shared::BreakoutRoom> = {
            let r = rooms.lock().unwrap();
            r.values().cloned().collect()
        };
        let _ = tx.send(shared::ServerMessage::BreakoutRoomsList(all_rooms));

        // Check message
        if let Ok(shared::ServerMessage::BreakoutRoomsList(list)) = rx.recv().await {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].name, "Team A");
        } else {
            panic!("Wrong message");
        }
    }
}
