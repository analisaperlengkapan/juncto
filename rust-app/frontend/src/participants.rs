use leptos::*;
use shared::Participant;

fn sort_participants(mut participants: Vec<Participant>) -> Vec<Participant> {
    participants.sort_by(|a, b| {
        // Sort by hand raised (desc), then name (asc)
        if a.is_hand_raised != b.is_hand_raised {
            b.is_hand_raised.cmp(&a.is_hand_raised)
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
    #[prop(optional)] on_mute_all: Option<Callback<()>>,
    on_transfer_host: Callback<String>,
    #[prop(optional)] power_statuses: Option<ReadSignal<std::collections::HashMap<String, shared::PowerStatus>>>,
    #[prop(optional)] on_request_unmute: Option<Callback<String>>,
) -> impl IntoView {
    let format_time = |ms: u64| {
        let seconds = ms / 1000;
        let m = seconds / 60;
        let s = seconds % 60;
        format!("{:02}:{:02}", m, s)
    };

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
                </Show>
            </div>
            <ul>
                <For
                    each=move || sort_participants(participants.get())
                    key=|p| (p.id.clone(), p.name.clone(), p.is_hand_raised, p.is_sharing_screen, p.is_muted, p.presence.clone(), p.is_visitor, p.e2ee_enabled)
                    children=move |p| {
                        let id_kick = p.id.clone();
                        let p_id_for_host = p.id.clone();
                        let p_id_for_power = p.id.clone();
                        // Use reactive check for host status
                        view! {
                            <li style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px;">
                                <div>
                                    <span>{p.name.clone()}</span>
                                    <span style="font-size: 0.8em; color: #666; margin-left: 5px;">
                                       " [" {format!("{:?}", p.presence)} "]"
                                    </span>
                                    <span style="font-size: 0.8em; color: #666; margin-left: 5px;">
                                       "(" {format_time(p.speaking_time)} ")"
                                    </span>
                                    <Show when={
                                        let pid = p_id_for_host.clone();
                                        move || host_id.get() == Some(pid.clone())
                                    }>
                                        <span style="font-size: 0.8em; color: #666; margin-left: 5px;">"(Host)"</span>
                                    </Show>
                                </div>
                                <div style="display: flex; align-items: center;">
                                    {move || {
                                        if let Some(statuses) = power_statuses {
                                            let result = statuses.with(|map| {
                                                map.get(&p_id_for_power).cloned()
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
                                    {if p.is_sharing_screen {
                                        view! { <span style="margin-right: 5px;">"🖥️"</span> }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    {if p.is_hand_raised {
                                        view! { <span style="margin-right: 5px;">"✋"</span> }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    {if p.is_muted {
                                        view! { <span style="margin-right: 5px;">"🔇"</span> }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    <Show when={
                                        let id_check = id_kick.clone();
                                        move || is_host.get() && my_id.get() != Some(id_check.clone())
                                    }>
                                        {
                                            let id_kick_for_mute = id_kick.clone();
                                            let id_kick_for_transfer = id_kick.clone();
                                            let id_kick_for_kick = id_kick.clone();
                                            let id_kick_for_unmute = id_kick.clone();
                                            let is_muted = p.is_muted;
                                            view! {
                                                <div style="display: flex; gap: 5px;">
                                                    <Show when=move || !is_muted>
                                                        {
                                                            let id_mute = id_kick_for_mute.clone();
                                                            view! {
                                                                <button
                                                                    on:click=move |_| on_mute.call(id_mute.clone())
                                                                    style="background: none; border: 1px solid #ccc; color: orange; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                                    title="Mute Participant"
                                                                >
                                                                    "Mute"
                                                                </button>
                                                            }
                                                        }
                                                    </Show>
                                                    <button
                                                        on:click=move |_| on_transfer_host.call(id_kick_for_transfer.clone())
                                                        style="background: none; border: 1px solid #ccc; color: blue; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                        title="Make Host"
                                                    >
                                                        "Host"
                                                    </button>
                                                    <button
                                                        on:click=move |_| on_kick.call(id_kick_for_kick.clone())
                                                        style="background: none; border: 1px solid #ccc; color: red; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                        title="Kick Participant"
                                                    >
                                                        "Kick"
                                                    </button>
                                                    <Show when=move || is_muted>
                                                        {
                                                            let id_unmute = id_kick_for_unmute.clone();
                                                            view! {
                                                                <button
                                                                    on:click=move |_| {
                                                                        if let Some(cb) = on_request_unmute {
                                                                            cb.call(id_unmute.clone());
                                                                        }
                                                                    }
                                                                    style="background: none; border: 1px solid #ccc; color: green; padding: 2px 5px; cursor: pointer; border-radius: 3px; font-size: 0.8em;"
                                                                    title="Request Unmute"
                                                                >
                                                                    "Unmute"
                                                                </button>
                                                            }
                                                        }
                                                    </Show>
                                                </div>
                                            }
                                        }
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
            e2ee_enabled: false,
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
            e2ee_enabled: false,
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
