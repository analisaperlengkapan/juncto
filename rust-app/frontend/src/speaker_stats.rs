use leptos::*;
use shared::Participant;

#[component]
pub fn SpeakerStatsDialog(
    show: ReadSignal<bool>,
    participants: ReadSignal<Vec<Participant>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let sorted_participants = move || {
        let mut p = participants.get();
        p.sort_by(|a, b| b.speaking_time.cmp(&a.speaking_time));
        p
    };

    let format_time = |ms: u64| {
        let seconds = ms / 1000;
        let m = seconds / 60;
        let s = seconds % 60;
        format!("{:02}:{:02}", m, s)
    };

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>"Speaker Stats"</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>
                    <table style="width: 100%; border-collapse: collapse;">
                        <thead>
                            <tr style="border-bottom: 1px solid #ccc; text-align: left;">
                                <th style="padding: 8px;">"Name"</th>
                                <th style="padding: 8px; text-align: right;">"Time"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=sorted_participants
                                key=|p| p.id.clone()
                                children=move |p| {
                                    view! {
                                        <tr>
                                            <td style="padding: 8px;">{p.name}</td>
                                            <td style="padding: 8px; text-align: right;">{format_time(p.speaking_time)}</td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::Participant;

    #[test]
    fn test_format_time_logic() {
        let format_time = |ms: u64| {
            let seconds = ms / 1000;
            let m = seconds / 60;
            let s = seconds % 60;
            format!("{:02}:{:02}", m, s)
        };

        assert_eq!(format_time(0), "00:00");
        assert_eq!(format_time(1000), "00:01");
        assert_eq!(format_time(61000), "01:01");
        assert_eq!(format_time(3600000), "60:00");
    }
}
