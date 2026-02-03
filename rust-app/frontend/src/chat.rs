use leptos::*;
use shared::{ChatMessage, Participant, FileAttachment};
use gloo_timers::callback::Timeout;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024; // 2MB

// Helper for testing
fn extract_base64_from_data_url(data_url: &str) -> Option<String> {
    let parts: Vec<&str> = data_url.split(',').collect();
    if parts.len() == 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

fn format_typing_indicator(users: &HashSet<String>, participants: &[Participant], my_id: &Option<String>) -> String {
    let mut users_to_show = users.clone();
    if let Some(uid) = my_id {
        users_to_show.remove(uid);
    }

    if users_to_show.is_empty() {
        "".to_string()
    } else {
        // Lookup names
        let names: Vec<String> = users_to_show.iter().map(|uid| {
            participants.iter().find(|p| &p.id == uid).map(|p| p.name.clone()).unwrap_or(uid.clone())
        }).collect();

        if names.len() == 1 {
            format!("{} is typing...", names[0])
        } else {
            format!("{} users are typing...", names.len())
        }
    }
}

#[component]
pub fn Chat(
    messages: ReadSignal<Vec<ChatMessage>>,
    typing_users: ReadSignal<HashSet<String>>,
    participants: ReadSignal<Vec<Participant>>,
    on_send: Callback<(String, Option<String>, Option<FileAttachment>)>,
    on_typing: Callback<bool>,
    is_connected: ReadSignal<bool>,
    my_id: ReadSignal<Option<String>>,
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal("".to_string());
    let (recipient, set_recipient) = create_signal(None::<String>); // None = Everyone
    let (selected_file, set_selected_file) = create_signal(None::<FileAttachment>);
    let file_input_ref = create_node_ref::<html::Input>();

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
            let on_typing = on_typing;
            Timeout::new(3000, move || {
                on_typing.call(false);
            }).forget();
        }
    };

    let handle_file_change = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let filename = file.name();
                let mime_type = file.type_();
                let size = file.size() as u64;

                if size > MAX_FILE_SIZE {
                    let _ = web_sys::window().unwrap().alert_with_message("File too large. Max size is 2MB.");
                    input.set_value("");
                    return;
                }

                let reader = web_sys::FileReader::new().unwrap();
                let reader_clone = reader.clone();
                // Need to move necessary data into closure
                let on_load = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    if let Ok(res) = reader_clone.result() {
                        if let Some(data_url) = res.as_string() {
                            if let Some(content_base64) = extract_base64_from_data_url(&data_url) {
                                set_selected_file.set(Some(FileAttachment {
                                    filename: filename.clone(),
                                    mime_type: mime_type.clone(),
                                    size,
                                    content_base64,
                                }));
                            }
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
                on_load.forget();
                let _ = reader.read_as_data_url(&file);
            }
        }
    };

    let send = move |_| {
        let content = input_value.get();
        let target = recipient.get();
        let attachment = selected_file.get();

        if !content.is_empty() || attachment.is_some() {
            on_send.call((content, target, attachment));
            on_typing.call(false);
            set_input_value.set("".to_string());
            set_selected_file.set(None);
            if let Some(input) = file_input_ref.get() {
                input.set_value("");
            }
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
                        key=|msg| format!("{}_{}", msg.timestamp, msg.user_id)
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

                            // Format timestamp HH:MM
                            let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(msg.timestamp as f64));
                            let hours = date.get_hours();
                            let minutes = date.get_minutes();
                            let time_str = format!("{:02}:{:02}", hours, minutes);

                            view! {
                                <li style=style>
                                    <small style="color: #999; margin-right: 5px;">"[" {time_str} "] "</small>
                                    <small>{private_indicator}</small>
                                    <strong>{sender_name}": "</strong>
                                    <span>{msg.content.clone()}</span>
                                    {move || {
                                        if let Some(att) = &msg.attachment {
                                            if att.mime_type.starts_with("image/") {
                                                let src = format!("data:{};base64,{}", att.mime_type, att.content_base64);
                                                view! {
                                                    <div>
                                                        <img src=src style="max-width: 200px; max-height: 200px; display: block; margin-top: 5px;" />
                                                    </div>
                                                }.into_view()
                                            } else {
                                                let href = format!("data:{};base64,{}", att.mime_type, att.content_base64);
                                                view! {
                                                    <div>
                                                        <a href=href download=att.filename.clone() style="display: block; margin-top: 5px;">
                                                            "📎 " {att.filename.clone()}
                                                        </a>
                                                    </div>
                                                }.into_view()
                                            }
                                        } else {
                                            view! { <span></span> }.into_view()
                                        }
                                    }}
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
                    let my = my_id.get();
                    format_typing_indicator(&users, &parts, &my)
                }}
            </div>
            <div class="input-area" style="display: flex; flex-direction: column; gap: 5px;">
                <div style="display: flex; gap: 5px;">
                    <input
                        type="text"
                        prop:value=input_value
                        on:input=handle_input
                        placeholder="Type a message..."
                        style="flex: 1;"
                    />
                    <button
                        on:click=send
                        disabled=move || !is_connected.get()
                        style="width: 60px;">
                        {move || if is_connected.get() { "Send" } else { "..." }}
                    </button>
                </div>
                <div>
                     <input
                        type="file"
                        _ref=file_input_ref
                        on:change=handle_file_change
                        style="width: 100%; font-size: 0.8em;"
                     />
                     {move || if let Some(f) = selected_file.get() {
                         view! { <small style="color: green;">" Selected: " {f.filename}</small> }.into_view()
                     } else {
                         view! { <span/> }.into_view()
                     }}
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::Participant;
    use std::collections::HashSet;

    #[test]
    fn test_format_typing_indicator() {
        let participants = vec![
            Participant {
                id: "u1".to_string(),
                name: "Alice".to_string(),
                is_hand_raised: false,
                is_sharing_screen: false,
            },
            Participant {
                id: "u2".to_string(),
                name: "Bob".to_string(),
                is_hand_raised: false,
                is_sharing_screen: false,
            }
        ];

        let my_id = Some("u1".to_string());

        let mut typing = HashSet::new();
        assert_eq!(format_typing_indicator(&typing, &participants, &my_id), "");

        typing.insert("u1".to_string());
        // Should ignore self
        assert_eq!(format_typing_indicator(&typing, &participants, &my_id), "");

        typing.insert("u2".to_string());
        assert_eq!(format_typing_indicator(&typing, &participants, &my_id), "Bob is typing...");

        typing.insert("u3".to_string()); // Unknown user
        let res = format_typing_indicator(&typing, &participants, &my_id);
        assert!(res == "2 users are typing..." || res == "2 users are typing...");
    }

    #[test]
    fn test_extract_base64() {
        let data_url = "data:text/plain;base64,SGVsbG8=";
        assert_eq!(extract_base64_from_data_url(data_url), Some("SGVsbG8=".to_string()));

        let invalid = "invalid_data";
        assert_eq!(extract_base64_from_data_url(invalid), None);
    }
}
