use leptos::*;

#[component]
pub fn LobbyScreen(
    announcement: ReadSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="lobby-container" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background: #333; color: white;">
            <div class="card" style="background: #444; padding: 40px; border-radius: 8px; text-align: center; max-width: 500px; width: 90%;">
                <h2>"Waiting for host..."</h2>
                <p>"You have asked to join the meeting. Please wait for the host to let you in."</p>

                <Show when=move || announcement.get().is_some()>
                    <div class="announcement" style="margin-top: 20px; padding: 15px; background: #555; border-left: 4px solid #007bff; text-align: left;">
                        <strong style="display: block; margin-bottom: 5px; color: #007bff;">"Message from Host:"</strong>
                        <span>{move || announcement.get()}</span>
                    </div>
                </Show>

                <div class="spinner" style="margin-top: 30px; font-size: 32px; animation: spin 2s linear infinite;">"⏳"</div>
                <style>
                    "@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }"
                </style>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lobby_screen_compiles() {
        let _ = create_runtime();
        let (announcement, _) = create_signal(None::<String>);
        let _view = view! { <LobbyScreen announcement=announcement /> };
        let _ = true;
    }
}
