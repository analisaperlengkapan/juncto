use leptos::*;

#[component]
pub fn Toolbox(
    is_locked: ReadSignal<bool>,
    on_toggle_lock: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="toolbox" style="padding: 10px; border-top: 1px solid #ccc; text-align: center; background: #eee;">
            <button
                on:click=move |_| on_toggle_lock.call(())
                style="padding: 8px 16px; background-color: #f44336; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                {move || if is_locked.get() { "Unlock Room" } else { "Lock Room" }}
            </button>
        </div>
    }
}
