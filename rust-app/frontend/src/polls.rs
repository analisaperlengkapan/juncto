use leptos::*;
use shared::{Poll, PollOption};

#[component]
pub fn PollsDialog(
    show: ReadSignal<bool>,
    polls: ReadSignal<Vec<Poll>>,
    on_close: Callback<()>,
    on_create_poll: Callback<Poll>,
    on_vote: Callback<(String, u32)>, // poll_id, option_id
) -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("list");

    // Create Poll State
    let (question, set_question) = create_signal("".to_string());
    let (option1, set_option1) = create_signal("".to_string());
    let (option2, set_option2) = create_signal("".to_string());

    let create = move |_| {
        let q = question.get();
        let o1 = option1.get();
        let o2 = option2.get();

        if !q.is_empty() && !o1.is_empty() && !o2.is_empty() {
            let poll = Poll {
                id: "".to_string(), // Backend assigns ID
                question: q,
                options: vec![
                    PollOption { id: 0, text: o1, votes: 0 },
                    PollOption { id: 1, text: o2, votes: 0 },
                ],
                voters: std::collections::HashSet::new(),
            };
            on_create_poll.call(poll);
            // Reset and switch to list
            set_question.set("".to_string());
            set_option1.set("".to_string());
            set_option2.set("".to_string());
            set_active_tab.set("list");
        }
    };

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>"Polls"</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div class="tabs" style="display: flex; border-bottom: 1px solid #ccc; margin-bottom: 20px;">
                        <button
                            on:click=move |_| set_active_tab.set("list")
                            style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "list" { "#007bff" } else { "transparent" })
                        >
                            "Active Polls"
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("create")
                            style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "create" { "#007bff" } else { "transparent" })
                        >
                            "Create Poll"
                        </button>
                    </div>

                    <div class="tab-content">
                        <Show when=move || active_tab.get() == "list">
                            <div class="polls-list" style="max-height: 300px; overflow-y: auto;">
                                <For
                                    each=move || polls.get()
                                    key=|p| (p.id.clone(), p.options.iter().map(|o| o.votes).sum::<u32>())
                                    children=move |p| {
                                        let pid = p.id.clone();
                                        view! {
                                            <div class="poll-item" style="border: 1px solid #eee; padding: 10px; margin-bottom: 10px; border-radius: 4px;">
                                                <h4>{p.question}</h4>
                                                <ul style="list-style: none; padding: 0;">
                                                    <For
                                                        each=move || {
                                                            let opts = p.options.clone();
                                                            let total_votes: u32 = opts.iter().map(|o| o.votes).sum();
                                                            opts.into_iter().map(move |o| (o, total_votes)).collect::<Vec<_>>()
                                                        }
                                                        key=|tuple| tuple.0.id
                                                        children=move |(opt, total_votes)| {
                                                            let pid_clone = pid.clone();
                                                            let percent = if total_votes > 0 {
                                                                (opt.votes as f64 / total_votes as f64) * 100.0
                                                            } else {
                                                                0.0
                                                            };

                                                            view! {
                                                                <li style="margin-bottom: 10px; position: relative;">
                                                                    <div style="display: flex; justify-content: space-between; align-items: center; position: relative; z-index: 2;">
                                                                        <span>{opt.text}</span>
                                                                        <div>
                                                                            <span style="margin-right: 10px; font-weight: bold;">{opt.votes} " votes (" {format!("{:.0}", percent)} "%)"</span>
                                                                            <button
                                                                                on:click=move |_| on_vote.call((pid_clone.clone(), opt.id))
                                                                                style="padding: 4px 8px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;"
                                                                            >
                                                                                "Vote"
                                                                            </button>
                                                                        </div>
                                                                    </div>
                                                                    <div
                                                                        class="poll-bar"
                                                                        style=format!("position: absolute; top: 0; left: 0; height: 100%; width: {}%; background-color: rgba(0, 123, 255, 0.2); border-radius: 4px; z-index: 1; transition: width 0.3s ease;", percent)
                                                                    ></div>
                                                                </li>
                                                            }
                                                        }
                                                    />
                                                </ul>
                                            </div>
                                        }
                                    }
                                />
                                <Show when=move || polls.get().is_empty()>
                                    <p style="text-align: center; color: #666;">"No active polls."</p>
                                </Show>
                            </div>
                        </Show>
                        <Show when=move || active_tab.get() == "create">
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">"Question"</label>
                                <input
                                    type="text"
                                    prop:value=question
                                    on:input=move |ev| set_question.set(event_target_value(&ev))
                                    style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                                />
                            </div>
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">"Option 1"</label>
                                <input
                                    type="text"
                                    prop:value=option1
                                    on:input=move |ev| set_option1.set(event_target_value(&ev))
                                    style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                                />
                            </div>
                            <div class="form-group" style="margin-bottom: 15px;">
                                <label style="display: block; margin-bottom: 5px;">"Option 2"</label>
                                <input
                                    type="text"
                                    prop:value=option2
                                    on:input=move |ev| set_option2.set(event_target_value(&ev))
                                    style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                                />
                            </div>
                            <button
                                on:click=create
                                style="padding: 10px 20px; background-color: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer; width: 100%;"
                            >
                                "Create Poll"
                            </button>
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}
