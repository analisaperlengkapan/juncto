use leptos::*;
use shared::{ServerMessage, ChatMessage, Participant, Poll, DrawAction, ClientMessage};
use std::collections::{HashMap, HashSet};
use web_sys::{MediaStream, WebSocket};
use crate::components_ui::toast::{ToastType};
use crate::analytics::AnalyticsService;
use crate::webrtc::WebRTCManager;
use crate::media::AudioMonitor;
use crate::remote_control::RemoteControlService;
use crate::state::RoomConnectionState;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;

#[derive(Clone)]
pub struct HandlerContext {
    pub set_my_id: WriteSignal<Option<String>>,
    pub set_current_state: WriteSignal<RoomConnectionState>,
    pub analytics: AnalyticsService,
    pub start_media_on_join: ReadSignal<bool>,
    pub initial_cam_on: ReadSignal<bool>,
    pub start_media_stream: Callback<bool>,
    pub set_start_media_on_join: WriteSignal<bool>,
    pub is_muted: ReadSignal<bool>,
    pub ws: ReadSignal<Option<WebSocket>>,
    pub local_stream: ReadSignal<Option<MediaStream>>,
    pub raw_local_stream: ReadSignal<Option<MediaStream>>,
    pub add_toast: Callback<(String, ToastType)>,
    pub set_is_camera_off: WriteSignal<bool>,
    pub room_config: ReadSignal<shared::RoomConfig>,
    pub set_show_etherpad: WriteSignal<bool>,
    pub set_is_locked: WriteSignal<bool>,
    pub set_is_e2ee_enabled: WriteSignal<bool>,
    pub is_recording: ReadSignal<bool>,
    pub set_is_recording: WriteSignal<bool>,
    pub set_is_lobby_enabled: WriteSignal<bool>,
    pub is_subtitles_enabled: ReadSignal<bool>,
    pub set_is_subtitles_enabled: WriteSignal<bool>,
    pub set_subtitles: WriteSignal<Vec<(String, String, u64)>>,
    pub set_room_config: WriteSignal<shared::RoomConfig>,
    pub current_room_id: ReadSignal<Option<String>>,
    pub set_messages: WriteSignal<Vec<ChatMessage>>,
    pub set_is_connected: WriteSignal<bool>,
    pub set_knocking_participants: WriteSignal<Vec<Participant>>,
    pub set_participants: WriteSignal<Vec<Participant>>,
    pub my_id: ReadSignal<Option<String>>,
    pub webrtc_manager: WebRTCManager,
    pub set_typing_users: WriteSignal<HashSet<String>>,
    pub set_speaking_peers: WriteSignal<HashSet<String>>,
    pub set_power_statuses: WriteSignal<HashMap<String, shared::PowerStatus>>,
    pub set_remote_streams: WriteSignal<HashMap<String, Vec<MediaStream>>>,
    pub is_recording_locally: ReadSignal<bool>,
    pub local_recorder: Rc<RefCell<Option<crate::media_recorder::LocalRecorder>>>,
    pub pending_recorders: Rc<RefCell<Vec<crate::media_recorder::LocalRecorder>>>,
    pub recording_stream_id: Rc<RefCell<Option<String>>>,
    pub set_is_recording_locally: WriteSignal<bool>,
    pub set_is_muted: WriteSignal<bool>,
    pub set_audio_monitor: WriteSignal<Option<AudioMonitor>>,
    pub participants: ReadSignal<Vec<Participant>>,
    pub set_last_reaction: WriteSignal<Option<(String, String, u64)>>,
    pub set_breakout_rooms: WriteSignal<Vec<shared::BreakoutRoom>>,
    pub set_polls: WriteSignal<Vec<Poll>>,
    pub set_grid_layout_sig: WriteSignal<String>,
    pub set_whiteboard_history: WriteSignal<Vec<DrawAction>>,
    pub set_last_draw_action: WriteSignal<Option<DrawAction>>,
    pub set_shared_video_url: WriteSignal<Option<String>>,
    pub last_ping_time: ReadSignal<f64>,
    pub set_rtt: WriteSignal<u64>,
    pub set_is_authenticated: WriteSignal<bool>,
    pub set_show_login_dialog: WriteSignal<bool>,
    pub set_auth_error: WriteSignal<Option<String>>,
    pub set_calendar_events: WriteSignal<Vec<String>>,
    pub set_lobby_announcement: WriteSignal<Option<String>>,
    pub set_face_expression: WriteSignal<Option<(String, String, u64)>>,
    pub remote_control: RemoteControlService,
}

pub fn handle_server_message(server_msg: ServerMessage, ctx: &HandlerContext) {
    match server_msg {
        ServerMessage::Welcome { id } => {
            ctx.set_my_id.set(Some(id.clone()));
            ctx.set_current_state.set(RoomConnectionState::Joined);

            // Track join
            ctx.analytics.track_join(&id);

            // Persist the meeting room to recent rooms only after the server
            // confirms the join. This avoids adding rooms that rejected the
            // join attempt (locked, full, denied, etc.) to the user's list.
            if let Some(window) = web_sys::window() {
                if let Ok(pathname) = window.location().pathname() {
                    if let Some(rest) = pathname.strip_prefix("/room/") {
                        let room_id = rest.split('/').next().unwrap_or(rest);
                        if !room_id.is_empty() {
                            let decoded = urlencoding::decode(room_id)
                                .map(|s| s.into_owned())
                                .unwrap_or_else(|_| room_id.to_string());
                            crate::storage::add_recent_room(decoded);
                        }
                    }
                }
            }

            // Auto-start media if requested from prejoin
            if ctx.start_media_on_join.get_untracked() {
                ctx.start_media_stream.call(ctx.initial_cam_on.get_untracked());
                ctx.set_start_media_on_join.set(false);
            }

            // Sync mute state after joining (important for Lobby flow)
            if ctx.is_muted.get_untracked() {
                if let Some(socket) = ctx.ws.get_untracked() {
                    let msg = ClientMessage::SetMuteStatus(true);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = socket.send_with_str(&json);
                    }
                }
            }
        }
        ServerMessage::CameraMutedByHost(target_id) => {
            if let Some(my) = ctx.my_id.get_untracked() {
                if my == target_id {
                    let has_video = ctx.local_stream.with_untracked(|s| {
                        s.as_ref().is_some_and(|stream| stream.get_video_tracks().length() > 0)
                    });
                    if has_video {
                        ctx.add_toast.call((
                            "Your camera has been disabled by the host.".to_string(),
                            ToastType::Info,
                        ));
                        ctx.set_is_camera_off.set(true);
                        if let Some(raw) = ctx.raw_local_stream.get_untracked() {
                            let video_tracks = raw.get_video_tracks();
                            for i in 0..video_tracks.length() {
                                if let Ok(track) = video_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                                    track.set_enabled(false);
                                }
                            }
                        }
                        if let Some(stream) = ctx.local_stream.get_untracked() {
                            let video_tracks = stream.get_video_tracks();
                            for i in 0..video_tracks.length() {
                                if let Ok(track) = video_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                                    track.set_enabled(false);
                                }
                            }
                        }
                    }
                }
            }
        }
        ServerMessage::RoomUpdated(config) => {
            let old_etherpad_url = ctx.room_config.with_untracked(|c| c.etherpad_url.clone());
            if config.etherpad_url != old_etherpad_url {
                ctx.set_show_etherpad.set(config.etherpad_url.is_some());
            }

            ctx.set_is_locked.set(config.is_locked);
            ctx.set_is_e2ee_enabled.set(config.e2ee_enabled);

            let was_recording = ctx.is_recording.get_untracked();
            if config.is_recording != was_recording && ctx.my_id.get_untracked().is_some() {
                if config.is_recording {
                    ctx.add_toast.call(("Recording Started".to_string(), ToastType::Info));
                } else {
                    ctx.add_toast.call(("Recording Stopped".to_string(), ToastType::Info));
                }
            }
            ctx.set_is_recording.set(config.is_recording);

            ctx.set_is_lobby_enabled.set(config.is_lobby_enabled);

            if !config.is_subtitles_enabled && ctx.is_subtitles_enabled.get_untracked() {
                ctx.set_subtitles.set(Vec::new());
            }
            ctx.set_is_subtitles_enabled.set(config.is_subtitles_enabled);

            ctx.set_room_config.set(config);
        }
        ServerMessage::Chat { message, room_id } => {
            let current_room = ctx.current_room_id.get_untracked();
            if room_id == current_room {
                ctx.set_messages.update(|msgs| msgs.push(message));
            }
        }
        ServerMessage::ChatHistory(history) => {
            if ctx.current_room_id.get_untracked().is_none() {
                ctx.set_messages.set(history);
            }
        }
        ServerMessage::ParticipantJoined(p) => {
            ctx.set_knocking_participants.update(|list| list.retain(|x| x.id != p.id));
            ctx.set_participants.update(|list| {
                if !list.iter().any(|x| x.id == p.id) {
                    list.push(p.clone());
                }
            });

            if let Some(me) = ctx.my_id.get_untracked() {
                if me != p.id && me > p.id {
                    ctx.webrtc_manager.handle_participant_joined(p.id);
                }
            }
        }
        ServerMessage::KnockingParticipantLeft(id) => {
            ctx.set_knocking_participants.update(|list| list.retain(|x| x.id != id));
        }
        ServerMessage::ParticipantLeft { id, .. } => {
            ctx.set_participants.update(|list| list.retain(|p| p.id != id));
            ctx.set_typing_users.update(|users| {
                users.remove(&id);
            });
            ctx.set_speaking_peers.update(|s| {
                s.remove(&id);
            });
            ctx.set_power_statuses.update(|map| {
                map.remove(&id);
            });
            // If we were remote-controlling this peer, clear the overlay so
            // we don't keep capturing input for a peer that no longer exists.
            if ctx.remote_control.controlled_peer.get_untracked().as_deref() == Some(&id) {
                ctx.remote_control.set_controlled_peer(None);
                ctx.add_toast.call((
                    "Remote control session ended (peer disconnected)".to_string(),
                    ToastType::Info,
                ));
            }
            ctx.webrtc_manager.handle_participant_left(&id);
            ctx.set_remote_streams.update(|map| {
                map.remove(&id);
            });
        }
        ServerMessage::ParticipantList(list) => {
            ctx.set_participants.set(list.clone());

            if let Some(me) = ctx.my_id.get_untracked() {
                for p in list {
                    if me > p.id {
                        ctx.webrtc_manager.handle_participant_joined(p.id);
                    }
                }
            }
        }
        ServerMessage::Knocking => {
            ctx.set_current_state.set(RoomConnectionState::Lobby);
        }
        ServerMessage::AccessDenied => {
            ctx.add_toast.call(("Access Denied".to_string(), ToastType::Error));
            ctx.set_current_state.set(RoomConnectionState::Prejoin);
        }
        ServerMessage::Kicked { target_id, .. } => {
            if let Some(my) = ctx.my_id.get_untracked() {
                if my == target_id {
                    ctx.add_toast.call((
                        "You have been kicked from the room.".to_string(),
                        ToastType::Error,
                    ));
                    ctx.webrtc_manager.close_all_peers();
                    ctx.set_remote_streams.set(HashMap::new());
                    ctx.set_power_statuses.set(HashMap::new());
                    if ctx.is_recording_locally.get_untracked() {
                        if let Some(r) = ctx.local_recorder.borrow_mut().take() {
                            r.stop();
                            ctx.pending_recorders.borrow_mut().push(r);
                        }
                        *ctx.recording_stream_id.borrow_mut() = None;
                        ctx.set_is_recording_locally.set(false);
                    }
                    ctx.set_current_state.set(RoomConnectionState::Prejoin);

                    set_timeout(
                        move || {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/");
                            }
                        },
                        std::time::Duration::from_millis(1500),
                    );
                }
            }
        }
        ServerMessage::MutedByHost(target_id) => {
            if let Some(my) = ctx.my_id.get_untracked() {
                if my == target_id {
                    ctx.add_toast.call((
                        "You have been muted by the host.".to_string(),
                        ToastType::Info,
                    ));
                    ctx.set_is_muted.set(true);
                    if let Some(stream) = ctx.local_stream.get_untracked() {
                        let audio_tracks = stream.get_audio_tracks();
                        for i in 0..audio_tracks.length() {
                            if let Ok(track) = audio_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                                track.set_enabled(false);
                            }
                        }

                        ctx.set_audio_monitor.update(|monitor| {
                            if let Some(m) = monitor.as_mut() {
                                m.set_muted(true);
                            }
                        });
                    }
                }
            }
        }
        ServerMessage::RoomEnded => {
            ctx.add_toast.call((
                "The meeting has ended by the host.".to_string(),
                ToastType::Info,
            ));
            ctx.webrtc_manager.close_all_peers();
            ctx.set_remote_streams.set(HashMap::new());
            ctx.set_current_state.set(RoomConnectionState::Prejoin);
            ctx.set_participants.set(Vec::new());
            ctx.set_power_statuses.set(HashMap::new());
            if ctx.is_recording_locally.get_untracked() {
                if let Some(r) = ctx.local_recorder.borrow_mut().take() {
                    r.stop();
                    ctx.pending_recorders.borrow_mut().push(r);
                }
                *ctx.recording_stream_id.borrow_mut() = None;
                ctx.set_is_recording_locally.set(false);
            }
            ctx.set_is_connected.set(false);

            set_timeout(
                move || {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                },
                std::time::Duration::from_millis(1500),
            );
        }
        ServerMessage::KnockingParticipant(p) => {
            ctx.set_knocking_participants.update(|list| {
                if !list.iter().any(|x| x.id == p.id) {
                    list.push(p);
                }
            });
        }
        ServerMessage::ParticipantUpdated(p) => {
            ctx.set_participants.update(|list| {
                if let Some(existing) = list.iter_mut().find(|x| x.id == p.id) {
                    if p.is_hand_raised && !existing.is_hand_raised {
                        ctx.add_toast.call((
                            format!("{} raised their hand", p.name),
                            ToastType::Info,
                        ));
                    }
                    *existing = p;
                }
            });
        }
        ServerMessage::Reaction { sender_id, emoji } => {
            ctx.set_last_reaction.set(Some((
                sender_id,
                emoji,
                js_sys::Date::now() as u64,
            )));
        }
        ServerMessage::PeerTyping {
            user_id, is_typing, ..
        } => {
            ctx.set_typing_users.update(|users| {
                if is_typing {
                    users.insert(user_id);
                } else {
                    users.remove(&user_id);
                }
            });
        }
        ServerMessage::BreakoutRoomsList(rooms) => {
            ctx.set_breakout_rooms.set(rooms);
        }
        ServerMessage::PollCreated(poll) => {
            ctx.set_polls.update(|list| {
                if !list.iter().any(|p| p.id == poll.id) {
                    list.push(poll);
                }
            });
        }
        ServerMessage::PollUpdated(poll) => {
            ctx.set_polls.update(|list| {
                if let Some(existing) =
                    list.iter_mut().find(|x| x.id == poll.id)
                {
                    *existing = poll;
                }
            });
        }
        ServerMessage::PollClosed(poll_id) => {
            ctx.set_polls.update(|list| {
                if let Some(existing) = list.iter_mut().find(|x| x.id == poll_id) {
                    existing.is_closed = true;
                }
            });
        }
        ServerMessage::FollowMe(layout) => {
            ctx.set_grid_layout_sig.set(layout);
        }
        ServerMessage::FaceExpression { sender_id, expression } => {
            ctx.set_face_expression.set(Some((sender_id, expression.expression, expression.timestamp)));
        }
        ServerMessage::RemoteControlRequest { requester_id, target_id } => {
            if let Some(my) = ctx.my_id.get_untracked() {
                if my == target_id {
                    let parts = ctx.participants.get_untracked();
                    let name = parts.iter().find(|p| p.id == requester_id).map(|p| p.name.clone()).unwrap_or(requester_id.clone());

                    // Require explicit user consent before granting remote
                    // control. Falls back to deny if no window is available
                    // or the user dismisses the prompt.
                    let granted = web_sys::window()
                        .and_then(|w| {
                            w.confirm_with_message(&format!(
                                "{} is requesting remote control of your session. Allow?",
                                name
                            ))
                            .ok()
                        })
                        .unwrap_or(false);

                    let msg = if granted {
                        ctx.add_toast.call(("Remote control granted".to_string(), ToastType::Info));
                        ClientMessage::GrantRemoteControl(requester_id)
                    } else {
                        ctx.add_toast.call(("Remote control denied".to_string(), ToastType::Info));
                        ClientMessage::DenyRemoteControl(requester_id)
                    };
                    ctx.remote_control.send_signal.call(msg);
                }
            }
        }
        ServerMessage::RemoteControlAllowed { requester_id, target_id, allowed } => {
            if let Some(my) = ctx.my_id.get_untracked() {
                if my == requester_id {
                    if allowed {
                        ctx.remote_control.set_controlled_peer(Some(target_id));
                        ctx.add_toast.call(("Remote control granted".to_string(), ToastType::Success));
                    } else {
                        ctx.add_toast.call(("Remote control denied".to_string(), ToastType::Error));
                    }
                }
            }
        }
        ServerMessage::RemoteControlStopped { sender_id, peer_id } => {
            // Clear the overlay if we're the controller and the controlled
            // party ended the session.
            if ctx.remote_control.controlled_peer.get_untracked() == Some(sender_id.clone()) {
                ctx.remote_control.set_controlled_peer(None);
                ctx.add_toast.call(("Remote control session ended".to_string(), ToastType::Info));
            } else if let Some(my) = ctx.my_id.get_untracked() {
                // If we're the controlled party (i.e. `peer_id` identifies us
                // and the controller stopped the session), notify us too.
                if my == peer_id && my != sender_id {
                    ctx.add_toast.call(("Remote control session ended".to_string(), ToastType::Info));
                }
            }
        }
        ServerMessage::RemoteControlAction { .. } => {
            // In a real app we'd simulate the event locally if we are the target
            // web_sys doesn't allow easy event injection for security, so this is mostly protocol-level here
        }
        ServerMessage::PollsList(list) => {
            ctx.set_polls.set(list);
        }
        ServerMessage::Draw(action) => {
            ctx.set_last_draw_action.set(Some(action.clone()));
            ctx.set_whiteboard_history.update(|h| h.push(action));
        }
        ServerMessage::WhiteboardHistory(history) => {
            ctx.set_whiteboard_history.set(history);
        }
        ServerMessage::VideoShared(url) => {
            ctx.set_shared_video_url.set(Some(url));
        }
        ServerMessage::VideoStopped => {
            ctx.set_shared_video_url.set(None);
        }
        ServerMessage::PowerStatusUpdated { user_id, status } => {
            ctx.set_power_statuses.update(|map| {
                map.insert(user_id, status);
            });
        }
        ServerMessage::RecordingStatusChanged { user_id, is_recording: is_locally_recording } => {
            let is_self = ctx.my_id.get_untracked().as_deref() == Some(&user_id);
            if !is_self {
                if is_locally_recording {
                    ctx.add_toast.call(("A participant started recording locally".to_string(), ToastType::Info));
                } else {
                    ctx.add_toast.call(("A participant stopped their local recording".to_string(), ToastType::Info));
                }
            }
        }
        ServerMessage::UnmuteRequested { requester_id, target_id } => {
            if let Some(my) = ctx.my_id.get_untracked() {
                if my == target_id {
                    let parts = ctx.participants.get_untracked();
                    let sender_name = parts.iter().find(|p| p.id == requester_id).map(|p| p.name.clone()).unwrap_or(requester_id);
                    ctx.add_toast.call((format!("Host ({}) asked you to unmute", sender_name), ToastType::Info));
                }
            }
        }
        ServerMessage::LobbyAnnouncement(text) => {
            ctx.set_lobby_announcement.set(Some(text));
        }
        ServerMessage::VisitorPromoted(target_id) => {
            if let Some(my) = ctx.my_id.get_untracked() {
                if my == target_id {
                    ctx.add_toast.call(("You have been promoted to a full participant".to_string(), ToastType::Info));
                }
            }
        }
        ServerMessage::PeerSpeaking { user_id, speaking } => {
            ctx.set_speaking_peers.update(|s| {
                if speaking {
                    s.insert(user_id);
                } else {
                    s.remove(&user_id);
                }
            });
        }
        ServerMessage::Transcription { user_id, text, timestamp } => {
            ctx.set_subtitles.update(|subs| {
                subs.push((user_id, text, timestamp));
                if subs.len() > 5 {
                    subs.remove(0);
                }
            });
        }
        ServerMessage::Pong { .. } => {
            let now = js_sys::Date::now();
            let start = ctx.last_ping_time.get_untracked();
            if start > 0.0 {
                let latency = (now - start) as u64;
                ctx.set_rtt.set(latency);
            }
        }
        ServerMessage::AuthenticationResult(success) => {
            if success {
                ctx.set_is_authenticated.set(true);
                ctx.set_show_login_dialog.set(false);
                ctx.set_auth_error.set(None);
                ctx.add_toast.call(("Authenticated successfully (mock)".to_string(), ToastType::Info));
            } else {
                ctx.set_auth_error.set(Some("Invalid username or password".to_string()));
            }
        }
        ServerMessage::CalendarEvents(events) => {
            ctx.set_calendar_events.set(events);
        }
        ServerMessage::Error(err) => {
            ctx.add_toast.call((err, ToastType::Error));
        }
        ServerMessage::Offer { source_id, sdp, .. } => {
            ctx.webrtc_manager.handle_offer(source_id, sdp);
        }
        ServerMessage::Answer { source_id, sdp, .. } => {
            ctx.webrtc_manager.handle_answer(source_id, sdp);
        }
        ServerMessage::IceCandidate {
            source_id,
            candidate,
            sdp_mid,
            sdp_m_line_index,
            ..
        } => {
            ctx.webrtc_manager.handle_ice_candidate(
                source_id,
                candidate,
                sdp_mid,
                sdp_m_line_index,
            );
        }
        ServerMessage::EtherpadUrlUpdated { .. } => {}
        ServerMessage::GiphyShared { .. } => {}
    }
}
