use leptos::*;
use shared::{ChatMessage, Participant, ServerMessage, ClientMessage, Poll, DrawAction, FileAttachment};
use web_sys::{MessageEvent, WebSocket, MediaStream};
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::media::{get_user_media, get_display_media, AudioMonitor};
use crate::components_ui::toast::{ToastMessage, ToastType};
use gloo_timers::callback::Timeout;

#[derive(Clone, PartialEq, Debug)]
pub enum RoomConnectionState {
    Prejoin,
    Lobby,
    Joined,
}

#[derive(Clone)]
pub struct RoomState {
    pub connection_state: ReadSignal<RoomConnectionState>,
    pub messages: ReadSignal<Vec<ChatMessage>>,
    pub participants: ReadSignal<Vec<Participant>>,
    pub knocking_participants: ReadSignal<Vec<Participant>>,
    pub is_connected: ReadSignal<bool>,
    pub is_locked: ReadSignal<bool>,
    pub is_lobby_enabled: ReadSignal<bool>,
    pub is_recording: ReadSignal<bool>,
    pub show_settings: ReadSignal<bool>,
    pub show_polls: ReadSignal<bool>,
    pub show_shortcuts: ReadSignal<bool>,
    pub polls: ReadSignal<Vec<Poll>>,
    pub last_reaction: ReadSignal<Option<(String, String, u64)>>,
    pub show_whiteboard: ReadSignal<bool>,
    pub whiteboard_history: ReadSignal<Vec<DrawAction>>,
    pub my_id: ReadSignal<Option<String>>,
    pub typing_users: ReadSignal<HashSet<String>>,
    pub is_host: Signal<bool>,
    pub host_id: Signal<Option<String>>,
    pub current_room_id: ReadSignal<Option<String>>,
    pub breakout_rooms: ReadSignal<Vec<shared::BreakoutRoom>>,
    pub local_stream: ReadSignal<Option<MediaStream>>,
    pub local_screen_stream: ReadSignal<Option<MediaStream>>,
    pub toasts: ReadSignal<Vec<ToastMessage>>,
    pub is_muted: ReadSignal<bool>,
    pub shared_video_url: ReadSignal<Option<String>>,
    pub speaking_peers: ReadSignal<HashSet<String>>,
    // Setters or Actions
    pub set_show_settings: WriteSignal<bool>,
    pub set_show_polls: WriteSignal<bool>,
    pub set_show_shortcuts: WriteSignal<bool>,
    pub set_show_whiteboard: WriteSignal<bool>,
    pub send_message: Callback<(String, Option<String>, Option<FileAttachment>)>, // content, recipient_id, attachment
    pub start_share_video: Callback<String>,
    pub stop_share_video: Callback<()>,
    pub toggle_lock: Callback<()>,
    pub toggle_lobby: Callback<()>,
    pub toggle_recording: Callback<()>,
    pub grant_access: Callback<String>,
    pub deny_access: Callback<String>,
    pub join_meeting: Callback<String>,
    pub save_profile: Callback<String>,
    pub send_reaction: Callback<String>,
    pub toggle_raise_hand: Callback<()>,
    pub toggle_screen_share: Callback<()>,
    pub kick_participant: Callback<String>,
    pub create_poll: Callback<Poll>,
    pub vote_poll: Callback<(String, u32)>,
    pub send_draw: Callback<DrawAction>,
    pub set_is_typing: Callback<bool>,
    pub create_breakout_room: Callback<String>,
    pub join_breakout_room: Callback<Option<String>>,
    pub toggle_camera: Callback<()>,
    pub toggle_mic: Callback<()>,
    pub dismiss_toast: Callback<u64>,
    pub end_meeting: Callback<()>,
}

pub fn use_room_state() -> RoomState {
    let (current_state, set_current_state) = create_signal(RoomConnectionState::Prejoin);
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (typing_users, set_typing_users) = create_signal(HashSet::<String>::new());
    let (breakout_rooms, set_breakout_rooms) = create_signal(Vec::<shared::BreakoutRoom>::new());
    let (current_room_id, set_current_room_id) = create_signal(None::<String>);
    let (participants, set_participants) = create_signal(Vec::<Participant>::new());
    let (knocking_participants, set_knocking_participants) = create_signal(Vec::<Participant>::new());
    let (ws, set_ws) = create_signal(None::<WebSocket>);
    let (is_connected, set_is_connected) = create_signal(false);
    let (is_locked, set_is_locked) = create_signal(false);
    let (is_lobby_enabled, set_is_lobby_enabled) = create_signal(false);
    let (is_recording, set_is_recording) = create_signal(false);
    let (show_settings, set_show_settings) = create_signal(false);
    let (show_polls, set_show_polls) = create_signal(false);
    let (show_shortcuts, set_show_shortcuts) = create_signal(false);
    let (polls, set_polls) = create_signal(Vec::<Poll>::new());
    let (last_reaction, set_last_reaction) = create_signal(None::<(String, String, u64)>);
    let (show_whiteboard, set_show_whiteboard) = create_signal(false);
    let (whiteboard_history, set_whiteboard_history) = create_signal(Vec::<DrawAction>::new());
    let (_last_draw_action, set_last_draw_action) = create_signal(None::<DrawAction>);
    let (my_id, set_my_id) = create_signal(None::<String>);
    let (local_stream, set_local_stream) = create_signal(None::<MediaStream>);
    let (local_screen_stream, set_local_screen_stream) = create_signal(None::<MediaStream>);
    let (toasts, set_toasts) = create_signal(Vec::<ToastMessage>::new());
    let (is_muted, set_is_muted) = create_signal(false);
    let (shared_video_url, set_shared_video_url) = create_signal(None::<String>);
    let (speaking_peers, set_speaking_peers) = create_signal(HashSet::<String>::new());
    let (_audio_monitor, set_audio_monitor) = create_signal(None::<AudioMonitor>);

    // We assume the first participant in the list is the host for now,
    // or we'd need to send host_id in RoomConfig.
    // The previous implementation used host_id in backend but didn't expose it to frontend.
    // Let's rely on backend RoomUpdated message.
    // BUT, RoomConfig struct in shared was updated to include host_id.
    // So we can extract it from there.

    // We need to store the current room config to access host_id.
    let (room_config, set_room_config) = create_signal(shared::RoomConfig::default());

    let host_id = Signal::derive(move || {
        room_config.get().host_id
    });

    let is_host = Signal::derive(move || {
        let h = host_id.get();
        let my = my_id.get();

        match (h, my) {
            (Some(host), Some(me)) => host == me,
            _ => false,
        }
    });

    let add_toast = move |msg: String, type_: ToastType| {
        let id = js_sys::Date::now() as u64;
        set_toasts.update(|t| t.push(ToastMessage { id, message: msg, type_ }));

        // Auto dismiss
        Timeout::new(3000, move || {
            set_toasts.update(|t| t.retain(|x| x.id != id));
        }).forget();
    };

    let dismiss_toast = Callback::new(move |id: u64| {
        set_toasts.update(|t| t.retain(|x| x.id != id));
    });

    // Initialize WebSocket
    create_effect(move |_| {
        // Ensure my_id is reset on new connection logic if needed, but here we just connect.
        // Actually, if we reconnect, we might get a new ID.
        set_my_id.set(None);
        // Default config has host_id = None.
        set_room_config.set(shared::RoomConfig::default());

        // Reset host signal to false until we get new data
        // Derived signal updates automatically based on deps.

        let location = web_sys::window().unwrap().location();
        let protocol = if location.protocol().unwrap() == "https:" { "wss:" } else { "ws:" };
        let host = location.host().unwrap();
        let url = format!("{}//{}/ws/chat", protocol, host);

        if let Ok(socket) = WebSocket::new(&url) {
            // Handle incoming messages
            let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let txt: String = txt.into();
                    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                        match server_msg {
                            ServerMessage::Welcome { id } => {
                                set_my_id.set(Some(id));
                                set_current_state.set(RoomConnectionState::Joined);
                            },
                            ServerMessage::RoomUpdated(config) => {
                                set_is_locked.set(config.is_locked);

                                // Check for recording status change
                                let was_recording = is_recording.get_untracked();
                                if config.is_recording != was_recording {
                                    if config.is_recording {
                                        add_toast("Recording Started".to_string(), ToastType::Info);
                                    } else {
                                        add_toast("Recording Stopped".to_string(), ToastType::Info);
                                    }
                                }
                                set_is_recording.set(config.is_recording);

                                set_is_lobby_enabled.set(config.is_lobby_enabled);
                                set_room_config.set(config);
                            },
                            ServerMessage::Chat { message, .. } => {
                                set_messages.update(|msgs| msgs.push(message));
                            },
                            ServerMessage::ChatHistory(history) => {
                                set_messages.set(history);
                            },
                            ServerMessage::ParticipantJoined(p) => {
                                set_knocking_participants.update(|list| list.retain(|x| x.id != p.id));
                                set_participants.update(|list| {
                                    if !list.iter().any(|x| x.id == p.id) {
                                        list.push(p);
                                    }
                                });
                            },
                            ServerMessage::KnockingParticipantLeft(id) => {
                                set_knocking_participants.update(|list| list.retain(|x| x.id != id));
                            },
                            ServerMessage::ParticipantLeft(id) => {
                                set_participants.update(|list| list.retain(|p| p.id != id));
                                // Remove from typing users if present
                                set_typing_users.update(|users| { users.remove(&id); });
                            },
                            ServerMessage::ParticipantList(list) => {
                                set_participants.set(list);
                            },
                            // RoomUpdated is already handled above in the Welcome block? No, I added it twice by mistake in previous patch or I need to check where I added it.
                            // The previous SEARCH block in `read_file` output showed `RoomUpdated` *after* `ParticipantList`.
                            // But I inserted it *before* `Chat` which is *before* `ParticipantList` in the `Welcome` block in my previous `replace_with_git_merge_diff`.
                            // Wait, the file has a large `match server_msg`.
                            // Let's remove the duplicate `RoomUpdated` if present or just ensure it's handled.
                            // I added one handler near `Welcome`.
                            // The original `RoomUpdated` handler was further down.
                            // Let's check the file content first.
                            ServerMessage::Knocking => {
                                set_current_state.set(RoomConnectionState::Lobby);
                            },
                            ServerMessage::AccessGranted => {
                                set_current_state.set(RoomConnectionState::Joined);
                            },
                            ServerMessage::AccessDenied => {
                                add_toast("Access Denied".to_string(), ToastType::Error);
                                set_current_state.set(RoomConnectionState::Prejoin);
                            },
                            ServerMessage::Kicked(target_id) => {
                                if let Some(my) = my_id.get() {
                                    if my == target_id {
                                        add_toast("You have been kicked from the room.".to_string(), ToastType::Error);
                                        set_current_state.set(RoomConnectionState::Prejoin);
                                        // Close socket?
                                        // The effect cleanup will close it if we navigate away, but here we just change state.
                                        // Ideally we should force close or depend on state change to trigger cleanup if we moved socket creation inside a resource/effect dependent on state.
                                        // For now, state change to Prejoin is enough visual indication.
                                    }
                                }
                            },
                            ServerMessage::RoomEnded => {
                                add_toast("The meeting has ended by the host.".to_string(), ToastType::Info);
                                set_current_state.set(RoomConnectionState::Prejoin);
                                set_participants.set(Vec::new());
                                set_is_connected.set(false);
                            },
                            ServerMessage::KnockingParticipant(p) => {
                                set_knocking_participants.update(|list| {
                                    if !list.iter().any(|x| x.id == p.id) {
                                        list.push(p);
                                    }
                                });
                            },
                            ServerMessage::ParticipantUpdated(p) => {
                                set_participants.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|x| x.id == p.id) {
                                        // Check for hand raise
                                        if p.is_hand_raised && !existing.is_hand_raised {
                                            add_toast(format!("{} raised their hand", p.name), ToastType::Info);
                                        }
                                        *existing = p;
                                    }
                                });
                            },
                            ServerMessage::Reaction { sender_id, emoji } => {
                                set_last_reaction.set(Some((sender_id, emoji, js_sys::Date::now() as u64)));
                            },
                            ServerMessage::PeerTyping { user_id, is_typing, .. } => {
                                set_typing_users.update(|users| {
                                    // Map ID to Name if possible, or just use ID for now.
                                    // Better: store ID in set, and lookup name in `Chat` component.
                                    // Ideally `typing_users` should be `HashSet<String>` (IDs).
                                    // And we need access to `participants` map to get names.
                                    // For now, let's just stick to ID logic, but we might want to expose a helper or just let Chat handle it.
                                    // The current `Chat` implementation iterates `typing_users` and displays them.
                                    // If we want names, we need to pass `participants` to `Chat` too.

                                    if is_typing {
                                        users.insert(user_id);
                                    } else {
                                        users.remove(&user_id);
                                    }
                                });
                            },
                            ServerMessage::BreakoutRoomsList(rooms) => {
                                set_breakout_rooms.set(rooms);
                            },
                            ServerMessage::PollCreated(poll) => {
                                set_polls.update(|list| list.push(poll));
                            },
                            ServerMessage::PollUpdated(poll) => {
                                set_polls.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|x| x.id == poll.id) {
                                        *existing = poll;
                                    }
                                });
                            },
                            ServerMessage::Draw(action) => {
                                set_last_draw_action.set(Some(action.clone()));
                                set_whiteboard_history.update(|h| h.push(action));
                            },
                            ServerMessage::WhiteboardHistory(history) => {
                                set_whiteboard_history.set(history);
                            },
                            ServerMessage::VideoShared(url) => {
                                set_shared_video_url.set(Some(url));
                            },
                            ServerMessage::VideoStopped => {
                                set_shared_video_url.set(None);
                            },
                            ServerMessage::PeerSpeaking { user_id, speaking } => {
                                set_speaking_peers.update(|s| {
                                    if speaking {
                                        s.insert(user_id);
                                    } else {
                                        s.remove(&user_id);
                                    }
                                });
                            },
                            ServerMessage::Error(err) => {
                                add_toast(err, ToastType::Error);
                            }
                        }
                    }
                }
            });
            socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();

            // Handle connection open
            let onopen_callback = Closure::<dyn FnMut()>::new(move || {
                set_is_connected.set(true);
            });
            socket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();

            // Handle connection close
            let onclose_callback = Closure::<dyn FnMut()>::new(move || {
                set_is_connected.set(false);
            });
            socket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();

            // Handle error
            let onerror_callback = Closure::<dyn FnMut()>::new(move || {
                set_is_connected.set(false);
            });
            socket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            set_ws.set(Some(socket));
        }
    });

    on_cleanup(move || {
        if let Some(socket) = ws.get() {
            let _ = socket.close();
        }
    });

    let send_message = Callback::new(move |(content, recipient_id, attachment): (String, Option<String>, Option<FileAttachment>)| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Chat { content, recipient_id, attachment };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_lock = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleRoomLock;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_lobby = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleLobby;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let grant_access = Callback::new(move |id: String| {
        set_knocking_participants.update(|list| list.retain(|p| p.id != id));
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::GrantAccess(id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let deny_access = Callback::new(move |id: String| {
        set_knocking_participants.update(|list| list.retain(|p| p.id != id));
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::DenyAccess(id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_recording = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleRecording;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let save_profile = Callback::new(move |new_name: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::UpdateProfile(new_name);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let send_reaction = Callback::new(move |emoji: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Reaction(emoji);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_raise_hand = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleRaiseHand;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_screen_share = Callback::new(move |_: ()| {
        if local_screen_stream.get().is_some() {
            // Stop sharing
            if let Some(stream) = local_screen_stream.get() {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                        track.stop();
                    }
                }
            }
            set_local_screen_stream.set(None);

            // Notify server stopped
            if let Some(socket) = ws.get() {
                let msg = ClientMessage::ToggleScreenShare;
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send_with_str(&json);
                }
            }
        } else {
            // Start sharing
            spawn_local(async move {
                match get_display_media().await {
                    Ok(stream) => {
                        set_local_screen_stream.set(Some(stream));
                        // Notify server started
                        if let Some(socket) = ws.get() {
                            let msg = ClientMessage::ToggleScreenShare;
                            if let Ok(json) = serde_json::to_string(&msg) {
                                let _ = socket.send_with_str(&json);
                            }
                        }
                    },
                    Err(e) => {
                        web_sys::console::error_1(&e);
                    }
                }
            });
        }
    });

    let create_poll = Callback::new(move |poll: Poll| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::CreatePoll(poll);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let vote_poll = Callback::new(move |(poll_id, option_id): (String, u32)| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Vote { poll_id, option_id };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let send_draw = Callback::new(move |action: DrawAction| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Draw(action);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let join_meeting = Callback::new(move |display_name: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Join(display_name);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let set_is_typing = Callback::new(move |is_typing: bool| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Typing(is_typing);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let create_breakout_room = Callback::new(move |name: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::CreateBreakoutRoom(name);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let join_breakout_room = Callback::new(move |room_id: Option<String>| {
        set_current_room_id.set(room_id.clone());
        // Clear messages when switching rooms? Maybe.
        set_messages.set(Vec::new());

        if let Some(socket) = ws.get() {
            let msg = ClientMessage::JoinBreakoutRoom(room_id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let kick_participant = Callback::new(move |id: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::KickParticipant(id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let end_meeting = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::EndMeeting;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_camera = Callback::new(move |_: ()| {
        if local_stream.get().is_some() {
            // Turn off
            // Stop tracks
            if let Some(stream) = local_stream.get() {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                        track.stop();
                    }
                }
            }
            set_local_stream.set(None);
            set_audio_monitor.set(None);
        } else {
            // Turn on
            spawn_local(async move {
                if let Ok(stream) = get_user_media(None, None).await {
                    set_local_stream.set(Some(stream.clone()));

                    let on_speaking = Box::new(move |is_speaking: bool| {
                        if let Some(socket) = ws.get_untracked() {
                             let msg = ClientMessage::Speaking(is_speaking);
                             if let Ok(json) = serde_json::to_string(&msg) {
                                 let _ = socket.send_with_str(&json);
                             }
                        }
                    });

                    if let Ok(monitor) = AudioMonitor::new(&stream, on_speaking) {
                        set_audio_monitor.set(Some(monitor));
                    }
                }
            });
        }
    });

    let start_share_video = Callback::new(move |url: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::StartShareVideo(url);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let stop_share_video = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::StopShareVideo;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_mic = Callback::new(move |_: ()| {
        let new_state = !is_muted.get();
        set_is_muted.set(new_state);

        if let Some(stream) = local_stream.get() {
            let audio_tracks = stream.get_audio_tracks();
            for i in 0..audio_tracks.length() {
                if let Ok(track) = audio_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.set_enabled(!new_state); // enabled = !muted
                }
            }
        }
    });

    RoomState {
        connection_state: current_state,
        messages,
        participants,
        knocking_participants,
        is_connected,
        is_locked,
        is_lobby_enabled,
        is_recording,
        show_settings,
        show_polls,
        show_shortcuts,
        polls,
        last_reaction,
        show_whiteboard,
        whiteboard_history,
        my_id,
        typing_users,
        is_host,
        host_id,
        current_room_id,
        breakout_rooms,
        local_stream,
        local_screen_stream,
        toasts,
        is_muted,
        shared_video_url,
        speaking_peers,
        set_show_settings,
        set_show_polls,
        set_show_shortcuts,
        set_show_whiteboard,
        send_message,
        toggle_lock,
        toggle_lobby,
        toggle_recording,
        grant_access,
        deny_access,
        join_meeting,
        save_profile,
        send_reaction,
        toggle_raise_hand,
        toggle_screen_share,
        kick_participant,
        create_poll,
        vote_poll,
        send_draw,
        set_is_typing,
        create_breakout_room,
        join_breakout_room,
        toggle_camera,
        toggle_mic,
        dismiss_toast,
        end_meeting,
        start_share_video,
        stop_share_video,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_connection_state_equality() {
        assert_eq!(RoomConnectionState::Prejoin, RoomConnectionState::Prejoin);
        assert_ne!(RoomConnectionState::Prejoin, RoomConnectionState::Joined);
    }
}
