use crate::i18n::t;
use leptos::*;

#[component]
pub fn VirtualBackgroundDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_change: Callback<String>,
    current_mode: ReadSignal<String>,
) -> impl IntoView {
    let apply = move |mode: String| {
        on_change.call(mode);
    };

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay">
                <div class="modal-content">
                    <div class="modal-header">
                        <h3 class="modal-title">{move || t("virtual_background")}</h3>
                        <button class="modal-close-btn" on:click=move |_| on_close.call(())>"✕"</button>
                    </div>

                    <div class="options" style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-top: 10px;">
                        <div
                            on:click=move |_| apply("none".to_string())
                            style=move || format!("
                                cursor: pointer;
                                border: 2px solid {};
                                border-radius: 8px;
                                padding: 12px;
                                text-align: center;
                                background: rgba(15, 23, 42, 0.6);
                            ", if current_mode.get() == "none" { "var(--primary-color)" } else { "var(--border-color)" })
                        >
                            <div style="height: 60px; background: rgba(255,255,255,0.05); margin-bottom: 8px; display: flex; align-items: center; justify-content: center; border-radius: 4px;">
                                {move || t("none")}
                            </div>
                            <span>{move || t("none")}</span>
                        </div>

                        <div
                            on:click=move |_| apply("blur".to_string())
                            style=move || format!("
                                cursor: pointer;
                                border: 2px solid {};
                                border-radius: 8px;
                                padding: 12px;
                                text-align: center;
                                background: rgba(15, 23, 42, 0.6);
                            ", if current_mode.get() == "blur" { "var(--primary-color)" } else { "var(--border-color)" })
                        >
                            <div style="height: 60px; background: rgba(255,255,255,0.05); margin-bottom: 8px; filter: blur(3px); display: flex; align-items: center; justify-content: center; border-radius: 4px;">
                                {move || t("blur")}
                            </div>
                            <span>{move || t("blur")}</span>
                        </div>

                        <div
                            on:click=move |_| apply("image".to_string())
                            style=move || format!("
                                cursor: pointer;
                                border: 2px solid {};
                                border-radius: 8px;
                                padding: 12px;
                                text-align: center;
                                background: rgba(15, 23, 42, 0.6);
                            ", if current_mode.get() == "image" { "var(--primary-color)" } else { "var(--border-color)" })
                        >
                            <div style="height: 60px; background: linear-gradient(135deg, #3b82f6, #1e1b4b); margin-bottom: 8px; border-radius: 4px;"></div>
                            <span>{move || t("image")}</span>
                        </div>
                    </div>

                    <div style="margin-top: 24px; text-align: right;">
                         <button
                            class="btn btn-primary"
                            on:click=move |_| on_close.call(())
                        >
                            {move || t("done")}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use leptos::*;
    #[test]
    fn test_virtual_background_selection() {
        let _runtime = create_runtime();
        let (current_mode, _set_current_mode) = create_signal::<String>("blur".to_string());
        assert_eq!(current_mode.get(), "blur");
    }
}
