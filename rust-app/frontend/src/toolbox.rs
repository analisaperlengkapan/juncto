use leptos::*;

#[component]
pub fn Toolbox(
    is_locked: ReadSignal<bool>,
    on_toggle_lock: Callback<()>,
    on_settings: Callback<()>,
    on_reaction: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="toolbox" style="padding: 10px; border-top: 1px solid #ccc; text-align: center; background: #eee; display: flex; justify-content: center; gap: 10px;">
            <button
                on:click=move |_| on_toggle_lock.call(())
                style="padding: 8px 16px; background-color: #f44336; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                {move || if is_locked.get() { "Unlock Room" } else { "Lock Room" }}
            </button>
            <button
                on:click=move |_| on_settings.call(())
                style="padding: 8px 16px; background-color: #007bff; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Settings"
            </button>
            <div class="reactions" style="display: flex; gap: 5px;">
                <button on:click=move |_| on_reaction.call("👍".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"👍"</button>
                <button on:click=move |_| on_reaction.call("👏".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"👏"</button>
                <button on:click=move |_| on_reaction.call("😂".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"😂"</button>
            </div>
        </div>
    }
}
