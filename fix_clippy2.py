with open('rust-app/backend/src/handlers/ws.rs', 'r') as f:
    content = f.read()

# Fix clippy warning for identical if-else blocks
content = content.replace('                                                            } else if message.user_id == my_id_clone {\n                                                                true // Echo to self\n                                                            } else {\n                                                                true\n                                                            }', '                                                            } else {\n                                                                true\n                                                            }')
content = content.replace('                                                    } else if message.user_id == my_id_clone {\n                                                        true // Echo to self\n                                                    } else {\n                                                        true\n                                                    }', '                                                    } else {\n                                                        true\n                                                    }')


with open('rust-app/backend/src/handlers/ws.rs', 'w') as f:
    f.write(content)

# Fix complex type in chat
with open('rust-app/frontend/src/chat.rs', 'r') as f:
    content = f.read()

type_def = "pub type ChatSendCallback = Callback<(String, Option<String>, Option<FileAttachment>, Option<String>)>;\n"
content = content.replace('use web_sys::{Event, FileList, HtmlInputElement};\n\n', f'use web_sys::{{Event, FileList, HtmlInputElement}};\n{type_def}\n')
content = content.replace('on_send: Callback<(String, Option<String>, Option<FileAttachment>, Option<String>)>,', 'on_send: ChatSendCallback,')

with open('rust-app/frontend/src/chat.rs', 'w') as f:
    f.write(content)

# Fix complex type in state
with open('rust-app/frontend/src/state.rs', 'r') as f:
    content = f.read()

content = content.replace('pub send_message: Callback<(String, Option<String>, Option<FileAttachment>, Option<String>)>,', 'pub send_message: crate::chat::ChatSendCallback,')

with open('rust-app/frontend/src/state.rs', 'w') as f:
    f.write(content)
