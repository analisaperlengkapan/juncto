mod components_ui;
mod pages;
mod state;
mod utils;
mod chat;
mod participants;
mod toolbox;
mod settings;
mod reactions;
mod polls;
mod whiteboard;
mod media;
mod shortcuts;
mod speaker_stats;
mod virtual_background;
mod connection_stats;
mod i18n;

use leptos::*;
use leptos_router::*;
use pages::home::Home;
use pages::room::Room;
use wasm_bindgen::prelude::*;
use crate::components_ui::toast::{provide_toast_context, ToastContainer};
use crate::i18n::provide_i18n_context;

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
