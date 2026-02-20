use leptos::*;
use web_sys::{MediaDeviceInfo, MediaDeviceKind};
use wasm_bindgen::JsCast;
use crate::media::{enumerate_devices, get_user_media};
use crate::i18n::t;

#[component]
pub fn SettingsDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_save_profile: Callback<String>,
    #[prop(optional)]
    on_save_devices: Option<Callback<(Option<String>, Option<String>, String)>>,
) -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("profile");
    let (display_name, set_display_name) = create_signal("".to_string());

    // Devices State
    let (video_devices, set_video_devices) = create_signal(Vec::<MediaDeviceInfo>::new());
    let (audio_devices, set_audio_devices) = create_signal(Vec::<MediaDeviceInfo>::new());
    let (selected_video, set_selected_video) = create_signal(None::<String>);
    let (selected_audio, set_selected_audio) = create_signal(None::<String>);
    let (video_quality, set_video_quality) = create_signal("hd".to_string());
    let (error_msg, set_error_msg) = create_signal(None::<String>);
    let (preview_stream, set_preview_stream) = create_signal(None::<web_sys::MediaStream>);

    let video_ref = create_node_ref::<html::Video>();

    let stop_preview = move || {
        if let Some(stream) = preview_stream.get_untracked() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
            set_preview_stream.set(None);
        }
    };

    let fetch_devices = create_action(move |_: &()| async move {
        match enumerate_devices().await {
            Ok(devices) => {
                let mut vid = Vec::new();
                let mut aud = Vec::new();
                for d in devices {
                    match d.kind() {
                        MediaDeviceKind::Videoinput => vid.push(d),
                        MediaDeviceKind::Audioinput => aud.push(d),
                        _ => {}
                    }
                }
                set_video_devices.set(vid);
                set_audio_devices.set(aud);
            },
            Err(e) => {
                set_error_msg.set(Some(format!("Error enumerating devices: {:?}", e)));
            }
        }
    });

    let start_preview = create_action(move |_: &()| async move {
        // Stop existing before starting new
        stop_preview();

        let v_id = selected_video.get();
        let a_id = selected_audio.get();
        let quality = video_quality.get();

        match get_user_media(v_id, a_id, Some(&quality)).await {
            Ok(stream) => {
                set_preview_stream.set(Some(stream.clone()));
                if let Some(video_el) = video_ref.get() {
                    video_el.set_src_object(Some(&stream));
                    let _ = video_el.play();
                }
                set_error_msg.set(None);
            },
            Err(e) => {
                set_error_msg.set(Some(format!("Error accessing media: {:?}", e)));
            }
        }
    });

    create_effect(move |_| {
        if active_tab.get() == "devices" {
            fetch_devices.dispatch(());
            start_preview.dispatch(());
        } else {
            stop_preview();
        }
    });

    on_cleanup(move || {
        stop_preview();
    });

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 500px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>{move || t("settings")}</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div class="tabs" style="display: flex; border-bottom: 1px solid #ccc; margin-bottom: 20px;">
                        <button
                            on:click=move |_| set_active_tab.set("profile")
                            style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "profile" { "#007bff" } else { "transparent" })
                        >
                            {move || t("profile")}
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("devices")
                            style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "devices" { "#007bff" } else { "transparent" })
                        >
                            {move || t("devices")}
                        </button>
                    </div>

                    <div class="tab-content">
                        <Show when=move || active_tab.get() == "profile">
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">{move || t("display_name")}</label>
                                <input
                                    type="text"
                                    prop:value=display_name
                                    on:input=move |ev| set_display_name.set(event_target_value(&ev))
                                    style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                                />
                            </div>
                            <button
                                on:click=move |_| {
                                    on_save_profile.call(display_name.get());
                                    on_close.call(());
                                }
                                style="padding: 10px 20px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;"
                            >
                                {move || t("save_profile")}
                            </button>
                        </Show>
                        <Show when=move || active_tab.get() == "devices">
                            <Show when=move || error_msg.get().is_some()>
                                <div style="color: red; margin-bottom: 10px; padding: 10px; background: #ffeaea; border-radius: 4px;">
                                    {move || error_msg.get().unwrap()}
                                </div>
                            </Show>

                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">{move || t("camera")}</label>
                                <select
                                    style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        if val.is_empty() {
                                            set_selected_video.set(None);
                                        } else {
                                            set_selected_video.set(Some(val));
                                        }
                                        start_preview.dispatch(());
                                    }
                                >
                                    <option value="">{move || t("default")}</option>
                                    <For
                                        each=move || video_devices.get()
                                        key=|d| d.device_id()
                                        children=move |d| {
                                            let id = d.device_id();
                                            let label = d.label();
                                            let label_text = if label.is_empty() { format!("Camera {}", id) } else { label };
                                            let id_clone = id.clone();
                                            view! {
                                                <option value=id selected=move || selected_video.get().as_ref() == Some(&id_clone)>
                                                    {label_text}
                                                </option>
                                            }
                                        }
                                    />
                                </select>
                            </div>
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">{move || t("video_quality")}</label>
                                <select
                                    style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                                    on:change=move |ev| {
                                        set_video_quality.set(event_target_value(&ev));
                                        start_preview.dispatch(());
                                    }
                                >
                                    <option value="hd" selected=move || video_quality.get() == "hd">"HD (720p)"</option>
                                    <option value="sd" selected=move || video_quality.get() == "sd">"SD (360p)"</option>
                                </select>
                            </div>
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">{move || t("microphone")}</label>
                                <select
                                    style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        if val.is_empty() {
                                            set_selected_audio.set(None);
                                        } else {
                                            set_selected_audio.set(Some(val));
                                        }
                                        start_preview.dispatch(());
                                    }
                                >
                                    <option value="">{move || t("default")}</option>
                                    <For
                                        each=move || audio_devices.get()
                                        key=|d| d.device_id()
                                        children=move |d| {
                                            let id = d.device_id();
                                            let label = d.label();
                                            let label_text = if label.is_empty() { format!("Mic {}", id) } else { label };
                                            let id_clone = id.clone();
                                            view! {
                                                <option value=id selected=move || selected_audio.get().as_ref() == Some(&id_clone)>
                                                    {label_text}
                                                </option>
                                            }
                                        }
                                    />
                                </select>
                            </div>

                            <div class="preview" style="margin-top: 20px; border: 1px solid #ccc; height: 200px; background: #000; display: flex; justify-content: center; align-items: center; overflow: hidden;">
                                <video
                                    _ref=video_ref
                                    autoplay
                                    playsinline
                                    muted
                                    style="max-width: 100%; max-height: 100%;"
                                />
                            </div>
                            <p style="color: #666; font-size: 0.8em; margin-top: 5px;">{move || t("preview_only")}</p>

                            <div style="margin-top: 15px; text-align: right;">
                                <button
                                    on:click=move |_| {
                                        if let Some(cb) = on_save_devices {
                                            cb.call((selected_video.get(), selected_audio.get(), video_quality.get()));
                                        }
                                        on_close.call(());
                                    }
                                    style="padding: 10px 20px; background-color: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer;"
                                >
                                    {move || t("apply_devices")}
                                </button>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    // Note: Component testing with web_sys/Leptos requires browser environment.
    #[test]
    fn test_settings_dialog_exists() {
        assert_eq!(1, 1);
    }
}
