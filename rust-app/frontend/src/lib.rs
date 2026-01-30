mod components;
mod chat;
mod participants;
mod toolbox;

use leptos::*;
use leptos_router::*;
use components::*;
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes>
                    <Route path="" view=WelcomePage/>
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
