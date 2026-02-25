use crate::components_ui::toast::{use_toast, ToastType};
use crate::i18n::t;
use leptos::*;
use shared::Feedback;

#[component]
pub fn FeedbackDialog(show: ReadSignal<bool>, on_close: Callback<()>) -> impl IntoView {
    let (stars, set_stars) = create_signal(0u8);
    let (comment, set_comment) = create_signal("".to_string());
    let toast = use_toast();

    let submit = move |_| {
        let s = stars.get();
        if s == 0 {
            toast.add(t("please_select_rating"), ToastType::Error);
            return;
        }
        let c = comment.get();

        spawn_local(async move {
            let feedback = Feedback {
                stars: s,
                comment: c,
                user_id: None,
            };

            let client = gloo_net::http::Request::post("/api/feedback").json(&feedback);

            match client {
                Ok(req) => match req.send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            toast.add(t("feedback_submitted"), ToastType::Success);
                            on_close.call(());
                            set_stars.set(0);
                            set_comment.set("".to_string());
                        } else {
                            toast.add(t("feedback_error"), ToastType::Error);
                        }
                    }
                    Err(_) => toast.add(t("feedback_error"), ToastType::Error),
                },
                Err(_) => toast.add(t("feedback_error"), ToastType::Error),
            }
        });
    };

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 2000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>{move || t("feedback")}</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div style="margin-bottom: 20px; display: flex; justify-content: center; gap: 10px;">
                        <For
                            each=move || 1..=5
                            key=|i| *i
                            children=move |i| {
                                view! {
                                    <span
                                        on:click=move |_| set_stars.set(i)
                                        style=move || format!("cursor: pointer; font-size: 30px; color: {};", if stars.get() >= i { "#ffc107" } else { "#ccc" })
                                    >
                                        "★"
                                    </span>
                                }
                            }
                        />
                    </div>

                    <textarea
                        prop:value=comment
                        on:input=move |ev| set_comment.set(event_target_value(&ev))
                        placeholder=move || t("feedback_placeholder")
                        style="width: 100%; height: 100px; padding: 8px; border: 1px solid #ccc; border-radius: 4px; margin-bottom: 20px; box-sizing: border-box;"
                    />

                    <div style="text-align: right;">
                        <button
                            class="submit-feedback-btn" // Added class for E2E testing
                            on:click=submit
                            style="padding: 10px 20px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        >
                            {move || t("submit")}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
