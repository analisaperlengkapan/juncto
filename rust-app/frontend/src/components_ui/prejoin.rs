use leptos::*;

#[component]
pub fn PrejoinScreen(
    on_join: Callback<String>,
) -> impl IntoView {
    let (display_name, set_display_name) = create_signal("Guest".to_string());

    view! {
        <div class="prejoin-container" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background: #f0f0f0;">
            <div class="card" style="background: white; padding: 40px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); text-align: center;">
                <h2>"Ready to join?"</h2>
                <div style="margin: 20px 0;">
                    <label style="display: block; margin-bottom: 8px;">"Display Name"</label>
                    <input
                        type="text"
                        prop:value=display_name
                        on:input=move |ev| set_display_name.set(event_target_value(&ev))
                        style="padding: 10px; width: 200px; border: 1px solid #ccc; border-radius: 4px;"
                    />
                </div>
                <button
                    on:click=move |_| on_join.call(display_name.get())
                    class="join-btn"
                    style="padding: 10px 20px; background-color: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px;"
                >
                    "Join Meeting"
                </button>
            </div>
        </div>
    }
}
