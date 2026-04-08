use leptos::*;

#[component]
pub fn Toolbox(
    is_locked: ReadSignal<bool>,
    is_host: Signal<bool>,
    #[prop(optional)] _is_lobby_enabled: Option<ReadSignal<bool>>,
    is_recording: ReadSignal<bool>,
    is_subtitles_enabled: ReadSignal<bool>,
    on_toggle_e2ee: Callback<()>,
    is_e2ee_enabled: ReadSignal<bool>,
    on_toggle_etherpad: Callback<()>,
    is_etherpad_active: ReadSignal<bool>,
    current_presence: Signal<shared::PresenceStatus>,
    #[prop(optional)] _on_toggle_lock: Option<Callback<()>>,
    #[prop(optional)] _on_toggle_lobby: Option<Callback<()>>,
    on_toggle_recording: Callback<()>,
    on_toggle_subtitles: Callback<()>,
    on_set_presence: Callback<shared::PresenceStatus>,
    on_invite: Callback<()>,
    on_toggle_chat: Callback<()>,
    on_toggle_participants: Callback<()>,
    on_settings: Callback<()>,
    on_polls: Callback<()>,
    on_shortcuts: Callback<()>,
    on_speaker_stats: Callback<()>,
    on_virtual_background: Callback<()>,
    on_feedback: Callback<()>,    on_embed: Callback<()>,
    on_raise_hand: Callback<()>,
    on_screen_share: Callback<()>,
    on_share_video: Callback<()>,
    on_stop_share_video: Callback<()>,
    is_sharing_video: Signal<bool>,
    on_whiteboard: Callback<()>,
    on_reaction: Callback<String>,
    on_toggle_camera: Callback<()>,
    on_toggle_mic: Callback<()>,
    is_muted: ReadSignal<bool>,
    on_auth_dialog: Callback<()>,
    on_calendar: Callback<()>,
    #[prop(optional)] on_leave: Option<Callback<()>>,
    #[prop(optional)] on_end_meeting: Option<Callback<()>>,
    #[prop(optional)] class: &'static str,
    #[prop(optional)] style: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!("toolbox {}", class) style=format!("padding: 10px; border-top: 1px solid #ccc; text-align: center; background: #eee; display: flex; flex-wrap: wrap; justify-content: center; gap: 10px; {}", style)>
            <button
                on:click=move |_| {
                    if let Some(cb) = on_leave {
                        cb.call(());
                    }
                }
                style="padding: 8px 16px; background-color: #dc3545; color: white; border: none; cursor: pointer; border-radius: 4px; font-weight: bold;"
            >
                "Leave"
            </button>
            <Show when=move || is_host.get() fallback=|| ()>
                <button
                    on:click=move |_| {
                        if let Some(cb) = on_end_meeting {
                            cb.call(());
                        }
                    }
                    style="padding: 8px 16px; background-color: #8b0000; color: white; border: none; cursor: pointer; border-radius: 4px; font-weight: bold;"
                >
                    "End Meeting"
                </button>
                <button
                    on:click=move |_| on_toggle_subtitles.call(())
                    style=move || format!("padding: 8px 16px; background-color: {}; color: white; border: none; cursor: pointer; border-radius: 4px;", if is_subtitles_enabled.get() { "#28a745" } else { "#6c757d" })
                    title="Subtitles"
                >
                    {move || if is_subtitles_enabled.get() { "Hide Subtitles" } else { "Show Subtitles" }}
                </button>
                <button
                    on:click=move |_| on_toggle_recording.call(())
                    style=move || format!("padding: 8px 16px; background-color: {}; color: white; border: none; cursor: pointer; border-radius: 4px;", if is_recording.get() { "#dc3545" } else { "#6c757d" })
                >
                    {move || if is_recording.get() { "Stop Recording" } else { "Start Recording" }}
                </button>
                <button
                    on:click=move |_| on_toggle_e2ee.call(())
                    style=move || format!("padding: 8px 16px; background-color: {}; color: white; border: none; cursor: pointer; border-radius: 4px;", if is_e2ee_enabled.get() { "#28a745" } else { "#6c757d" })
                    title="End-to-End Encryption"
                >
                    {move || if is_e2ee_enabled.get() { "Disable E2EE" } else { "Enable E2EE" }}
                </button>
                <button
                    on:click=move |_| on_toggle_etherpad.call(())
                    style=move || format!("padding: 8px 16px; background-color: {}; color: white; border: none; cursor: pointer; border-radius: 4px;", if is_etherpad_active.get() { "#28a745" } else { "#6c757d" })
                    title="Shared Document (Etherpad)"
                >
                    {move || if is_etherpad_active.get() { "Close Pad" } else { "Open Pad" }}
                </button>
            </Show>
            <button
                on:click=move |_| on_invite.call(())
                style="padding: 8px 16px; background-color: #007bff; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Invite"
            </button>
            <button
                on:click=move |_| on_toggle_camera.call(())
                style="padding: 8px 16px; background-color: #007bff; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Toggle Camera"
            </button>
            <button
                on:click=move |_| on_toggle_mic.call(())
                style=move || format!("padding: 8px 16px; background-color: {}; color: white; border: none; cursor: pointer; border-radius: 4px;", if is_muted.get() { "#dc3545" } else { "#28a745" })
            >
                {move || if is_muted.get() { "Unmute" } else { "Mute" }}
            </button>
            <button
                on:click=move |_| on_screen_share.call(())
                style="padding: 8px 16px; background-color: #6610f2; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Share Screen"
            </button>
            <Show when=move || is_host.get() fallback=|| ()>
                <button
                    on:click=move |_| {
                        if is_sharing_video.get() {
                            on_stop_share_video.call(());
                        } else {
                            on_share_video.call(());
                        }
                    }
                    style=move || format!("padding: 8px 16px; background-color: {}; color: white; border: none; cursor: pointer; border-radius: 4px;", if is_sharing_video.get() { "#dc3545" } else { "#fd7e14" })
                >
                    {move || if is_sharing_video.get() { "Stop Video" } else { "Share Video" }}
                </button>
            </Show>
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
            <Show when=move || !is_host.get()>
                <div style="padding: 8px 16px; background-color: #ccc; color: white; border-radius: 4px;">
                    {move || if is_locked.get() { "Locked" } else { "Unlocked" }}
                </div>
            </Show>
            <button
                on:click=move |_| on_toggle_chat.call(())
                style="padding: 8px 16px; background-color: #6610f2; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Chat"
            </button>
            <button
                on:click=move |_| on_toggle_participants.call(())
                style="padding: 8px 16px; background-color: #6610f2; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Participants"
            </button>
            <button
                on:click=move |_| on_speaker_stats.call(())
                style="padding: 8px 16px; background-color: #6610f2; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Stats"
            </button>
            <button
                on:click=move |_| on_virtual_background.call(())
                style="padding: 8px 16px; background-color: #fd7e14; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Background"
            </button>

            <button
                on:click=move |_| on_embed.call(())
                style="padding: 8px 16px; background-color: #6c757d; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Embed Meeting"
            </button>
            <button
                on:click=move |_| on_feedback.call(())
                style="padding: 8px 16px; background-color: #28a745; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Feedback"
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
            <button
                on:click=move |_| on_auth_dialog.call(())
                style="padding: 8px 16px; background-color: #f8f9fa; color: #333; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Login"
            </button>
            <button
                on:click=move |_| on_calendar.call(())
                style="padding: 8px 16px; background-color: #f8f9fa; color: #333; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Calendar"
            </button>
            <button
                on:click=move |_| on_shortcuts.call(())
                style="padding: 8px 16px; background-color: #666; color: white; border: none; cursor: pointer; border-radius: 4px;"
                title="Keyboard Shortcuts"
            >
                "?"
            </button>
            <div class="reactions" style="display: flex; gap: 5px;">
                <button on:click=move |_| on_reaction.call("👍".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"👍"</button>
                <button on:click=move |_| on_reaction.call("👏".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"👏"</button>
                <button on:click=move |_| on_reaction.call("😂".to_string()) style="cursor: pointer; border: none; background: none; font-size: 20px;">"😂"</button>
            </div>
            <div class="presence-selector" style="display: flex; gap: 5px; align-items: center;">
                <label for="presence-select" style="font-size: 0.9em;">"Presence:"</label>
                <select
                    id="presence-select"
                    prop:value=move || match current_presence.get() {
                        shared::PresenceStatus::Connected => "Connected",
                        shared::PresenceStatus::Busy => "Busy",
                        shared::PresenceStatus::Calling => "Calling",
                        shared::PresenceStatus::Ringing => "Ringing",
                        _ => "Connected",
                    }
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        let status = match value.as_str() {
                            "Connected" => shared::PresenceStatus::Connected,
                            "Busy" => shared::PresenceStatus::Busy,
                            "Calling" => shared::PresenceStatus::Calling,
                            "Ringing" => shared::PresenceStatus::Ringing,
                            _ => shared::PresenceStatus::Connected,
                        };
                        on_set_presence.call(status);
                    }
                    style="padding: 4px; border-radius: 4px; border: 1px solid #ccc;"
                >
                    <option value="Connected">"Connected"</option>
                    <option value="Busy">"Busy"</option>
                    <option value="Calling">"Calling"</option>
                    <option value="Ringing">"Ringing"</option>
                </select>
            </div>
        </div>
    }
}
#[cfg(test)]
mod tests {

    #[test]
    fn test_toolbox_compiles() {
        // dummy test
        assert!(true);
    }
}
