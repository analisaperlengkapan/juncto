use leptos::*;
use shared::Participant;

#[component]
pub fn ParticipantsList(
    participants: ReadSignal<Vec<Participant>>
) -> impl IntoView {
    view! {
        <div class="participants-list" style="width: 200px; background: #eee; padding: 20px; height: 100%;">
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
