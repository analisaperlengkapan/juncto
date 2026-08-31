use leptos::*;

/// Blocking overlay shown when the WebSocket drops mid-meeting.
/// Calls a rejoin action supplied by the caller (typically a page reload).
#[component]
pub fn RejoinOverlay(
    show: ReadSignal<bool>,
    on_rejoin: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="rejoin-overlay">
                <div class="rejoin-dialog">
                    <h2>"You were disconnected"</h2>
                    <p>"Your session to the server was interrupted. Rejoin to reconnect."</p>
                    <button class="btn btn-primary" on:click=move |_| on_rejoin.call(())>"Rejoin now"</button>
                </div>
            </div>
        </Show>
    }
}
