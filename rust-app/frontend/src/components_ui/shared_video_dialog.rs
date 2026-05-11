use crate::i18n::t;
use leptos::*;

#[component]
pub fn SharedVideoDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_submit: Callback<String>,
) -> impl IntoView {
    let (url, set_url) = create_signal("".to_string());

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>{move || t("share_video")}</h3>
                        <button id="close-shared-video-btn" on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div class="form-group" style="margin-bottom: 15px;">
                        <label style="display: block; margin-bottom: 5px;">{move || t("youtube_url")}</label>
                        <input
                            type="text"
                            prop:value=url
                            on:input=move |ev| set_url.set(event_target_value(&ev))
                            placeholder="https://www.youtube.com/watch?v=..."
                            style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                        />
                    </div>

                    <div style="display: flex; justify-content: flex-end; gap: 10px;">
                        <button
                            on:click=move |_| on_close.call(())
                            style="padding: 8px 16px; background-color: #6c757d; color: white; border: none; cursor: pointer; border-radius: 4px;"
                        >
                            {move || t("cancel")}
                        </button>
                        <button
                            id="submit-shared-video-btn"
                            on:click=move |_| {
                                on_submit.call(url.get());
                                on_close.call(());
                                set_url.set("".to_string());
                            }
                            style="padding: 8px 16px; background-color: #007bff; color: white; border: none; cursor: pointer; border-radius: 4px;"
                        >
                            {move || t("share")}
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
    fn test_shared_video_dialog_compiles() {
        // Minimal test to ensure the component definition is valid Rust
        let _ = SharedVideoDialog;
    }
}
