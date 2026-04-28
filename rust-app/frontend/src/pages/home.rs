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
        <div class="welcome-container" style="text-align: center; margin-top: 50px;">
            <h1>"Welcome to Juncto (Rust Edition)"</h1>
            <p>"Migration to Leptos + Axum complete."</p>
            <input
                type="text"
                id="meeting-name"
                on:input=move |ev| set_room_name.set(event_target_value(&ev))
                prop:value=room_name
                style="padding: 10px; margin: 10px;"
                placeholder="Meeting Name"
            />
            <button
                on:click=create_meeting
                class="create-btn"
                style="padding: 10px 20px; background-color: #007bff; color: white; border: none; cursor: pointer;"
            >
                "Start Meeting"
            </button>

            <Show when=move || !recent_rooms.get().is_empty()>
                <div style="margin-top: 30px; text-align: center;">
                    <h3>"Recent Meetings"</h3>
                    <ul style="list-style: none; padding: 0;">
                        <For
                            each=move || recent_rooms.get()
                            key=|r| r.clone()
                            children={
                                let nav_loop = navigate.clone();
                                move |r| {
                                    let r_clone = r.clone();
                                    let nav = nav_loop.clone();
                                    view! {
                                        <li style="margin-bottom: 8px;">
                                            <button
                                                on:click=move |_| {
                                                    let url = format!("/room/{}", urlencoding::encode(&r_clone));
                                                    nav(&url, Default::default());
                                                }
                                                style="background: none; border: 1px solid #ccc; padding: 5px 15px; border-radius: 20px; cursor: pointer; color: #007bff;"
                                            >
                                                {r}
                                            </button>
                                        </li>
                                    }
                                }
                            }
                        />
                    </ul>
                </div>
            </Show>
        </div>
    }
}
#[cfg(test)]
mod tests {

    #[test]
    fn test_home_compiles() {
        // dummy test
        let _ = true;
    }
}
