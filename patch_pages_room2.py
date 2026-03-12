with open('rust-app/frontend/src/pages/room.rs', 'r') as f:
    content = f.read()

# Add on_embed to Toolbox component call in room.rs
content = content.replace('                                on_feedback=Callback::new(move |_| set_show_feedback.set(true))\n', '                                on_feedback=Callback::new(move |_| set_show_feedback.set(true))\n                                on_embed=Callback::new(move |_| set_show_embed.set(true))\n')

with open('rust-app/frontend/src/pages/room.rs', 'w') as f:
    f.write(content)
