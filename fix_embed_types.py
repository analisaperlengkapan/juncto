with open('rust-app/frontend/src/components_ui/embed_meeting.rs', 'r') as f:
    content = f.read()

content = content.replace('use leptos::*;', 'use leptos::*;\nuse wasm_bindgen::JsCast;')

with open('rust-app/frontend/src/components_ui/embed_meeting.rs', 'w') as f:
    f.write(content)
