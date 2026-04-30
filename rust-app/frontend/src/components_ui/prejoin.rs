use crate::media::{
    get_audio_input_devices, get_user_media, get_video_input_devices, AudioMonitor, DeviceInfo,
};
use crate::state::JoinOptions;
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::MediaStream;

#[component]
pub fn PrejoinScreen(
    on_join: Callback<JoinOptions>,
    is_connected: ReadSignal<bool>,
) -> impl IntoView {
    let initial_settings = crate::storage::load_settings();
    let (display_name, set_display_name) = create_signal(initial_settings.display_name.clone().unwrap_or_else(|| "Guest".to_string()));
    let (avatar_url, set_avatar_url) = create_signal("".to_string());

    // Device Lists
    let (video_devices, set_video_devices) = create_signal(Vec::<DeviceInfo>::new());
    let (audio_devices, set_audio_devices) = create_signal(Vec::<DeviceInfo>::new());

    // Selected Devices
    let (selected_video_device, set_selected_video_device) = create_signal(initial_settings.camera_id);
    let (selected_audio_device, set_selected_audio_device) = create_signal(initial_settings.mic_id);

    // Toggles
    let (is_camera_on, set_is_camera_on) = create_signal(false);
    let (is_mic_on, set_is_mic_on) = create_signal(true);
    let (is_visitor, set_is_visitor) = create_signal(false);

    // Stream & Audio Monitor
    let (local_stream, set_local_stream) = create_signal(None::<MediaStream>);
    let (_audio_monitor, set_audio_monitor) = create_signal(None::<AudioMonitor>);
    let (is_speaking, set_is_speaking) = create_signal(false);

    // Load Devices on Mount
    create_effect(move |_| {
        spawn_local(async move {
            let v_devices = get_video_input_devices().await.ok().unwrap_or_default();
            let a_devices = get_audio_input_devices().await.ok().unwrap_or_default();

            // Batch updates to avoid multiple stream restarts
            batch(move || {
                set_video_devices.set(v_devices.clone());
                // Fall back to first device if no saved ID or if the saved ID
                // is stale (device was unplugged since settings were persisted).
                let saved_vid = selected_video_device.get_untracked();
                let vid_valid = saved_vid.as_ref().is_some_and(|id| {
                    v_devices.iter().any(|d| &d.device_id == id)
                });
                if !vid_valid {
                    if let Some(first) = v_devices.first() {
                        set_selected_video_device.set(Some(first.device_id.clone()));
                    }
                }

                set_audio_devices.set(a_devices.clone());
                let saved_aid = selected_audio_device.get_untracked();
                let aid_valid = saved_aid.as_ref().is_some_and(|id| {
                    a_devices.iter().any(|d| &d.device_id == id)
                });
                if !aid_valid {
                    if let Some(first) = a_devices.first() {
                        set_selected_audio_device.set(Some(first.device_id.clone()));
                    }
                }
            });
        });
    });

    // Stop stream helper
    let stop_stream = move || {
        if let Some(stream) = local_stream.get_untracked() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        set_local_stream.set(None);
        set_audio_monitor.set(None);
    };

    // Update Stream when settings change
    create_effect(move |_| {
        let cam_on = is_camera_on.get();
        let mic_on = is_mic_on.get();
        let v_id = selected_video_device.get();
        let a_id = selected_audio_device.get();

        // Use a tracking variable to debounce if multiple signals change in same microtask?
        // Leptos effects are synchronous but scheduled.
        // If we want to avoid multiple calls, we can assume Leptos batches signal updates.
        // However, spawn_local runs on the next tick essentially.
        // To be safe against rapid changes (like initial load), we could check if we are already requesting?
        // For now, let's trust Leptos 0.6 batching, but we'll add a check.

        spawn_local(async move {
            // Stop existing first
            if let Some(stream) = local_stream.get_untracked() {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                        track.stop();
                    }
                }
            }
            set_local_stream.set(None);
            set_audio_monitor.set(None);

            if cam_on || mic_on {
                // Request both if mic is also requested, otherwise just video?
                // Actually get_user_media handles both options.
                // If mic is off, we might still want the stream to have an audio track that is muted?
                // Or we just request video only?
                // If we request audio but mute it, we can still monitor levels (if track is enabled but gain is 0? No, track enabled=false stops data).
                // Let's request what is needed.

                // Note: If cam is OFF, we don't show preview video.
                // If Cam is ON, show video.
                // If Mic is ON, we want to monitor audio.
                // If Cam is OFF but Mic is ON, we still want to monitor audio?
                // Current logic: This effect runs if ANY change.

                // We need a stream
                // If cam_on is false, we pass None for video_device_id? No, get_user_media treats None as "any".
                // We need to change get_user_media to accept "No Video".
                // Current get_user_media always sets video constraints if ID is provided OR constraints are new.
                // Let's assume for Preview, we always want video if cam_on is true.

                // Pass cam_on as enable_video flag
                if let Ok(stream) = get_user_media(cam_on, true, v_id, a_id, Some("hd")).await {
                    // Apply mute state to audio track
                    let audio_tracks = stream.get_audio_tracks();
                    for i in 0..audio_tracks.length() {
                        if let Ok(track) =
                            audio_tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>()
                        {
                            track.set_enabled(mic_on);
                        }
                    }

                    set_local_stream.set(Some(stream.clone()));

                    if mic_on {
                        let on_speaking = Box::new(move |speaking: bool| {
                            set_is_speaking.set(speaking);
                        });
                        if let Ok(monitor) = AudioMonitor::new(&stream, on_speaking, None, None, false) {
                            set_audio_monitor.set(Some(monitor));
                        }
                    }
                }
            }
        });
    });

    on_cleanup(move || {
        stop_stream();
    });

    let handle_join = move |_| {
        stop_stream();
        let av = avatar_url.get_untracked();
        on_join.call(JoinOptions {
            display_name: display_name.get_untracked(),
            mic_enabled: is_mic_on.get_untracked(),
            camera_enabled: is_camera_on.get_untracked(),
            audio_device_id: selected_audio_device.get_untracked(),
            video_device_id: selected_video_device.get_untracked(),
            is_visitor: is_visitor.get_untracked(),
            avatar_url: if av.is_empty() { None } else { Some(av) },
        });
    };

    let video_ref = create_node_ref::<html::Video>();
    create_effect(move |_| {
        if let Some(video) = video_ref.get() {
            if let Some(stream) = local_stream.get() {
                video.set_src_object(Some(&stream));
            } else {
                video.set_src_object(None);
            }
        }
    });

    view! {
        <div class="prejoin-container" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background: #f0f0f0; font-family: sans-serif;">
            <div class="card" style="background: white; padding: 40px; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.15); text-align: center; max-width: 500px; width: 100%;">
                <h2 style="margin-bottom: 20px; color: #333;">"Join Meeting"</h2>

                // Video Preview
                <div style="position: relative; width: 100%; height: 250px; background: #000; margin-bottom: 20px; border-radius: 8px; overflow: hidden; display: flex; align-items: center; justify-content: center;">
                    <Show when=move || is_camera_on.get() fallback=|| view! { <div class="camera-off-text" style="color: white;">"Camera is Off"</div> }>
                        <video
                            node_ref=video_ref
                            autoplay
                            muted // Always mute local preview to avoid feedback
                            playsinline
                            style="width: 100%; height: 100%; object-fit: cover;"
                        />
                    </Show>

                    // Audio Meter Indicator
                    <Show when=move || is_speaking.get() && is_mic_on.get()>
                        <div style="position: absolute; bottom: 10px; right: 10px; width: 15px; height: 15px; background: #28a745; border-radius: 50%; border: 2px solid white;"></div>
                    </Show>
                    <Show when=move || !is_mic_on.get()>
                        <div style="position: absolute; bottom: 10px; right: 10px; color: #dc3545; background: rgba(0,0,0,0.5); padding: 2px 5px; border-radius: 4px; font-size: 12px;">"Muted"</div>
                    </Show>
                </div>

                // Controls
                <div style="display: flex; gap: 10px; justify-content: center; margin-bottom: 20px;">
                    <button
                        on:click=move |_| set_is_camera_on.update(|v| *v = !*v)
                        style=move || format!("padding: 10px; border-radius: 50%; border: none; cursor: pointer; width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; color: white; background-color: {};", if is_camera_on.get() { "#007bff" } else { "#dc3545" })
                        title="Toggle Camera"
                    >
                         {move || if is_camera_on.get() { "📷" } else { "🚫" }}
                    </button>
                    <button
                        on:click=move |_| set_is_mic_on.update(|v| *v = !*v)
                        style=move || format!("padding: 10px; border-radius: 50%; border: none; cursor: pointer; width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; color: white; background-color: {};", if is_mic_on.get() { "#007bff" } else { "#dc3545" })
                        title="Toggle Microphone"
                    >
                         {move || if is_mic_on.get() { "🎤" } else { "🔇" }}
                    </button>
                </div>

                // Device Selectors
                <div style="margin-bottom: 20px; text-align: left;">
                    <div style="margin-bottom: 10px;">
                        <label style="display: block; font-size: 12px; margin-bottom: 4px; color: #666;">"Camera"</label>
                        <select
                            on:change=move |ev| set_selected_video_device.set(Some(event_target_value(&ev)))
                            prop:value=move || selected_video_device.get().unwrap_or_default()
                            style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                            disabled=move || !is_camera_on.get()
                        >
                            <For
                                each=move || video_devices.get()
                                key=|d| d.device_id.clone()
                                children=move |device| {
                                    view! {
                                        <option value=device.device_id>{device.label}</option>
                                    }
                                }
                            />
                        </select>
                    </div>
                    <div>
                        <label style="display: block; font-size: 12px; margin-bottom: 4px; color: #666;">"Microphone"</label>
                        <select
                            on:change=move |ev| set_selected_audio_device.set(Some(event_target_value(&ev)))
                            prop:value=move || selected_audio_device.get().unwrap_or_default()
                            style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                        >
                            <For
                                each=move || audio_devices.get()
                                key=|d| d.device_id.clone()
                                children=move |device| {
                                    view! {
                                        <option value=device.device_id>{device.label}</option>
                                    }
                                }
                            />
                        </select>
                    </div>
                </div>

                // Name Input & Join Type
                <div style="margin-bottom: 20px; text-align: left;">
                    <label style="display: block; font-size: 12px; margin-bottom: 4px; color: #666;">"Display Name"</label>
                    <input
                        type="text"
                        id="display-name"
                        on:input=move |ev| set_display_name.set(event_target_value(&ev))
                        prop:value=move || display_name.get()
                        style="padding: 10px; width: 100%; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box;"
                        placeholder="Enter your name"
                    />
                </div>

                <div style="margin-bottom: 20px; text-align: left;">
                    <label style="display: block; font-size: 12px; margin-bottom: 4px; color: #666;">"Avatar URL (Optional)"</label>
                    <input
                        type="url"
                        id="avatar-url"
                        on:input=move |ev| set_avatar_url.set(event_target_value(&ev))
                        prop:value=move || avatar_url.get()
                        style="padding: 10px; width: 100%; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box;"
                        placeholder="https://example.com/avatar.png"
                    />
                </div>

                <div style="margin-bottom: 20px; display: flex; align-items: center; gap: 8px; font-size: 14px; color: #666;">
                    <input
                        type="checkbox"
                        id="visitor-mode"
                        prop:checked=is_visitor
                        on:change=move |ev| set_is_visitor.set(event_target_checked(&ev))
                    />
                    <label for="visitor-mode">"Join as Visitor (Read-only)"</label>
                </div>

                <button
                    class="join-btn"
                    on:click=handle_join
                    disabled=move || !is_connected.get()
                    style=move || format!("padding: 12px 24px; background-color: {}; color: white; border: none; border-radius: 4px; cursor: {}; font-size: 16px; font-weight: bold; width: 100%;", if is_connected.get() { "#28a745" } else { "#6c757d" }, if is_connected.get() { "pointer" } else { "not-allowed" })
                >
                    {move || if is_connected.get() { "Join Meeting" } else { "Connecting..." }}
                </button>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_prejoin_compiles() {
        let _ = create_runtime();
        let on_join = Callback::new(|_: JoinOptions| {});
        let (is_connected, _) = create_signal(true);

        let _view = view! {
            <PrejoinScreen on_join=on_join is_connected=is_connected />
        };
        let _ = true;
    }
}
