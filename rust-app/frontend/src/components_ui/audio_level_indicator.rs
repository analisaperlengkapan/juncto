use leptos::*;

const AUDIO_LEVEL_DOTS: i32 = 5;
const CENTER_DOT_INDEX: i32 = AUDIO_LEVEL_DOTS / 2;

#[component]
pub fn AudioLevelIndicator(#[prop(into)] audio_level: Signal<f64>) -> impl IntoView {
    // Generate the 5 dots based on the audio level
    let dots = (0..AUDIO_LEVEL_DOTS).map(|i| {
        let distance_from_center = CENTER_DOT_INDEX - i;
        let class_name = if distance_from_center == 0 {
            "audiodot-middle"
        } else if distance_from_center < 0 {
            "audiodot-bottom"
        } else {
            "audiodot-top"
        };

        view! {
            <span
                class=class_name
                style=move || {
                    let level = (audio_level.get() * 1.2).clamp(0.0, 1.0);
                    let stretched_audio_level = (AUDIO_LEVEL_DOTS as f64) * level;
                    let audio_level_from_center = stretched_audio_level - (distance_from_center.abs() as f64);
                    let capped_opacity = audio_level_from_center.clamp(0.0, 1.0);
                    format!("opacity: {}", capped_opacity)
                }
            />
        }
    }).collect::<Vec<_>>();

    view! {
        <span class="audioindicator">
            {dots}
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_level_indicator_logic() {
        // We can test the pure math logic if we extract it, but here we just test that the constants are correct.
        assert_eq!(AUDIO_LEVEL_DOTS, 5);
        assert_eq!(CENTER_DOT_INDEX, 2);
    }

    #[test]
    fn test_audio_level_indicator_compiles() {
        let _runtime = create_runtime();
        let (audio_level, _set_audio_level) = create_signal(0.5);
        let _view = view! { <AudioLevelIndicator audio_level=audio_level /> };
    }
}
