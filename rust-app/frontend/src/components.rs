use leptos::*;
use leptos_router::*;

#[component]
pub fn WelcomePage() -> impl IntoView {
    let (room_name, set_room_name) = create_signal("My Meeting".to_string());
    let navigate = use_navigate();

    let create_meeting = move |_| {
        let name = room_name.get();
        // Simple sanitization/encoding for URL
        let encoded_name = urlencoding::encode(&name);
        navigate(&format!("/room/{}", encoded_name), Default::default());
    };

    view! {
        <div class="welcome-container" style="text-align: center; margin-top: 50px;">
            <h1>"Welcome to Juncto (Rust Edition)"</h1>
            <p>"Migration to Leptos + Axum complete."</p>
            <input
                type="text"
                on:input=move |ev| set_room_name.set(event_target_value(&ev))
                prop:value=room_name
                style="padding: 10px; margin: 10px;"
            />
            <button
                on:click=create_meeting
                class="create-btn"
                style="padding: 10px 20px; background-color: #007bff; color: white; border: none; cursor: pointer;"
            >
                "Start Meeting"
            </button>
        </div>
    }
}

#[component]
pub fn Room() -> impl IntoView {
    let params = use_params_map();
    let room_id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());

    view! {
        <div class="room-container" style="display: flex; height: 100vh;">
            <div class="sidebar" style="width: 200px; background: #eee; padding: 20px;">
                <h3>"Participants"</h3>
                <ul>
                    <li>"You (Host)"</li>
                </ul>
            </div>
            <div class="main-content" style="flex: 1; display: flex; justify-content: center; align-items: center; background: #333; color: white;">
                <div>
                    <h2>"Meeting Room: " {room_id}</h2>
                    <div class="video-placeholder" style="width: 640px; height: 360px; background: black; border: 2px solid #555;">
                        <p style="text-align: center; margin-top: 160px;">"Video Stream Placeholder"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
