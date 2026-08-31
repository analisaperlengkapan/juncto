use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Logic for testing key mapping
pub fn get_action_for_key(key: &str) -> Option<&'static str> {
    match key.to_lowercase().as_str() {
        "m" => Some("toggle_mic"),
        "v" => Some("toggle_camera"),
        "h" => Some("raise_hand"),
        "s" => Some("screen_share"),
        "c" => Some("toggle_chat"),
        "p" => Some("toggle_participants"),
        "r" => Some("toggle_local_recording"),
        _ => None,
    }
}

#[component]
pub fn KeyboardShortcuts(
    on_toggle_mic: Callback<()>,
    on_toggle_camera: Callback<()>,
    on_raise_hand: Callback<()>,
    on_screen_share: Callback<()>,
    #[prop(optional)] on_toggle_chat: Option<Callback<()>>,
    #[prop(optional)] on_toggle_participants: Option<Callback<()>>,
    #[prop(optional)] on_toggle_local_recording: Option<Callback<()>>,
) -> impl IntoView {
    create_effect(move |_| {
        let handle_keydown = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
            // Ignore if user is typing in an input, textarea, or select
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    let tag = el.tag_name().to_lowercase();
                    if tag == "input" || tag == "textarea" || tag == "select" {
                        return;
                    }
                    if el.is_content_editable() {
                        return;
                    }
                }
            }
            let key = ev.key();

            match get_action_for_key(&key) {
                Some("toggle_mic") => on_toggle_mic.call(()),
                Some("toggle_camera") => on_toggle_camera.call(()),
                Some("raise_hand") => on_raise_hand.call(()),
                Some("screen_share") => on_screen_share.call(()),
                Some("toggle_chat") => {
                    if let Some(cb) = on_toggle_chat {
                        cb.call(());
                    }
                }
                Some("toggle_participants") => {
                    if let Some(cb) = on_toggle_participants {
                        cb.call(());
                    }
                }
                Some("toggle_local_recording") => {
                    if let Some(cb) = on_toggle_local_recording {
                        cb.call(());
                    }
                }
                _ => {}
            }
        }) as Box<dyn FnMut(_)>);

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document
            .add_event_listener_with_callback("keydown", handle_keydown.as_ref().unchecked_ref())
            .unwrap();

        on_cleanup(move || {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let _ = document.remove_event_listener_with_callback(
                "keydown",
                handle_keydown.as_ref().unchecked_ref(),
            );
            // handle_keydown is moved into this closure, so it lives until cleanup is called.
            // After cleanup, it drops and the Closure is properly freed.
        });

        // Ownership of handle_keydown is moved into the cleanup closure,
        // which keeps it alive until the component is unmounted.
    });

    view! {
        // Invisible component
    }
}

#[component]
pub fn ShortcutsDialog(show: ReadSignal<bool>, on_close: Callback<()>) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay">
                <div class="modal-content">
                    <div class="modal-header">
                        <h3 class="modal-title">"⌨️ Keyboard Shortcuts"</h3>
                        <button id="close-shortcuts-btn" class="modal-close-btn" on:click=move |_| on_close.call(())>"✕"</button>
                    </div>
                    <ul class="modal-body custom-scrollbar" style="list-style: none; padding: 0; display: flex; flex-direction: column; gap: 10px;">
                        <li style="display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: rgba(15, 23, 42, 0.6); border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
                            <kbd style="background: var(--card-bg); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border-strong); font-weight: bold; color: var(--primary-color);">"M"</kbd>
                            <span style="color: var(--text-primary);">"Toggle Microphone"</span>
                        </li>
                        <li style="display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: rgba(15, 23, 42, 0.6); border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
                            <kbd style="background: var(--card-bg); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border-strong); font-weight: bold; color: var(--primary-color);">"V"</kbd>
                            <span style="color: var(--text-primary);">"Toggle Camera"</span>
                        </li>
                        <li style="display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: rgba(15, 23, 42, 0.6); border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
                            <kbd style="background: var(--card-bg); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border-strong); font-weight: bold; color: var(--primary-color);">"H"</kbd>
                            <span style="color: var(--text-primary);">"Raise/Lower Hand"</span>
                        </li>
                        <li style="display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: rgba(15, 23, 42, 0.6); border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
                            <kbd style="background: var(--card-bg); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border-strong); font-weight: bold; color: var(--primary-color);">"S"</kbd>
                            <span style="color: var(--text-primary);">"Share Screen"</span>
                        </li>
                        <li style="display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: rgba(15, 23, 42, 0.6); border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
                            <kbd style="background: var(--card-bg); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border-strong); font-weight: bold; color: var(--primary-color);">"C"</kbd>
                            <span style="color: var(--text-primary);">"Toggle Chat"</span>
                        </li>
                        <li style="display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: rgba(15, 23, 42, 0.6); border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
                            <kbd style="background: var(--card-bg); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border-strong); font-weight: bold; color: var(--primary-color);">"P"</kbd>
                            <span style="color: var(--text-primary);">"Toggle Participants"</span>
                        </li>
                        <li style="display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: rgba(15, 23, 42, 0.6); border-radius: var(--radius-sm); border: 1px solid var(--border-color);">
                            <kbd style="background: var(--card-bg); padding: 4px 8px; border-radius: 4px; border: 1px solid var(--border-strong); font-weight: bold; color: var(--primary-color);">"R"</kbd>
                            <span style="color: var(--text-primary);">"Local Recording"</span>
                        </li>
                    </ul>
                </div>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_mapping() {
        assert_eq!(get_action_for_key("m"), Some("toggle_mic"));
        assert_eq!(get_action_for_key("M"), Some("toggle_mic"));
        assert_eq!(get_action_for_key("v"), Some("toggle_camera"));
        assert_eq!(get_action_for_key("h"), Some("raise_hand"));
        assert_eq!(get_action_for_key("s"), Some("screen_share"));
        assert_eq!(get_action_for_key("c"), Some("toggle_chat"));
        assert_eq!(get_action_for_key("p"), Some("toggle_participants"));
        assert_eq!(get_action_for_key("r"), Some("toggle_local_recording"));
        assert_eq!(get_action_for_key("a"), None);
    }
}
