use leptos::*;

#[component]
pub fn SettingsDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_save_profile: Callback<String>,
) -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("profile");
    let (display_name, set_display_name) = create_signal("".to_string());

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
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
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">"Camera"</label>
                                <select style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;">
                                    <option>"Facetime HD Camera"</option>
                                    <option>"Mock Camera 1"</option>
                                </select>
                            </div>
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">"Microphone"</label>
                                <select style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;">
                                    <option>"Default Microphone"</option>
                                    <option>"Mock Mic 1"</option>
                                </select>
                            </div>
                            <p style="color: #666; font-size: 0.9em;">"Device selection is simulated in this migration."</p>
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}
