use leptos::*;

#[component]
pub fn Etherpad(#[prop(into)] url: Signal<Option<String>>) -> impl IntoView {
    view! {
        <div class="etherpad-container" style="width: 100%; height: 100%; border: none;">
            <Show when=move || url.get().is_some() fallback=|| view! {
                <div style="display: flex; align-items: center; justify-content: center; height: 100%; color: #888;">
                    "No Etherpad URL set by host."
                </div>
            }>
                <iframe
                    src=move || url.get().unwrap_or_default()
                    sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
                    style="width: 100%; height: 100%; border: none;"
                    title="Etherpad"
                ></iframe>
            </Show>
        </div>
    }
}
