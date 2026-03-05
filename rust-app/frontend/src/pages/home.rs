use crate::utils::create_room_url;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Home() -> impl IntoView {
    let (room_name, set_room_name) = create_signal("My Meeting".to_string());
    let navigate = use_navigate();

    let create_meeting = move |_| {
        let name = room_name.get();
        let url = create_room_url(&name);
        navigate(&url, Default::default());
    };

    view! {
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
            <div class="max-w-md w-full space-y-8 bg-white p-10 rounded-xl shadow-lg">
                <div>
                    <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
                        "Welcome to Juncto"
                    </h2>
                    <p class="mt-2 text-center text-sm text-gray-600">
                        "Secure, high-quality video meetings built with Rust."
                    </p>
                </div>
                <div class="mt-8 space-y-6">
                    <div class="rounded-md shadow-sm -space-y-px">
                        <div>
                            <label for="room-name" class="sr-only">"Room Name"</label>
                            <input
                                id="room-name"
                                name="room-name"
                                type="text"
                                required
                                class="appearance-none rounded-md relative block w-full px-3 py-3 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm transition duration-150 ease-in-out"
                                placeholder="Enter a room name to join or create"
                                on:input=move |ev| set_room_name.set(event_target_value(&ev))
                                prop:value=room_name
                            />
                        </div>
                    </div>

                    <div>
                        <button
                            on:click=create_meeting
                            class="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition duration-150 ease-in-out"
                        >
                            "Start Meeting"
                        </button>
                    </div>
                </div>
            </div>
            <div class="mt-8 text-center text-xs text-gray-500">
                <p>"Powered by Leptos & Axum"</p>
            </div>
        </div>
    }
}
