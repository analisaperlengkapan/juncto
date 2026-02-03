use leptos::*;

#[component]
pub fn Toolbox(
    is_locked: ReadSignal<bool>,
    is_lobby_enabled: ReadSignal<bool>,
    is_recording: ReadSignal<bool>,
    on_toggle_lock: Callback<()>,
    on_toggle_lobby: Callback<()>,
    on_toggle_recording: Callback<()>,
    on_settings: Callback<()>,
    on_polls: Callback<()>,
    on_raise_hand: Callback<()>,
    on_screen_share: Callback<()>,
    on_whiteboard: Callback<()>,
    on_reaction: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="toolbox" style="padding: 10px; border-top: 1px solid #ccc; text-align: center; background: #eee; display: flex; justify-content: center; gap: 10px;">
            <button
                on:click=move |_| on_screen_share.call(())
                style="padding: 8px 16px; background-color: #6610f2; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Share Screen"
            </button>
            <button
                on:click=move |_| on_whiteboard.call(())
                style="padding: 8px 16px; background-color: #fd7e14; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Whiteboard"
            </button>
            <button
                on:click=move |_| on_raise_hand.call(())
                style="padding: 8px 16px; background-color: #ffc107; color: black; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Raise Hand"
            </button>
            <button
                on:click=move |_| on_toggle_lock.call(())
                style="padding: 8px 16px; background-color: #f44336; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                {move || if is_locked.get() { "Unlock Room" } else { "Lock Room" }}
            </button>
            <button
                on:click=move |_| on_toggle_lobby.call(())
                style="padding: 8px 16px; background-color: #20c997; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                {move || if is_lobby_enabled.get() { "Disable Lobby" } else { "Enable Lobby" }}
            </button>
            <button
                on:click=move |_| on_toggle_recording.call(())
                style=move || format!("padding: 8px 16px; background-color: {}; color: white; border: none; cursor: pointer; border-radius: 4px;", if is_recording.get() { "#dc3545" } else { "#6c757d" })
            >
                {move || if is_recording.get() { "Stop Recording" } else { "Start Recording" }}
            </button>
            <button
                on:click=move |_| on_settings.call(())
                style="padding: 8px 16px; background-color: #007bff; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Settings"
            </button>
            <button
                on:click=move |_| on_polls.call(())
                style="padding: 8px 16px; background-color: #17a2b8; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Polls"
            </button>
            <div class="reactions" style="display: flex; gap: 5px;">
                <button on:click=move |_| on_reaction.call("👍".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"👍"</button>
                <button on:click=move |_| on_reaction.call("👏".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"👏"</button>
                <button on:click=move |_| on_reaction.call("😂".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"😂"</button>
            </div>
        </div>
    }
}
