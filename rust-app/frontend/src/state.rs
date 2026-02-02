use leptos::*;
use shared::{ChatMessage, Participant, ServerMessage, ClientMessage, Poll, DrawAction};
use web_sys::{MessageEvent, WebSocket};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, PartialEq, Debug)]
pub enum RoomConnectionState {
    Prejoin,
    Lobby,
    Joined,
}

#[derive(Clone)]
pub struct RoomState {
    pub connection_state: ReadSignal<RoomConnectionState>,
    pub messages: ReadSignal<Vec<ChatMessage>>,
    pub participants: ReadSignal<Vec<Participant>>,
    pub knocking_participants: ReadSignal<Vec<Participant>>,
    pub is_connected: ReadSignal<bool>,
    pub is_locked: ReadSignal<bool>,
    pub is_lobby_enabled: ReadSignal<bool>,
    pub is_recording: ReadSignal<bool>,
    pub show_settings: ReadSignal<bool>,
    pub show_polls: ReadSignal<bool>,
    pub polls: ReadSignal<Vec<Poll>>,
    pub last_reaction: ReadSignal<Option<(String, String, u64)>>,
    pub show_whiteboard: ReadSignal<bool>,
    pub whiteboard_history: ReadSignal<Vec<DrawAction>>,
    pub my_id: ReadSignal<Option<String>>,
    // Setters or Actions
    pub set_show_settings: WriteSignal<bool>,
    pub set_show_polls: WriteSignal<bool>,
    pub set_show_whiteboard: WriteSignal<bool>,
    pub send_message: Callback<String>,
    pub toggle_lock: Callback<()>,
    pub toggle_lobby: Callback<()>,
    pub toggle_recording: Callback<()>,
    pub grant_access: Callback<String>,
    pub deny_access: Callback<String>,
    pub join_meeting: Callback<String>,
    pub save_profile: Callback<String>,
    pub send_reaction: Callback<String>,
    pub toggle_raise_hand: Callback<()>,
    pub toggle_screen_share: Callback<()>,
    pub create_poll: Callback<Poll>,
    pub vote_poll: Callback<(String, u32)>,
    pub send_draw: Callback<DrawAction>,
}

pub fn use_room_state() -> RoomState {
    let (current_state, set_current_state) = create_signal(RoomConnectionState::Prejoin);
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (participants, set_participants) = create_signal(Vec::<Participant>::new());
    let (knocking_participants, set_knocking_participants) = create_signal(Vec::<Participant>::new());
    let (ws, set_ws) = create_signal(None::<WebSocket>);
    let (is_connected, set_is_connected) = create_signal(false);
    let (is_locked, set_is_locked) = create_signal(false);
    let (is_lobby_enabled, set_is_lobby_enabled) = create_signal(false);
    let (is_recording, set_is_recording) = create_signal(false);
    let (show_settings, set_show_settings) = create_signal(false);
    let (show_polls, set_show_polls) = create_signal(false);
    let (polls, set_polls) = create_signal(Vec::<Poll>::new());
    let (last_reaction, set_last_reaction) = create_signal(None::<(String, String, u64)>);
    let (show_whiteboard, set_show_whiteboard) = create_signal(false);
    let (whiteboard_history, set_whiteboard_history) = create_signal(Vec::<DrawAction>::new());
    let (_last_draw_action, set_last_draw_action) = create_signal(None::<DrawAction>);
    let (my_id, set_my_id) = create_signal(None::<String>);

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
                            ServerMessage::Welcome { id } => {
                                set_my_id.set(Some(id));
                                set_current_state.set(RoomConnectionState::Joined);
                            },
                            ServerMessage::Chat(msg) => {
                                set_messages.update(|msgs| msgs.push(msg));
                            },
                            ServerMessage::ChatHistory(history) => {
                                set_messages.set(history);
                            },
                            ServerMessage::ParticipantJoined(p) => {
                                set_knocking_participants.update(|list| list.retain(|x| x.id != p.id));
                                set_participants.update(|list| {
                                    if !list.iter().any(|x| x.id == p.id) {
                                        list.push(p);
                                    }
                                });
                            },
                            ServerMessage::KnockingParticipantLeft(id) => {
                                set_knocking_participants.update(|list| list.retain(|x| x.id != id));
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
                                set_is_lobby_enabled.set(config.is_lobby_enabled);
                            },
                            ServerMessage::Knocking => {
                                set_current_state.set(RoomConnectionState::Lobby);
                            },
                            ServerMessage::AccessGranted => {
                                set_current_state.set(RoomConnectionState::Joined);
                            },
                            ServerMessage::AccessDenied => {
                                let _ = web_sys::window().unwrap().alert_with_message("Access Denied");
                                set_current_state.set(RoomConnectionState::Prejoin);
                            },
                            ServerMessage::KnockingParticipant(p) => {
                                set_knocking_participants.update(|list| {
                                    if !list.iter().any(|x| x.id == p.id) {
                                        list.push(p);
                                    }
                                });
                            },
                            ServerMessage::ParticipantUpdated(p) => {
                                set_participants.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|x| x.id == p.id) {
                                        *existing = p;
                                    }
                                });
                            },
                            ServerMessage::Reaction { sender_id, emoji } => {
                                set_last_reaction.set(Some((sender_id, emoji, js_sys::Date::now() as u64)));
                            },
                            ServerMessage::PollCreated(poll) => {
                                set_polls.update(|list| list.push(poll));
                            },
                            ServerMessage::PollUpdated(poll) => {
                                set_polls.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|x| x.id == poll.id) {
                                        *existing = poll;
                                    }
                                });
                            },
                            ServerMessage::Draw(action) => {
                                set_last_draw_action.set(Some(action.clone()));
                                set_whiteboard_history.update(|h| h.push(action));
                            },
                            ServerMessage::WhiteboardHistory(history) => {
                                set_whiteboard_history.set(history);
                            },
                            ServerMessage::Error(err) => {
                                // For now, just alert. In a real app, use a toast or modal.
                                let _ = web_sys::window().unwrap().alert_with_message(&err);
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

            // Handle connection close
            let onclose_callback = Closure::<dyn FnMut()>::new(move || {
                set_is_connected.set(false);
            });
            socket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();

            // Handle error
            let onerror_callback = Closure::<dyn FnMut()>::new(move || {
                set_is_connected.set(false);
            });
            socket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            set_ws.set(Some(socket));
        }
    });

    on_cleanup(move || {
        if let Some(socket) = ws.get() {
            let _ = socket.close();
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

    let toggle_lobby = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleLobby;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let grant_access = Callback::new(move |id: String| {
        set_knocking_participants.update(|list| list.retain(|p| p.id != id));
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::GrantAccess(id);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let deny_access = Callback::new(move |id: String| {
        set_knocking_participants.update(|list| list.retain(|p| p.id != id));
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::DenyAccess(id);
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

    let toggle_raise_hand = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleRaiseHand;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let toggle_screen_share = Callback::new(move |_: ()| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::ToggleScreenShare;
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let create_poll = Callback::new(move |poll: Poll| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::CreatePoll(poll);
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let vote_poll = Callback::new(move |(poll_id, option_id): (String, u32)| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Vote { poll_id, option_id };
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = socket.send_with_str(&json);
            }
        }
    });

    let send_draw = Callback::new(move |action: DrawAction| {
        if let Some(socket) = ws.get() {
            let msg = ClientMessage::Draw(action);
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
        }
    });

    RoomState {
        connection_state: current_state,
        messages,
        participants,
        knocking_participants,
        is_connected,
        is_locked,
        is_lobby_enabled,
        is_recording,
        show_settings,
        show_polls,
        polls,
        last_reaction,
        show_whiteboard,
        whiteboard_history,
        my_id,
        set_show_settings,
        set_show_polls,
        set_show_whiteboard,
        send_message,
        toggle_lock,
        toggle_lobby,
        toggle_recording,
        grant_access,
        deny_access,
        join_meeting,
        save_profile,
        send_reaction,
        toggle_raise_hand,
        toggle_screen_share,
        create_poll,
        vote_poll,
        send_draw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_connection_state_equality() {
        assert_eq!(RoomConnectionState::Prejoin, RoomConnectionState::Prejoin);
        assert_ne!(RoomConnectionState::Prejoin, RoomConnectionState::Joined);
    }
}
