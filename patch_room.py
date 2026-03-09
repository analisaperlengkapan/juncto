import re

with open("rust-app/frontend/src/pages/room.rs", "r") as f:
    content = f.read()

pattern = r'''                                current_presence=Signal::derive\(move \|\| \{
                                    if let Some\(my_id\) = state\.my_id\.get\(\) \{
                                        if let Some\(me\) = state\.participants\.get\(\)\.get\(&my_id\) \{
                                            return me\.presence\.clone\(\);
                                        \}
                                    \}
                                    shared::PresenceStatus::Connected
                                \}\)'''

replacement = r'''                                current_presence=Signal::derive(move || {
                                    if let Some(my_id) = state.my_id.get() {
                                        if let Some(me) = state.participants.get().iter().find(|p| p.id == my_id) {
                                            return me.presence.clone();
                                        }
                                    }
                                    shared::PresenceStatus::Connected
                                })'''

content = re.sub(pattern, replacement, content)

with open("rust-app/frontend/src/pages/room.rs", "w") as f:
    f.write(content)
