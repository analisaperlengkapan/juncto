use crate::components_ui::toast::{use_toast, ToastType};
use crate::i18n::t;
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn InviteDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    room_url: Signal<String>,
) -> impl IntoView {
    let toast = use_toast();

    let copy_link = move |_| {
        let url = room_url.get();
        // Capture translated strings here to ensure correct locale context
        // This fixes Bug 2: Locale defaults to English inside spawn_local async block
        let msg_success = t("link_copied");
        let msg_error = t("failed_to_copy");

        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();

            // Defensively check for clipboard API (it might be undefined in insecure contexts)
            // This fixes Bug 1: Navigator.clipboard() will panic in insecure (HTTP) contexts
            let clipboard_prop = js_sys::Reflect::get(&navigator, &"clipboard".into());

            if let Ok(val) = clipboard_prop {
                if !val.is_undefined() && !val.is_null() {
                    if let Ok(clipboard) = val.dyn_into::<web_sys::Clipboard>() {
                        let promise = clipboard.write_text(&url);
                        wasm_bindgen_futures::spawn_local(async move {
                            match wasm_bindgen_futures::JsFuture::from(promise).await {
                                Ok(_) => toast.add(msg_success, ToastType::Success),
                                Err(_) => toast.add(msg_error, ToastType::Error),
                            }
                        });
                        return;
                    }
                }
            }

            // Fallback if clipboard API is not available
            toast.add(
                "Clipboard API not available (Secure context required)".to_string(),
                ToastType::Error,
            );
        }
    };

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 2000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>{move || t("invite_people")}</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <p style="margin-bottom: 10px; color: #666;">
                        {move || t("share_link_hint")}
                    </p>

                    <div style="display: flex; gap: 10px; margin-bottom: 20px;">
                        <input
                            type="text"
                            readonly
                            prop:value=room_url
                            style="flex: 1; padding: 8px; border: 1px solid #ccc; border-radius: 4px; background: #f9f9f9;"
                        />
                        <button
                            on:click=copy_link
                            style="padding: 8px 16px; background-color: #007bff; color: white; border: none; cursor: pointer; border-radius: 4px;"
                        >
                            {move || t("copy_link")}
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
    fn test_invite_dialog_render() {
        // Basic syntax check for component macro usage
        let (_show, _set_show) = create_signal(true);
        let (_url, _set_url) = create_signal("http://test.com".to_string());

        // In a unit test environment without DOM, we can mainly check if logic compiles.
        // Leptos components are functions, so we can technically call it, but without a reactive root it panics.
        // We will rely on E2E for render verification.
    }
}
