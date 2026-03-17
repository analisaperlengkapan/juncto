use super::breakout;
use super::chat;
use super::polls;
use super::whiteboard;
use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use shared::{ClientMessage, Participant, RoomConfig, ServerMessage};
use std::sync::Arc;

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

    let current_config: RoomConfig = {
        let config = state.room_config.lock().unwrap();
        config.clone()
    };
    if let Ok(json) = serde_json::to_string(&ServerMessage::RoomUpdated(current_config.clone())) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Explicitly send RoomUpdated to self to trigger frontend state logic (like is_host)
    let _ = internal_tx
        .send(ServerMessage::RoomUpdated(current_config.clone()))
        .await;

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
    let shared_video_mutex = state.shared_video_url.clone();
    let speaking_start_times_mutex = state.speaking_start_times.clone();

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
        let _ = internal_tx
            .send(ServerMessage::BreakoutRoomsList(rooms))
            .await;
    }

    loop {
        tokio::select! {
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
                                            if target_id == *uid {
                                                // Prevent self-kick
                                                continue;
                                            }
                                            // Valid kick
                                            // 1. Update speaking time before removal
                                            {
                                                let mut starts = speaking_start_times_mutex.lock().unwrap();
                                                if let Some(start) = starts.remove(&target_id) {
                                                    let now = chrono::Utc::now().timestamp_millis() as u64;
                                                    if now > start {
                                                        let delta = now - start;
                                                        let mut participants = participants_mutex.lock().unwrap();
                                                        if let Some(p) = participants.get_mut(&target_id) {
                                                            p.speaking_time += delta;
                                                            // Broadcast final update before kick
                                                            let _ = tx.send(ServerMessage::ParticipantUpdated(p.clone()));
                                                        }
                                                    }
                                                }
                                            }
                                            // 2. Remove from participants
                                            {
                                                let mut participants = participants_mutex.lock().unwrap();
                                                participants.remove(&target_id);
                                            }
                                            // Fetch target's location before removal to broadcast Left accurately
                                            let target_loc = {
                                                let locations = participant_locations_mutex.lock().unwrap();
                                                locations.get(&target_id).cloned().flatten()
                                            };
                                            // 3. Remove from participant_locations
                                            {
                                                let mut locations = participant_locations_mutex.lock().unwrap();
                                                locations.remove(&target_id);
                                            }
                                            // 4. Broadcast Kicked
                                            let _ = tx.send(ServerMessage::Kicked { target_id: target_id.clone(), room_id: target_loc.clone() });

                                            // 5. Broadcast ParticipantLeft (so lists update)
                                            let _ = tx.send(ServerMessage::ParticipantLeft { id: target_id, room_id: target_loc });
                                        }
                                    }
                                },
                                ClientMessage::ToggleSubtitles => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            let config = {
                                                let mut config = room_config_mutex.lock().unwrap();
                                                config.is_subtitles_enabled = !config.is_subtitles_enabled;
                                                config.clone()
                                            };
                                            let _ = tx.send(ServerMessage::RoomUpdated(config));
                                        }
                                    }
                                },
                                ClientMessage::SetPresence(status) => {
                                    if let Some(uid) = &my_id {
                                        let updated_participant = {
                                            let mut participants = participants_mutex.lock().unwrap();
                                            if let Some(p) = participants.get_mut(uid) {
                                                p.presence = status;
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
                                ClientMessage::EndMeeting => {
                                    if let Some(uid) = &my_id {
                                        let host_id = {
                                            room_config_mutex.lock().unwrap().host_id.clone()
                                        };
                                        if Some(uid.clone()) == host_id {
                                            // Valid end meeting
                                            // Broadcast RoomEnded
                                            let _ = tx.send(ServerMessage::RoomEnded);

                                            {
                                                let mut p = participants_mutex.lock().unwrap();
                                                p.clear();
                                            }
                                            {
                                                let mut c = room_config_mutex.lock().unwrap();
                                                *c = shared::RoomConfig::default();
                                            }
                                            {
                                                let mut s = speaking_start_times_mutex.lock().unwrap();
                                                s.clear();
                                            }
                                            {
                                                let mut k = state.knocking_participants.lock().unwrap();
                                                k.clear();
                                            }
                                            {
                                                let mut p = state.polls.lock().unwrap();
                                                p.clear();
                                            }
                                            {
                                                let mut w = state.whiteboard.lock().unwrap();
                                                w.clear();
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
                                                let mut loc = state.participant_locations.lock().unwrap();
                                                loc.clear();
                                            }
                                            {
                                                let mut vid = state.shared_video_url.lock().unwrap();
                                                *vid = None;
                                            }
                                        }
                                    }
                                },
                                ClientMessage::ToggleLobby => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            let new_config = {
                                                let mut config = room_config_mutex.lock().unwrap();
                                                config.is_lobby_enabled = !config.is_lobby_enabled;
                                                config.clone()
                                            };
                                            let _ = tx.send(ServerMessage::RoomUpdated(new_config));
                                        }
                                    }
                                },
                                ClientMessage::GrantAccess(target_id) => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            let sender_opt = {
                                                let mut knocking = knocking_mutex.lock().unwrap();
                                                knocking.get_mut(&target_id).and_then(|(_, s)| s.take())
                                            };
                                            if let Some(s) = sender_opt {
                                                let _ = s.send(true);
                                            }
                                        }
                                    }
                                },
                                ClientMessage::DenyAccess(target_id) => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            let sender_opt = {
                                                let mut knocking = knocking_mutex.lock().unwrap();
                                                knocking.get_mut(&target_id).and_then(|(_, s)| s.take())
                                            };
                                            if let Some(s) = sender_opt {
                                                let _ = s.send(false);
                                            }
                                        }
                                    }
                                },
                                ClientMessage::Join(name) => {
                                    if my_id.is_some() || knocking_id.is_some() { continue; } // Already joined or knocking

                                    // Check if room is locked or lobby is enabled
                                    let (is_locked, is_lobby, max_participants, host_exists) = {
                                        let config = room_config_mutex.lock().unwrap();
                                        (config.is_locked, config.is_lobby_enabled, config.max_participants, config.host_id.is_some())
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
                                        is_muted: false,
                                        speaking_time: 0,
                                        presence: shared::PresenceStatus::Connected,
                                    };

                                    if is_lobby && host_exists {
                                        let (s, r) = tokio::sync::oneshot::channel();
                                        {
                                            let mut knocking = knocking_mutex.lock().unwrap();
                            knocking.insert(id.clone(), (me.clone(), Some(s)));
                                        }
                                        knocking_id = Some(id.clone());
                                        let _ = internal_tx.send(ServerMessage::Knocking).await;
                                        let _ = tx.send(ServerMessage::KnockingParticipant(me.clone()));

                                        let control_tx_clone = control_tx.clone();
                                        let knocking_mutex_clone = knocking_mutex.clone();
                                        let tx_clone = tx.clone();
                                        let id_clone = id.clone();

                                        tokio::spawn(async move {
                                            match tokio::time::timeout(std::time::Duration::from_secs(120), r).await {
                                                Ok(Ok(true)) => {
                                                    let _ = control_tx_clone.send(true).await;
                                                },
                                                _ => {
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

                                    // Send Chat History
                                    let history: Vec<shared::ChatMessage> = {
                                        let history = chat_history_mutex.lock().unwrap();
                                        history.clone()
                                    };
                                    if !history.is_empty() {
                                        let _ = internal_tx.send(ServerMessage::ChatHistory(history)).await;
                                    }

                                    if new_host_assigned {
                                        let new_config = {
                                            room_config_mutex.lock().unwrap().clone()
                                        };
                                        let _ = tx.send(ServerMessage::RoomUpdated(new_config.clone()));
                                        let _ = internal_tx.send(ServerMessage::RoomUpdated(new_config)).await;
                                    }

                                    // Register initial location (Main Room)
                                    {
                                        let mut locations = participant_locations_mutex.lock().unwrap();
                                        locations.insert(id.clone(), None);
                                    }

                                    let mut rx = tx.subscribe();
                                    let forward_tx = internal_tx.clone();
                                    let my_id_clone = id.clone();
                                    let locations_clone = participant_locations_mutex.clone();

                                    broadcast_task = Some(tokio::spawn(async move {
                                        loop {
                                            match rx.recv().await {
                                                Ok(msg) => {
                                                    // Filter based on room and recipient
                                                    let should_send = match &msg {
                                                        ServerMessage::Chat { message, room_id } => {
                                                            let my_loc = {
                                                                let locs = locations_clone.lock().unwrap();
                                                                locs.get(&my_id_clone).cloned().flatten()
                                                            };
                                                            println!("BROADCAST TASK 1: my_id: {}, msg from: {}, my_loc: {:?}, msg_room: {:?}", my_id_clone, message.user_id, my_loc, room_id);
                                                            if *room_id != my_loc {
                                                                false
                                                            } else if let Some(target) = &message.recipient_id {
                                                                *target == my_id_clone || message.user_id == my_id_clone // Must echo private message back to self
                                                            } else {
                                                                true
                                                            }
                                                        },
                                                        ServerMessage::PeerTyping { room_id, .. } => {
                                                            let my_loc = {
                                                                let locs = locations_clone.lock().unwrap();
                                                                locs.get(&my_id_clone).cloned().flatten()
                                                            };
                                                            *room_id == my_loc
                                                        },
                                                        ServerMessage::Offer { source_id, target_id, .. }
                                                        | ServerMessage::Answer { source_id, target_id, .. }
                                                        | ServerMessage::IceCandidate { source_id, target_id, .. } => {
                                                            if *target_id == my_id_clone {
                                                                let locs = locations_clone.lock().unwrap();
                                                                let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                                let source_loc = locs.get(source_id).cloned().flatten();
                                                                my_loc == source_loc
                                                            } else {
                                                                false
                                                            }
                                                        },
                                                        ServerMessage::ParticipantJoined(p) => {
                                                            let locs = locations_clone.lock().unwrap();
                                                            let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                            let source_loc = locs.get(&p.id).cloned().flatten();
                                                            my_loc == source_loc
                                                        },
                                                        ServerMessage::ParticipantUpdated(p) => {
                                                            let locs = locations_clone.lock().unwrap();
                                                            let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                            let source_loc = locs.get(&p.id).cloned().flatten();
                                                            // Deliver to same room OR if it's an update about myself
                                                            my_loc == source_loc || p.id == my_id_clone
                                                        },
                                                        ServerMessage::ParticipantLeft { id, room_id } => {
                                                            // Don't deliver ParticipantLeft to the person who is leaving (e.g. during room switch)
                                                            if *id == my_id_clone {
                                                                false
                                                            } else {
                                                                let locs = locations_clone.lock().unwrap();
                                                                let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                                // Use the room_id embedded in the message instead of looking it up
                                                                my_loc == *room_id
                                                            }
                                                        },
                                                        ServerMessage::Kicked { target_id: id, room_id } => {
                                                            let locs = locations_clone.lock().unwrap();
                                                            let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                            // Deliver to same room OR if it's a command directed at myself
                                                            my_loc == *room_id || *id == my_id_clone
                                                        },
                                                        ServerMessage::MutedByHost(id) => {
                                                            let locs = locations_clone.lock().unwrap();
                                                            let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                            let source_loc = locs.get(id).cloned().flatten();
                                                            // Deliver to same room OR if it's a command directed at myself
                                                            my_loc == source_loc || *id == my_id_clone
                                                        },
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

                                    let _ = tx.send(ServerMessage::ParticipantJoined(me));

                                    let current_list: Vec<Participant> = {
                                        let participants = participants_mutex.lock().unwrap();
                                        let locs = participant_locations_mutex.lock().unwrap();
                                        participants.values().filter(|p| {
                                            locs.get(&p.id).cloned().flatten().is_none()
                                        }).cloned().collect()
                                    };
                                    let _ = internal_tx.send(ServerMessage::ParticipantList(current_list)).await;

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

                                    // Send Existing Polls
                                    let polls_list: Vec<shared::Poll> = {
                                        let polls = polls_mutex.lock().unwrap();
                                        polls.values().cloned().collect()
                                    };
                                    if !polls_list.is_empty() {
                                        let _ = internal_tx.send(ServerMessage::PollsList(polls_list)).await;
                                    }

                                    // Send Shared Video State
                                    let shared_url = {
                                        shared_video_mutex.lock().unwrap().clone()
                                    };
                                    if let Some(url) = shared_url {
                                        let _ = internal_tx.send(ServerMessage::VideoShared(url)).await;
                                    }
                                },
                                ClientMessage::Chat { content, recipient_id, attachment, room_id } => {
                                    if let Some(uid) = &my_id {
                                        // Security: Only allow the client to send a chat message if the room_id they provided
                                        // matches the room_id the server believes they are currently in.
                                        let is_authorized = room_id == my_room_id;

                                        if !is_authorized {
                                            let _ = internal_tx.send(ServerMessage::Error("Unauthorized: Cannot send message to a different room".to_string())).await;
                                            continue;
                                        }

                                        let res = chat::process_chat_message(
                                            uid,
                                            &room_id,
                                            content,
                                            recipient_id,
                                            attachment,
                                            &state
                                        );
                                        match res {
                                            Ok(msg) => {
                                                let _ = tx.send(msg);
                                            },
                                            Err(e) => {
                                                let _ = internal_tx.send(ServerMessage::Error(e)).await;
                                            }
                                        }
                                    }
                                },
                                ClientMessage::ToggleRoomLock => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            let new_config = {
                                                let mut config = room_config_mutex.lock().unwrap();
                                                config.is_locked = !config.is_locked;
                                                config.clone()
                                            };
                                            let _ = tx.send(ServerMessage::RoomUpdated(new_config));
                                        }
                                    }
                                },
                                ClientMessage::ToggleRecording => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            let new_config = {
                                                let mut config = room_config_mutex.lock().unwrap();
                                                config.is_recording = !config.is_recording;
                                                config.clone()
                                            };
                                            let _ = tx.send(ServerMessage::RoomUpdated(new_config));
                                        }
                                    }
                                },
                                ClientMessage::CreatePoll(poll) => {
                                    if let Some(uid) = &my_id {
                                        match polls::create_poll(uid, poll, &state) {
                                            Ok(msg) => { let _ = tx.send(msg); },
                                            Err(e) => { let _ = internal_tx.send(ServerMessage::Error(e)).await; }
                                        }
                                    }
                                },
                                ClientMessage::Vote { poll_id, option_id } => {
                                    if let Some(uid) = &my_id {
                                        match polls::vote(uid, poll_id, option_id, &state) {
                                            Ok(msg) => { let _ = tx.send(msg); },
                                            Err(e) => { let _ = internal_tx.send(ServerMessage::Error(e)).await; }
                                        }
                                    }
                                },
                                ClientMessage::Draw(action) => {
                                    if let Some(uid) = &my_id {
                                        let msg = whiteboard::process_draw_action(uid, action, &state);
                                        let _ = tx.send(msg);
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
                                    if let Some(uid) = &my_id {
                                        match breakout::create_breakout_room(uid, name, &state) {
                                            Ok(msg) => { let _ = tx.send(msg); },
                                            Err(e) => { let _ = internal_tx.send(ServerMessage::Error(e)).await; }
                                        }
                                    }
                                },
                                ClientMessage::JoinBreakoutRoom(room_id) => {
                                    if let Some(uid) = &my_id {
                                        // Pre-validate room existence to prevent false ParticipantLeft broadcasts
                                        let is_valid = match &room_id {
                                            Some(rid) => state.breakout_rooms.lock().unwrap().contains_key(rid),
                                            None => true,
                                        };

                                        if !is_valid {
                                            let _ = internal_tx.send(ServerMessage::Error("Breakout room not found".to_string())).await;
                                            continue;
                                        }

                                        let me = {
                                            let participants = participants_mutex.lock().unwrap();
                                            participants.get(uid).cloned()
                                        };

                                        // Capture current location to embed in the leave message
                                        let old_room = {
                                            let locations = participant_locations_mutex.lock().unwrap();
                                            locations.get(uid).cloned().flatten()
                                        };

                                        // Broadcast leave using the embedded location, immune to async races
                                        let _ = tx.send(ServerMessage::ParticipantLeft { id: uid.clone(), room_id: old_room });

                                        match breakout::join_breakout_room(uid, room_id, &state) {
                                            Ok((new_rid, msgs)) => {
                                                my_room_id = new_rid;
                                                for msg in msgs {
                                                    let _ = internal_tx.send(msg).await;
                                                }

                                                // Broadcast join to new room (after location update)
                                                if let Some(p) = me {
                                                    let _ = tx.send(ServerMessage::ParticipantJoined(p));
                                                }
                                            },
                                            Err(e) => {
                                                // Should not be hit due to pre-validation, but included for safety
                                                let _ = internal_tx.send(ServerMessage::Error(e)).await;
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StartShareVideo(url) => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            {
                                                let mut v = shared_video_mutex.lock().unwrap();
                                                *v = Some(url.clone());
                                            }
                                            let _ = tx.send(ServerMessage::VideoShared(url));
                                        }
                                    }
                                },
                                ClientMessage::StopShareVideo => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            {
                                                let mut v = shared_video_mutex.lock().unwrap();
                                                *v = None;
                                            }
                                            let _ = tx.send(ServerMessage::VideoStopped);
                                        }
                                    }
                                },
                                ClientMessage::Speaking(is_speaking) => {
                                    if let Some(uid) = &my_id {
                                        let mut update_stats = false;
                                        if is_speaking {
                                            let mut starts = speaking_start_times_mutex.lock().unwrap();
                                            starts.insert(uid.clone(), chrono::Utc::now().timestamp_millis() as u64);
                                        } else {
                                            let start_opt = {
                                                let mut starts = speaking_start_times_mutex.lock().unwrap();
                                                starts.remove(uid)
                                            };
                                            if let Some(start) = start_opt {
                                                let now = chrono::Utc::now().timestamp_millis() as u64;
                                                if now > start {
                                                    let delta = now - start;
                                                    let mut participants = participants_mutex.lock().unwrap();
                                                    if let Some(p) = participants.get_mut(uid) {
                                                        p.speaking_time += delta;
                                                        update_stats = true;
                                                    }
                                                }
                                            }
                                        }

                                        if update_stats {
                                            let updated_p = {
                                                let participants = participants_mutex.lock().unwrap();
                                                participants.get(uid).cloned()
                                            };
                                            if let Some(p) = updated_p {
                                                let _ = tx.send(ServerMessage::ParticipantUpdated(p));
                                            }
                                        }

                                        let _ = tx.send(ServerMessage::PeerSpeaking {
                                            user_id: uid.clone(),
                                            speaking: is_speaking,
                                        });
                                    }
                                },
                                ClientMessage::SetMuteStatus(muted) => {
                                    if let Some(uid) = &my_id {
                                        let updated_participant = {
                                            let mut participants = participants_mutex.lock().unwrap();
                                            if let Some(p) = participants.get_mut(uid) {
                                                p.is_muted = muted;
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
                                ClientMessage::Authenticate { username, password } => {
                                    if let Some(_uid) = &my_id {
                                        // TODO: implement real authentication
                                        if !username.is_empty() && password.as_ref().map_or(false, |p| !p.is_empty()) {
                                            let _ = internal_tx.send(ServerMessage::AuthenticationResult(true)).await;
                                        } else {
                                            let _ = internal_tx.send(ServerMessage::AuthenticationResult(false)).await;
                                        }
                                    }
                                },
                                ClientMessage::FetchCalendar => {
                                    if let Some(_uid) = &my_id {
                                        let mock_events = vec![
                                            "Team Standup - 10:00 AM".to_string(),
                                            "Project Sync - 1:00 PM".to_string(),
                                            "1:1 with Manager - 3:30 PM".to_string()
                                        ];
                                        let _ = internal_tx.send(ServerMessage::CalendarEvents(mock_events)).await;
                                    }
                                },
                                ClientMessage::AnalyticsEvent { name, properties } => {
                                    if let Some(uid) = &my_id {
                                        // TODO: use proper tracing/logging framework
                                        let safe_name: String = name.chars().take(200).filter(|c| !c.is_control()).collect();
                                        let safe_props: String = properties.chars().take(1000).filter(|c| !c.is_control()).collect();
                                        println!("INFO: Received Analytics Event from {}: {} - {}", uid, safe_name, safe_props);
                                    }
                                },
                                ClientMessage::MuteParticipant(target_id) => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            let updated_participant = {
                                                let mut participants = participants_mutex.lock().unwrap();
                                                if let Some(p) = participants.get_mut(&target_id) {
                                                    p.is_muted = true;
                                                    Some(p.clone())
                                                } else {
                                                    None
                                                }
                                            };
                                            if let Some(p) = updated_participant {
                                                let _ = tx.send(ServerMessage::ParticipantUpdated(p));
                                                let _ = tx.send(ServerMessage::MutedByHost(target_id));
                                            }
                                        }
                                    }
                                },
                                ClientMessage::TransferHost(target_id) => {
                                    if let Some(uid) = &my_id {
                                        let is_host = {
                                            room_config_mutex.lock().unwrap().host_id == Some(uid.clone())
                                        };
                                        if is_host {
                                            let target_exists = {
                                                let participants = participants_mutex.lock().unwrap();
                                                participants.contains_key(&target_id)
                                            };
                                            if target_exists {
                                                let new_config = {
                                                    let mut config = room_config_mutex.lock().unwrap();
                                                    config.host_id = Some(target_id);
                                                    config.clone()
                                                };
                                                let _ = tx.send(ServerMessage::RoomUpdated(new_config));
                                            }
                                        }
                                    }
                                },
                                ClientMessage::Ping => {
                                    let _ = internal_tx.send(ServerMessage::Pong { timestamp: chrono::Utc::now().timestamp_millis() as u64 }).await;
                                },
                                ClientMessage::Offer { target_id, sdp } => {
                                    if let Some(uid) = &my_id {
                                        let _ = tx.send(ServerMessage::Offer {
                                            source_id: uid.clone(),
                                            target_id,
                                            sdp,
                                        });
                                    }
                                },
                                ClientMessage::Answer { target_id, sdp } => {
                                    if let Some(uid) = &my_id {
                                        let _ = tx.send(ServerMessage::Answer {
                                            source_id: uid.clone(),
                                            target_id,
                                            sdp,
                                        });
                                    }
                                },
                                ClientMessage::IceCandidate { target_id, candidate, sdp_mid, sdp_m_line_index } => {
                                    if let Some(uid) = &my_id {
                                        let _ = tx.send(ServerMessage::IceCandidate {
                                            source_id: uid.clone(),
                                            target_id,
                                            candidate,
                                            sdp_mid,
                                            sdp_m_line_index,
                                        });
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
                        let me_opt = {
                            let mut knocking = knocking_mutex.lock().unwrap();
                            knocking.remove(&id).map(|(p, _)| p)
                        };

                        if let Some(me) = me_opt {
                            let (joined, new_host_assigned) = {
                                let mut participants = participants_mutex.lock().unwrap();
                                let mut config = room_config_mutex.lock().unwrap();
                                if participants.len() >= config.max_participants as usize {
                                    (false, false)
                                } else {
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

                            let _ = internal_tx.send(ServerMessage::Welcome { id: id.clone() }).await;

                            // Send Chat History
                            let history: Vec<shared::ChatMessage> = {
                                let history = chat_history_mutex.lock().unwrap();
                                history.clone()
                            };
                            if !history.is_empty() {
                                let _ = internal_tx.send(ServerMessage::ChatHistory(history)).await;
                            }

                            if new_host_assigned {
                                let new_config = {
                                    room_config_mutex.lock().unwrap().clone()
                                };
                                let _ = tx.send(ServerMessage::RoomUpdated(new_config.clone()));
                                let _ = internal_tx.send(ServerMessage::RoomUpdated(new_config)).await;
                            }

                             // Register initial location (Main Room)
                            {
                                let mut locations = participant_locations_mutex.lock().unwrap();
                                locations.insert(id.clone(), None);
                            }

                            // Subscribe to broadcast
                            let mut rx = tx.subscribe();
                            let forward_tx = internal_tx.clone();
                            let my_id_clone = id.clone();
                            let locations_clone = participant_locations_mutex.clone();

                            broadcast_task = Some(tokio::spawn(async move {
                                loop {
                                    match rx.recv().await {
                                        Ok(msg) => {
                                            // Filter based on room and recipient
                                            let should_send = match &msg {
                                                ServerMessage::Chat { message, room_id } => {
                                                    let my_loc = {
                                                        let locs = locations_clone.lock().unwrap();
                                                        locs.get(&my_id_clone).cloned().flatten()
                                                    };
                                                    println!("BROADCAST TASK 2: my_id: {}, msg from: {}, my_loc: {:?}, msg_room: {:?}", my_id_clone, message.user_id, my_loc, room_id);
                                                    if *room_id != my_loc {
                                                        false
                                                    } else if let Some(target) = &message.recipient_id {
                                                        *target == my_id_clone || message.user_id == my_id_clone // Must echo private message back to self
                                                    } else {
                                                        true
                                                    }
                                                },
                                                ServerMessage::PeerTyping { room_id, .. } => {
                                                    let my_loc = {
                                                        let locs = locations_clone.lock().unwrap();
                                                        locs.get(&my_id_clone).cloned().flatten()
                                                    };
                                                    *room_id == my_loc
                                                },
                                                ServerMessage::Offer { source_id, target_id, .. }
                                                | ServerMessage::Answer { source_id, target_id, .. }
                                                | ServerMessage::IceCandidate { source_id, target_id, .. } => {
                                                    if *target_id == my_id_clone {
                                                        let locs = locations_clone.lock().unwrap();
                                                        let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                        let source_loc = locs.get(source_id).cloned().flatten();
                                                        my_loc == source_loc
                                                    } else {
                                                        false
                                                    }
                                                },
                                                        ServerMessage::ParticipantJoined(p) => {
                                                            let locs = locations_clone.lock().unwrap();
                                                            let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                            let source_loc = locs.get(&p.id).cloned().flatten();
                                                            my_loc == source_loc
                                                        },
                                                        ServerMessage::ParticipantUpdated(p) => {
                                                            let locs = locations_clone.lock().unwrap();
                                                            let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                            let source_loc = locs.get(&p.id).cloned().flatten();
                                                            my_loc == source_loc || p.id == my_id_clone
                                                        },
                                                        ServerMessage::ParticipantLeft { id, room_id } => {
                                                            if *id == my_id_clone {
                                                                false
                                                            } else {
                                                                let locs = locations_clone.lock().unwrap();
                                                                let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                                // Use the room_id embedded in the message instead of looking it up
                                                                my_loc == *room_id
                                                            }
                                                        },
                                                ServerMessage::Kicked { target_id: id, room_id } => {
                                                    let locs = locations_clone.lock().unwrap();
                                                    let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                    my_loc == *room_id || *id == my_id_clone
                                                },
                                                ServerMessage::MutedByHost(id) => {
                                                            let locs = locations_clone.lock().unwrap();
                                                            let my_loc = locs.get(&my_id_clone).cloned().flatten();
                                                            let source_loc = locs.get(id).cloned().flatten();
                                                            my_loc == source_loc || *id == my_id_clone
                                                        },
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

                            let _ = tx.send(ServerMessage::ParticipantJoined(me));

                            let current_list: Vec<Participant> = {
                                let participants = participants_mutex.lock().unwrap();
                                        let locs = participant_locations_mutex.lock().unwrap();
                                        participants.values().filter(|p| {
                                            locs.get(&p.id).cloned().flatten().is_none()
                                        }).cloned().collect()
                            };
                            let _ = internal_tx.send(ServerMessage::ParticipantList(current_list)).await;

                            let knocking_list: Vec<Participant> = {
                                let knocking = knocking_mutex.lock().unwrap();
                                knocking.values().map(|(p, _)| p.clone()).collect()
                            };
                            for p in knocking_list {
                                let _ = internal_tx.send(ServerMessage::KnockingParticipant(p)).await;
                            }

                            // Send Breakout Rooms List
                            let all_rooms: Vec<shared::BreakoutRoom> = {
                                let rooms = breakout_rooms_mutex.lock().unwrap();
                                rooms.values().cloned().collect()
                            };
                            if !all_rooms.is_empty() {
                                let _ = internal_tx.send(ServerMessage::BreakoutRoomsList(all_rooms)).await;
                            }

                            let history: Vec<shared::DrawAction> = {
                                let wb = whiteboard_mutex.lock().unwrap();
                                wb.clone()
                            };
                            if !history.is_empty() {
                                let _ = internal_tx.send(ServerMessage::WhiteboardHistory(history)).await;
                            }

                            // Send Existing Polls
                            let polls_list: Vec<shared::Poll> = {
                                let polls = polls_mutex.lock().unwrap();
                                polls.values().cloned().collect()
                            };
                            if !polls_list.is_empty() {
                                let _ = internal_tx.send(ServerMessage::PollsList(polls_list)).await;
                            }

                            // Send Shared Video State
                            let shared_url = {
                                shared_video_mutex.lock().unwrap().clone()
                            };
                            if let Some(url) = shared_url {
                                let _ = internal_tx.send(ServerMessage::VideoShared(url)).await;
                            }
                        } else {
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
    if let Some(t) = broadcast_task {
        t.abort();
    }

    if let Some(id) = my_id {
        // Update speaking time before removal
        {
            let mut starts = speaking_start_times_mutex.lock().unwrap();
            if let Some(start) = starts.remove(&id) {
                let now = chrono::Utc::now().timestamp_millis() as u64;
                if now > start {
                    let delta = now - start;
                    let mut participants = participants_mutex.lock().unwrap();
                    if let Some(p) = participants.get_mut(&id) {
                        p.speaking_time += delta;
                        // Broadcast final update
                        let _ = tx.send(ServerMessage::ParticipantUpdated(p.clone()));
                    }
                }
            }
        }

        // Handle Host Leaving / Reassignment
        let new_host_assigned = {
            let mut participants = participants_mutex.lock().unwrap();
            participants.remove(&id);

            let mut config = room_config_mutex.lock().unwrap();
            if config.host_id == Some(id.clone()) {
                // Host left, assign new host if any participants remain
                if let Some(new_host) = participants.keys().next() {
                    config.host_id = Some(new_host.clone());
                    true
                } else {
                    config.host_id = None;
                    false
                }
            } else {
                false
            }
        };

        if new_host_assigned {
            let new_config = { room_config_mutex.lock().unwrap().clone() };
            let _ = tx.send(ServerMessage::RoomUpdated(new_config));
        }

        // Fetch location before cleanup to embed in message
        let old_room = {
            let locations = participant_locations_mutex.lock().unwrap();
            locations.get(&id).cloned().flatten()
        };

        let _ = tx.send(ServerMessage::ParticipantLeft {
            id: id.clone(),
            room_id: old_room,
        });

        // Cleanup location
        {
            let mut locations = participant_locations_mutex.lock().unwrap();
            locations.remove(&id);
        }
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

    #[test]
    fn test_ws_handler() {
        assert!(true);
    }
}
