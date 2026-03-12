with open('rust-app/frontend/src/pages/room.rs', 'r') as f:
    content = f.read()

# Add the dialog correctly
embed_dialog = """
                                <crate::components_ui::embed_meeting::EmbedMeetingDialog
                                    show=show_embed.read_only()
                                    on_close=Callback::new(move |_| set_show_embed.set(false))
                                />
"""
content = content.replace('                                <InviteDialog', embed_dialog + '                                <InviteDialog')

with open('rust-app/frontend/src/pages/room.rs', 'w') as f:
    f.write(content)
