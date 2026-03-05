use crate::media::{
    get_audio_input_devices, get_user_media, get_video_input_devices, AudioMonitor, DeviceInfo,
};
use crate::state::JoinOptions;
use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::MediaStream;

#[component]
pub fn PrejoinScreen(on_join: Callback<JoinOptions>) -> impl IntoView {
    let (display_name, set_display_name) = create_signal("Guest".to_string());

    // Device Lists
    let (video_devices, set_video_devices) = create_signal(Vec::<DeviceInfo>::new());
    let (audio_devices, set_audio_devices) = create_signal(Vec::<DeviceInfo>::new());

    // Selected Devices
    let (selected_video_device, set_selected_video_device) = create_signal(None::<String>);
    let (selected_audio_device, set_selected_audio_device) = create_signal(None::<String>);

    // Toggles
    let (is_camera_on, set_is_camera_on) = create_signal(false);
    let (is_mic_on, set_is_mic_on) = create_signal(true);

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
                if let Some(first) = v_devices.first() {
                    set_selected_video_device.set(Some(first.device_id.clone()));
                }

                set_audio_devices.set(a_devices.clone());
                if let Some(first) = a_devices.first() {
                    set_selected_audio_device.set(Some(first.device_id.clone()));
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
                        if let Ok(monitor) = AudioMonitor::new(&stream, on_speaking) {
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
        on_join.call(JoinOptions {
            display_name: display_name.get_untracked(),
            mic_enabled: is_mic_on.get_untracked(),
            camera_enabled: is_camera_on.get_untracked(),
            audio_device_id: selected_audio_device.get_untracked(),
            video_device_id: selected_video_device.get_untracked(),
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
        <div class="min-h-screen bg-gray-100 flex items-center justify-center p-4">
            <div class="max-w-4xl w-full bg-white rounded-2xl shadow-xl overflow-hidden flex flex-col md:flex-row">

                // Left side: Video Preview
                <div class="md:w-3/5 bg-gray-900 p-6 flex flex-col items-center justify-center relative min-h-[300px] md:min-h-[500px]">
                    <Show when=move || is_camera_on.get() fallback=|| view! {
                        <div class="flex flex-col items-center justify-center text-gray-400 space-y-4">
                            <svg class="w-16 h-16" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"></path></svg>
                            <p class="text-lg font-medium">"Camera is off"</p>
                        </div>
                    }>
                        <video
                            node_ref=video_ref
                            autoplay
                            muted
                            playsinline
                            class="w-full h-full object-cover rounded-xl shadow-lg border border-gray-700 transform scale-x-[-1]"
                        />
                    </Show>

                    // Audio Meter / Mute indicator
                    <div class="absolute bottom-10 right-10 flex items-center space-x-2">
                        <Show when=move || is_speaking.get() && is_mic_on.get()>
                            <div class="w-4 h-4 bg-green-500 rounded-full border-2 border-white animate-pulse"></div>
                        </Show>
                        <Show when=move || !is_mic_on.get()>
                            <div class="bg-red-500 text-white text-xs px-2 py-1 rounded shadow-sm flex items-center space-x-1">
                                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" clip-rule="evenodd"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"></path></svg>
                                <span>"Muted"</span>
                            </div>
                        </Show>
                    </div>

                    // Quick Toggle Controls overlaid on video
                    <div class="absolute bottom-8 flex space-x-4 bg-gray-900 bg-opacity-60 px-6 py-3 rounded-full backdrop-blur-sm border border-gray-700">
                        <button
                            on:click=move |_| set_is_mic_on.update(|v| *v = !*v)
                            class=move || format!("p-3 rounded-full transition duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900 {}",
                                if is_mic_on.get() { "bg-gray-700 text-white hover:bg-gray-600" } else { "bg-red-500 text-white hover:bg-red-600" })
                            title="Toggle Microphone"
                        >
                            <Show when=move || is_mic_on.get() fallback=|| view! {
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" clip-rule="evenodd"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"></path></svg>
                            }>
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"></path></svg>
                            </Show>
                        </button>
                        <button
                            on:click=move |_| set_is_camera_on.update(|v| *v = !*v)
                            class=move || format!("p-3 rounded-full transition duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900 {}",
                                if is_camera_on.get() { "bg-gray-700 text-white hover:bg-gray-600" } else { "bg-red-500 text-white hover:bg-red-600" })
                            title="Toggle Camera"
                        >
                            <Show when=move || is_camera_on.get() fallback=|| view! {
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"></path></svg>
                            }>
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"></path></svg>
                            </Show>
                        </button>
                    </div>
                </div>

                // Right side: Join Form
                <div class="md:w-2/5 p-8 flex flex-col justify-center bg-white">
                    <h2 class="text-3xl font-extrabold text-gray-900 mb-6 tracking-tight">"Ready to join?"</h2>

                    <div class="space-y-5">
                        // Name Input
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">"Display Name"</label>
                            <input
                                type="text"
                                prop:value=display_name
                                on:input=move |ev| set_display_name.set(event_target_value(&ev))
                                class="w-full px-4 py-3 border border-gray-300 rounded-lg shadow-sm focus:ring-blue-500 focus:border-blue-500 text-gray-900"
                                placeholder="Enter your name"
                            />
                        </div>

                        // Settings Accordion / Details
                        <div class="pt-4 border-t border-gray-200">
                            <h3 class="text-sm font-medium text-gray-900 mb-3">"Device Settings"</h3>

                            <div class="space-y-4">
                                <div>
                                    <label class="block text-xs font-medium text-gray-500 mb-1 uppercase tracking-wider">"Microphone"</label>
                                    <select
                                        on:change=move |ev| set_selected_audio_device.set(Some(event_target_value(&ev)))
                                        prop:value=move || selected_audio_device.get().unwrap_or_default()
                                        class="w-full pl-3 pr-10 py-2 text-sm border border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 rounded-md bg-gray-50"
                                    >
                                        <For
                                            each=move || audio_devices.get()
                                            key=|d| d.device_id.clone()
                                            children=move |device| {
                                                view! { <option value=device.device_id>{device.label}</option> }
                                            }
                                        />
                                    </select>
                                </div>

                                <div>
                                    <label class="block text-xs font-medium text-gray-500 mb-1 uppercase tracking-wider">"Camera"</label>
                                    <select
                                        on:change=move |ev| set_selected_video_device.set(Some(event_target_value(&ev)))
                                        prop:value=move || selected_video_device.get().unwrap_or_default()
                                        disabled=move || !is_camera_on.get()
                                        class="w-full pl-3 pr-10 py-2 text-sm border border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 rounded-md bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                    >
                                        <For
                                            each=move || video_devices.get()
                                            key=|d| d.device_id.clone()
                                            children=move |device| {
                                                view! { <option value=device.device_id>{device.label}</option> }
                                            }
                                        />
                                    </select>
                                </div>
                            </div>
                        </div>

                        // Join Button
                        <div class="pt-6">
                            <button
                                on:click=handle_join
                                class="w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-md text-base font-semibold text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 transition duration-150 ease-in-out transform hover:-translate-y-0.5"
                            >
                                "Join Meeting"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
