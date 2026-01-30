use leptos::*;

#[derive(Clone, Debug)]
struct ActiveReaction {
    id: u64,
    emoji: String,
    left: f64, // Position
}

#[component]
pub fn ReactionDisplay(
    // Signal that updates when a new reaction arrives. Tuple: (sender_id, emoji)
    last_reaction: ReadSignal<Option<(String, String)>>,
) -> impl IntoView {
    let (reactions, set_reactions) = create_signal(Vec::<ActiveReaction>::new());

    create_effect(move |_| {
        if let Some((_, emoji)) = last_reaction.get() {
            // Spawn a new floating emoji
            let id = js_sys::Date::now() as u64 + (js_sys::Math::random() * 1000.0) as u64;
            let left = 10.0 + (js_sys::Math::random() * 80.0); // Random horizontal position 10-90%

            let reaction = ActiveReaction {
                id,
                emoji,
                left,
            };

            set_reactions.update(|list| list.push(reaction));

            // Remove after animation (e.g., 2 seconds)
            set_timeout(move || {
                set_reactions.update(|list| list.retain(|r| r.id != id));
            }, std::time::Duration::from_secs(2));
        }
    });

    view! {
        <div class="reaction-layer" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; overflow: hidden;">
            <For
                each=move || reactions.get()
                key=|r| r.id
                children=move |r| {
                    view! {
                        <div
                            style=format!(
                                "position: absolute; bottom: 100px; left: {}%; font-size: 40px; animation: floatUp 2s ease-out forwards;",
                                r.left
                            )
                        >
                            {r.emoji}
                        </div>
                    }
                }
            />
            // We need to inject the keyframes if not present, usually better in CSS file.
            // For now, assume a style block or inline style won't work well for keyframes without global css.
            // But we can add a style tag here.
            <style>
                "@keyframes floatUp {
                    0% { transform: translateY(0); opacity: 1; }
                    100% { transform: translateY(-200px); opacity: 0; }
                }"
            </style>
        </div>
    }
}
