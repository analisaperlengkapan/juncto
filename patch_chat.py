import re

with open("rust-app/backend/src/handlers/chat.rs", "r") as f:
    content = f.read()

pattern = r'''    if recipient_id\.is_none\(\) \{ // && room_id\.is_none\(\) -> store history for breakout room chat too so it displays on reconnect
        let mut history = state\.chat_history\.lock\(\)\.unwrap\(\);
        history\.push\(chat_msg\.clone\(\)\);
    \}'''

replacement = r'''    // Only store history for global chat (main room, public messages)
    if recipient_id.is_none() && room_id.is_none() {
        let mut history = state.chat_history.lock().unwrap();
        history.push(chat_msg.clone());
    }'''

content = re.sub(pattern, replacement, content)

with open("rust-app/backend/src/handlers/chat.rs", "w") as f:
    f.write(content)
