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
                let intent_url = href.replace("https://", "intent://")
                    .replace("http://", "intent://") + "#Intent;scheme=org.juncto.meet;package=org.juncto.meet;end";

                // We typically show a prompt before redirecting, but here we just log for parity
                web_sys::console::log_1(&format!("Deep link candidate (Android): {}", intent_url).into());
            } else if user_agent.contains("iphone") || user_agent.contains("ipad") {
                 let href = window.location().href().unwrap_or_default();
                 let app_url = href.replace("https://", "org.juncto.meet://")
                    .replace("http://", "org.juncto.meet://");
                 web_sys::console::log_1(&format!("Deep link candidate (iOS): {}", app_url).into());
            }
        }
    });

    view! { <span style="display:none;" /> }
}
