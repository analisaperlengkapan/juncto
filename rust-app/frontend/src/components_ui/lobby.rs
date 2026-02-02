use leptos::*;

#[component]
pub fn LobbyScreen() -> impl IntoView {
    view! {
        <div class="lobby-container" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background: #333; color: white;">
            <div class="card" style="background: #444; padding: 40px; border-radius: 8px; text-align: center;">
                <h2>"Waiting for host..."</h2>
                <p>"You have asked to join the meeting. Please wait for the host to let you in."</p>
                <div class="spinner" style="margin-top: 20px; font-size: 24px;">"⏳"</div>
            </div>
        </div>
    }
}
