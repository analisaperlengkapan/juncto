with open('rust-app/frontend/src/toolbox.rs', 'r') as f:
    content = f.read()

# Add embed button correctly
button_code = """
            <button
                on:click=move |_| on_embed.call(())
                style="padding: 8px 16px; background-color: #6c757d; color: white; border: none; cursor: pointer; border-radius: 4px;"
            >
                "Embed Meeting"
            </button>
"""

content = content.replace('            <button\n                on:click=move |_| on_feedback.call(())', button_code + '            <button\n                on:click=move |_| on_feedback.call(())')

with open('rust-app/frontend/src/toolbox.rs', 'w') as f:
    f.write(content)
