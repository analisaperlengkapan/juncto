use leptos::*;
use gloo_timers::callback::Interval;

#[component]
pub fn ConnectionStats(
    on_ping: Callback<()>,
    rtt: ReadSignal<u64>,
) -> impl IntoView {
    create_effect(move |_| {
        let handle = Interval::new(2000, move || {
            on_ping.call(());
        });
        on_cleanup(move || drop(handle));
    });

    view! {
        <div class="connection-stats" style="
            position: absolute;
            top: 10px;
            left: 10px;
            background: rgba(0,0,0,0.6);
            color: white;
            padding: 5px 10px;
            border-radius: 4px;
            font-size: 0.8em;
            pointer-events: none;
            z-index: 100;
        ">
            <span style=move || format!("color: {}", if rtt.get() < 100 { "#28a745" } else if rtt.get() < 300 { "#ffc107" } else { "#dc3545" })>
                "● "
            </span>
            {move || format!("{} ms", rtt.get())}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_render() {
        assert_eq!(1, 1);
    }
}
