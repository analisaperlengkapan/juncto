with open('rust-app/frontend/src/chat.rs', 'r') as f:
    content = f.read()

if 'pub type ChatSendCallback' not in content:
    content = 'pub type ChatSendCallback = Callback<(String, Option<String>, Option<shared::FileAttachment>, Option<String>)>;\n' + content

content = content.replace('use web_sys::{Event, FileList, HtmlInputElement};\npub type ChatSendCallback = Callback<(String, Option<String>, Option<FileAttachment>, Option<String>)>;\n\n', 'use web_sys::{Event, FileList, HtmlInputElement};\n\n')

with open('rust-app/frontend/src/chat.rs', 'w') as f:
    f.write(content)
