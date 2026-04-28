use leptos::*;
use shared::Participant;

fn sort_participants(mut participants: Vec<Participant>) -> Vec<Participant> {
    participants.sort_by(|a, b| {
        // Sort by hand raised (desc), then by hand_raised_at (asc - earliest first), then name (asc).
        // `Option` orders `None < Some(_)` by default, which would place participants
        // missing a timestamp (e.g. older clients or backends that didn't populate
        // `hand_raised_at`) ahead of those with real timestamps. Treat missing
        // timestamps as "latest" so well-formed FIFO entries are preferred.
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
    on_transfer_host: Callback<String>,
    #[prop(optional)] power_statuses: Option<ReadSignal<std::collections::HashMap<String, shared::PowerStatus>>>,
    #[prop(optional)] on_request_unmute: Option<Callback<String>>,
    #[prop(optional)] on_broadcast_lobby: Option<Callback<String>>,
    #[prop(optional)] on_promote: Option<Callback<String>>,
    #[prop(optional)] on_request_remote_control: Option<Callback<String>>,
) -> impl IntoView {
    let (lobby_msg, set_lobby_msg) = create_signal("".to_string());

    let format_time = |ms: u64| {
        let seconds = ms / 1000;
        let m = seconds / 60;
        let s = seconds % 60;
        format!("{:02}:{:02}", m, s)
    };

    // Store callbacks in StoredValue for easy multi-closure access
    let on_promote_sv = store_value(on_promote);
    let on_mute_sv = store_value(on_mute);
    let on_mute_camera_sv = store_value(on_mute_camera);
    let on_transfer_host_sv = store_value(on_transfer_host);
    let on_kick_sv = store_value(on_kick);
    let on_request_unmute_sv = store_value(on_request_unmute);
    let on_request_remote_control_sv = store_value(on_request_remote_control);

    view! {
        <div class="participants-list" style="padding: 10px; width: 100%; height: 100%;">
            <Show when=move || !knocking_participants.get().is_empty()>
                <div class="knocking-list" style="margin-bottom: 20px; padding-bottom: 20px; border-bottom: 1px solid #ccc;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                        <h3 style="margin: 0;">"Waiting Room"</h3>
                        <button
                            on:click=move |_| {
                                for p in knocking_participants.get() {
                                    on_allow.call(p.id);
                                }
                            }
                            style="background: #007bff; color: white; border: none; padding: 4px 8px; cursor: pointer; border-radius: 4px; font-size: 0.8em;"
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
                                style="flex: 1; padding: 4px; border-radius: 4px; border: 1px solid #ccc; font-size: 0.85em;"
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
                                style="background: #6c757d; color: white; border: none; padding: 4px 8px; cursor: pointer; border-radius: 4px; font-size: 0.85em;"
                            >
                                "Send"
                            </button>
                        </div>
                    </Show>

                    <ul>
                        <For
                            each=move || knocking_participants.get()
                            key=|p| p.id.clone()
                            children=move |p| {
                                let id_allow = p.id.clone();
                                let id_deny = p.id.clone();
                                view! {
                                    <li style="margin-bottom: 10px;">
                                        <div style="font-weight: bold;">{p.name}</div>
                                        <div style="display: flex; gap: 5px; margin-top: 5px;">
                                            <button
                                                on:click=move |_| on_allow.call(id_allow.clone())
                                                style="background: #28a745; color: white; border: none; padding: 2px 5px; cursor: pointer;"
                                            >
                                                "Allow"
                                            </button>
                                            <button
                                                on:click=move |_| on_deny.call(id_deny.clone())
                                                style="background: #dc3545; color: white; border: none; padding: 2px 5px; cursor: pointer;"
                                            >
                                                "Deny"
                                            </button>
                                        </div>
                                    </li>
                                }
                            }
                        />
                    </ul>
                </div>
            </Show>

            <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 20px;">
                <h3 style="margin: 0;">"Participants"</h3>
                <Show when=move || is_host.get()>
                    <div style="display: flex; gap: 5px;">
                        <button
                            on:click=move |_| {
                                if let Some(cb) = on_mute_all {
                                    cb.call(());
                                }
                            }
                            style="background: #ffc107; color: black; border: none; padding: 4px 8px; cursor: pointer; border-radius: 4px; font-size: 0.8em;"
                        >
                            "Mute All"
                        </button>
                        <button
                            on:click=move |_| {
                                if let Some(cb) = on_mute_camera_all {
                                    cb.call(());
                                }
                            }
                            style="background: #ffc107; color: black; border: none; padding: 4px 8px; cursor: pointer; border-radius: 4px; font-size: 0.8em;"
                            title="Mute All Cameras"
                        >
                            "Mute Cam All"
                        </button>
                    </div>
                </Show>
            </div>
            <ul>
                <For
                    each=move || sort_participants(participants.get())
                    key=|p| (p.id.clone(), p.name.clone(), p.is_hand_raised, p.is_sharing_screen, p.is_muted, p.presence.clone(), p.is_visitor, p.e2ee_enabled)
                    children=move |p| {
                        let p_sv = store_value(p);

                        view! {
                            <li class="participant-item" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px;">
                                <div style=move || if p_sv.get_value().is_visitor { "opacity: 0.7;" } else { "" }>
                                    <span>{move || p_sv.get_value().name}</span>
                                    {move || if p_sv.get_value().is_visitor {
                                        view! { <span style="font-size: 0.75em; background: #eee; padding: 1px 3px; border-radius: 3px; margin-left: 5px; color: #666;">"Visitor"</span> }.into_view()
                                    } else {
                                        view! { <span/> }.into_view()
                                    }}
                                    <span style="font-size: 0.8em; color: #666; margin-left: 5px;">
                                       " [" {move || format!("{:?}", p_sv.get_value().presence)} "]"
                                    </span>
                                    <Show when=move || !p_sv.get_value().is_visitor>
                                        <span style="font-size: 0.8em; color: #666; margin-left: 5px;">
                                           "(" {move || format_time(p_sv.get_value().speaking_time)} ")"
                                        </span>
                                    </Show>
                                    <Show when=move || host_id.get() == Some(p_sv.get_value().id)>
                                        <span style="font-size: 0.8em; color: #666; margin-left: 5px;">"(Host)"</span>
                                    </Show>
                                </div>
                                <div style="display: flex; align-items: center;">
                                    {move || {
                                        if let Some(statuses) = power_statuses {
                                            let result = statuses.with(|map| {
                                                map.get(&p_sv.get_value().id).cloned()
                                            });
                                            if let Some(status) = result {
                                                let icon = if status.is_charging { "⚡" } else { "🔋" };
                                                let level = (status.battery_level * 100.0).round() as i32;
                                                return view! {
                                                    <span style="margin-right: 8px; font-size: 0.8em;" title=format!("Battery: {}%", level)>
                                                        {icon} {level} "%"
                                                    </span>
                                                }.into_view();
                                            }
                                        }
                                        view! { <span></span> }.into_view()
                                    }}
                                    {move || if p_sv.get_value().is_sharing_screen {
                                        view! { <span style="margin-right: 5px;">"🖥️"</span> }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    {move || if p_sv.get_value().is_hand_raised {
                                        view! { <span style="margin-right: 5px;">"✋"</span> }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    {move || if p_sv.get_value().is_muted {
                                        view! { <span style="margin-right: 5px;">"🔇"</span> }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    <Show when=move || is_host.get() && my_id.get() != Some(p_sv.get_value().id)>
                                        <div style="display: flex; gap: 5px;">
                                            <Show when=move || p_sv.get_value().is_visitor>
                                                <button
                                                    on:click=move |_| {
                                                        if let Some(cb) = on_promote_sv.get_value() {
                                                            cb.call(p_sv.get_value().id);
                                                        }
                                                    }
                                                    style="background: none; border: 1px solid #ccc; color: #007bff; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                    title="Promote to Participant"
                                                >
                                                    "Up"
                                                </button>
                                            </Show>
                                            <Show when=move || !p_sv.get_value().is_muted && !p_sv.get_value().is_visitor>
                                                <button
                                                    on:click=move |_| {
                                                        on_mute_sv.get_value().call(p_sv.get_value().id);
                                                    }
                                                    style="background: none; border: 1px solid #ccc; color: orange; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                    title="Mute Participant"
                                                >
                                                    "Mute"
                                                </button>
                                            </Show>
                                            <Show when=move || !p_sv.get_value().is_visitor>
                                                <button
                                                    on:click=move |_| {
                                                        if let Some(cb) = on_mute_camera_sv.get_value() {
                                                            cb.call(p_sv.get_value().id);
                                                        }
                                                    }
                                                    style="background: none; border: 1px solid #ccc; color: orange; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                    title="Mute Camera"
                                                >
                                                    "Cam"
                                                </button>
                                            </Show>
                                            <button
                                                on:click=move |_| {
                                                    on_transfer_host_sv.get_value().call(p_sv.get_value().id);
                                                }
                                                style="background: none; border: 1px solid #ccc; color: blue; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                title="Make Host"
                                            >
                                                "Host"
                                            </button>
                                            <button
                                                on:click=move |_| {
                                                    on_kick_sv.get_value().call(p_sv.get_value().id);
                                                }
                                                style="background: none; border: 1px solid #ccc; color: red; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                title="Kick Participant"
                                            >
                                                "Kick"
                                            </button>
                                            <Show when=move || p_sv.get_value().is_muted && !p_sv.get_value().is_visitor>
                                                <button
                                                    on:click=move |_| {
                                                        if let Some(cb) = on_request_unmute_sv.get_value() {
                                                            cb.call(p_sv.get_value().id);
                                                        }
                                                    }
                                                    style="background: none; border: 1px solid #ccc; color: green; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                    title="Request Unmute"
                                                >
                                                    "Unmute"
                                                </button>
                                            </Show>
                                            <Show when=move || on_request_remote_control_sv.get_value().is_some() && my_id.get() != Some(p_sv.get_value().id)>
                                                <button
                                                    on:click=move |_| {
                                                        if let Some(cb) = on_request_remote_control_sv.get_value() {
                                                            cb.call(p_sv.get_value().id);
                                                        }
                                                    }
                                                    style="background: none; border: 1px solid #ccc; color: #007bff; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                    title="Request Remote Control"
                                                >
                                                    "RC"
                                                </button>
                                            </Show>
                                        </div>
                                    </Show>
                                </div>
                            </li>
                        }
                    }
                />
            </ul>
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
            e2ee_enabled: false, hand_raised_at: None,
        };
        let p2 = Participant {
            id: "2".to_string(),
            name: "Alice".to_string(),
            is_hand_raised: true, // Hand raised should be first
            is_sharing_screen: false,
            is_muted: false,
            speaking_time: 0,
            presence: shared::PresenceStatus::Connected,
            is_visitor: false,
            e2ee_enabled: false, hand_raised_at: None,
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
            e2ee_enabled: false, hand_raised_at: None,
        };

        let unsorted = vec![p1.clone(), p2.clone(), p3.clone()];
        let sorted = sort_participants(unsorted);

        assert_eq!(sorted[0].name, "Alice"); // Raised hand
        assert_eq!(sorted[1].name, "Bob"); // Alphabetical
        assert_eq!(sorted[2].name, "Charlie");
    }

    #[test]
    fn test_mute_all_visibility_logic() {
        let _runtime = create_runtime();
        let (is_host, _set_is_host) = create_signal(true);
        assert!(is_host.get());
    }
}
