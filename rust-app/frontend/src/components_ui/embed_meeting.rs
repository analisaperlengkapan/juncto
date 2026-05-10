use leptos::*;
use wasm_bindgen::JsCast;

fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
}

#[component]
pub fn EmbedMeetingDialog(show: ReadSignal<bool>, on_close: Callback<()>) -> impl IntoView {
    let (copy_success, set_copy_success) = create_signal(false);
    let (iframe_code, set_iframe_code) = create_signal("".to_string());

    // Update the URL on show
    create_effect(move |_| {
        if show.get() {
            if let Some(window) = web_sys::window() {
                if let Ok(loc) = window.location().href() {
                    set_iframe_code.set(format!(
                        "<iframe src=\"{}\" allow=\"camera; microphone; display-capture; fullscreen\" width=\"100%\" height=\"600px\" style=\"border: none;\"></iframe>",
                        escape_html(&loc)
                    ));
                    set_copy_success.set(false);
                }
            }
        }
    });

    let copy_to_clipboard = move |_| {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            let clipboard_prop = js_sys::Reflect::get(&navigator, &"clipboard".into());

            if let Ok(val) = clipboard_prop {
                if !val.is_undefined() && !val.is_null() {
                    if let Ok(clipboard) = val.dyn_into::<web_sys::Clipboard>() {
                        let promise = clipboard.write_text(&iframe_code.get());
                        wasm_bindgen_futures::spawn_local(async move {
                            if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
                                set_copy_success.set(true);
                            }
                        });
                    }
                }
            }
        }
    };

    view! {
        <Show when=move || show.get()>
            <div class="dialog-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000;">
                <div class="dialog-content" style="background: white; padding: 20px; border-radius: 8px; color: black; min-width: 400px; max-width: 90%;">
                    <h3 style="margin-top: 0;">"Embed Meeting"</h3>
                    <p>"Copy the iframe code below to embed this meeting on your website:"</p>
                    <textarea
                        readonly=true
                        style="width: 100%; height: 80px; margin-bottom: 10px; font-family: monospace; resize: none;"
                        prop:value=move || iframe_code.get()
                    ></textarea>

                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <button on:click=copy_to_clipboard style="padding: 8px 16px; background-color: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer;">
                            {move || if copy_success.get() { "Copied!" } else { "Copy Iframe Code" }}
                        </button>
                        <button on:click=move |_| on_close.call(()) style="padding: 8px 16px; background-color: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer;">
                            "Close"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_embed_meeting_compiles() {
        let _ = create_runtime();
        let show = create_rw_signal(true);
        let on_close = Callback::new(|_: ()| {});

        let _view = view! {
            <EmbedMeetingDialog show=show.read_only() on_close=on_close />
        };
        let _ = true;
    }
}
