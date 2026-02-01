use leptos::*;
use leptos_router::*;
use crate::chat::Chat;
use crate::participants::ParticipantsList;
use crate::toolbox::Toolbox;
use crate::prejoin::PrejoinScreen;
use crate::settings::SettingsDialog;
use crate::reactions::ReactionDisplay;
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
    let (is_recording, set_is_recording) = create_signal(false);
    let (show_settings, set_show_settings) = create_signal(false);
    let (last_reaction, set_last_reaction) = create_signal(None::<(String, String)>);

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
                                set_is_recording.set(config.is_recording);
                            },
                            ServerMessage::ParticipantUpdated(p) => {
                                set_participants.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|x| x.id == p.id) {
                                        *existing = p;
                                    }
                                });
                            },
                            ServerMessage::Reaction { sender_id, emoji } => {
                                set_last_reaction.set(Some((sender_id, emoji)));
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

    let toggle_recording = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleRecording;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let save_profile = Callback::new(move |new_name: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::UpdateProfile(new_name);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let send_reaction = Callback::new(move |emoji: String| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Reaction(emoji);
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
                            <div style="position: relative; flex: 1; width: 100%; height: 100%;">
                                <div class="video-container" style="display: flex; justify-content: center; align-items: center; height: 100%;">
                                    <div>
                                        <h2>"Meeting Room: " {room_id}</h2>
                                        <Show when=move || is_recording.get()>
                                            <div style="background: red; color: white; padding: 5px; border-radius: 4px; display: inline-block; margin-bottom: 10px;">
                                                "REC"
                                            </div>
                                        </Show>
                                        <div class="video-placeholder" style="width: 640px; height: 360px; background: black; border: 2px solid #555;">
                                            <p style="text-align: center; margin-top: 160px;">"Video Stream Placeholder"</p>
                                        </div>
                                    </div>
                                </div>
                                <ReactionDisplay last_reaction=last_reaction />
                            </div>
                            <Toolbox
                                is_locked=is_locked
                                is_recording=is_recording
                                on_toggle_lock=toggle_lock
                                on_toggle_recording=toggle_recording
                                on_settings=Callback::new(move |_| set_show_settings.set(true))
                                on_reaction=send_reaction
                            />
                        </div>
                        <Chat
                            messages=messages
                            on_send=send_message
                            is_connected=is_connected
                        />
                        <SettingsDialog
                            show=show_settings
                            on_close=Callback::new(move |_| set_show_settings.set(false))
                            on_save_profile=save_profile
                        />
                    </div>
                }.into_view()
            }}
        </div>
    }
}
