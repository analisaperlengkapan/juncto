with open('rust-app/frontend/src/components_ui/breakout.rs', 'r') as f:
    content = f.read()
content = content.replace('let _show = create_rw_signal(true);\n', 'let show = create_rw_signal(true);\n')
content = content.replace('let _on_close = Callback::new(|_: ()| {});\n', 'let on_close = Callback::new(|_: ()| {});\n')
content = content.replace('let _view = view! {', 'let is_host = create_rw_signal(true);\n        let _view = view! {')
with open('rust-app/frontend/src/components_ui/breakout.rs', 'w') as f:
    f.write(content)

with open('rust-app/frontend/src/components_ui/feedback.rs', 'r') as f:
    content = f.read()
content = content.replace('let _view = view! {', 'let show = create_rw_signal(true);\n        let _view = view! {')
with open('rust-app/frontend/src/components_ui/feedback.rs', 'w') as f:
    f.write(content)
