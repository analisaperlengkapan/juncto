use leptos::*;
use shared::ChatMessage;
use web_sys::{MessageEvent, WebSocket};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Chat() -> impl IntoView {
    let (messages, set_messages) = create_signal(Vec::<ChatMessage>::new());
    let (input_value, set_input_value) = create_signal("".to_string());
    let (ws, set_ws) = create_signal(None::<WebSocket>);
    let (is_connected, set_is_connected) = create_signal(false);

    // Initialize WebSocket on mount
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
                    if let Ok(msg) = serde_json::from_str::<ChatMessage>(&txt) {
                        set_messages.update(|msgs| msgs.push(msg));
                    }
                }
            });
            socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget(); // Keep callback alive

            // Handle connection open
            let onopen_callback = Closure::<dyn FnMut()>::new(move || {
                set_is_connected.set(true);
            });
            socket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();

            set_ws.set(Some(socket));
        }
    });

    let send_message = move |_| {
        if let Some(socket) = ws.get() {
            let content = input_value.get();
            if !content.is_empty() {
                let msg = ChatMessage {
                    user_id: "Me".to_string(), // Placeholder user ID
                    content,
                    timestamp: js_sys::Date::now() as u64,
                };

                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = socket.send_with_str(&json);
                    set_input_value.set("".to_string());
                }
            }
        }
    };

    view! {
        <div class="chat-container" style="border-left: 1px solid #ccc; width: 300px; padding: 10px; display: flex; flex-direction: column;">
            <h3>"Chat"</h3>
            <div class="messages" style="flex: 1; overflow-y: auto; height: 300px; border: 1px solid #eee; margin-bottom: 10px;">
                <ul>
                    <For
                        each=move || messages.get()
                        key=|msg| msg.timestamp
                        children=move |msg| {
                            view! {
                                <li>
                                    <strong>{msg.user_id}": "</strong>
                                    <span>{msg.content}</span>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>
            <div class="input-area">
                <input
                    type="text"
                    prop:value=input_value
                    on:input=move |ev| set_input_value.set(event_target_value(&ev))
                    placeholder="Type a message..."
                    style="width: 70%;"
                />
                <button
                    on:click=send_message
                    disabled=move || !is_connected.get()
                    style="width: 25%;">
                    {move || if is_connected.get() { "Send" } else { "Connecting..." }}
                </button>
            </div>
        </div>
    }
}
