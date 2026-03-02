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
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
