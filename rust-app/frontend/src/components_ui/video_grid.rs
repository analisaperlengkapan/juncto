use leptos::*;
use shared::Participant;
use web_sys::MediaStream;
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
enum GridItem {
    User(Participant),
    RemoteScreen(Participant),
    SharedVideo(String), // URL
}

impl GridItem {
    // Helper for key generation to ensure reactivity when state changes
    fn unique_key(&self) -> String {
        match self {
            GridItem::User(p) => format!("{}_{}_{}", p.id, p.is_hand_raised, p.is_sharing_screen),
            GridItem::RemoteScreen(p) => format!("{}_screen_{}", p.id, p.is_sharing_screen),
            GridItem::SharedVideo(url) => format!("shared_video_{}", url),
        }
    }

    fn is_screen(&self) -> bool {
        matches!(self, GridItem::RemoteScreen(_))
    }

    fn is_shared_video(&self) -> bool {
        matches!(self, GridItem::SharedVideo(_))
    }

    fn participant(&self) -> Option<&Participant> {
        match self {
            GridItem::User(p) => Some(p),
            GridItem::RemoteScreen(p) => Some(p),
            GridItem::SharedVideo(_) => None,
        }
    }
}

#[component]
pub fn VideoGrid(
    participants: ReadSignal<Vec<Participant>>,
    local_stream: ReadSignal<Option<MediaStream>>,
    local_screen_stream: ReadSignal<Option<MediaStream>>,
    my_id: ReadSignal<Option<String>>,
    shared_video_url: ReadSignal<Option<String>>,
    speaking_peers: ReadSignal<HashSet<String>>,
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

    // Prepare grid items: remote users + remote screens + shared video
    let grid_items = create_memo(move |_| {
        let mut items = Vec::new();
        if let Some(url) = shared_video_url.get() {
            items.push(GridItem::SharedVideo(url));
        }
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
                    match item {
                        GridItem::SharedVideo(url) => {
                            // Extract video ID if YouTube
                            let video_id = if url.contains("youtube.com") || url.contains("youtu.be") {
                                // Basic extraction
                                if let Some(idx) = url.find("v=") {
                                    url[idx+2..].split('&').next().unwrap_or("").to_string()
                                } else if let Some(idx) = url.rfind('/') {
                                    url[idx+1..].to_string()
                                } else {
                                    "".to_string()
                                }
                            } else {
                                "".to_string()
                            };

                            let embed_url = format!("https://www.youtube.com/embed/{}?autoplay=1", video_id);

                            view! {
                                <div class="video-card shared-video" style="width: 640px; height: 360px; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #fd7e14;">
                                    <iframe
                                        width="100%"
                                        height="100%"
                                        src=embed_url
                                        frameborder="0"
                                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                                        allowfullscreen
                                    ></iframe>
                                    <div class="name-tag" style="position: absolute; top: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">
                                        "Shared Video"
                                    </div>
                                </div>
                            }
                        },
                        _ => {
                            let p = item.participant().unwrap().clone();
                            let is_screen = item.is_screen();
                            let p_name = if is_screen { format!("{}'s Screen", p.name) } else { p.name.clone() };
                            let initial_char = p.name.chars().next().unwrap_or('?').to_uppercase().to_string();
                            let is_hand_raised = p.is_hand_raised;
                            let id_clone = p.id.clone();
                            let is_speaking = move || speaking_peers.get().contains(&id_clone);

                            view! {
                                <div class="video-card" style=move || format!("width: 320px; height: 240px; background: #222; border-radius: 8px; position: relative; display: flex; align-items: center; justify-content: center; border: {} solid {};", if is_speaking() { "3px" } else { "1px" }, if is_speaking() { "#28a745" } else { "#444" })>
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

        let item_video = GridItem::SharedVideo("http://test".to_string());
        assert_eq!(item_video.unique_key(), "shared_video_http://test");
        assert!(item_video.is_shared_video());
    }
}
