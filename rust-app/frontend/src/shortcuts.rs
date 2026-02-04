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
        _ => None,
    }
}

#[component]
pub fn KeyboardShortcuts(
    on_toggle_mic: Callback<()>,
    on_toggle_camera: Callback<()>,
    on_raise_hand: Callback<()>,
    on_screen_share: Callback<()>,
) -> impl IntoView {
    create_effect(move |_| {
        let handle_keydown = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
            // Ignore if user is typing in an input or textarea
            if let Some(target) = ev.target() {
                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    let tag = el.tag_name().to_lowercase();
                    if tag == "input" || tag == "textarea" {
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
            let _ = document.remove_event_listener_with_callback("keydown", handle_keydown.as_ref().unchecked_ref());
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
pub fn ShortcutsDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 2000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>"Keyboard Shortcuts"</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>
                    <ul style="list-style: none; padding: 0;">
                        <li style="margin-bottom: 10px; display: flex; justify-content: space-between;">
                            <strong>"M"</strong> <span>"Toggle Microphone"</span>
                        </li>
                        <li style="margin-bottom: 10px; display: flex; justify-content: space-between;">
                            <strong>"V"</strong> <span>"Toggle Camera"</span>
                        </li>
                        <li style="margin-bottom: 10px; display: flex; justify-content: space-between;">
                            <strong>"H"</strong> <span>"Raise/Lower Hand"</span>
                        </li>
                        <li style="margin-bottom: 10px; display: flex; justify-content: space-between;">
                            <strong>"S"</strong> <span>"Share Screen"</span>
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
        assert_eq!(get_action_for_key("a"), None);
    }
}
