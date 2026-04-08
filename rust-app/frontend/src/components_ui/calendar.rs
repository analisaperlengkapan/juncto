use leptos::*;

#[component]
pub fn CalendarList(
    #[prop(into)] events: Signal<Vec<String>>,
    on_refresh: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    // Automatically refresh on mount
    create_effect(move |_| {
        // Trigger fetch asynchronously to avoid potential sync borrow issues during mount
        set_timeout(move || {
            on_refresh.call(());
        }, std::time::Duration::from_millis(100));
    });

    view! {
        <div class="calendar-list-overlay" style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
            <div class="calendar-list-dialog" style="background: #2a2a2a; padding: 20px; border-radius: 8px; width: 400px; color: white; max-height: 80vh; display: flex; flex-direction: column;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                    <h3 style="margin: 0;">"Upcoming Meetings"</h3>
                    <button
                        on:click=move |_| on_close.call(())
                        style="background: transparent; border: none; color: white; font-size: 20px; cursor: pointer;"
                    >
                        "×"
                    </button>
                </div>

                <div style="flex: 1; overflow-y: auto; margin-bottom: 15px;">
                    {move || if events.get().is_empty() {
                        view! {
                            <div style="text-align: center; color: #999; padding: 20px;">
                                "No upcoming events"
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <ul style="list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 10px;">
                                {
                                    let events_list = events.get().into_iter().enumerate().collect::<Vec<_>>();
                                    view! {
                                        <For
                                            each=move || events_list.clone()
                                            key=|(i, _)| *i
                                            children=move |(_i, evt)| {
                                                view! {
                                                    <li style="background: #333; padding: 10px; border-radius: 4px; border-left: 4px solid #007bff;">
                                                        {evt}
                                                    </li>
                                                }
                                            }
                                        />
                                    }
                                }
                            </ul>
                        }.into_view()
                    }}
                </div>

                <div style="display: flex; justify-content: flex-end;">
                    <button
                        on:click=move |_| on_refresh.call(())
                        style="padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;"
                    >
                        "Refresh"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_list_compiles() {
        let _ = create_runtime();
        let (events, _set_events) = create_signal::<Vec<String>>(Vec::new());
        let on_refresh = Callback::new(|_: ()| {});
        let on_close = Callback::new(|_: ()| {});

        let _view = CalendarList(
            CalendarListProps {
                events: events.into(),
                on_refresh,
                on_close,
            }
        );
        assert!(true); // Verifies that instantiation succeeds within a reactive scope
    }
}
