use crate::utils::create_room_url;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Home() -> impl IntoView {
    let (room_name, set_room_name) = create_signal("My Meeting".to_string());
    let navigate = use_navigate();

    let settings = crate::storage::load_settings();
    let (recent_rooms, _set_recent_rooms) = create_signal(settings.recent_rooms);

    let nav_create = navigate.clone();
    let create_meeting = move |_| {
        let name = room_name.get();
        let url = create_room_url(&name);
        nav_create(&url, Default::default());
    };

    view! {
        <div class="welcome-container">
            <div class="hero-card">
                <h1>"Welcome to Juncto"</h1>
                <p>"High-Performance WebRTC Video Conferencing in Rust"</p>

                <div class="input-group">
                    <label class="input-label" for="meeting-name">"Room Name"</label>
                    <input
                        type="text"
                        id="meeting-name"
                        class="styled-input"
                        on:input=move |ev| set_room_name.set(event_target_value(&ev))
                        prop:value=room_name
                        placeholder="Enter meeting name..."
                    />
                </div>

                <button
                    on:click=create_meeting
                    class="create-btn btn btn-primary"
                    style="width: 100%; padding: 12px; font-size: 1rem; font-weight: 600;"
                >
                    "🚀 Start Meeting"
                </button>

                <Show when=move || !recent_rooms.get().is_empty()>
                    <div style="margin-top: 32px; border-top: 1px solid var(--border-color); padding-top: 24px; text-align: left;">
                        <span class="input-label">"Recent Meetings"</span>
                        <div style="display: flex; flex-wrap: wrap; gap: 8px; margin-top: 12px;">
                            <For
                                each=move || recent_rooms.get()
                                key=|r| r.clone()
                                children={
                                    let nav_loop = navigate.clone();
                                    move |r| {
                                        let r_clone = r.clone();
                                        let nav = nav_loop.clone();
                                        view! {
                                            <button
                                                class="btn btn-outline"
                                                style="border-radius: var(--radius-full); padding: 6px 14px; font-size: var(--font-size-xs);"
                                                on:click=move |_| {
                                                    let url = format!("/room/{}", urlencoding::encode(&r_clone));
                                                    nav(&url, Default::default());
                                                }
                                            >
                                                "📌 " {r}
                                            </button>
                                        }
                                    }
                                }
                            />
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_home_compiles() {
        let _ = true;
    }
}
