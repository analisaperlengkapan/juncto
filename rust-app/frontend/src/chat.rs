use leptos::*;
use shared::ChatMessage;

#[component]
pub fn Chat(
    messages: ReadSignal<Vec<ChatMessage>>,
    on_send: Callback<String>,
    is_connected: ReadSignal<bool>,
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal("".to_string());

    let send = move |_| {
        let content = input_value.get();
        if !content.is_empty() {
            on_send.call(content);
            set_input_value.set("".to_string());
        }
    };

    view! {
        <div class="chat-container" style="border-left: 1px solid #ccc; width: 300px; padding: 10px; display: flex; flex-direction: column; background: white;">
            <h3>"Chat"</h3>
            <div class="messages" style="flex: 1; overflow-y: auto; height: 300px; border: 1px solid #eee; margin-bottom: 10px; padding: 5px;">
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
                    on:click=send
                    disabled=move || !is_connected.get()
                    style="width: 25%;">
                    {move || if is_connected.get() { "Send" } else { "Connecting..." }}
                </button>
            </div>
        </div>
    }
}
