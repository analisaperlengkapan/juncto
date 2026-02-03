use leptos::*;
use shared::Participant;
use web_sys::MediaStream;

#[derive(Clone, PartialEq)]
enum GridItem {
    User(Participant),
    RemoteScreen(Participant),
}

impl GridItem {
    // Helper for key generation to ensure reactivity when state changes
    fn unique_key(&self) -> String {
        match self {
            GridItem::User(p) => format!("{}_{}_{}", p.id, p.is_hand_raised, p.is_sharing_screen),
            GridItem::RemoteScreen(p) => format!("{}_screen_{}", p.id, p.is_sharing_screen),
        }
    }

    fn is_screen(&self) -> bool {
        matches!(self, GridItem::RemoteScreen(_))
    }

    fn participant(&self) -> &Participant {
        match self {
            GridItem::User(p) => p,
            GridItem::RemoteScreen(p) => p,
        }
    }
}

#[component]
pub fn VideoGrid(
    participants: ReadSignal<Vec<Participant>>,
    local_stream: ReadSignal<Option<MediaStream>>,
    local_screen_stream: ReadSignal<Option<MediaStream>>,
    my_id: ReadSignal<Option<String>>,
) -> impl IntoView {
    let video_ref = create_node_ref::<html::Video>();
    let screen_ref = create_node_ref::<html::Video>();
    let (layout, set_layout) = create_signal("grid"); // "grid" or "spotlight"

    create_effect(move |_| {
        if let Some(stream) = local_stream.get() {
            if let Some(video_el) = video_ref.get() {
                video_el.set_src_object(Some(&stream));
                let _ = video_el.play();
            }
        }
    });

    create_effect(move |_| {
        if let Some(stream) = local_screen_stream.get() {
            if let Some(video_el) = screen_ref.get() {
                video_el.set_src_object(Some(&stream));
                let _ = video_el.play();
            }
        }
    });

    // Prepare grid items: remote users + remote screens
    let grid_items = create_memo(move |_| {
        let mut items = Vec::new();
        let my_id_val = my_id.get();
        for p in participants.get() {
            if my_id_val != Some(p.id.clone()) {
                items.push(GridItem::User(p.clone()));
                if p.is_sharing_screen {
                    items.push(GridItem::RemoteScreen(p.clone()));
                }
            }
        }
        items
    });

    view! {
        <div class="video-grid-container" style="display: flex; flex-direction: column; width: 100%; height: 100%; position: relative;">
            <div class="layout-controls" style="position: absolute; top: 10px; right: 10px; z-index: 100;">
                <button
                    on:click=move |_| set_layout.update(|l| *l = if *l == "grid" { "spotlight" } else { "grid" })
                    style="padding: 5px 10px; background: rgba(0,0,0,0.6); color: white; border: 1px solid white; border-radius: 4px; cursor: pointer;"
                >
                    {move || if layout.get() == "grid" { "Switch to Spotlight" } else { "Switch to Grid" }}
                </button>
            </div>

            <div
                class=move || format!("video-grid {}", layout.get())
                style=move || if layout.get() == "grid" {
                    "display: flex; flex-wrap: wrap; justify-content: center; gap: 10px; padding: 20px; box-sizing: border-box; overflow-y: auto; height: 100%; align-items: center;"
                } else {
                    "display: flex; flex-direction: column; gap: 10px; padding: 20px; box-sizing: border-box; overflow-y: auto; height: 100%;"
                }
            >
            // Local Screen Share
            <Show when=move || local_screen_stream.get().is_some()>
                <div class="video-card screen-share" style=move || if layout.get() == "spotlight" {
                    "width: 100%; flex: 1; min-height: 0; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #28a745;"
                } else {
                    "width: 320px; height: 240px; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #28a745;"
                }>
                    <video
                        _ref=screen_ref
                        autoplay
                        playsinline
                        muted
                        style="width: 100%; height: 100%; object-fit: contain;"
                    />
                    <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">
                        "My Screen"
                    </div>
                </div>
            </Show>

            // Local User Video
            <div class="video-card local-video" style=move || if layout.get() == "spotlight" && local_screen_stream.get().is_none() {
                 // If spotlight and no screen share, make local video big (or first remote)
                 // For simplicity, just making local big if it's the only priority content
                 "width: 100%; flex: 1; min-height: 0; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #007bff;"
            } else {
                 "width: 320px; height: 240px; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #007bff;"
            }>
                <Show when=move || local_stream.get().is_some() fallback=move || view! {
                    <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; color: white;">
                        "Camera Off"
                    </div>
                }>
                    <video
                        _ref=video_ref
                        autoplay
                        playsinline
                        muted // Mute local video to avoid feedback
                        style="width: 100%; height: 100%; object-fit: cover; transform: scaleX(-1);" // Mirror
                    />
                </Show>
                <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">
                    "Me"
                </div>
            </div>

            // Remote Items
            <For
                each=move || grid_items.get()
                key=|item| item.unique_key()
                children=move |item| {
                    let p = item.participant().clone();
                    let is_screen = item.is_screen();
                    let p_name = if is_screen { format!("{}'s Screen", p.name) } else { p.name.clone() };
                    let initial_char = p.name.chars().next().unwrap_or('?').to_uppercase().to_string();
                    let is_hand_raised = p.is_hand_raised;

                    view! {
                        <div class="video-card" style="width: 320px; height: 240px; background: #222; border-radius: 8px; position: relative; display: flex; align-items: center; justify-content: center; border: 1px solid #444;">
                            <Show when=move || is_screen fallback=move || view!{
                                <div class="avatar" style="width: 80px; height: 80px; background: #555; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 32px; color: white;">
                                    {initial_char.clone()}
                                </div>
                            }>
                                <div class="screen-placeholder" style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; color: #aaa; background: #111;">
                                    "Remote Screen"
                                </div>
                            </Show>

                            <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">
                                {p_name}
                            </div>

                            <div class="status-icons" style="position: absolute; top: 10px; right: 10px; display: flex; gap: 5px;">
                                <Show when=move || is_hand_raised && !is_screen>
                                    <span style="font-size: 20px;" title="Hand Raised">"✋"</span>
                                </Show>
                            </div>
                        </div>
                    }
                }
            />
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::Participant;

    #[test]
    fn test_grid_item_key() {
        let p = Participant {
            id: "user1".to_string(),
            name: "Alice".to_string(),
            is_hand_raised: false,
            is_sharing_screen: false,
        };

        let item_user = GridItem::User(p.clone());
        // Key format: id_hand_screen
        assert_eq!(item_user.unique_key(), "user1_false_false");
        assert!(!item_user.is_screen());

        let item_screen = GridItem::RemoteScreen(p.clone());
        // Key format: id_screen_screen
        assert_eq!(item_screen.unique_key(), "user1_screen_false");
        assert!(item_screen.is_screen());
    }
}
