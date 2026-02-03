use leptos::*;
use shared::Participant;

#[component]
pub fn ParticipantsList(
    participants: ReadSignal<Vec<Participant>>,
    knocking_participants: ReadSignal<Vec<Participant>>,
    on_allow: Callback<String>,
    on_deny: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="participants-list" style="width: 200px; background: #eee; padding: 20px; height: 100%;">
            <Show when=move || !knocking_participants.get().is_empty()>
                <div class="knocking-list" style="margin-bottom: 20px; padding-bottom: 20px; border-bottom: 1px solid #ccc;">
                    <h3>"Waiting Room"</h3>
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

            <h3>"Participants"</h3>
            <ul>
                <For
                    each=move || participants.get()
                    key=|p| (p.id.clone(), p.name.clone(), p.is_hand_raised, p.is_sharing_screen)
                    children=move |p| {
                        view! {
                            <li style="display: flex; justify-content: space-between;">
                                <span>{p.name}</span>
                                <div>
                                    {if p.is_sharing_screen {
                                        view! { <span style="margin-right: 5px;">"🖥️"</span> }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                    {if p.is_hand_raised {
                                        view! { <span>"✋"</span> }.into_view()
                                    } else {
                                        view! { <span></span> }.into_view()
                                    }}
                                </div>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}
