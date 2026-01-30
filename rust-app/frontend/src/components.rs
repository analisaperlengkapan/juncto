use leptos::*;
use leptos_router::*;
use crate::chat::Chat;
use crate::participants::ParticipantsList;
use crate::toolbox::Toolbox;
use crate::prejoin::PrejoinScreen;
use shared::{ChatMessage, Participant, ServerMessage, ClientMessage};
use web_sys::{MessageEvent, WebSocket};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, PartialEq)]
enum RoomState {
    Prejoin,
    Joined,
}

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

    // State
    let (current_state, set_current_state) = create_signal(RoomState::Prejoin);
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (participants, set_participants) = create_signal(Vec::<Participant>::new());
    let (ws, set_ws) = create_signal(None::<WebSocket>);
    let (is_connected, set_is_connected) = create_signal(false);
    let (is_locked, set_is_locked) = create_signal(false);

    // Initialize WebSocket
    create_effect(move |_| {
        let location = web_sys::window().unwrap().location();
        let protocol = if location.protocol().unwrap() == "https:" { "wss:" } else { "ws:" };
        let host = location.host().unwrap();
        let url = format!("{}//{}/ws/chat", protocol, host);

        if let Ok(socket) = WebSocket::new(&url) {
            // Handle incoming messages
            let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let txt: String = txt.into();
                    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&txt) {
                        match server_msg {
                            ServerMessage::Chat(msg) => {
                                set_messages.update(|msgs| msgs.push(msg));
                            },
                            ServerMessage::ParticipantJoined(p) => {
                                set_participants.update(|list| list.push(p));
                            },
                            ServerMessage::ParticipantLeft(id) => {
                                set_participants.update(|list| list.retain(|p| p.id != id));
                            },
                            ServerMessage::ParticipantList(list) => {
                                set_participants.set(list);
                            },
                            ServerMessage::RoomUpdated(config) => {
                                set_is_locked.set(config.is_locked);
                            }
                        }
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

            set_ws.set(Some(socket));
        }
    });

    let send_message = Callback::new(move |content: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Chat(content);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_lock = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleRoomLock;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let join_meeting = Callback::new(move |display_name: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Join(display_name);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
            set_current_state.set(RoomState::Joined);
        }
    });

    view! {
        <div style="height: 100vh;">
            {move || match current_state.get() {
                RoomState::Prejoin => view! {
                    <PrejoinScreen on_join=join_meeting />
                }.into_view(),
                RoomState::Joined => view! {
                    <div class="room-container" style="display: flex; height: 100vh;">
                        <ParticipantsList participants=participants />
                        <div class="main-content" style="flex: 1; display: flex; flex-direction: column; background: #333; color: white;">
                            <div style="flex: 1; display: flex; justify-content: center; align-items: center;">
                                <div>
                                    <h2>"Meeting Room: " {room_id}</h2>
                                    <div class="video-placeholder" style="width: 640px; height: 360px; background: black; border: 2px solid #555;">
                                        <p style="text-align: center; margin-top: 160px;">"Video Stream Placeholder"</p>
                                    </div>
                                </div>
                            </div>
                            <Toolbox is_locked=is_locked on_toggle_lock=toggle_lock />
                        </div>
                        <Chat
                            messages=messages
                            on_send=send_message
                            is_connected=is_connected
                        />
                    </div>
                }.into_view()
            }}
        </div>
    }
}
