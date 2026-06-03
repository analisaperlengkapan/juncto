use leptos::*;
use shared::ChatMessage;
use wasm_bindgen::JsCast;

#[component]
pub fn FileSharing(messages: ReadSignal<Vec<ChatMessage>>) -> impl IntoView {
    let files = Signal::derive(move || {
        messages
            .get()
            .into_iter()
            .filter_map(|m| m.attachment.map(|a| (m.user_id.clone(), m.timestamp, a)))
            .collect::<Vec<_>>()
    });

    view! {
        <div class="file-sharing" style="padding: 10px; width: 100%;">
            <h3 style="margin-top: 0;">"Shared Files"</h3>
            <Show when=move || files.get().is_empty() fallback=move || view! {
                <ul style="list-style: none; padding: 0;">
                    <For
                        each=move || files.get()
                        key=|(_, ts, a)| format!("{}-{}", a.filename, ts)
                        children=move |(user_id, ts, a)| {
                            let filename = a.filename.clone();
                            let content = a.content_base64.clone();
                            let mime = a.mime_type.clone();

                            let download = move |_| {
                                if let Some(window) = web_sys::window() {
                                    let _ = window.alert_with_message(&format!("Downloading {}...", filename));
                                    // In a real app, we'd use a blob URL here
                                    let document = window.document().unwrap();
                                    let link = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                                    let _ = link.set_attribute("href", &format!("data:{};base64,{}", mime, content));
                                    link.set_download(&filename);
                                    link.click();
                                }
                            };

                            let filename_display = a.filename.clone();
                            let filename_for_dropbox = a.filename.clone();
                            view! {
                                <li style="padding: 10px; border: 1px solid #eee; border-radius: 4px; margin-bottom: 10px; display: flex; flex-direction: column; gap: 5px;">
                                    <div style="font-weight: bold; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{filename_display}</div>
                                    <div style="font-size: 0.8em; color: #666;">
                                        "Shared by " {user_id} " at " {ts}
                                    </div>
                                    <div style="font-size: 0.8em; color: #666;">
                                        {(a.size as f64 / 1024.0).round()} " KB"
                                    </div>
                                    <div style="display: flex; gap: 5px;">
                                        <button
                                            on:click=download
                                            style="padding: 4px 8px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; align-self: flex-start;"
                                        >
                                            "Download"
                                        </button>
                                        <button
                                            on:click={
                                                let dropbox_svc = crate::dropbox::use_dropbox();
                                                move |_| {
                                                    dropbox_svc.save_file(filename_for_dropbox.clone());
                                                }
                                            }
                                            class="save-dropbox-btn"
                                            style="padding: 4px 8px; background: #0061ff; color: white; border: none; border-radius: 4px; cursor: pointer; align-self: flex-start;"
                                        >
                                            "Save to Dropbox"
                                        </button>
                                    </div>
                                </li>
                            }
                        }
                    />
                </ul>
            }>
                <p style="color: #666; font-style: italic;">"No files shared yet."</p>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{ChatMessage, FileAttachment};

    #[test]
    fn test_file_filtering_logic() {
        let _runtime = create_runtime();
        let m1 = ChatMessage {
            user_id: "u1".to_string(),
            content: "hello".to_string(),
            recipient_id: None,
            timestamp: 100,
            attachment: None,
            room_id: None,
        };
        let m2 = ChatMessage {
            user_id: "u2".to_string(),
            content: "here is a file".to_string(),
            recipient_id: None,
            timestamp: 200,
            attachment: Some(FileAttachment {
                filename: "test.txt".to_string(),
                mime_type: "text/plain".to_string(),
                size: 1024,
                content_base64: "YWJj".to_string(),
            }),
            room_id: None,
        };

        let messages = create_rw_signal(vec![m1, m2]);

        // This is a simple logic test that mirrors the derive in component
        let files: Vec<_> = messages
            .get()
            .into_iter()
            .filter_map(|m| m.attachment.map(|a| (m.user_id.clone(), m.timestamp, a)))
            .collect();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].2.filename, "test.txt");
    }
}
