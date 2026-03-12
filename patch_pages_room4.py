with open('rust-app/frontend/src/pages/room.rs', 'r') as f:
    content = f.read()

# Make sure we only add it once
if 'let (show_embed, set_show_embed)' not in content:
    content = content.replace('    let (show_invite, set_show_invite) = create_signal(false);', '    let (show_invite, set_show_invite) = create_signal(false);\n    let (show_embed, set_show_embed) = create_signal(false);')

with open('rust-app/frontend/src/pages/room.rs', 'w') as f:
    f.write(content)
