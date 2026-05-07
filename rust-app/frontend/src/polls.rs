use leptos::*;
use shared::{Poll, PollOption};

#[component]
pub fn PollsDialog(
    show: ReadSignal<bool>,
    polls: ReadSignal<Vec<Poll>>,
    is_host: Signal<bool>,
    on_close: Callback<()>,
    on_create_poll: Callback<Poll>,
    on_vote: Callback<(String, u32)>, // poll_id, option_id
    on_close_poll: Callback<String>,
) -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("active");

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
                    PollOption {
                        id: 0,
                        text: o1,
                        votes: 0,
                    },
                    PollOption {
                        id: 1,
                        text: o2,
                        votes: 0,
                    },
                ],
                voters: std::collections::HashSet::new(),
                is_closed: false,
            };
            on_create_poll.call(poll);
            // Reset and switch to active
            set_question.set("".to_string());
            set_option1.set("".to_string());
            set_option2.set("".to_string());
            set_active_tab.set("active");
        }
    };

    let active_polls = create_memo(move |_| {
        polls.get().into_iter().filter(|p| !p.is_closed).collect::<Vec<_>>()
    });

    let history_polls = create_memo(move |_| {
        polls.get().into_iter().filter(|p| p.is_closed).collect::<Vec<_>>()
    });

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 450px; max-width: 95%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>"Polls"</h3>
                        <button id="close-polls-btn" on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div class="tabs" style="display: flex; border-bottom: 1px solid #ccc; margin-bottom: 20px;">
                        <button
                            on:click=move |_| set_active_tab.set("active")
                            style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "active" { "#007bff" } else { "transparent" })
                        >
                            "Active Polls"
                        </button>
                        <button
                            on:click=move |_| set_active_tab.set("history")
                            style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "history" { "#007bff" } else { "transparent" })
                        >
                            "History"
                        </button>
                        <Show when=move || is_host.get()>
                            <button
                                on:click=move |_| set_active_tab.set("create")
                                style=move || format!("padding: 10px; border: none; background: none; cursor: pointer; border-bottom: 2px solid {}", if active_tab.get() == "create" { "#007bff" } else { "transparent" })
                            >
                                "Create Poll"
                            </button>
                        </Show>
                    </div>

                    <div class="tab-content" style="max-height: 400px; overflow-y: auto;">
                        <Show when=move || active_tab.get() == "active">
                            <div class="polls-list">
                                <For
                                    each=move || active_polls.get()
                                    key=|p| (p.id.clone(), p.options.iter().map(|o| o.votes).sum::<u32>())
                                    children=move |p| {
                                        let pid = p.id.clone();
                                        let pid_for_votes = pid.clone();
                                        view! {
                                            <div class="poll-item" style="border: 1px solid #eee; padding: 10px; margin-bottom: 10px; border-radius: 4px;">
                                                <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                                                    <h4 style="margin-top: 0;">{p.question.clone()}</h4>
                                                    <Show when=move || is_host.get()>
                                                        <button
                                                            on:click={
                                                                let pid_inner = pid.clone();
                                                                move |_| on_close_poll.call(pid_inner.clone())
                                                            }
                                                            style="padding: 2px 6px; background-color: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                        >
                                                            "Close Poll"
                                                        </button>
                                                    </Show>
                                                </div>
                                                <ul style="list-style: none; padding: 0;">
                                                    <For
                                                        each={
                                                            let opts = p.options.clone();
                                                            let total_votes: u32 = opts.iter().map(|o| o.votes).sum();
                                                            move || opts.clone().into_iter().map(move |o| (o, total_votes)).collect::<Vec<_>>()
                                                        }
                                                        key=|tuple| tuple.0.id
                                                        children={
                                                            let pid_inner_cap = pid_for_votes.clone();
                                                            move |(opt, total_votes)| {
                                                                let pid_inner2 = pid_inner_cap.clone();
                                                                let percent = if total_votes > 0 {
                                                                    (opt.votes as f64 / total_votes as f64) * 100.0
                                                                } else {
                                                                    0.0
                                                                };

                                                                view! {
                                                                    <li style="margin-bottom: 10px; position: relative; padding: 8px; border-radius: 4px; background: #f8f9fa;">
                                                                        <div style="display: flex; justify-content: space-between; align-items: center; position: relative; z-index: 2;">
                                                                            <span>{opt.text}</span>
                                                                            <div style="display: flex; align-items: center;">
                                                                                <span style="margin-right: 10px; font-weight: bold; font-size: 12px;">
                                                                                    <span>{opt.votes} " votes"</span>
                                                                                    " (" {format!("{:.0}", percent)} "%)"
                                                                                </span>
                                                                                <button
                                                                                    on:click={
                                                                                        let pid_inner3 = pid_inner2.clone();
                                                                                        move |_| on_vote.call((pid_inner3.clone(), opt.id))
                                                                                    }
                                                                                    style="padding: 4px 8px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                                                                >
                                                                                    "Vote"
                                                                                </button>
                                                                            </div>
                                                                        </div>
                                                                        <div
                                                                            class="poll-bar"
                                                                            style=format!("position: absolute; top: 0; left: 0; height: 100%; width: {}%; background-color: rgba(0, 123, 255, 0.15); border-radius: 4px; z-index: 1; transition: width 0.3s ease;", percent)
                                                                        ></div>
                                                                    </li>
                                                                }
                                                            }
                                                        }
                                                    />
                                                </ul>
                                            </div>
                                        }
                                    }
                                />
                                <Show when=move || active_polls.get().is_empty()>
                                    <p style="text-align: center; color: #666;">"No active polls."</p>
                                </Show>
                            </div>
                        </Show>

                        <Show when=move || active_tab.get() == "history">
                            <div class="polls-list">
                                <For
                                    each=move || history_polls.get()
                                    key=|p| (p.id.clone(), p.options.iter().map(|o| o.votes).sum::<u32>())
                                    children=move |p| {
                                        view! {
                                            <div class="poll-item" style="border: 1px solid #eee; padding: 10px; margin-bottom: 10px; border-radius: 4px; opacity: 0.8; background: #fafafa;">
                                                <div style="display: flex; justify-content: space-between;">
                                                    <h4 style="margin-top: 0; color: #666;">{p.question.clone()}</h4>
                                                    <span style="font-size: 10px; background: #eee; padding: 2px 4px; border-radius: 3px;">"CLOSED"</span>
                                                </div>
                                                <ul style="list-style: none; padding: 0;">
                                                    <For
                                                        each={
                                                            let opts = p.options.clone();
                                                            let total_votes: u32 = opts.iter().map(|o| o.votes).sum();
                                                            move || opts.clone().into_iter().map(move |o| (o, total_votes)).collect::<Vec<_>>()
                                                        }
                                                        key=|tuple| tuple.0.id
                                                        children=move |(opt, total_votes)| {
                                                            let percent = if total_votes > 0 {
                                                                (opt.votes as f64 / total_votes as f64) * 100.0
                                                            } else {
                                                                0.0
                                                            };
                                                            view! {
                                                                <li style="margin-bottom: 5px; position: relative; padding: 6px; border-radius: 4px;">
                                                                    <div style="display: flex; justify-content: space-between; align-items: center; position: relative; z-index: 2; font-size: 13px;">
                                                                        <span>{opt.text}</span>
                                                                        <span style="font-weight: bold;">
                                                                            <span>{opt.votes} " votes"</span>
                                                                            " (" {format!("{:.0}", percent)} "%)"
                                                                        </span>
                                                                    </div>
                                                                    <div
                                                                        style=format!("position: absolute; top: 0; left: 0; height: 100%; width: {}%; background-color: rgba(0, 0, 0, 0.05); border-radius: 4px; z-index: 1;", percent)
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
                                <Show when=move || history_polls.get().is_empty()>
                                    <p style="text-align: center; color: #666;">"No poll history."</p>
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
                                    placeholder="e.g. What is your favorite color?"
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
                                style="padding: 10px 20px; background-color: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer; width: 100%; font-weight: bold;"
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

#[cfg(test)]
mod tests {
    use shared::Poll;
    use std::collections::HashSet;

    #[test]
    fn test_poll_filtering_logic() {
        let p1 = Poll {
            id: "1".to_string(),
            question: "Q1".to_string(),
            options: vec![],
            voters: HashSet::new(),
            is_closed: false,
        };
        let p2 = Poll {
            id: "2".to_string(),
            question: "Q2".to_string(),
            options: vec![],
            voters: HashSet::new(),
            is_closed: true,
        };

        let polls = vec![p1.clone(), p2.clone()];

        let active: Vec<_> = polls.iter().filter(|p| !p.is_closed).collect();
        let history: Vec<_> = polls.iter().filter(|p| p.is_closed).collect();

        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "1");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, "2");
    }
}
