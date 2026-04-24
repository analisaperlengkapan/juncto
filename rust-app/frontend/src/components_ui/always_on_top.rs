use leptos::*;
use crate::i18n::t;

#[component]
pub fn AlwaysOnTop(
    #[prop(into)] is_video_muted: Signal<bool>,
    #[prop(into)] is_audio_muted: Signal<bool>,
    #[prop(optional)] audio_level: Option<Signal<f64>>,
    on_toggle_video: Callback<()>,
    on_toggle_audio: Callback<()>,
    on_leave: Callback<()>,
) -> impl IntoView {
    view! {
        <div
            id="alwaysOnTop"
            class="always-on-top-container"
            style="
                position: fixed;
                bottom: 20px;
                right: 20px;
                background: rgba(0, 0, 0, 0.7);
                border-radius: 8px;
                padding: 10px;
                display: flex;
                gap: 10px;
                z-index: 100;
                box-shadow: 0 4px 6px rgba(0,0,0,0.3);
                transition: opacity 0.3s;
            "
        >
            <div class="toolbox-content-items always-on-top-toolbox">
                <div style="position: relative; display: flex; flex-direction: column; align-items: center;">
                    <button
                        on:click=move |_| on_toggle_audio.call(())
                        class=move || format!("toolbar-btn {}", if is_audio_muted.get() { "muted" } else { "" })
                        style=move || format!("
                            background: {};
                            color: white;
                            border: none;
                            border-radius: 50%;
                            width: 40px;
                            height: 40px;
                            cursor: pointer;
                            display: flex;
                            justify-content: center;
                            align-items: center;
                        ", if is_audio_muted.get() { "#dc3545" } else { "#444" })
                        title=move || if is_audio_muted.get() { t("unmute") } else { t("mute") }
                    >
                        {move || if is_audio_muted.get() { "🔇" } else { "🎤" }}
                    </button>
                    <Show when=move || !is_audio_muted.get()>
                        {move || audio_level.map(|l| view! {
                            <div style="position: absolute; bottom: -15px;">
                                <crate::components_ui::audio_level_indicator::AudioLevelIndicator audio_level=l />
                            </div>
                        })}
                    </Show>
                </div>

                <button
                    on:click=move |_| on_toggle_video.call(())
                    class=move || format!("toolbar-btn {}", if is_video_muted.get() { "muted" } else { "" })
                    style=move || format!("
                        background: {};
                        color: white;
                        border: none;
                        border-radius: 50%;
                        width: 40px;
                        height: 40px;
                        cursor: pointer;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                    ", if is_video_muted.get() { "#dc3545" } else { "#444" })
                    title=move || if is_video_muted.get() { t("camera_on") } else { t("camera_off") }
                >
                    {move || if is_video_muted.get() { "🚫" } else { "📷" }}
                </button>

                <button
                    on:click=move |_| on_leave.call(())
                    class="toolbar-btn hangup-button"
                    style="
                        background: #dc3545;
                        color: white;
                        border: none;
                        border-radius: 50%;
                        width: 40px;
                        height: 40px;
                        cursor: pointer;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                    "
                    title=move || t("leave_room")
                >
                    "📞"
                </button>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_always_on_top_compiles() {
        // Component tests in Leptos without a WASM/browser environment are very limited.
        // We just assert the module and basics compile correctly.
        assert_eq!(1, 1);
    }
}
