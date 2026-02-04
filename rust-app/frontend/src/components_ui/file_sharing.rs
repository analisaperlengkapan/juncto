use leptos::*;
use web_sys::FormData;

#[component]
pub fn FileSharingDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_upload: Callback<String>, // Returns the URL of the uploaded file
) -> impl IntoView {
    let (is_uploading, set_is_uploading) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal(None::<String>);
    let file_input_ref = create_node_ref::<html::Input>();

    let upload_action = create_action(move |_: &()| async move {
        set_is_uploading.set(true);
        set_error_msg.set(None);

        if let Some(input) = file_input_ref.get() {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let form_data = FormData::new().unwrap();
                    // append_with_blob_and_filename is the method for 3 args
                    let _ = form_data.append_with_blob_and_filename("file", &file, &file.name());

                    // Perform Fetch request
                    let request = gloo_net::http::Request::post("/api/upload")
                        .body(form_data);
                    
                    if let Ok(req) = request {
                         if let Ok(resp) = req.send().await {
                            if resp.ok() {
                                if let Ok(json) = resp.json::<serde_json::Value>().await {
                                    if let Some(url) = json.get("url").and_then(|v| v.as_str()) {
                                        on_upload.call(url.to_string());
                                        on_close.call(());
                                    } else {
                                        set_error_msg.set(Some("Invalid response from server".to_string()));
                                    }
                                } else {
                                     set_error_msg.set(Some("Failed to parse JSON".to_string()));
                                }
                            } else {
                                set_error_msg.set(Some(format!("Upload failed: {}", resp.status())));
                            }
                         } else {
                            set_error_msg.set(Some("Network error".to_string()));
                         }
                    } else {
                        set_error_msg.set(Some("Request prep failed".to_string()));
                    }
                } else {
                     set_error_msg.set(Some("No file selected".to_string()));
                }
            }
        }
        set_is_uploading.set(false);
    });

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>"Share File"</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div class="form-group" style="margin-bottom: 20px;">
                        <input type="file" _ref=file_input_ref disabled=is_uploading />
                    </div>

                    <Show when=move || error_msg.get().is_some()>
                        <p style="color: red; margin-bottom: 10px;">{move || error_msg.get().unwrap()}</p>
                    </Show>

                    <Show when=move || is_uploading.get()>
                         <p style="color: blue;">"Uploading..."</p>
                    </Show>

                    <div style="display: flex; justify-content: flex-end; gap: 10px;">
                        <button
                            on:click=move |_| on_close.call(())
                            disabled=is_uploading
                            style="padding: 8px 16px; border: 1px solid #ccc; background: white; border-radius: 4px; cursor: pointer;"
                        >
                            "Cancel"
                        </button>
                        <button
                            on:click=move |_| upload_action.dispatch(())
                            disabled=is_uploading
                            style="padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;"
                        >
                            "Upload"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
