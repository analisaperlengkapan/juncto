import re

with open("rust-app/frontend/src/toolbox.rs", "r") as f:
    content = f.read()

pattern1 = r'''pub fn Toolbox\(
    is_locked: ReadSignal<bool>,
    is_host: Signal<bool>,
    is_lobby_enabled: ReadSignal<bool>,
    is_recording: ReadSignal<bool>,
    is_subtitles_enabled: ReadSignal<bool>,
    on_toggle_lock: Callback<\(\)>,
    on_toggle_lobby: Callback<\(\)>,
    on_toggle_recording: Callback<\(\)>,
    on_toggle_subtitles: Callback<\(\)>,
    on_set_presence: Callback<shared::PresenceStatus>,'''

replacement1 = r'''pub fn Toolbox(
    is_locked: ReadSignal<bool>,
    is_host: Signal<bool>,
    is_lobby_enabled: ReadSignal<bool>,
    is_recording: ReadSignal<bool>,
    is_subtitles_enabled: ReadSignal<bool>,
    current_presence: Signal<shared::PresenceStatus>,
    on_toggle_lock: Callback<()>,
    on_toggle_lobby: Callback<()>,
    on_toggle_recording: Callback<()>,
    on_toggle_subtitles: Callback<()>,
    on_set_presence: Callback<shared::PresenceStatus>,'''

content = re.sub(pattern1, replacement1, content)

pattern2 = r'''                <select
                    id="presence-select"
                    on:change=move \|ev\| \{'''

replacement2 = r'''                <select
                    id="presence-select"
                    prop:value=move || match current_presence.get() {
                        shared::PresenceStatus::Connected => "Connected",
                        shared::PresenceStatus::Busy => "Busy",
                        shared::PresenceStatus::Calling => "Calling",
                        shared::PresenceStatus::Ringing => "Ringing",
                        _ => "Connected",
                    }
                    on:change=move |ev| {'''

content = re.sub(pattern2, replacement2, content)

with open("rust-app/frontend/src/toolbox.rs", "w") as f:
    f.write(content)
