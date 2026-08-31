use leptos::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Quality {
    Good,
    Fair,
    Poor,
    Unknown,
}

impl Quality {
    fn from_rtt(rtt: u64) -> Self {
        match rtt {
            0 => Quality::Unknown,
            1..=150 => Quality::Good,
            151..=400 => Quality::Fair,
            _ => Quality::Poor,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Quality::Good => "good",
            Quality::Fair => "fair",
            Quality::Poor => "poor",
            Quality::Unknown => "—",
        }
    }
    fn css_class(self) -> &'static str {
        match self {
            Quality::Good => "quality-good",
            Quality::Fair => "quality-fair",
            Quality::Poor => "quality-poor",
            Quality::Unknown => "quality-unknown",
        }
    }
}

/// Small badge visualising connection quality derived from recent RTT.
#[component]
pub fn ConnectionIndicator(rtt: ReadSignal<u64>) -> impl IntoView {
    let quality = Signal::derive(move || Quality::from_rtt(rtt.get()));
    view! {
        <span
            class=move || format!("connection-indicator {}", quality.get().css_class())
            title=move || format!("Connection quality: {}", quality.get().label())
        >
            <span class="connection-dot"></span>
            {move || quality.get().label()}
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_thresholds() {
        assert_eq!(Quality::from_rtt(0), Quality::Unknown);
        assert_eq!(Quality::from_rtt(80), Quality::Good);
        assert_eq!(Quality::from_rtt(200), Quality::Fair);
        assert_eq!(Quality::from_rtt(600), Quality::Poor);
    }
}
