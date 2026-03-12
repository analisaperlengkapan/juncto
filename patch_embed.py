with open('rust-app/frontend/src/components_ui/embed_meeting.rs', 'r') as f:
    content = f.read()

content = content.replace('if let Some(clipboard) = window.navigator().clipboard() {', 'let clipboard = window.navigator().clipboard();\n            if true {')

with open('rust-app/frontend/src/components_ui/embed_meeting.rs', 'w') as f:
    f.write(content)
