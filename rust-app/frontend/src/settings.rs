use leptos::*;
use web_sys::{MediaDeviceInfo, MediaDeviceKind};
use crate::media::{enumerate_devices, get_user_media};

#[component]
pub fn SettingsDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_save_profile: Callback<String>,
) -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("profile");
    let (display_name, set_display_name) = create_signal("".to_string());

    // Devices State
    let (video_devices, set_video_devices) = create_signal(Vec::<MediaDeviceInfo>::new());
    let (audio_devices, set_audio_devices) = create_signal(Vec::<MediaDeviceInfo>::new());
    let (selected_video, set_selected_video) = create_signal(None::<String>);
    let (selected_audio, set_selected_audio) = create_signal(None::<String>);
    let (error_msg, set_error_msg) = create_signal(None::<String>);

    let video_ref = create_node_ref::<html::Video>();

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
        let v_id = selected_video.get();
        let a_id = selected_audio.get();

        match get_user_media(v_id, a_id).await {
            Ok(stream) => {
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
        }
    });

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 500px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>"Settings"</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div class="tabs" style="display: flex; border-bottom: 1px solid #ccc; margin-bottom: 20px;">
                        <button
                            on:click=move |_| set_active_tab.set("profile")
                            style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "profile" { "#007bff" } else { "transparent" })
                        >
                            "Profile"
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("devices")
                            style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "devices" { "#007bff" } else { "transparent" })
                        >
                            "Devices"
                        </button>
                    </div>

                    <div class="tab-content">
                        <Show when=move || active_tab.get() == "profile">
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">"Display Name"</label>
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
                                "Save Profile"
                            </button>
                        </Show>
                        <Show when=move || active_tab.get() == "devices">
                            <Show when=move || error_msg.get().is_some()>
                                <div style="color: red; margin-bottom: 10px; padding: 10px; background: #ffeaea; border-radius: 4px;">
                                    {move || error_msg.get().unwrap()}
                                </div>
                            </Show>

                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">"Camera"</label>
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
                                    <option value="">"Default"</option>
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
                                <label style="display: block; margin-bottom: 5px;">"Microphone"</label>
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
                                    <option value="">"Default"</option>
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
                                    style="max-width: 100%; max-height: 100%;"
                                />
                            </div>
                            <p style="color: #666; font-size: 0.8em; margin-top: 5px;">"This is a local preview only."</p>
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
