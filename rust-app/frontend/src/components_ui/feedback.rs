use leptos::*;
use shared::Feedback;
#[component]
pub fn FeedbackDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    user_id: Option<String>,
) -> impl IntoView {
    let (rating, set_rating) = create_signal(0u8);
    let (comment, set_comment) = create_signal("".to_string());
    let (is_submitting, set_is_submitting) = create_signal(false);
    let (submitted, set_submitted) = create_signal(false);

    let user_id_clone = user_id.clone();
    let submit_action = create_action(move |_: &()| {
        let uid = user_id_clone.clone();
        async move {
            set_is_submitting.set(true);
            let feedback = Feedback {
                rating: rating.get(),
                comment: comment.get(),
                user_id: uid,
            };

        if let Ok(req) = gloo_net::http::Request::post("/api/feedback")
            .json(&feedback) 
        {
             if let Ok(resp) = req.send().await {
                 if resp.ok() {
                     set_submitted.set(true);
                     // Auto close after 2 seconds
                     let _ = gloo_timers::future::TimeoutFuture::new(2000).await;
                     on_close.call(());
                     // Reset state
                     set_submitted.set(false);
                     set_rating.set(0);
                     set_comment.set("".to_string());
                 }
             }
        }
        set_is_submitting.set(false);
        }
    });

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%; text_align: center;">
                    <Show when=move || !submitted.get() fallback=|| view! {
                        <div style="padding: 40px 0;">
                            <h3 style="color: green; margin-bottom: 10px;">"Thank You!"</h3>
                            <p>"Your feedback has been submitted."</p>
                        </div>
                    }>
                        <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                            <h3>"Rate Your Experience"</h3>
                            <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                        </div>

                        <div class="rating-stars" style="font-size: 32px; margin-bottom: 20px; cursor: pointer;">
                            {(1..=5).map(|i| view! {
                                <span
                                    on:click=move |_| set_rating.set(i)
                                    style=move || format!("color: {}; margin: 0 5px;", if i <= rating.get() { "#ffc107" } else { "#e4e5e9" })
                                >
                                    "★"
                                </span>
                            }).collect::<Vec<_>>()}
                        </div>

                        <div class="form-group" style="margin-bottom: 20px;">
                            <textarea
                                prop:value=comment
                                on:input=move |ev| set_comment.set(event_target_value(&ev))
                                placeholder="Tell us more (optional)..."
                                style="width: 100%; min-height: 100px; padding: 10px; border: 1px solid #ccc; border-radius: 4px; resize: vertical;"
                            />
                        </div>

                        <div style="display: flex; justify-content: flex-end; gap: 10px;">
                            <button
                                on:click=move |_| on_close.call(())
                                disabled=is_submitting
                                style="padding: 8px 16px; border: 1px solid #ccc; background: white; border-radius: 4px; cursor: pointer;"
                            >
                                "Cancel"
                            </button>
                            <button
                                on:click=move |_| submit_action.dispatch(())
                                disabled=move || is_submitting.get() || rating.get() == 0
                                style="padding: 8px 16px; background: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer;"
                            >
                                {move || if is_submitting.get() { "Sending..." } else { "Submit" }}
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
