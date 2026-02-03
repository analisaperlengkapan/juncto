use leptos::*;
use shared::{ChatMessage, Participant};
use gloo_timers::callback::Timeout;
use std::collections::HashSet;

#[component]
pub fn Chat(
    messages: ReadSignal<Vec<ChatMessage>>,
    typing_users: ReadSignal<HashSet<String>>,
    participants: ReadSignal<Vec<Participant>>,
    on_send: Callback<(String, Option<String>)>,
    on_typing: Callback<bool>,
    is_connected: ReadSignal<bool>,
    my_id: ReadSignal<Option<String>>,
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal("".to_string());
    let (recipient, set_recipient) = create_signal(None::<String>); // None = Everyone
    // Store timer handle in a ref to clear it if needed, or just let it fire.
    // In Leptos, we can't easily store non-Clone types in signals.
    // We'll rely on a simple logic: send true on input, set a timeout to send false.
    // If input happens again, we send true again (server handles it idempotently).
    // Ideally we debounce 'true' and debounce 'false'.
    // For simplicity: Send true on every input (throttled?) and false after delay.

    // Using a ref to store the last time we sent "start typing" to avoid spamming
    let last_typing_sent = create_rw_signal(0.0);

    let handle_input = move |ev: web_sys::Event| {
        set_input_value.set(event_target_value(&ev));

        let now = js_sys::Date::now();
        if now - last_typing_sent.get() > 2000.0 {
            on_typing.call(true);
            last_typing_sent.set(now);

            // Schedule stop typing
            // Note: In a real app we would cancel previous timer.
            // Here we just send false after 3s. If user types again, we send true again.
            let on_typing = on_typing;
            Timeout::new(3000, move || {
                on_typing.call(false);
            }).forget();
        }
    };

    let send = move |_| {
        let content = input_value.get();
        let target = recipient.get();
        if !content.is_empty() {
            on_send.call((content, target));
            on_typing.call(false); // Stop typing immediately on send
            set_input_value.set("".to_string());
        }
    };

    view! {
        <div class="chat-container" style="border-left: 1px solid #ccc; width: 300px; padding: 10px; display: flex; flex-direction: column; background: white;">
            <h3>"Chat"</h3>
            <div class="recipient-selector" style="margin-bottom: 10px;">
                <label>"To: "</label>
                <select
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        if val.is_empty() {
                            set_recipient.set(None);
                        } else {
                            set_recipient.set(Some(val));
                        }
                    }
                    style="width: 100%; padding: 5px;"
                >
                    <option value="">"Everyone"</option>
                    <For
                        each=move || participants.get()
                        key=|p| p.id.clone()
                        children=move |p| {
                            let id = p.id.clone();
                            // Don't show myself in recipient list
                            let is_me = my_id.get() == Some(id.clone());
                            let p_name = p.name.clone();
                            view! {
                                <Show when=move || !is_me>
                                    <option value=id.clone()>{p_name.clone()}</option>
                                </Show>
                            }
                        }
                    />
                </select>
            </div>
            <div class="messages" style="flex: 1; overflow-y: auto; height: 300px; border: 1px solid #eee; margin-bottom: 10px; padding: 5px;">
                <ul>
                    <For
                        each=move || messages.get()
                        key=|msg| msg.timestamp
                        children=move |msg| {
                            let parts = participants.get();
                            let my = my_id.get();
                            let sender_name = if Some(msg.user_id.clone()) == my {
                                "Me".to_string()
                            } else {
                                parts.iter().find(|p| p.id == msg.user_id).map(|p| p.name.clone()).unwrap_or(msg.user_id.clone())
                            };

                            let mut style = if Some(msg.user_id.clone()) == my { "color: blue;" } else { "color: black;" };
                            let private_indicator = if msg.recipient_id.is_some() {
                                style = "color: purple;"; // Private msg style
                                "(Private) "
                            } else {
                                ""
                            };

                            view! {
                                <li style=style>
                                    <small>{private_indicator}</small>
                                    <strong>{sender_name}": "</strong>
                                    <span>{msg.content}</span>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>
            <div class="typing-indicator" style="height: 20px; font-style: italic; color: #666; font-size: 0.8em;">
                {move || {
                    let users = typing_users.get();
                    let parts = participants.get();
                    if users.is_empty() {
                        "".to_string()
                    } else {
                        // Lookup names
                        let names: Vec<String> = users.iter().map(|uid| {
                            parts.iter().find(|p| &p.id == uid).map(|p| p.name.clone()).unwrap_or(uid.clone())
                        }).collect();

                        if names.len() == 1 {
                            format!("{} is typing...", names[0])
                        } else {
                            format!("{} users are typing...", names.len())
                        }
                    }
                }}
            </div>
            <div class="input-area">
                <input
                    type="text"
                    prop:value=input_value
                    on:input=handle_input
                    placeholder="Type a message..."
                    style="width: 70%;"
                />
                <button
                    on:click=send
                    disabled=move || !is_connected.get()
                    style="width: 25%;">
                    {move || if is_connected.get() { "Send" } else { "Connecting..." }}
                </button>
            </div>
        </div>
    }
}
