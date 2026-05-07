use leptos::*;
use shared::BreakoutRoom;

#[component]
pub fn BreakoutRooms(
    breakout_rooms: ReadSignal<Vec<BreakoutRoom>>,
    current_room_id: ReadSignal<Option<String>>,
    is_host: Signal<bool>,
    on_create: Callback<String>,
    on_join: Callback<Option<String>>,
    #[prop(optional)] on_remove: Option<Callback<String>>,
    #[prop(optional)] on_rename: Option<Callback<(String, String)>>,
    #[prop(optional)] on_close_all: Option<Callback<()>>,
    #[prop(optional)] on_auto_assign: Option<Callback<()>>,
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
        <div class="breakout-rooms">
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <h4 style="margin: 0;">"Breakout Rooms"</h4>
                <div style="display: flex; gap: 8px; align-items: center;">
                    <Show when=move || is_host.get()>
                        <button
                            on:click=move |_| { if let Some(cb) = on_auto_assign { cb.call(()); } }
                            class="btn btn-info" style="font-size: 0.8rem; padding: 4px 8px;"
                        >
                            "Auto Assign"
                        </button>
                        <button
                            on:click=move |_| { if let Some(cb) = on_close_all { cb.call(()); } }
                            class="btn btn-danger" style="font-size: 0.8rem; padding: 4px 8px;"
                        >
                            "Close All"
                        </button>
                    </Show>
                    <Show when=move || current_room_id.get().is_some()>
                        <button
                            on:click=move |_| on_join.call(None)
                            class="btn btn-secondary" style="font-size: 0.8rem; padding: 4px 8px;"
                        >
                            "Return to Main"
                        </button>
                    </Show>
                </div>
            </div>

            <div class="rooms-list">
                <For
                    each=move || breakout_rooms.get()
                    key=|r| r.id.clone()
                    children=move |r| {
                        let rid = Some(r.id.clone());
                        let rid_active = rid.clone();
                        let rid_show = rid.clone();
                        let rid_remove = rid.clone();
                        let rid_rename = rid.clone();
                        let r_name = r.name.clone();

                        view! {
                            <div class=move || format!("breakout-room-tag {}", if current_room_id.get() == rid_active { "active" } else { "" })>
                                <span style="font-weight: 500;">{r_name.clone()}</span>
                                <Show when=move || current_room_id.get() != rid_show>
                                    {
                                        let rid = rid.clone();
                                        view! {
                                            <button on:click=move |_| on_join.call(rid.clone()) class="btn btn-primary" style="font-size: 0.7rem; padding: 2px 6px;">"Join"</button>
                                        }
                                    }
                                </Show>
                                <Show when=move || is_host.get()>
                                    <div style="display: flex; gap: 4px; border-left: 1px solid rgba(255,255,255,0.2); padding-left: 8px; margin-left: 4px;">
                                        <button
                                            on:click={
                                                let rid = rid_rename.clone();
                                                let rname = r_name.clone();
                                                move |_| {
                                                    if let Some(cb) = on_rename {
                                                        if let Some(new_name) = window().prompt_with_message_and_default("Rename room:", &rname).ok().flatten() {
                                                            if !new_name.is_empty() {
                                                                cb.call((rid.clone().unwrap(), new_name));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            class="btn btn-outline" style="padding: 2px 4px; font-size: 0.7rem;" title="Rename"
                                        >
                                            "✎"
                                        </button>
                                        <button
                                            on:click={
                                                let rid = rid_remove.clone();
                                                move |_| { if let Some(cb) = on_remove { cb.call(rid.clone().unwrap()); } }
                                            }
                                            class="btn btn-outline" style="padding: 2px 4px; font-size: 0.7rem; color: var(--danger-color); border-color: var(--danger-color);" title="Remove"
                                        >
                                            "×"
                                        </button>
                                    </div>
                                </Show>
                            </div>
                        }
                    }
                />
            </div>

            <Show when=move || is_host.get()>
                <div style="display: flex; gap: 8px; align-items: center; margin-top: 10px;">
                    <input
                        type="text"
                        prop:value=new_room_name
                        on:input=move |ev| set_new_room_name.set(event_target_value(&ev))
                        placeholder="New Room Name"
                        style="padding: 6px 12px; border-radius: 6px; border: 1px solid var(--border-color); background: var(--card-bg); color: white; flex: 1; max-width: 250px;"
                    />
                    <button on:click=create class="btn btn-success" style="font-size: 0.8rem; padding: 6px 12px;">"Create"</button>
                </div>
            </Show>
        </div>
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakout_compiles() {
        let _ = create_runtime();
        let _show = create_rw_signal(true);
        let breakout_rooms = create_rw_signal(Vec::new());
        let current_room_id = create_rw_signal(None);
        let _on_close = Callback::new(|_: ()| {});
        let on_create = Callback::new(|_: String| {});
        let on_join = Callback::new(|_: Option<String>| {});

        let is_host = create_rw_signal(true);
        let _view = view! {
            <BreakoutRooms
                breakout_rooms=breakout_rooms.read_only()
                current_room_id=current_room_id.read_only()
                on_create=on_create
                on_join=on_join
                is_host=is_host.into()
            />
        };
        let _ = true;
    }
}
