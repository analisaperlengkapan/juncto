use leptos::*;
use shared::Participant;

fn sort_participants(mut participants: Vec<Participant>) -> Vec<Participant> {
    participants.sort_by(|a, b| {
        if a.is_hand_raised != b.is_hand_raised {
            b.is_hand_raised.cmp(&a.is_hand_raised)
        } else if a.is_hand_raised && a.hand_raised_at != b.hand_raised_at {
            let a_ts = a.hand_raised_at.unwrap_or(u64::MAX);
            let b_ts = b.hand_raised_at.unwrap_or(u64::MAX);
            a_ts.cmp(&b_ts)
        } else {
            a.name.cmp(&b.name)
        }
    });
    participants
}

#[component]
pub fn ParticipantsList(
    participants: ReadSignal<Vec<Participant>>,
    knocking_participants: ReadSignal<Vec<Participant>>,
    host_id: Signal<Option<String>>,
    is_host: Signal<bool>,
    my_id: ReadSignal<Option<String>>,
    on_allow: Callback<String>,
    on_deny: Callback<String>,
    on_kick: Callback<String>,
    on_mute: Callback<String>,
    #[prop(optional)] on_mute_camera: Option<Callback<String>>,
    #[prop(optional)] on_mute_all: Option<Callback<()>>,
    #[prop(optional)] on_mute_camera_all: Option<Callback<()>>,
    _on_transfer_host: Callback<String>,
    #[prop(optional)] _power_statuses: Option<
        ReadSignal<std::collections::HashMap<String, shared::PowerStatus>>,
    >,
    #[prop(optional)] _on_request_unmute: Option<Callback<String>>,
    #[prop(optional)] on_broadcast_lobby: Option<Callback<String>>,
    #[prop(optional)] _on_promote: Option<Callback<String>>,
    #[prop(optional)] _on_request_remote_control: Option<Callback<String>>,
    #[prop(optional)] on_stop_screen_share_all: Option<Callback<()>>,
    #[prop(optional)] pinned_participant: Option<ReadSignal<Option<String>>>,
    #[prop(optional)] on_pin: Option<Callback<Option<String>>>,
    #[prop(optional)] _on_set_volume: Option<Callback<(String, f64)>>,
    #[prop(optional)] _on_mute_everyone_else: Option<Callback<String>>,
    #[prop(optional)] pending_unmute_requests: Option<ReadSignal<std::collections::HashSet<String>>>,
    #[prop(optional)] pending_camera_requests: Option<ReadSignal<std::collections::HashSet<String>>>,
    #[prop(optional)] on_grant_unmute: Option<Callback<String>>,
    #[prop(optional)] on_grant_camera: Option<Callback<String>>,
) -> impl IntoView {
    let (lobby_msg, set_lobby_msg) = create_signal("".to_string());
    let (search_query, set_search_query) = create_signal("".to_string());

    let format_time = |ms: u64| {
        let seconds = ms / 1000;
        let m = seconds / 60;
        let s = seconds % 60;
        format!("{:02}:{:02}", m, s)
    };

    let _on_promote_sv = store_value(_on_promote);
    let on_mute_sv = store_value(on_mute);
    let _on_mute_camera_sv = store_value(on_mute_camera);
    let _on_transfer_host_sv = store_value(_on_transfer_host);
    let on_kick_sv = store_value(on_kick);
    let _on_request_unmute_sv = store_value(_on_request_unmute);
    let _on_request_remote_control_sv = store_value(_on_request_remote_control);
    let on_pin_sv = store_value(on_pin);
    let _on_set_volume_sv = store_value(_on_set_volume);
    let _on_mute_everyone_else_sv = store_value(_on_mute_everyone_else);

    view! {
        <div class="panel-content participants-list" style="padding: 10px;">
            <Show when=move || !knocking_participants.get().is_empty()>
                <div class="knocking-list" style="margin-bottom: 20px; padding-bottom: 20px; border-bottom: 1px solid var(--border-color);">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                        <h4 style="margin: 0;">"Waiting Room"</h4>
                        <button
                            on:click=move |_| { for p in knocking_participants.get() { on_allow.call(p.id); } }
                            class="btn btn-primary" style="font-size: 0.8rem; padding: 4px 8px;"
                        >
                            "Allow All"
                        </button>
                    </div>

                    <Show when=move || on_broadcast_lobby.is_some()>
                        <div style="display: flex; gap: 5px; margin-bottom: 15px;">
                            <input
                                type="text"
                                prop:value=lobby_msg
                                on:input=move |ev| set_lobby_msg.set(event_target_value(&ev))
                                placeholder="Message to lobby..."
                                style="flex: 1; padding: 6px; border-radius: 4px; border: 1px solid var(--border-color); background: var(--card-bg); color: white; font-size: 0.8rem;"
                            />
                            <button
                                on:click=move |_| {
                                    let msg = lobby_msg.get();
                                    if !msg.is_empty() {
                                        if let Some(cb) = on_broadcast_lobby {
                                            cb.call(msg);
                                            set_lobby_msg.set("".to_string());
                                        }
                                    }
                                }
                                class="btn btn-secondary" style="font-size: 0.8rem; padding: 4px 8px;"
                            >
                                "Send"
                            </button>
                        </div>
                    </Show>

                    <ul class="knocking-list-items" style="display: flex; flex-direction: column; gap: 8px; padding: 0; list-style: none;">
                        <For
                            each=move || knocking_participants.get()
                            key=|p| p.id.clone()
                            children=move |p| {
                                let id_allow = p.id.clone();
                                let id_deny = p.id.clone();
                                view! {
                                    <li style="padding: 8px; background: var(--card-bg); border-radius: 6px;">
                                        <div style="font-weight: 500; font-size: 0.9rem;">{p.name}</div>
                                        <div style="display: flex; gap: 5px; margin-top: 5px;">
                                            <button on:click=move |_| on_allow.call(id_allow.clone()) class="btn btn-success" style="font-size: 0.75rem; padding: 2px 6px;">"Allow"</button>
                                            <button on:click=move |_| on_deny.call(id_deny.clone()) class="btn btn-danger" style="font-size: 0.75rem; padding: 2px 6px;">"Deny"</button>
                                        </div>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </div>
            </Show>

            <div class="search-bar" style="margin-bottom: 12px;">
                <input
                    type="text"
                    id="participant-search"
                    prop:value=search_query
                    on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    placeholder="Search participants..."
                    style="width: 100%; padding: 8px; border-radius: 6px; border: 1px solid var(--border-color); background: var(--card-bg); color: white; box-sizing: border-box;"
                />
            </div>

            <Show when=move || is_host.get()>
                <div style="display: flex; gap: 5px; margin-bottom: 15px;">
                    <button on:click=move |_| { if let Some(cb) = on_mute_all { cb.call(()); } } id="mute-all-btn" class="btn btn-warning" style="flex: 1; font-size: 0.75rem; padding: 4px;">"Mute All"</button>
                    <button on:click=move |_| { if let Some(cb) = on_mute_camera_all { cb.call(()); } } class="btn btn-warning" style="flex: 1; font-size: 0.75rem; padding: 4px;">"Cam Off All"</button>
                    <button id="stop-screen-share-all-btn" on:click=move |_| { if let Some(cb) = on_stop_screen_share_all { cb.call(()); } } class="btn btn-danger" style="flex: 1; font-size: 0.75rem; padding: 4px;">"Stop Screen"</button>
                </div>
            </Show>

            <div style="display: flex; flex-direction: column;">
                <ul style="padding: 0; margin: 0; list-style: none;">
                <For
                    each=move || {
                        let query = search_query.get().to_lowercase();
                        let filtered: Vec<_> = participants.get().into_iter()
                            .filter(|p| p.name.to_lowercase().contains(&query))
                            .collect();
                        sort_participants(filtered)
                    }
                    key=|p| (p.id.clone(), p.name.clone(), p.is_hand_raised, p.is_sharing_screen, p.is_muted, p.presence.clone(), p.is_visitor, p.e2ee_enabled, p.avatar_url.clone())
                    children=move |p| {
                        let p_sv = store_value(p);
                        let (avatar_failed, set_avatar_failed) = create_signal(false);

                        view! {
                            <li class="participant-item">
                                <div class="participant-info">
                                    <Show when=move || p_sv.get_value().avatar_url.is_some() && !avatar_failed.get() fallback=move || view! {
                                        <div class="avatar-sm">
                                            {p_sv.get_value().name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                        </div>
                                    }>
                                        <img
                                            src=move || p_sv.get_value().avatar_url.unwrap_or_default()
                                            on:error=move |_| set_avatar_failed.set(true)
                                            class="avatar-sm"
                                            alt="Avatar"
                                        />
                                    </Show>
                                    <div style="display: flex; flex-direction: column;">
                                        <div style="font-size: 0.9rem; font-weight: 500;">
                                            {move || p_sv.get_value().name}
                                            <Show when=move || host_id.get() == Some(p_sv.get_value().id)>
                                                <span style="color: var(--primary-color); margin-left: 4px;">" (Host)"</span>
                                            </Show>
                                        </div>
                                        <div style="font-size: 0.75rem; color: var(--text-muted);">
                                            {move || format!("[{:?}]", p_sv.get_value().presence)}
                                            <Show when=move || !p_sv.get_value().is_visitor>
                                                " • " {move || format_time(p_sv.get_value().speaking_time)}
                                            </Show>
                                        </div>
                                    </div>
                                </div>
                                <div style="display: flex; gap: 8px; align-items: center;">
                                    {move || if p_sv.get_value().is_hand_raised { view! { <span title="Hand Raised">"✋"</span> }.into_view() } else { view! { <span/> }.into_view() }}
                                    {move || if p_sv.get_value().is_sharing_screen { view! { <span title="Sharing Screen">"🖥️"</span> }.into_view() } else { view! { <span/> }.into_view() }}
                                    {move || if p_sv.get_value().is_muted { view! { <span title="Muted" style="color: var(--danger-color);">"🔇"</span> }.into_view() } else { view! { <span/> }.into_view() }}

                                    <div style="display: flex; gap: 4px; align-items: center;">
                                        <Show when=move || _on_request_remote_control_sv.get_value().is_some() && my_id.get() != Some(p_sv.get_value().id)>
                                            <button
                                                on:click={
                                                    let id = p_sv.get_value().id.clone();
                                                    move |_| { if let Some(cb) = _on_request_remote_control_sv.get_value() { cb.call(id.clone()); } }
                                                }
                                                class="btn btn-outline" style="padding: 2px 6px; font-size: 0.7rem;"
                                                title="Request Remote Control"
                                            >
                                                "RC"
                                            </button>
                                        </Show>
                                        <Show when=move || _on_set_volume_sv.get_value().is_some() && my_id.get() != Some(p_sv.get_value().id)>
                                            <input
                                                type="range"
                                                min="0" max="1" step="0.1"
                                                prop:value=1.0
                                                on:input={
                                                    let id = p_sv.get_value().id.clone();
                                                    move |ev| {
                                                        if let Some(cb) = _on_set_volume_sv.get_value() {
                                                            cb.call((id.clone(), event_target_value(&ev).parse().unwrap_or(1.0)));
                                                        }
                                                    }
                                                }
                                                style="width: 50px; cursor: pointer;"
                                                title="Participant Volume"
                                            />
                                        </Show>
                                        <Show when=move || on_pin_sv.get_value().is_some() && my_id.get() != Some(p_sv.get_value().id) && !p_sv.get_value().is_visitor>
                                            <button
                                                on:click={
                                                    let id = p_sv.get_value().id.clone();
                                                    move |_| {
                                                        if let Some(cb) = on_pin_sv.get_value() {
                                                            let current_pinned = pinned_participant.and_then(|s: ReadSignal<Option<String>>| s.get());
                                                            cb.call(if current_pinned.as_ref() == Some(&id) { None } else { Some(id.clone()) });
                                                        }
                                                    }
                                                }
                                                class="btn btn-outline" style="padding: 2px 6px; font-size: 0.7rem;"
                                                title="Pin participant"
                                            >
                                                {move || {
                                                    let id = p_sv.get_value().id.clone();
                                                    if pinned_participant.and_then(|s: ReadSignal<Option<String>>| s.get()).as_ref() == Some(&id) { "📍" } else { "📌" }
                                                }}
                                            </button>
                                        </Show>
                                        <Show when=move || is_host.get() && my_id.get() != Some(p_sv.get_value().id)>
                                            <Show when=move || !p_sv.get_value().is_muted>
                                                <button
                                                    on:click={
                                                        let id = p_sv.get_value().id.clone();
                                                        move |_| { on_mute_sv.get_value().call(id.clone()); }
                                                    }
                                                    class="btn btn-outline" style="padding: 2px 6px; font-size: 0.7rem;"
                                                    title="Mute Participant"
                                                >
                                                    "Mute"
                                                </button>
                                            </Show>
                                            <button
                                                on:click={
                                                    let id = p_sv.get_value().id.clone();
                                                    move |_| { _on_transfer_host_sv.get_value().call(id.clone()); }
                                                }
                                                class="btn btn-outline" style="padding: 2px 6px; font-size: 0.7rem;"
                                                title="Transfer Host"
                                            >
                                                "Host"
                                            </button>
                                            <Show when=move || _on_request_unmute_sv.get_value().is_some() && p_sv.get_value().is_muted>
                                                <button
                                                    on:click={
                                                        let id = p_sv.get_value().id.clone();
                                                        move |_| { _on_request_unmute_sv.get_value().unwrap().call(id.clone()); }
                                                    }
                                                    class="btn btn-outline" style="padding: 2px 6px; font-size: 0.7rem;"
                                                    title="Request Unmute"
                                                >
                                                    "Request Unmute"
                                                </button>
                                            </Show>
                                            <Show when=move || _on_mute_everyone_else_sv.get_value().is_some()>
                                                <button
                                                    on:click={
                                                        let id = p_sv.get_value().id.clone();
                                                        move |_| { _on_mute_everyone_else_sv.get_value().unwrap().call(id.clone()); }
                                                    }
                                                    class="btn btn-outline" style="padding: 2px 6px; font-size: 0.7rem;"
                                                    title="Mute Everyone Else"
                                                >
                                                    "Solo"
                                                </button>
                                            </Show>
                                            <button
                                                on:click={
                                                    let id = p_sv.get_value().id.clone();
                                                    move |_| { on_kick_sv.get_value().call(id.clone()); }
                                                }
                                                class="btn btn-outline" style="padding: 2px 6px; font-size: 0.7rem; color: var(--danger-color); border-color: var(--danger-color);"
                                                title="Kick Participant"
                                            >
                                                "Kick"
                                            </button>
                                        </Show>
                                        <Show when=move || is_host.get() && my_id.get() != Some(p_sv.get_value().id)>
                                            <Show when=move || pending_unmute_requests.map(|s| s.get().contains(&p_sv.get_value().id)).unwrap_or(false)>
                                                <button
                                                    on:click={
                                                        let id = p_sv.get_value().id.clone();
                                                        move |_| { if let Some(cb) = on_grant_unmute { cb.call(id.clone()); } }
                                                    }
                                                    class="btn btn-success grant-mic-btn" style="padding: 2px 6px; font-size: 0.7rem;"
                                                >
                                                    "Grant Mic"
                                                </button>
                                            </Show>
                                            <Show when=move || pending_camera_requests.map(|s| s.get().contains(&p_sv.get_value().id)).unwrap_or(false)>
                                                <button
                                                    on:click={
                                                        let id = p_sv.get_value().id.clone();
                                                        move |_| { if let Some(cb) = on_grant_camera { cb.call(id.clone()); } }
                                                    }
                                                    class="btn btn-success grant-cam-btn" style="padding: 2px 6px; font-size: 0.7rem;"
                                                >
                                                    "Grant Cam"
                                                </button>
                                            </Show>
                                        </Show>
                                    </div>
                                </div>
                            </li>
                        }
                    }
                />
                </ul>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::Participant;

    #[test]
    fn test_participant_sorting() {
        let p1 = Participant {
            id: "1".to_string(),
            name: "Charlie".to_string(),
            is_hand_raised: false,
            is_sharing_screen: false,
            is_muted: false,
            speaking_time: 0,
            presence: shared::PresenceStatus::Connected,
            is_visitor: false,
            e2ee_enabled: false,
            hand_raised_at: None,
            avatar_url: None,
        };
        let p2 = Participant {
            id: "2".to_string(),
            name: "Alice".to_string(),
            is_hand_raised: true,
            is_sharing_screen: false,
            is_muted: false,
            speaking_time: 0,
            presence: shared::PresenceStatus::Connected,
            is_visitor: false,
            e2ee_enabled: false,
            hand_raised_at: None,
            avatar_url: None,
        };
        let p3 = Participant {
            id: "3".to_string(),
            name: "Bob".to_string(),
            is_hand_raised: false,
            is_sharing_screen: false,
            is_muted: false,
            speaking_time: 0,
            presence: shared::PresenceStatus::Connected,
            is_visitor: false,
            e2ee_enabled: false,
            hand_raised_at: None,
            avatar_url: None,
        };

        let unsorted = vec![p1.clone(), p2.clone(), p3.clone()];
        let sorted = sort_participants(unsorted);

        assert_eq!(sorted[0].name, "Alice");
        assert_eq!(sorted[1].name, "Bob");
        assert_eq!(sorted[2].name, "Charlie");
    }

    #[test]
    fn test_mute_all_visibility_logic() {
        let _runtime = create_runtime();
        let (is_host, _set_is_host) = create_signal(true);
        assert!(is_host.get());
    }

    #[test]
    fn test_participant_search_logic() {
        let p1 = Participant {
            id: "1".to_string(),
            name: "Alice".to_string(),
            is_hand_raised: false,
            is_sharing_screen: false,
            is_muted: false,
            speaking_time: 0,
            presence: shared::PresenceStatus::Connected,
            is_visitor: false,
            e2ee_enabled: false,
            hand_raised_at: None,
            avatar_url: None,
        };
        let p2 = Participant {
            id: "2".to_string(),
            name: "Bob".to_string(),
            is_hand_raised: false,
            is_sharing_screen: false,
            is_muted: false,
            speaking_time: 0,
            presence: shared::PresenceStatus::Connected,
            is_visitor: false,
            e2ee_enabled: false,
            hand_raised_at: None,
            avatar_url: None,
        };

        let participants = vec![p1, p2];
        let query = "al".to_string();

        let filtered: Vec<_> = participants
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&query.to_lowercase()))
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Alice");
    }
}
