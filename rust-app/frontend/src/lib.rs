mod chat;
mod components_ui;
mod connection_stats;
mod i18n;
mod media;
mod pages;
mod participants;
mod polls;
mod reactions;
mod settings;
mod shortcuts;
mod speaker_stats;
mod state;
mod toolbox;
mod utils;
mod virtual_background;
mod webrtc;
mod whiteboard;

use crate::components_ui::toast::{provide_toast_context, ToastContainer};
use crate::i18n::provide_i18n_context;
use leptos::*;
use leptos_router::*;
use pages::home::Home;
use pages::room::Room;
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    provide_i18n_context();
    provide_toast_context();

    // Check for WebRTC support (RTCPeerConnection)
    let is_webrtc_supported = if let Some(window) = web_sys::window() {
        js_sys::Reflect::has(&window, &JsValue::from_str("RTCPeerConnection")).unwrap_or(false)
    } else {
        false
    };

    if !is_webrtc_supported {
        return view! {
            <div class="unsupported-browser-container" style="display: flex; justify-content: center; align-items: center; height: 100vh; background-color: #f8d7da; color: #721c24; text-align: center; font-family: sans-serif;">
                <div>
                    <h1 style="font-size: 2em; margin-bottom: 20px;">"Unsupported Browser"</h1>
                    <p style="font-size: 1.2em;">"WebRTC is required to use this application."</p>
                    <p>"Please upgrade your browser to a modern version (e.g., Chrome, Firefox, Safari, Edge) that supports WebRTC."</p>
                </div>
            </div>
        }.into_view();
    }

    view! {
        <ToastContainer />
        <Router>
            <main>
                <Routes>
                    <Route path="" view=Home/>
                    <Route path="/room/:id" view=Room/>
                </Routes>
            </main>
        </Router>
    }.into_view()
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
