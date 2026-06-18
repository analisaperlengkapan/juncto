use leptos::*;
use web_sys::{MediaStream, WebSocket, MediaStreamTrack};
use wasm_bindgen::JsCast;
use shared::ClientMessage;
use crate::analytics::AnalyticsService;
use crate::media::AudioMonitor;
use crate::components_ui::toast::{ToastType, use_toast};

pub fn toggle_mic_logic(
    is_visitor: Signal<bool>,
    is_muted: ReadSignal<bool>,
    set_is_muted: WriteSignal<bool>,
    room_config: ReadSignal<shared::RoomConfig>,
    has_unmute_permission: ReadSignal<bool>,
    is_host: Signal<bool>,
    analytics: AnalyticsService,
    local_stream: ReadSignal<Option<MediaStream>>,
    set_audio_monitor: WriteSignal<Option<AudioMonitor>>,
    ws: ReadSignal<Option<WebSocket>>,
) {
    if is_visitor.get_untracked() {
        return;
    }

    let is_currently_muted = is_muted.get_untracked();
    if is_currently_muted {
        let is_mod_enabled = room_config.with_untracked(|c| c.audio_moderation_enabled);
        let has_perm = has_unmute_permission.get_untracked();
        if is_mod_enabled && !has_perm && !is_host.get_untracked() {
            let toast = use_toast();
            toast.add(
                "Audio is moderated. Request permission to unmute.".to_string(),
                ToastType::Error,
            );
            return;
        }
    }

    let new_state = !is_currently_muted;
    set_is_muted.set(new_state);
    analytics.track_toggle_media("microphone", !new_state);

    if let Some(stream) = local_stream.get_untracked() {
        let audio_tracks = stream.get_audio_tracks();
        for i in 0..audio_tracks.length() {
            if let Ok(track) = audio_tracks.get(i).dyn_into::<MediaStreamTrack>() {
                track.set_enabled(!new_state); // enabled = !muted
            }
        }

        set_audio_monitor.update(|monitor: &mut Option<AudioMonitor>| {
            if let Some(m) = monitor.as_mut() {
                m.set_muted(new_state);
            }
        });
    }

    if let Some(socket) = ws.get_untracked() {
        let msg = ClientMessage::SetMuteStatus(new_state);
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = socket.send_with_str(&json);
        }
    }
}

pub fn toggle_camera_logic(
    is_visitor: Signal<bool>,
    local_stream: ReadSignal<Option<MediaStream>>,
    room_config: ReadSignal<shared::RoomConfig>,
    has_camera_permission: ReadSignal<bool>,
    is_host: Signal<bool>,
    is_camera_off: ReadSignal<bool>,
    set_is_camera_off: WriteSignal<bool>,
    analytics: AnalyticsService,
    raw_local_stream: ReadSignal<Option<MediaStream>>,
    ws: ReadSignal<Option<WebSocket>>,
    start_media_stream: Callback<bool>,
) {
    if is_visitor.get_untracked() {
        return;
    }

    let is_currently_off = if let Some(stream) = local_stream.get_untracked() {
        stream.get_video_tracks().length() == 0
    } else {
        true
    };

    if is_currently_off {
        let is_mod_enabled = room_config.with_untracked(|c| c.video_moderation_enabled);
        let has_perm = has_camera_permission.get_untracked();
        if is_mod_enabled && !has_perm && !is_host.get_untracked() {
            let toast = use_toast();
            toast.add(
                "Camera is moderated. Request permission to enable.".to_string(),
                ToastType::Error,
            );
            return;
        }
    }

    if is_camera_off.get_untracked() {
        set_is_camera_off.set(false);
        analytics.track_toggle_media("camera", true);
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

    let has_video = if let Some(stream) = local_stream.get_untracked() {
        stream.get_video_tracks().length() > 0
    } else {
        false
    };
    let new_state = !has_video;
    analytics.track_toggle_media("camera", new_state);

    if let Some(socket) = ws.get_untracked() {
        let msg = ClientMessage::SetCameraMuteStatus(!new_state);
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = socket.send_with_str(&json);
        }
    }

    start_media_stream.call(new_state);
}
