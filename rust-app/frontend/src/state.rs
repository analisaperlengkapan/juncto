use crate::components_ui::toast::{use_toast, ToastType};
use crate::media::{get_display_media, get_user_media, AudioMonitor};
use crate::webrtc::WebRTCManager;
use leptos::*;
use serde::{Deserialize, Serialize};
use shared::{
    ChatMessage, ClientMessage, DrawAction, FileAttachment, Participant, Poll, ServerMessage,
};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MediaStream, MessageEvent, WebSocket};

#[derive(Clone, PartialEq, Debug)]
pub enum RoomConnectionState {
    Prejoin,
    Lobby,
    Joined,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JoinOptions {
    pub display_name: String,
    pub mic_enabled: bool,
    pub camera_enabled: bool,
    pub audio_device_id: Option<String>,
    pub video_device_id: Option<String>,
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
    pub is_muted: ReadSignal<bool>,
    pub shared_video_url: ReadSignal<Option<String>>,
    pub speaking_peers: ReadSignal<HashSet<String>>,
    pub show_speaker_stats: ReadSignal<bool>,
    pub show_virtual_background: ReadSignal<bool>,
    pub show_feedback: ReadSignal<bool>,
    pub rtt: ReadSignal<u64>,
    pub remote_streams: ReadSignal<HashMap<String, MediaStream>>,
    #[allow(dead_code)]
    pub selected_camera_id: ReadSignal<Option<String>>,
    #[allow(dead_code)]
    pub selected_mic_id: ReadSignal<Option<String>>,
    #[allow(dead_code)]
    pub video_resolution: ReadSignal<String>,
    // Setters or Actions
    pub set_input_devices: Callback<(Option<String>, Option<String>, String)>,
    pub set_show_settings: WriteSignal<bool>,
    pub set_show_polls: WriteSignal<bool>,
    pub set_show_shortcuts: WriteSignal<bool>,
    pub set_show_whiteboard: WriteSignal<bool>,
    pub set_show_speaker_stats: WriteSignal<bool>,
    pub set_show_virtual_background: WriteSignal<bool>,
    pub set_show_feedback: WriteSignal<bool>,
    pub send_ping: Callback<()>,
    pub send_message: Callback<(String, Option<String>, Option<FileAttachment>)>, // content, recipient_id, attachment
    pub start_share_video: Callback<String>,
    pub stop_share_video: Callback<()>,
    pub toggle_lock: Callback<()>,
    pub toggle_lobby: Callback<()>,
    pub toggle_recording: Callback<()>,
    pub grant_access: Callback<String>,
    pub deny_access: Callback<String>,
    pub join_meeting: Callback<JoinOptions>,
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
    pub end_meeting: Callback<()>,
    pub mute_participant: Callback<String>,
    pub transfer_host: Callback<String>,
}

pub fn use_room_state() -> RoomState {
    let toast_ctx = use_toast();
    let (current_state, set_current_state) = create_signal(RoomConnectionState::Prejoin);
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (typing_users, set_typing_users) = create_signal(HashSet::<String>::new());
    let (breakout_rooms, set_breakout_rooms) = create_signal(Vec::<shared::BreakoutRoom>::new());
    let (current_room_id, set_current_room_id) = create_signal(None::<String>);
    let (participants, set_participants) = create_signal(Vec::<Participant>::new());
    let (knocking_participants, set_knocking_participants) =
        create_signal(Vec::<Participant>::new());
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
    let (is_muted, set_is_muted) = create_signal(false);
    let (shared_video_url, set_shared_video_url) = create_signal(None::<String>);
    let (speaking_peers, set_speaking_peers) = create_signal(HashSet::<String>::new());
    let (_audio_monitor, set_audio_monitor) = create_signal(None::<AudioMonitor>);
    let (show_speaker_stats, set_show_speaker_stats) = create_signal(false);
    let (show_virtual_background, set_show_virtual_background) = create_signal(false);
    let (show_feedback, set_show_feedback) = create_signal(false);
    let (rtt, set_rtt) = create_signal(0u64);
    let (last_ping_time, set_last_ping_time) = create_signal(0f64);
    let (selected_camera_id, set_selected_camera_id) = create_signal(None::<String>);
    let (selected_mic_id, set_selected_mic_id) = create_signal(None::<String>);
    let (video_resolution, set_video_resolution) = create_signal("hd".to_string());

    let (remote_streams, set_remote_streams) = create_signal(HashMap::<String, MediaStream>::new());

    // WebRTC Manager Setup
    let ws_clone_for_webrtc = ws;
    let send_signal_cb = move |msg: ClientMessage| {
        if let Some(socket) = ws_clone_for_webrtc.get_untracked() {
             if let Ok(json) = serde_json::to_string(&msg) {
                 let _ = socket.send_with_str(&json);
             }
        }
    };

    let on_track_cb = move |peer_id: String, stream: MediaStream| {
        set_remote_streams.update(|map| {
            map.insert(peer_id, stream);
        });
    };

    let webrtc_manager = WebRTCManager::new(
        send_signal_cb,
        on_track_cb,
        local_stream.into(),
        local_screen_stream.into(),
        my_id.into(),
    );

    // Dynamic Stream Handling: Initiate connections when local stream becomes available
    let participants_for_effect = participants.clone();
    let my_id_for_effect = my_id.clone();
    let webrtc_manager_clone = webrtc_manager.clone();

    create_effect(move |_| {
        if local_stream.get().is_some() {
            let my_id_val = my_id_for_effect.get_untracked();
            if let Some(me) = my_id_val {
                // Update tracks for existing connections (e.g. from incoming offers)
                webrtc_manager_clone.update_local_tracks();

                let list = participants_for_effect.get_untracked();
                for p in list {
                    if p.id != me {
                        // Only initiate connection if one doesn't exist
                        if !webrtc_manager_clone.has_peer(&p.id) {
                            webrtc_manager_clone.handle_participant_joined(p.id);
                        }
                    }
                }
            }
        }
    });

    // Internal state to trigger media start after joining
    let (start_media_on_join, set_start_media_on_join) = create_signal(false);
    let (initial_cam_on, set_initial_cam_on) = create_signal(false);

    // We assume the first participant in the list is the host for now,
    // or we'd need to send host_id in RoomConfig.
    // The previous implementation used host_id in backend but didn't expose it to frontend.
    // Let's rely on backend RoomUpdated message.
    // BUT, RoomConfig struct in shared was updated to include host_id.
    // So we can extract it from there.

    // We need to store the current room config to access host_id.
    let (room_config, set_room_config) = create_signal(shared::RoomConfig::default());

    let host_id = Signal::derive(move || room_config.get().host_id);

    let is_host = Signal::derive(move || {
        let h = host_id.get();
        let my = my_id.get();

        match (h, my) {
            (Some(host), Some(me)) => host == me,
            _ => false,
        }
    });

    let add_toast = move |msg: String, type_: ToastType| {
        toast_ctx.add(msg, type_);
    };

    // Extract start_media_stream logic
    let start_media_stream = Callback::new(move |enable_video: bool| {
        // Stop existing stream tracks
        if let Some(stream) = local_stream.get_untracked() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        // Don't set to None immediately to avoid flicker if possible, but for clean state -> None
        set_local_stream.set(None);
        set_audio_monitor.set(None);

        spawn_local(async move {
            let v_id = selected_camera_id.get_untracked();
            let a_id = selected_mic_id.get_untracked();
            let res = video_resolution.get_untracked();

            // Always request audio (true) unless explicitly unwanted, but here we assume "meeting" means audio capacity.
            // If user is muted, we still request audio but mute track.
            // If user has NO microphone, this might fail?
            // Assuming typical WebRTC flow: request audio=true.

            if let Ok(stream) = get_user_media(enable_video, true, v_id, a_id, Some(&res)).await {
                // Apply existing mute state to new stream
                if is_muted.get_untracked() {
                    let audio_tracks = stream.get_audio_tracks();
                    for i in 0..audio_tracks.length() {
                        if let Ok(track) =
                            audio_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>()
                        {
                            track.set_enabled(false);
                        }
                    }
                }

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
    });

    // Initialize WebSocket
    let webrtc_manager_for_ws = webrtc_manager.clone();
    create_effect(move |_| {
        // Ensure my_id is reset on new connection logic if needed, but here we just connect.
        // Actually, if we reconnect, we might get a new ID.
        set_my_id.set(None);
        // Default config has host_id = None.
        set_room_config.set(shared::RoomConfig::default());

        // Reset host signal to false until we get new data
        // Derived signal updates automatically based on deps.

        let location = web_sys::window().unwrap().location();
        let protocol = if location.protocol().unwrap() == "https:" {
            "wss:"
        } else {
            "ws:"
        };
        let host = location.host().unwrap();
        let url = format!("{}//{}/ws/chat", protocol, host);

        if let Ok(socket) = WebSocket::new(&url) {
            // Handle incoming messages
            let webrtc_manager = webrtc_manager_for_ws.clone();
            let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let txt: String = txt.into();
                    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                        match server_msg {
                            ServerMessage::Welcome { id } => {
                                set_my_id.set(Some(id));
                                set_current_state.set(RoomConnectionState::Joined);

                                // Auto-start media if requested from prejoin
                                if start_media_on_join.get_untracked() {
                                    start_media_stream.call(initial_cam_on.get_untracked());
                                    set_start_media_on_join.set(false);
                                }

                                // Sync mute state after joining (important for Lobby flow)
                                if is_muted.get_untracked() {
                                    if let Some(socket) = ws.get_untracked() {
                                        let msg = ClientMessage::SetMuteStatus(true);
                                        if let Ok(json) = serde_json::to_string(&msg) {
                                            let _ = socket.send_with_str(&json);
                                        }
                                    }
                                }
                            }
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
                            }
                            ServerMessage::Chat { message, .. } => {
                                set_messages.update(|msgs| msgs.push(message));
                            }
                            ServerMessage::ChatHistory(history) => {
                                set_messages.set(history);
                            }
                            ServerMessage::ParticipantJoined(p) => {
                                set_knocking_participants
                                    .update(|list| list.retain(|x| x.id != p.id));
                                set_participants.update(|list| {
                                    if !list.iter().any(|x| x.id == p.id) {
                                        list.push(p.clone());
                                    }
                                });
                                // Initiate WebRTC connection (Polite Peer)
                                // Only connect if it's NOT me AND local stream is ready
                                if my_id.get_untracked() != Some(p.id.clone()) {
                                    if local_stream.get_untracked().is_some() && !webrtc_manager.has_peer(&p.id) {
                                        webrtc_manager.handle_participant_joined(p.id);
                                    }
                                }
                                }
                            }
                            ServerMessage::KnockingParticipantLeft(id) => {
                                set_knocking_participants
                                    .update(|list| list.retain(|x| x.id != id));
                            }
                            ServerMessage::ParticipantLeft(id) => {
                                set_participants.update(|list| list.retain(|p| p.id != id));
                                // Remove from typing users if present
                                set_typing_users.update(|users| {
                                    users.remove(&id);
                                });
                                // Cleanup WebRTC
                                webrtc_manager.handle_participant_left(&id);
                                set_remote_streams.update(|map| {
                                    map.remove(&id);
                                });
                            }
                            ServerMessage::ParticipantList(list) => {
                                set_participants.set(list);
                            }
                            ServerMessage::Knocking => {
                                set_current_state.set(RoomConnectionState::Lobby);
                            }
                            ServerMessage::AccessDenied => {
                                add_toast("Access Denied".to_string(), ToastType::Error);
                                set_current_state.set(RoomConnectionState::Prejoin);
                            }
                            ServerMessage::Kicked(target_id) => {
                                if let Some(my) = my_id.get() {
                                    if my == target_id {
                                        add_toast(
                                            "You have been kicked from the room.".to_string(),
                                            ToastType::Error,
                                        );
                                        set_current_state.set(RoomConnectionState::Prejoin);
                                    }
                                }
                            }
                            ServerMessage::MutedByHost(target_id) => {
                                if let Some(my) = my_id.get() {
                                    if my == target_id {
                                        add_toast(
                                            "You have been muted by the host.".to_string(),
                                            ToastType::Info,
                                        );
                                        set_is_muted.set(true);
                                        if let Some(stream) = local_stream.get_untracked() {
                                            let audio_tracks = stream.get_audio_tracks();
                                            for i in 0..audio_tracks.length() {
                                                if let Ok(track) = audio_tracks
                                                    .get(i)
                                                    .dyn_into::<web_sys::MediaStreamTrack>(
                                                ) {
                                                    track.set_enabled(false);
                                                }
                                            }
                                        }
                                        // Confirm state to server
                                        if let Some(socket) = ws.get_untracked() {
                                            let msg = ClientMessage::SetMuteStatus(true);
                                            if let Ok(json) = serde_json::to_string(&msg) {
                                                let _ = socket.send_with_str(&json);
                                            }
                                        }
                                    }
                                }
                            }
                            ServerMessage::RoomEnded => {
                                add_toast(
                                    "The meeting has ended by the host.".to_string(),
                                    ToastType::Info,
                                );
                                set_current_state.set(RoomConnectionState::Prejoin);
                                set_participants.set(Vec::new());
                                set_is_connected.set(false);
                            }
                            ServerMessage::KnockingParticipant(p) => {
                                set_knocking_participants.update(|list| {
                                    if !list.iter().any(|x| x.id == p.id) {
                                        list.push(p);
                                    }
                                });
                            }
                            ServerMessage::ParticipantUpdated(p) => {
                                set_participants.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|x| x.id == p.id) {
                                        // Check for hand raise
                                        if p.is_hand_raised && !existing.is_hand_raised {
                                            add_toast(
                                                format!("{} raised their hand", p.name),
                                                ToastType::Info,
                                            );
                                        }
                                        *existing = p;
                                    }
                                });
                            }
                            ServerMessage::Reaction { sender_id, emoji } => {
                                set_last_reaction.set(Some((
                                    sender_id,
                                    emoji,
                                    js_sys::Date::now() as u64,
                                )));
                            }
                            ServerMessage::PeerTyping {
                                user_id, is_typing, ..
                            } => {
                                set_typing_users.update(|users| {
                                    if is_typing {
                                        users.insert(user_id);
                                    } else {
                                        users.remove(&user_id);
                                    }
                                });
                            }
                            ServerMessage::BreakoutRoomsList(rooms) => {
                                set_breakout_rooms.set(rooms);
                            }
                            ServerMessage::PollCreated(poll) => {
                                set_polls.update(|list| {
                                    if !list.iter().any(|p| p.id == poll.id) {
                                        list.push(poll);
                                    }
                                });
                            }
                            ServerMessage::PollUpdated(poll) => {
                                set_polls.update(|list| {
                                    if let Some(existing) =
                                        list.iter_mut().find(|x| x.id == poll.id)
                                    {
                                        *existing = poll;
                                    }
                                });
                            }
                            ServerMessage::PollsList(list) => {
                                set_polls.set(list);
                            }
                            ServerMessage::Draw(action) => {
                                set_last_draw_action.set(Some(action.clone()));
                                set_whiteboard_history.update(|h| h.push(action));
                            }
                            ServerMessage::WhiteboardHistory(history) => {
                                set_whiteboard_history.set(history);
                            }
                            ServerMessage::VideoShared(url) => {
                                set_shared_video_url.set(Some(url));
                            }
                            ServerMessage::VideoStopped => {
                                set_shared_video_url.set(None);
                            }
                            ServerMessage::PeerSpeaking { user_id, speaking } => {
                                set_speaking_peers.update(|s| {
                                    if speaking {
                                        s.insert(user_id);
                                    } else {
                                        s.remove(&user_id);
                                    }
                                });
                            }
                            ServerMessage::Pong { .. } => {
                                let now = js_sys::Date::now();
                                let start = last_ping_time.get_untracked();
                                if start > 0.0 {
                                    let latency = (now - start) as u64;
                                    set_rtt.set(latency);
                                }
                            }
                            ServerMessage::Error(err) => {
                                add_toast(err, ToastType::Error);
                            }
                            ServerMessage::Offer { source_id, sdp, .. } => {
                                // Always handle offer even if local stream is not ready.
                                // We can answer without local tracks, and add them later via update_local_tracks()
                                // when the stream becomes available (handled by the create_effect above).
                                webrtc_manager.handle_offer(source_id, sdp);
                            }
                            ServerMessage::Answer { source_id, sdp, .. } => {
                                webrtc_manager.handle_answer(source_id, sdp);
                            }
                            ServerMessage::IceCandidate { source_id, candidate, sdp_mid, sdp_m_line_index, .. } => {
                                webrtc_manager.handle_ice_candidate(source_id, candidate, sdp_mid, sdp_m_line_index);
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

    let send_message = Callback::new(
        move |(content, recipient_id, attachment): (
            String,
            Option<String>,
            Option<FileAttachment>,
        )| {
            if let Some(socket) = ws.get() {
                let msg = ClientMessage::Chat {
                    content,
                    recipient_id,
                    attachment,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send_with_str(&json);
                }
            }
        },
    );

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

    let webrtc_manager_for_screen = webrtc_manager.clone();
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
            webrtc_manager_for_screen.update_local_tracks();

            // Notify server stopped
            if let Some(socket) = ws.get() {
                let msg = ClientMessage::ToggleScreenShare;
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send_with_str(&json);
                }
            }
        } else {
            // Start sharing
            let webrtc_manager_for_spawn = webrtc_manager_for_screen.clone();
            spawn_local(async move {
                match get_display_media().await {
                    Ok(stream) => {
                        set_local_screen_stream.set(Some(stream));
                        webrtc_manager_for_spawn.update_local_tracks();

                        // Notify server started
                        if let Some(socket) = ws.get() {
                            let msg = ClientMessage::ToggleScreenShare;
                            if let Ok(json) = serde_json::to_string(&msg) {
                                let _ = socket.send_with_str(&json);
                            }
                        }
                    }
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

    let join_meeting = Callback::new(move |options: JoinOptions| {
        // Set initial state
        set_is_muted.set(!options.mic_enabled);
        set_selected_mic_id.set(options.audio_device_id);
        set_selected_camera_id.set(options.video_device_id);

        // Start media if either mic or cam is on
        set_start_media_on_join.set(options.mic_enabled || options.camera_enabled);
        set_initial_cam_on.set(options.camera_enabled);

        let display_name = options.display_name;

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

    let mute_participant = Callback::new(move |id: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::MuteParticipant(id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let transfer_host = Callback::new(move |id: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::TransferHost(id);
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
        // Check if we currently have video tracks active
        let has_video = if let Some(stream) = local_stream.get_untracked() {
            stream.get_video_tracks().length() > 0
        } else {
            false
        };
        start_media_stream.call(!has_video);
    });

    let set_input_devices = Callback::new(
        move |(vid, aid, res): (Option<String>, Option<String>, String)| {
            set_selected_camera_id.set(vid.clone());
            set_selected_mic_id.set(aid.clone());
            set_video_resolution.set(res.clone());

            // Use same logic as toggle_camera: preserve video state?
            // Or if user selected a camera, enable video?
            // Usually settings dialog allows selecting device. If "None" is passed for video, maybe disable?
            // But settings dialog passes "current selection".
            // Let's assume if stream is running, we restart it with same video state (unless video was disabled? No, if stream running, restart with current capabilities).
            // Actually, simpler: check if video is currently running.
            let has_video = if let Some(stream) = local_stream.get_untracked() {
                stream.get_video_tracks().length() > 0
            } else {
                false // If no stream, maybe don't start one? Or start if devices selected?
                      // If user explicitly changes devices in settings, they probably want to see them.
                      // But if they were Audio Only, and changed Mic...
                      // Let's stick to: if stream exists, restart with current video state.
            };

            if local_stream.get_untracked().is_some() {
                start_media_stream.call(has_video);
            }
        },
    );

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

        if let Some(socket) = ws.get() {
            let msg = ClientMessage::SetMuteStatus(new_state);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let send_ping = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            set_last_ping_time.set(js_sys::Date::now());
            let msg = ClientMessage::Ping;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
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
        is_muted,
        shared_video_url,
        speaking_peers,
        show_speaker_stats,
        show_virtual_background,
        show_feedback,
        rtt,
        remote_streams,
        selected_camera_id,
        selected_mic_id,
        video_resolution,
        set_input_devices,
        set_show_settings,
        set_show_polls,
        set_show_shortcuts,
        set_show_whiteboard,
        set_show_speaker_stats,
        set_show_virtual_background,
        set_show_feedback,
        send_ping,
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
        end_meeting,
        mute_participant,
        transfer_host,
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
