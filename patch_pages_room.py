with open('rust-app/frontend/src/pages/room.rs', 'r') as f:
    content = f.read()

# Add signal for embed
content = content.replace('let (show_feedback, set_show_feedback) = create_signal(false);', 'let (show_feedback, set_show_feedback) = create_signal(false);\n    let (show_embed, set_show_embed) = create_signal(false);')

# Add embed dialog
embed_dialog = """
                                <crate::components_ui::embed_meeting::EmbedMeetingDialog
                                    show=show_embed.read_only()
                                    on_close=Callback::new(move |_| set_show_embed.set(false))
                                />
"""
content = content.replace('                                <FeedbackDialog', embed_dialog + '                                <FeedbackDialog')

# Add on_embed to toolbox
content = content.replace('                                on_feedback=Callback::new(move |_| set_show_feedback.set(true))', '                                on_feedback=Callback::new(move |_| set_show_feedback.set(true))\n                                on_embed=Callback::new(move |_| set_show_embed.set(true))')

with open('rust-app/frontend/src/pages/room.rs', 'w') as f:
    f.write(content)
