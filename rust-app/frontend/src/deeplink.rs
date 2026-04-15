use leptos::*;

#[component]
pub fn DeepLinking() -> impl IntoView {
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            let user_agent = navigator.user_agent().unwrap_or_default().to_lowercase();

            // Check if mobile
            if user_agent.contains("android") {
                // For Android, we can try to redirect to intent://
                let href = window.location().href().unwrap_or_default();
                let intent_url = href.replacen("https://", "intent://", 1)
                    .replacen("http://", "intent://", 1) + "#Intent;scheme=org.juncto.meet;package=org.juncto.meet;end";

                // We typically show a prompt before redirecting, but here we just log for parity
                web_sys::console::log_1(&format!("Deep link candidate (Android): {}", intent_url).into());
            } else if user_agent.contains("iphone") || user_agent.contains("ipad") {
                 let href = window.location().href().unwrap_or_default();
                 let app_url = href.replacen("https://", "org.juncto.meet://", 1)
                    .replacen("http://", "org.juncto.meet://", 1);
                 web_sys::console::log_1(&format!("Deep link candidate (iOS): {}", app_url).into());
            }
        }
    });

    view! { <span style="display:none;" /> }
}
