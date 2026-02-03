use leptos::*;
use shared::BreakoutRoom;

#[component]
pub fn BreakoutRooms(
    breakout_rooms: ReadSignal<Vec<BreakoutRoom>>,
    current_room_id: ReadSignal<Option<String>>,
    is_host: Signal<bool>,
    on_create: Callback<String>,
    on_join: Callback<Option<String>>,
) -> impl IntoView {
    let (new_room_name, set_new_room_name) = create_signal("".to_string());

    let create = move |_| {
        let name = new_room_name.get();
        if !name.is_empty() {
            on_create.call(name);
            set_new_room_name.set("".to_string());
        }
    };

    view! {
        <div class="breakout-rooms" style="padding: 10px; background: #f8f9fa; border-bottom: 1px solid #ccc;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                <h4 style="margin: 0;">"Breakout Rooms"</h4>
                <Show when=move || current_room_id.get().is_some()>
                    <button
                        on:click=move |_| on_join.call(None) // Join Main
                        style="padding: 5px 10px; background-color: #6c757d; color: white; border: none; cursor: pointer; border-radius: 4px;"
                    >
                        "Return to Main"
                    </button>
                </Show>
            </div>

            <div class="rooms-list" style="display: flex; gap: 10px; flex-wrap: wrap; margin-bottom: 10px;">
                <For
                    each=move || breakout_rooms.get()
                    key=|r| r.id.clone()
                    children=move |r| {
                        let rid = Some(r.id.clone());
                        let is_current = current_room_id.get() == rid;
                        view! {
                            <div style=move || format!("padding: 5px 10px; border: 1px solid #ccc; border-radius: 4px; background: {};", if is_current { "#e9ecef" } else { "white" })>
                                <span style="margin-right: 5px;">{r.name}</span>
                                <Show when=move || !is_current>
                                    {
                                        let rid = rid.clone();
                                        view! {
                                            <button
                                                on:click=move |_| on_join.call(rid.clone())
                                                style="padding: 2px 5px; background-color: #007bff; color: white; border: none; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                            >
                                                "Join"
                                            </button>
                                        }
                                    }
                                </Show>
                            </div>
                        }
                    }
                />
            </div>

            <Show when=move || is_host.get()>
                <div style="display: flex; gap: 5px;">
                    <input
                        type="text"
                        prop:value=new_room_name
                        on:input=move |ev| set_new_room_name.set(event_target_value(&ev))
                        placeholder="New Room Name"
                        style="padding: 5px; border: 1px solid #ccc; border-radius: 4px;"
                    />
                    <button
                        on:click=create
                        style="padding: 5px 10px; background-color: #28a745; color: white; border: none; cursor: pointer; border-radius: 4px;"
                    >
                        "Create"
                    </button>
                </div>
            </Show>
        </div>
    }
}
