with open('rust-app/frontend/src/state.rs', 'r') as f:
    content = f.read()

# Update AudioMonitor initialization
patch = """
            if stream.get_audio_tracks().length() > 0 {
                let on_speaking = Box::new(move |is_speaking: bool| {
                    if let Some(socket) = ws_clone_audio.get() {
                        let msg = ClientMessage::Speaking(is_speaking);
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = socket.send_with_str(&json);
                        }
                    }
                });

                let add_toast_clone = add_toast.clone();
                let on_no_audio = Box::new(move || {
                    add_toast_clone("No audio input detected. Please check your microphone.".to_string(), crate::components_ui::toast::ToastType::Warning);
                });

                if let Ok(monitor) = AudioMonitor::new(&stream, on_speaking, Some(on_no_audio)) {
                    set_audio_monitor.set(Some(monitor));
                }
            }
"""

content = content.replace("""            if stream.get_audio_tracks().length() > 0 {
                let on_speaking = Box::new(move |is_speaking: bool| {
                    if let Some(socket) = ws_clone_audio.get() {
                        let msg = ClientMessage::Speaking(is_speaking);
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = socket.send_with_str(&json);
                        }
                    }
                });
                if let Ok(monitor) = AudioMonitor::new(&stream, on_speaking) {
                    set_audio_monitor.set(Some(monitor));
                }
            }""", patch)

with open('rust-app/frontend/src/state.rs', 'w') as f:
    f.write(content)
