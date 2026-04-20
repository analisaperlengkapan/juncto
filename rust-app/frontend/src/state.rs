use crate::analytics::{provide_analytics_context, use_analytics, AnalyticsService};
use crate::state_handlers::{handle_server_message, HandlerContext};
use crate::storage::{load_settings, update_setting};
use crate::components_ui::toast::{use_toast, ToastType};
use crate::media::{get_display_media, get_user_media, AudioMonitor};
use crate::webrtc::WebRTCManager;
use leptos::*;
use serde::{Deserialize, Serialize};
use shared::{
    ChatMessage, ClientMessage, DrawAction, FileAttachment, Participant, Poll, ServerMessage,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
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
    pub is_visitor: bool,
}

#[derive(Clone)]
pub struct RoomState {
    pub connection_state: ReadSignal<RoomConnectionState>,
    pub messages: ReadSignal<Vec<ChatMessage>>,
    pub participants: ReadSignal<Vec<Participant>>,
    pub knocking_participants: ReadSignal<Vec<Participant>>,
    pub is_connected: ReadSignal<bool>,
    pub is_locked: ReadSignal<bool>,
    pub is_e2ee_enabled: ReadSignal<bool>,
    pub is_lobby_enabled: ReadSignal<bool>,
    pub is_recording: ReadSignal<bool>,
    pub is_subtitles_enabled: ReadSignal<bool>,
    pub subtitles: ReadSignal<Vec<(String, String, u64)>>,
    pub show_settings: ReadSignal<bool>,
    pub show_polls: ReadSignal<bool>,
    pub show_shortcuts: ReadSignal<bool>,
    pub polls: ReadSignal<Vec<Poll>>,
    pub last_reaction: ReadSignal<Option<(String, String, u64)>>,
    pub show_whiteboard: ReadSignal<bool>,
    pub show_etherpad: ReadSignal<bool>,
    pub whiteboard_history: ReadSignal<Vec<DrawAction>>,
    pub my_id: ReadSignal<Option<String>>,
    pub typing_users: ReadSignal<HashSet<String>>,
    pub is_host: Signal<bool>,
    pub host_id: Signal<Option<String>>,
    pub current_room_id: ReadSignal<Option<String>>,
    pub breakout_rooms: ReadSignal<Vec<shared::BreakoutRoom>>,
    #[allow(dead_code)]
    pub is_authenticated: ReadSignal<bool>,
    pub show_login_dialog: ReadSignal<bool>,
    pub auth_error: ReadSignal<Option<String>>,
    pub calendar_events: ReadSignal<Vec<String>>,
    pub show_calendar: ReadSignal<bool>,
    pub local_stream: ReadSignal<Option<MediaStream>>,
    pub local_screen_stream: ReadSignal<Option<MediaStream>>,
    pub is_muted: ReadSignal<bool>,
    pub shared_video_url: ReadSignal<Option<String>>,
    pub speaking_peers: ReadSignal<HashSet<String>>,
    #[allow(dead_code)]
    pub audio_monitor: ReadSignal<Option<AudioMonitor>>,
    #[allow(dead_code)]
    pub raw_local_stream: ReadSignal<Option<MediaStream>>,
    pub show_speaker_stats: ReadSignal<bool>,
    pub show_virtual_background: ReadSignal<bool>,
    pub show_feedback: ReadSignal<bool>,
    pub rtt: ReadSignal<u64>,
    pub remote_streams: ReadSignal<HashMap<String, Vec<MediaStream>>>,
    pub selected_camera_id: ReadSignal<Option<String>>,
    pub selected_mic_id: ReadSignal<Option<String>>,
    pub video_resolution: ReadSignal<String>,
    pub is_noise_suppression_enabled: ReadSignal<bool>,
    pub background_mode: ReadSignal<String>,
    pub grid_layout: ReadSignal<String>,
    pub is_visitor: Signal<bool>,
    pub room_config: ReadSignal<shared::RoomConfig>,
    pub power_statuses: ReadSignal<std::collections::HashMap<String, shared::PowerStatus>>,
    pub is_recording_locally: ReadSignal<bool>,
    pub lobby_announcement: ReadSignal<Option<String>>,
    // Setters or Actions
    pub set_input_devices: Callback<(Option<String>, Option<String>, String, bool)>,
    pub set_background_mode: Callback<String>,
    pub set_grid_layout: Callback<String>,
    pub set_show_settings: WriteSignal<bool>,
    pub set_show_polls: WriteSignal<bool>,
    pub set_show_shortcuts: WriteSignal<bool>,
    pub set_show_whiteboard: WriteSignal<bool>,
    pub set_show_etherpad: WriteSignal<bool>,
    pub set_show_speaker_stats: WriteSignal<bool>,
    pub set_show_virtual_background: WriteSignal<bool>,
    pub set_show_feedback: WriteSignal<bool>,
    pub set_show_login_dialog: WriteSignal<bool>,
    pub set_auth_error: WriteSignal<Option<String>>,
    pub authenticate: Callback<(String, Option<String>)>,
    pub set_show_calendar: WriteSignal<bool>,
    pub fetch_calendar: Callback<()>,
    pub send_ping: Callback<()>,
    pub send_message: crate::chat::ChatSendCallback, // content, recipient_id, attachment
    pub start_share_video: Callback<String>,
    pub stop_share_video: Callback<()>,
    pub toggle_lock: Callback<()>,
    pub toggle_e2ee: Callback<()>,
    pub toggle_etherpad: Callback<Option<String>>,
    pub toggle_lobby: Callback<()>,
    pub toggle_recording: Callback<()>,
    pub toggle_subtitles: Callback<()>,
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
    pub close_poll: Callback<String>,
    pub send_draw: Callback<DrawAction>,
    pub set_is_typing: Callback<bool>,
    pub create_breakout_room: Callback<String>,
    pub join_breakout_room: Callback<Option<String>>,
    pub toggle_camera: Callback<()>,
    pub toggle_mic: Callback<()>,
    pub end_meeting: Callback<()>,
    pub mute_participant: Callback<String>,
    pub mute_camera_participant: Callback<String>,
    pub mute_all: Callback<()>,
    pub mute_camera_all: Callback<()>,
    pub transfer_host: Callback<String>,
    pub set_presence: Callback<shared::PresenceStatus>,
    pub toggle_local_recording: Callback<()>,
    pub request_unmute: Callback<String>,
    pub update_power_status: Callback<shared::PowerStatus>,
    pub broadcast_to_lobby: Callback<String>,
    pub promote_visitor: Callback<String>,
    #[allow(dead_code)]
    pub analytics: AnalyticsService,
}

pub fn use_room_state() -> RoomState {
    let settings = load_settings();
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
    let (is_e2ee_enabled, set_is_e2ee_enabled) = create_signal(false);
    let (is_lobby_enabled, set_is_lobby_enabled) = create_signal(false);
    let (is_recording, set_is_recording) = create_signal(false);
    let (is_subtitles_enabled, set_is_subtitles_enabled) = create_signal(false);
    let (subtitles, set_subtitles) = create_signal(Vec::<(String, String, u64)>::new());
    let (show_settings, set_show_settings) = create_signal(false);
    let (is_authenticated, set_is_authenticated) = create_signal(false);
    let (show_login_dialog, set_show_login_dialog) = create_signal(false);
    let (auth_error, set_auth_error) = create_signal(None::<String>);
    let (calendar_events, set_calendar_events) = create_signal(Vec::<String>::new());
    let (show_calendar, set_show_calendar) = create_signal(false);
    let (show_polls, set_show_polls) = create_signal(false);
    let (show_shortcuts, set_show_shortcuts) = create_signal(false);
    let (polls, set_polls) = create_signal(Vec::<Poll>::new());
    let (last_reaction, set_last_reaction) = create_signal(None::<(String, String, u64)>);
    let (show_whiteboard, set_show_whiteboard) = create_signal(false);
    let (show_etherpad, set_show_etherpad) = create_signal(false);
    let (whiteboard_history, set_whiteboard_history) = create_signal(Vec::<DrawAction>::new());
    let (_last_draw_action, set_last_draw_action) = create_signal(None::<DrawAction>);
    let (my_id, set_my_id) = create_signal(None::<String>);
    let (local_stream, set_local_stream) = create_signal(None::<MediaStream>);
    let (local_screen_stream, set_local_screen_stream) = create_signal(None::<MediaStream>);
    let (is_muted, set_is_muted) = create_signal(false);
    let (is_camera_off, set_is_camera_off) = create_signal(false);
    let (shared_video_url, set_shared_video_url) = create_signal(None::<String>);
    let (speaking_peers, set_speaking_peers) = create_signal(HashSet::<String>::new());
    let (audio_monitor, set_audio_monitor) = create_signal(None::<AudioMonitor>);
    let (show_speaker_stats, set_show_speaker_stats) = create_signal(false);
    let (show_virtual_background, set_show_virtual_background) = create_signal(false);
    let (show_feedback, set_show_feedback) = create_signal(false);
    let (rtt, set_rtt) = create_signal(0u64);
    let (last_ping_time, set_last_ping_time) = create_signal(0f64);
    let (selected_camera_id, set_selected_camera_id) = create_signal(settings.camera_id);
    let (selected_mic_id, set_selected_mic_id) = create_signal(settings.mic_id);
    let (video_resolution, set_video_resolution) = create_signal(settings.resolution.unwrap_or("hd".to_string()));
    let (is_noise_suppression_enabled, set_is_noise_suppression_enabled) = create_signal(false);
    let (background_mode, set_background_mode_sig) = create_signal("none".to_string());
    let (grid_layout, set_grid_layout_sig) = create_signal("grid".to_string());
    let (room_config, set_room_config) = create_signal(shared::RoomConfig::default());
    let (lobby_announcement, set_lobby_announcement) = create_signal(None::<String>);

    let (remote_streams, set_remote_streams) =
        create_signal(HashMap::<String, Vec<MediaStream>>::new());

    let (power_statuses, set_power_statuses) = create_signal(std::collections::HashMap::<String, shared::PowerStatus>::new());
    let (is_recording_locally, set_is_recording_locally) = create_signal(false);
    let local_recorder: Rc<RefCell<Option<crate::media_recorder::LocalRecorder>>> = Rc::new(RefCell::new(None));
    // Holds previously-stopped recorders whose async `onstop` callbacks may
    // not have fired yet. They are kept alive here so the wasm-bindgen
    // Closures remain valid until the browser event loop processes the stop
    // event. Entries are cleared each time a new recording starts.
    let pending_recorders: Rc<RefCell<Vec<crate::media_recorder::LocalRecorder>>> = Rc::new(RefCell::new(Vec::new()));
    // Tracks the stream ID that the active LocalRecorder was created with.
    // Used by a reactive effect to detect when `local_stream` is replaced
    // (e.g. camera toggle, device switch) and automatically restart the
    // recording on the new stream.
    let recording_stream_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // Video Processing for Virtual Background
    let (raw_local_stream, set_raw_local_stream) = create_signal(None::<MediaStream>);

    // Reactive signals that need to be defined before callbacks
    let host_id = Signal::derive(move || room_config.get().host_id);

    let is_host = Signal::derive(move || {
        let h = host_id.get();
        let my = my_id.get();

        match (h, my) {
            (Some(host), Some(me)) => host == me,
            _ => false,
        }
    });

    let is_visitor = Signal::derive(move || {
        if let Some(me_id) = my_id.get() {
            participants.get().iter().find(|p| p.id == me_id).map(|p| p.is_visitor).unwrap_or(false)
        } else {
            false
        }
    });

    // Note: Reactive Noise Suppression effect is created after start_media_stream below.

    // Track the stream ID so we can detect when only the mode changed (same
    // stream) vs when the underlying raw stream was replaced (camera toggle,
    // device switch, noise-suppression restart).
    let prev_stream_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    create_effect({
        let prev_stream_id = prev_stream_id.clone();
        move |prev_processor: Option<Option<crate::media::VideoProcessor>>| -> Option<crate::media::VideoProcessor> {
            let mode = background_mode.get();
            let stream = raw_local_stream.get();

            if let Some(s) = stream {
                let has_video = s.get_video_tracks().length() > 0;
                if mode == "none" || !has_video {
                    // No processing needed — drop any existing processor and pass through.
                    // Stop old canvas stream video tracks to avoid leaking captureStream
                    // resources. Only stop video tracks since audio tracks are shared
                    // references to the raw stream's tracks and must remain active.
                    if prev_processor.as_ref().is_some_and(|p| p.is_some()) {
                        if let Some(old_processed) = local_stream.get_untracked() {
                            let video_tracks = old_processed.get_video_tracks();
                            for i in 0..video_tracks.length() {
                                if let Ok(track) = video_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                                    track.stop();
                                }
                            }
                        }
                    }
                    if let Some(Some(prev)) = prev_processor {
                        drop(prev);
                    }
                    *prev_stream_id.borrow_mut() = Some(s.id());
                    set_local_stream.set(Some(s));
                    None
                } else {
                    // Check whether the raw stream changed since the last run.
                    let stream_changed = {
                        let old_id = prev_stream_id.borrow();
                        old_id.as_deref() != Some(&s.id())
                    };
                    *prev_stream_id.borrow_mut() = Some(s.id());

                    if !stream_changed {
                        if let Some(Some(prev)) = prev_processor {
                            // Same stream, only mode changed — update in-place instead of
                            // recreating the canvas, video element, interval, and captureStream.
                            prev.set_mode(mode);
                            return Some(prev);
                        }
                    }

                    // Either the stream changed or no processor exists yet — (re)create.
                    // Stop old canvas video tracks before dropping the processor.
                    if prev_processor.as_ref().is_some_and(|p| p.is_some()) {
                        if let Some(old_processed) = local_stream.get_untracked() {
                            let video_tracks = old_processed.get_video_tracks();
                            for i in 0..video_tracks.length() {
                                if let Ok(track) = video_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                                    track.stop();
                                }
                            }
                        }
                    }
                    if let Some(Some(prev)) = prev_processor {
                        drop(prev);
                    }
                    match crate::media::VideoProcessor::new(&s, mode) {
                        Ok((processor, processed)) => {
                            set_local_stream.set(Some(processed));
                            Some(processor)
                        }
                        Err(e) => {
                            web_sys::console::error_1(&e);
                            set_local_stream.set(Some(s));
                            None
                        }
                    }
                }
            } else {
                // No stream — drop any existing processor
                if let Some(Some(prev)) = prev_processor {
                    drop(prev);
                }
                *prev_stream_id.borrow_mut() = None;
                set_local_stream.set(None);
                None
            }
        }
    });

    let set_grid_layout = Callback::new(move |layout: String| {
        set_grid_layout_sig.set(layout.clone());
        if is_host.get_untracked() {
            if let Some(socket) = ws.get() {
                let msg = ClientMessage::FollowMe(layout);
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send_with_str(&json);
                }
            }
        }
    });

    let close_poll = Callback::new(move |poll_id: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ClosePoll(poll_id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    // WebRTC Manager Setup
    let ws_clone_for_webrtc = ws;
    let send_signal_cb = move |msg: ClientMessage| {
        if let Some(socket) = ws_clone_for_webrtc.get_untracked() {
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    };

    // Provide Analytics context
    provide_analytics_context(Callback::new(send_signal_cb));
    let analytics = use_analytics();

    let on_track_cb_clone = set_remote_streams;
    let on_track_cb = move |peer_id: String, stream: MediaStream| {
        // Add cleanup listener for tracks
        let tracks = stream.get_tracks();
        for i in 0..tracks.length() {
            if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                let peer_id_clone = peer_id.clone();
                let stream_id = stream.id();
                let set_remote_streams = on_track_cb_clone; // captured

                let onended = Closure::wrap(Box::new(move || {
                    set_remote_streams.update(|map: &mut HashMap<String, Vec<MediaStream>>| {
                        if let Some(streams) = map.get_mut(&peer_id_clone) {
                            streams.retain(|s| {
                                // If the stream matches the one ending, check if it's still active
                                if s.id() == stream_id {
                                    s.active()
                                } else {
                                    true
                                }
                            });
                        }
                    });
                }) as Box<dyn FnMut()>);
                track.set_onended(Some(onended.as_ref().unchecked_ref()));
                onended.forget();
            }
        }

        set_remote_streams.update(|map: &mut HashMap<String, Vec<MediaStream>>| {
            // Append the new stream to the list for this peer, checking for duplicates
            let streams = map.entry(peer_id).or_default();
            let stream_id = stream.id();
            if !streams.iter().any(|s| s.id() == stream_id) {
                streams.push(stream);
            }
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
    let participants_for_effect = participants;
    let my_id_for_effect = my_id;
    let webrtc_manager_clone = webrtc_manager.clone();

    create_effect(move |_| {
        // Run when my_id or local_stream changes
        let my_id_val = my_id_for_effect.get();
        // We track local_stream and local_screen_stream to trigger track updates
        let _ = local_stream.get();
        let _ = local_screen_stream.get();

        if let Some(me) = my_id_val {
            // Update tracks for existing connections (e.g. from incoming offers)
            // This is safe to call even if local_stream is None (it effectively clears tracks or does nothing)
            webrtc_manager_clone.update_local_tracks();

            let list = participants_for_effect.get_untracked();
            for p in list {
                if p.id != me {
                    // Only initiate connection if one doesn't exist AND I am the impolite peer (higher ID)
                    // Deterministic rule: Higher ID initiates.
                    if !webrtc_manager_clone.has_peer(&p.id) && me > p.id {
                        webrtc_manager_clone.handle_participant_joined(p.id);
                    }
                }
            }
        }
    });

    // Internal state to trigger media start after joining
    let (start_media_on_join, set_start_media_on_join) = create_signal(false);
    let (initial_cam_on, set_initial_cam_on) = create_signal(false);

    // Setup listener for "talk while muted"
    let toast_context = crate::components_ui::toast::use_toast();
    create_effect(move |_| {
        use wasm_bindgen::JsCast;
        if let Some(window) = web_sys::window() {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                toast_context.add(
                    "You are muted. Please unmute to speak.".to_string(),
                    crate::components_ui::toast::ToastType::Error,
                );
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback("talk_while_muted", closure.as_ref().unchecked_ref());

            // Clean up the event listener when the component is unmounted
            on_cleanup(move || {
                if let Some(win) = web_sys::window() {
                    let _ = win.remove_event_listener_with_callback("talk_while_muted", closure.as_ref().unchecked_ref());
                }
            });
        }
    });

    let add_toast = move |msg: String, type_: ToastType| {
        toast_ctx.add(msg, type_);
    };

    // Extract start_media_stream logic
    let start_media_stream = Callback::new(move |enable_video: bool| {
        // Clear host-muted camera state since we are replacing the stream
        set_is_camera_off.set(false);
        spawn_local(async move {
            let v_id = selected_camera_id.get_untracked();
            let a_id = selected_mic_id.get_untracked();
            let res = video_resolution.get_untracked();

            if let Ok(stream) = get_user_media(enable_video, true, v_id, a_id, Some(&res)).await {
                // Stop existing raw stream tracks to release camera/mic hardware.
                if let Some(old_raw) = raw_local_stream.get_untracked() {
                    let tracks = old_raw.get_tracks();
                    for i in 0..tracks.length() {
                        if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                            track.stop();
                        }
                    }
                }
                // Also stop processed stream tracks (canvas video tracks) for cleanup
                if let Some(old_stream) = local_stream.get_untracked() {
                    let tracks = old_stream.get_tracks();
                    for i in 0..tracks.length() {
                        if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                            track.stop();
                        }
                    }
                }
                set_audio_monitor.set(None); // Reset monitor for new stream

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

                set_raw_local_stream.set(Some(stream.clone()));

                let on_speaking = Box::new(move |is_speaking: bool| {
                    if let Some(socket) = ws.get_untracked() {
                        let msg = ClientMessage::Speaking(is_speaking);
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = socket.send_with_str(&json);
                        }
                    }
                });


                let add_toast_clone = add_toast;
                let on_no_audio = Box::new(move || {
                    add_toast_clone("No audio input detected. Please check your microphone.".to_string(), crate::components_ui::toast::ToastType::Error);
                });

            if let Ok(monitor) = AudioMonitor::new(
                &stream,
                on_speaking,
                Some(on_no_audio as Box<dyn FnMut()>),
                is_noise_suppression_enabled.get_untracked(),
            ) {
                // Inherit the current mute state so the monitor doesn't fire
                // false-positive speaking callbacks while the user is muted
                // (e.g. after a noise-suppression restart or device change).
                monitor.set_muted(is_muted.get_untracked());
                set_audio_monitor.set(Some(monitor));
            }
            }
        });
    });

    // Reactive Noise Suppression Update
    create_effect(move |_| {
        let enabled = is_noise_suppression_enabled.get();
        let needs_restart = audio_monitor.with_untracked(|monitor| {
            if let Some(m) = monitor {
                // Returns Err if the monitor needs to be recreated (e.g. enabling
                // suppression when no compressor node exists in the audio graph).
                m.set_noise_suppression(enabled).is_err()
            } else {
                false
            }
        });
        if needs_restart {
            // Restart media to recreate the AudioMonitor with the correct setting.
            // Preserve current video state.
            let has_video = local_stream.with_untracked(|s| {
                s.as_ref().is_some_and(|stream| stream.get_video_tracks().length() > 0)
            });
            if local_stream.get_untracked().is_some() {
                start_media_stream.call(has_video);
            }
        }
    });

    // Initialize WebSocket
    let webrtc_manager_for_ws = webrtc_manager.clone();
    let analytics_for_ws = analytics.clone();
    let local_recorder_for_ws = local_recorder.clone();
    let pending_recorders_for_ws = pending_recorders.clone();
    let recording_stream_id_for_ws = recording_stream_id.clone();
    create_effect(move |_| {
        let analytics = analytics_for_ws.clone();
        let local_recorder_for_cleanup = local_recorder_for_ws.clone();
        let pending_recorders_for_cleanup = pending_recorders_for_ws.clone();
        let recording_stream_id_for_cleanup = recording_stream_id_for_ws.clone();
        let set_messages = set_messages;
        let set_participants = set_participants;
        let set_typing_users = set_typing_users;
        let set_speaking_peers = set_speaking_peers;
        let set_knocking_participants = set_knocking_participants;
        let set_polls = set_polls;
        let set_whiteboard_history = set_whiteboard_history;
        let set_subtitles = set_subtitles;
        let set_rtt = set_rtt;

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
                        let ctx = HandlerContext {
                            set_my_id,
                            set_current_state,
                            analytics: analytics.clone(),
                            start_media_on_join,
                            initial_cam_on,
                            start_media_stream,
                            set_start_media_on_join,
                            is_muted,
                            ws,
                            local_stream,
                            raw_local_stream,
                            add_toast: Callback::new(move |(msg, t)| add_toast(msg, t)),
                            set_is_camera_off,
                            room_config,
                            set_show_etherpad,
                            set_is_locked,
                            set_is_e2ee_enabled,
                            is_recording,
                            set_is_recording,
                            set_is_lobby_enabled,
                            is_subtitles_enabled,
                            set_is_subtitles_enabled,
                            set_subtitles,
                            set_room_config,
                            current_room_id,
                            set_messages,
                            set_is_connected,
                            set_knocking_participants,
                            set_participants,
                            my_id,
                            webrtc_manager: webrtc_manager.clone(),
                            set_typing_users,
                            set_speaking_peers,
                            set_power_statuses,
                            set_remote_streams,
                            is_recording_locally,
                            local_recorder: local_recorder_for_cleanup.clone(),
                            pending_recorders: pending_recorders_for_cleanup.clone(),
                            recording_stream_id: recording_stream_id_for_cleanup.clone(),
                            set_is_recording_locally,
                            set_is_muted,
                            set_audio_monitor,
                            participants,
                            set_last_reaction,
                            set_breakout_rooms,
                            set_polls,
                            set_grid_layout_sig,
                            set_whiteboard_history,
                            set_last_draw_action,
                            set_shared_video_url,
                            last_ping_time,
                            set_rtt,
                            set_is_authenticated,
                            set_show_login_dialog,
                            set_auth_error,
                            set_calendar_events,
                            set_lobby_announcement,
                        };
                        handle_server_message(server_msg, &ctx);
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

    let webrtc_manager_cleanup = webrtc_manager.clone();
    on_cleanup(move || {
        if let Some(socket) = ws.get() {
            let _ = socket.close();
        }
        webrtc_manager_cleanup.close_all_peers();
    });

    let send_message = Callback::new(
        move |(content, recipient_id, attachment, room_id): (
            String,
            Option<String>,
            Option<FileAttachment>,
            Option<String>,
        )| {
            if let Some(socket) = ws.get() {
                let msg = ClientMessage::Chat {
                    content,
                    recipient_id,
                    attachment,
                    room_id,
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send_with_str(&json);
                }
            }
        },
    );

    let analytics_for_lock = analytics.clone();
    let toggle_lock = Callback::new(move |_: ()| {
        analytics_for_lock.track_interaction("toggle_lock");
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleRoomLock;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_e2ee = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleE2EE;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_etherpad = Callback::new(move |url: Option<String>| {
        if let Some(url_str) = &url {
            if !url_str.starts_with("https://") && !url_str.starts_with("http://") {
                add_toast("Invalid Etherpad URL".to_string(), ToastType::Error);
                return;
            }
        }

        if let Some(socket) = ws.get() {
            let msg = ClientMessage::SetEtherpadUrl(url);
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

    let toggle_subtitles = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleSubtitles;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let set_presence = Callback::new(move |status: shared::PresenceStatus| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::SetPresence(status);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let set_background_mode = Callback::new(move |mode: String| {
        set_background_mode_sig.set(mode);
    });

    let grant_access = Callback::new(move |id: String| {
        set_knocking_participants.update(|list: &mut Vec<Participant>| list.retain(|p| p.id != id));
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::GrantAccess(id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let deny_access = Callback::new(move |id: String| {
        set_knocking_participants.update(|list: &mut Vec<Participant>| list.retain(|p| p.id != id));
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

    let analytics_for_reaction = analytics.clone();
    let send_reaction = Callback::new(move |emoji: String| {
        if is_visitor.get_untracked() { return; }
        let props = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&props, &JsValue::from_str("emoji"), &JsValue::from_str(&emoji));
        analytics_for_reaction.track_event("send_reaction", props.into());
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Reaction(emoji);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let analytics_for_raise_hand = analytics.clone();
    let toggle_raise_hand = Callback::new(move |_: ()| {
        if is_visitor.get_untracked() { return; }
        analytics_for_raise_hand.track_interaction("toggle_raise_hand");
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleRaiseHand;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let webrtc_manager_for_screen = webrtc_manager.clone();
    let toggle_screen_share = Callback::new(move |_: ()| {
        if is_visitor.get_untracked() { return; }
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

    let analytics_for_create_poll = analytics.clone();
    let create_poll = Callback::new(move |poll: Poll| {
        analytics_for_create_poll.track_interaction("create_poll");
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::CreatePoll(poll);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let analytics_for_vote_poll = analytics.clone();
    let vote_poll = Callback::new(move |(poll_id, option_id): (String, u32)| {
        analytics_for_vote_poll.track_interaction("vote_poll");
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
        // Persist settings
        let display_name_clone = options.display_name.clone();
        let cam_id_clone = options.video_device_id.clone();
        let mic_id_clone = options.audio_device_id.clone();

        update_setting(move |s| {
            s.display_name = Some(display_name_clone);
            s.camera_id = cam_id_clone;
            s.mic_id = mic_id_clone;
        });

        // Set initial state
        set_is_muted.set(!options.mic_enabled);
        set_selected_mic_id.set(options.audio_device_id);
        set_selected_camera_id.set(options.video_device_id);

        // Start media if either mic or cam is on
        set_start_media_on_join.set(options.mic_enabled || options.camera_enabled);
        set_initial_cam_on.set(options.camera_enabled);

        let display_name = options.display_name;
        let is_visitor = options.is_visitor;

        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Join { name: display_name, is_visitor };
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

    let webrtc_manager_for_breakout = webrtc_manager.clone();
    let join_breakout_room = Callback::new(move |room_id: Option<String>| {
        set_current_room_id.set(room_id.clone());
        // Clear messages when switching rooms
        set_messages.set(Vec::new());
        // Clear stale indicators
        set_speaking_peers.update(|s: &mut HashSet<String>| s.clear());
        set_typing_users.update(|u: &mut HashSet<String>| u.clear());
        // Clear stale power statuses from the previous room; each
        // participant's PowerMonitor will re-send their status on the
        // next 60-second poll cycle.
        set_power_statuses.set(std::collections::HashMap::new());

        // Cleanup existing WebRTC connections on room switch
        webrtc_manager_for_breakout.close_all_peers();
        set_remote_streams.set(HashMap::new());

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

    let mute_camera_participant = Callback::new(move |id: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::MuteCameraParticipant(id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let mute_all = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::MuteAll;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let mute_camera_all = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::MuteCameraAll;
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

    let toggle_local_recording = Callback::new({
        let local_recorder = local_recorder.clone();
        let pending_recorders = pending_recorders.clone();
        let recording_stream_id = recording_stream_id.clone();
        move |_: ()| {
            let is_active = is_recording_locally.get_untracked();
            if is_active {
                // Call stop() but keep the LocalRecorder alive so its Closure
                // callbacks (_on_data_available, _on_stop) remain valid when
                // the browser asynchronously fires the stop event. Move the
                // recorder into `pending_recorders`; it will be dropped the
                // next time the user starts a new recording.
                if let Some(r) = local_recorder.borrow_mut().take() {
                    r.stop();
                    pending_recorders.borrow_mut().push(r);
                }
                *recording_stream_id.borrow_mut() = None;
                set_is_recording_locally.set(false);
                if let Some(socket) = ws.get() {
                    let _ = socket.send_with_str(&serde_json::to_string(&ClientMessage::ToggleLocalRecording(false)).unwrap());
                }
            } else if let Some(stream) = local_stream.get_untracked() {
                // Drop any previously-stopped recorders. By now their async
                // `onstop` callbacks have had ample time to fire (the user
                // had to click stop, wait, then click start again).
                pending_recorders.borrow_mut().clear();
                match crate::media_recorder::LocalRecorder::new(stream.clone(), Callback::new(move |msg: String| {
                    add_toast(format!("Recording error: {}", msg), ToastType::Error);
                })) {
                    Ok(r) => {
                        *local_recorder.borrow_mut() = Some(r);
                        *recording_stream_id.borrow_mut() = Some(stream.id());
                        set_is_recording_locally.set(true);
                        if let Some(socket) = ws.get() {
                            let _ = socket.send_with_str(&serde_json::to_string(&ClientMessage::ToggleLocalRecording(true)).unwrap());
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&e);
                        add_toast("Failed to start local recording".to_string(), ToastType::Error);
                    }
                }
            } else {
                add_toast("Enable camera or microphone first to start recording".to_string(), ToastType::Error);
            }
        }
    });

    // Reactive effect: when local_stream changes while recording, automatically
    // stop the old recorder and start a new one on the fresh stream. This handles
    // camera toggles, device switches, and noise-suppression restarts that replace
    // the underlying MediaStream whose tracks the recorder depends on.
    {
        let local_recorder = local_recorder.clone();
        let pending_recorders = pending_recorders.clone();
        let recording_stream_id = recording_stream_id.clone();
        create_effect(move |_| {
            let current_stream = local_stream.get();
            let is_active = is_recording_locally.get_untracked();
            if !is_active {
                return;
            }

            let old_id = recording_stream_id.borrow().clone();
            let new_id = current_stream.as_ref().map(|s| s.id());

            // Only act when the stream identity actually changed
            if old_id == new_id {
                return;
            }

            // Stop the old recorder, keeping it alive for async onstop
            if let Some(r) = local_recorder.borrow_mut().take() {
                r.stop();
                pending_recorders.borrow_mut().push(r);
            }

            // Trim very old pending recorders to prevent unbounded growth
            // during rapid stream changes. Keep the most recent entries
            // whose async onstop callbacks may not have fired yet; older
            // ones have had enough time (each stream change is user- or
            // system-initiated with perceptible delay).
            {
                let mut pending = pending_recorders.borrow_mut();
                if pending.len() > 3 {
                    let drain_count = pending.len() - 3;
                    pending.drain(..drain_count);
                }
            }

            if let Some(stream) = current_stream {
                // Start a new recorder on the replacement stream.
                // Do NOT clear pending_recorders here — the old recorder
                // was just pushed above and its async onstop callback has
                // not fired yet. Clearing now would drop the Closure and
                // lose the recorded data. Stale recorders are cleaned up
                // the next time the user manually starts a new recording
                // (in toggle_local_recording) after an interactive delay.
                match crate::media_recorder::LocalRecorder::new(stream.clone(), Callback::new(move |msg: String| {
                    add_toast(format!("Recording error: {}", msg), ToastType::Error);
                })) {
                    Ok(r) => {
                        *local_recorder.borrow_mut() = Some(r);
                        *recording_stream_id.borrow_mut() = Some(stream.id());
                        // No WS notification — the recording session continues
                        // from the other participants' perspective.
                    }
                    Err(e) => {
                        web_sys::console::error_1(&e);
                        *recording_stream_id.borrow_mut() = None;
                        set_is_recording_locally.set(false);
                        add_toast("Recording stopped: failed to record new stream".to_string(), ToastType::Error);
                        if let Some(socket) = ws.get() {
                            let _ = socket.send_with_str(&serde_json::to_string(&ClientMessage::ToggleLocalRecording(false)).unwrap());
                        }
                    }
                }
            } else {
                // Stream was removed entirely (e.g. all media disabled)
                *recording_stream_id.borrow_mut() = None;
                set_is_recording_locally.set(false);
                add_toast("Recording stopped: media stream ended".to_string(), ToastType::Info);
                if let Some(socket) = ws.get() {
                    let _ = socket.send_with_str(&serde_json::to_string(&ClientMessage::ToggleLocalRecording(false)).unwrap());
                }
            }
        });
    }

    let request_unmute = Callback::new(move |target_id: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::RequestUnmute(target_id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let update_power_status = Callback::new(move |status: shared::PowerStatus| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::UpdatePowerStatus(status);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let broadcast_to_lobby = Callback::new(move |text: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::BroadcastToLobby(text);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let promote_visitor = Callback::new(move |id: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::PromoteVisitor(id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let analytics_for_camera = analytics.clone();
    let toggle_camera = Callback::new(move |_: ()| {
        if is_visitor.get_untracked() { return; }

        // If the camera was disabled by the host, re-enable the existing
        // tracks instead of restarting the media stream. This mirrors
        // toggle_mic which checks is_muted to determine the current state.
        if is_camera_off.get_untracked() {
            set_is_camera_off.set(false);
            analytics_for_camera.track_toggle_media("camera", true);
            if let Some(raw) = raw_local_stream.get_untracked() {
                let video_tracks = raw.get_video_tracks();
                for i in 0..video_tracks.length() {
                    if let Ok(track) = video_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                        track.set_enabled(true);
                    }
                }
            }
            if let Some(stream) = local_stream.get_untracked() {
                let video_tracks = stream.get_video_tracks();
                for i in 0..video_tracks.length() {
                    if let Ok(track) = video_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                        track.set_enabled(true);
                    }
                }
            }
            return;
        }

        // Check if we currently have video tracks active
        let has_video = if let Some(stream) = local_stream.get_untracked() {
            stream.get_video_tracks().length() > 0
        } else {
            false
        };
        let new_state = !has_video;
        analytics_for_camera.track_toggle_media("camera", new_state);
        start_media_stream.call(new_state);
    });

    let set_input_devices = Callback::new(
        move |(vid, aid, res, ns): (Option<String>, Option<String>, String, bool)| {
            update_setting(|s| {
                s.camera_id = vid.clone();
                s.mic_id = aid.clone();
                s.resolution = Some(res.clone());
            });
            let old_ns = is_noise_suppression_enabled.get_untracked();
            set_selected_camera_id.set(vid);
            set_selected_mic_id.set(aid);
            set_video_resolution.set(res);
            set_is_noise_suppression_enabled.set(ns);

            let has_video = if let Some(stream) = local_stream.get_untracked() {
                stream.get_video_tracks().length() > 0
            } else {
                false
            };

            if local_stream.get_untracked().is_some() {
                let ns_will_trigger_restart = ns && !old_ns && audio_monitor.with_untracked(|m: &Option<AudioMonitor>| {
                    m.as_ref().is_some_and(|monitor| !monitor.has_compressor())
                });
                if !ns_will_trigger_restart {
                    start_media_stream.call(has_video);
                }
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

    let analytics_for_mic = analytics.clone();
    let toggle_mic = Callback::new(move |_: ()| {
        if is_visitor.get_untracked() { return; }
        let new_state = !is_muted.get();
        set_is_muted.set(new_state);
        analytics_for_mic.track_toggle_media("microphone", !new_state);

        if let Some(stream) = local_stream.get() {
            let audio_tracks = stream.get_audio_tracks();
            for i in 0..audio_tracks.length() {
                if let Ok(track) = audio_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.set_enabled(!new_state); // enabled = !muted
                }
            }

            set_audio_monitor.update(|monitor: &mut Option<AudioMonitor>| {
                if let Some(m) = monitor.as_mut() {
                    m.set_muted(new_state);
                }
            });
        }

        if let Some(socket) = ws.get() {
            let msg = ClientMessage::SetMuteStatus(new_state);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let authenticate = Callback::new(move |(username, password): (String, Option<String>)| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Authenticate { username, password };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let fetch_calendar = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::FetchCalendar;
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
        is_e2ee_enabled,
        is_lobby_enabled,
        is_recording,
        is_subtitles_enabled,
        subtitles,
        show_settings,
        show_polls,
        show_shortcuts,
        polls,
        last_reaction,
        show_whiteboard,
        show_etherpad,
        whiteboard_history,
        my_id,
        typing_users,
        is_host,
        host_id,
        current_room_id,
        breakout_rooms,
        is_authenticated,
        show_login_dialog,
        auth_error,
        calendar_events,
        show_calendar,
        local_stream,
        local_screen_stream,
        is_muted,
        shared_video_url,
        speaking_peers,
        audio_monitor,
        raw_local_stream,
        show_speaker_stats,
        show_virtual_background,
        show_feedback,
        rtt,
        remote_streams,
        selected_camera_id,
        selected_mic_id,
        video_resolution,
        is_noise_suppression_enabled,
        background_mode,
        grid_layout,
        is_visitor,
        room_config,
        power_statuses,
        is_recording_locally,
        lobby_announcement,
        set_input_devices,
        set_background_mode,
        set_grid_layout,
        set_show_settings,
        set_show_polls,
        set_show_shortcuts,
        set_show_whiteboard,
        set_show_etherpad,
        set_show_speaker_stats,
        set_show_virtual_background,
        set_show_feedback,
        set_show_login_dialog,
        set_auth_error,
        authenticate,
        set_show_calendar,
        fetch_calendar,
        send_ping,
        send_message,
        toggle_lock,
        toggle_e2ee,
        toggle_etherpad,
        toggle_lobby,
        toggle_recording,
        toggle_subtitles,
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
        close_poll,
        send_draw,
        set_is_typing,
        create_breakout_room,
        join_breakout_room,
        toggle_camera,
        toggle_mic,
        end_meeting,
        mute_participant,
        mute_camera_participant,
        mute_all,
        mute_camera_all,
        transfer_host,
        start_share_video,
        stop_share_video,
        set_presence,
        toggle_local_recording,
        request_unmute,
        update_power_status,
        broadcast_to_lobby,
        promote_visitor,
        analytics,
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
