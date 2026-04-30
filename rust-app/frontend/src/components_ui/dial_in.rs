use leptos::*;

// NOTE: The phone number and meeting ID rendered below are placeholder values
// for the UI scaffold. Real dial-in details are not yet provisioned by the
// backend — when a telephony provider is integrated, these should be sourced
// from the room configuration (e.g. `RoomConfig`) rather than hardcoded here.
#[component]
pub fn DialInDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 25px; border-radius: 8px; width: 400px; max-width: 90%; text-align: center;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                        <h3 style="margin: 0;">"Dial-in Information"</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; border: 1px solid #dee2e6; margin-bottom: 20px;">
                        <p style="margin: 0 0 10px 0; font-size: 0.9em; color: #666;">"To join by phone, dial one of these numbers:"</p>
                        <div style="font-size: 1.2em; font-weight: bold; margin-bottom: 15px; color: #007bff;">
                            "+1 555 012 3456"
                        </div>
                        <p style="margin: 0 0 5px 0; font-size: 0.9em; color: #666;">"Meeting ID:"</p>
                        <div style="font-size: 1.1em; font-weight: bold; letter-spacing: 2px;">
                            "123 456 789"
                        </div>
                    </div>

                    <p style="font-size: 0.85em; color: #666; line-height: 1.4;">
                        "Standard call rates apply. International numbers are available in the meeting invitation."
                    </p>

                    <button
                        on:click=move |_| on_close.call(())
                        style="margin-top: 20px; padding: 10px 20px; background: #6c757d; color: white; border: none; border-radius: 4px; cursor: pointer; width: 100%;"
                    >
                        "Close"
                    </button>
                </div>
            </div>
        </Show>
    }
}
