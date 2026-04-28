use leptos::*;
use crate::components_ui::audio_level_indicator::AudioLevelIndicator;
use shared::Participant;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsCast;
use web_sys::MediaStream;

#[derive(Clone, PartialEq)]
enum GridItem {
    User(Participant),
    RemoteScreen(Participant),
    SharedVideo(String), // URL
}

impl GridItem {
    // Helper for key generation for DOM elements.
    // It should strictly represent identity, not mutable properties,
    // to prevent component teardown and flickering on state changes.
    fn unique_key(&self) -> String {
        match self {
            GridItem::User(p) => p.id.clone(),
            GridItem::RemoteScreen(p) => format!("{}_screen", p.id),
            GridItem::SharedVideo(url) => format!("shared_video_{}", url),
        }
    }

    fn is_screen(&self) -> bool {
        matches!(self, GridItem::RemoteScreen(_))
    }

    #[allow(dead_code)]
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
    my_audio_level: Signal<f64>,
    my_id: ReadSignal<Option<String>>,
    shared_video_url: ReadSignal<Option<String>>,
    speaking_peers: ReadSignal<HashSet<String>>,
    dominant_speaker: ReadSignal<Option<String>>,
    remote_streams: ReadSignal<HashMap<String, Vec<MediaStream>>>,
    layout: ReadSignal<String>,
    on_set_layout: Callback<String>,
    is_host: Signal<bool>,
) -> impl IntoView {
    let video_ref = create_node_ref::<html::Video>();
    let screen_ref = create_node_ref::<html::Video>();

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

        let is_spotlight = layout.get() == "spotlight";
        let dominant = dominant_speaker.get();
        let my_id_val = my_id.get();

        let list = participants.get();

        if is_spotlight {
            // Find dominant speaker or first participant
            let spotlight_id = dominant.or_else(|| {
                list.iter()
                    .find(|p| Some(p.id.clone()) != my_id_val)
                    .map(|p| p.id.clone())
            });

            // Push the spotlighted participant first (rendered as the main tile),
            // then push the remaining remote participants so they appear as
            // thumbnails. Without this, switching to spotlight would hide every
            // other participant, which is confusing for users.
            if let Some(sid) = &spotlight_id {
                if let Some(p) = list.iter().find(|p| &p.id == sid) {
                    if Some(p.id.clone()) != my_id_val {
                        items.push(GridItem::User(p.clone()));
                        if p.is_sharing_screen {
                            items.push(GridItem::RemoteScreen(p.clone()));
                        }
                    }
                }
            }

            for p in &list {
                if my_id_val.as_ref() == Some(&p.id) {
                    continue;
                }
                if spotlight_id.as_ref() == Some(&p.id) {
                    continue;
                }
                items.push(GridItem::User(p.clone()));
                if p.is_sharing_screen {
                    items.push(GridItem::RemoteScreen(p.clone()));
                }
            }
        } else {
            for p in list {
                if my_id_val != Some(p.id.clone()) {
                    items.push(GridItem::User(p.clone()));
                    if p.is_sharing_screen {
                        items.push(GridItem::RemoteScreen(p.clone()));
                    }
                }
            }
        }
        items
    });

    view! {
        <div class="video-grid-container" style="display: flex; flex-direction: column; width: 100%; height: 100%; position: relative;">
            <div class="layout-controls" style="position: absolute; top: 10px; right: 10px; z-index: 100;">
                <button
                    on:click=move |_| on_set_layout.call(if layout.get() == "grid" { "spotlight".to_string() } else { "grid".to_string() })
                    style="padding: 5px 10px; background: rgba(0,0,0,0.6); color: white; border: 1px solid white; border-radius: 4px; cursor: pointer;"
                >
                    <Show when=move || is_host.get() fallback=|| "Switch View">
                        {move || if layout.get() == "grid" { "Switch to Spotlight" } else { "Switch to Grid" }}
                    </Show>
                </button>
            </div>

            <div
                class=move || format!("video-grid {}", layout.get())
                style=move || if layout.get() == "grid" {
                    "display: flex; flex-wrap: wrap; justify-content: center; gap: 10px; padding: 10px; box-sizing: border-box; overflow-y: auto; height: 100%; align-items: center; align-content: flex-start;"
                } else {
                    "display: flex; flex-direction: column; gap: 10px; padding: 10px; box-sizing: border-box; overflow-y: auto; height: 100%;"
                }
            >
            // Local Screen Share
            <Show when=move || local_screen_stream.get().is_some()>
                <div class="video-card screen-share" style=move || if layout.get() == "spotlight" {
                    "width: 100%; flex: 1; min-height: 0; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #28a745;"
                } else {
                    "flex: 1 1 300px; max-width: 100%; height: 240px; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #28a745;"
                }>
                    <video
                        node_ref=screen_ref
                        autoplay
                        playsinline
                        muted
                        style="width: 100%; height: 100%; object-fit: contain;"
                    />
                    <button
                        on:click=move |_| {
                            if let Some(video) = screen_ref.get() {
                                let js_video: &wasm_bindgen::JsValue = video.as_ref();
                                let prop = wasm_bindgen::JsValue::from_str("requestPictureInPicture");
                                if let Ok(func) = js_sys::Reflect::get(js_video, &prop) {
                                    if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                                        let promise = func.call0(js_video);
                                        let _ = promise;
                                    }
                                }
                            }
                        }
                        style="position: absolute; top: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; z-index: 10;"
                        title="Picture-in-Picture"
                    >
                        "PiP"
                    </button>
                    <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">
                        "My Screen"
                    </div>
                </div>
            </Show>

            // Local User Video
            <div class="video-card local-video" style=move || if layout.get() == "spotlight" && local_screen_stream.get().is_none() {
                 "width: 100%; flex: 1; min-height: 0; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #007bff;"
            } else {
                 "flex: 1 1 300px; max-width: 100%; height: 240px; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #007bff;"
            }>
                <Show when=move || {
                    local_stream.get()
                        .map(|s| s.get_video_tracks().length() > 0)
                        .unwrap_or(false)
                } fallback=move || view! {
                    <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; color: white;">
                        "Camera Off"
                    </div>
                }>
                    <video
                        node_ref=video_ref
                        autoplay
                        playsinline
                        muted // Mute local video to avoid feedback
                        style="width: 100%; height: 100%; object-fit: cover; transform: scaleX(-1);" // Mirror
                    />
                    <button
                        on:click=move |_| {
                            if let Some(video) = video_ref.get() {
                                let js_video: &wasm_bindgen::JsValue = video.as_ref();
                                let prop = wasm_bindgen::JsValue::from_str("requestPictureInPicture");
                                if let Ok(func) = js_sys::Reflect::get(js_video, &prop) {
                                    if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                                        let promise = func.call0(js_video);
                                        let _ = promise;
                                    }
                                }
                            }
                        }
                        style="position: absolute; top: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; z-index: 10;"
                        title="Picture-in-Picture"
                    >
                        "PiP"
                    </button>
                </Show>

                <Show when=move || speaking_peers.get().contains(&my_id.get().unwrap_or_default())>
                    <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 3px solid #28a745; box-sizing: border-box; border-radius: 8px; pointer-events: none; z-index: 5;"></div>
                </Show>
                <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">
                    "Me"
                </div>
                <div class="status-icons" style="position: absolute; top: 10px; right: 10px; display: flex; gap: 5px;">
                    <Show when=move || {
                        my_id.get().map(|id| {
                            participants.with(|ps| {
                                ps.iter().find(|p| p.id == id).map(|p| p.e2ee_enabled).unwrap_or(false)
                            })
                        }).unwrap_or(false)
                    }>
                        <span style="font-size: 20px;" title="End-to-End Encrypted">"🔒"</span>
                    </Show>
                    // The AudioMonitor reports 0.0 while muted, so guarding on
                    // a non-zero level avoids rendering invisible indicator dots
                    // when there is no signal to display. This mirrors the
                    // explicit mute guard used by the AlwaysOnTop toolbar.
                    <Show when=move || my_audio_level.get() > 0.0>
                        <AudioLevelIndicator audio_level=my_audio_level />
                    </Show>
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
                                    url[idx+1..].split('?').next().unwrap_or("").to_string()
                                } else {
                                    "".to_string()
                                }
                            } else {
                                "".to_string()
                            };

                            let embed_url = if !video_id.is_empty() {
                                format!("https://www.youtube.com/embed/{}?autoplay=1", video_id)
                            } else {
                                "".to_string()
                            };

                            view! {
                                <div class="video-card shared-video" style="flex: 1 1 100%; max-width: 800px; height: 450px; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #fd7e14;">
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
                            let id_clone = p.id.clone();
                            let id_clone_2 = id_clone.clone();

                            // Derive reactive participant properties using the `participants` signal
                            let p_id_for_props = p.id.clone();
                            let p_name = Signal::derive(move || {
                                participants.with(|ps| {
                                    ps.iter()
                                        .find(|pp| pp.id == p_id_for_props)
                                        .map(|pp| {
                                            if is_screen {
                                                format!("{}'s Screen", pp.name)
                                            } else {
                                                pp.name.clone()
                                            }
                                        })
                                        .unwrap_or_else(|| "Unknown".to_string())
                                })
                            });

                            let p_id_for_hand = p.id.clone();

                            let p_id_for_presence = p.id.clone();
                            let _is_connected = Signal::derive(move || {
                                participants.with(|ps| {
                                    ps.iter()
                                        .find(|pp| pp.id == p_id_for_presence)
                                        .map(|pp| pp.presence == shared::PresenceStatus::Connected)
                                        .unwrap_or(false)
                                })
                            });

                            let is_hand_raised = Signal::derive(move || {
                                participants.with(|ps| {
                                    ps.iter()
                                        .find(|pp| pp.id == p_id_for_hand)
                                        .map(|pp| pp.is_hand_raised)
                                        .unwrap_or(false)
                                })
                            });

                            let p_id_for_e2ee = p.id.clone();
                            let is_p_e2ee_enabled = Signal::derive(move || {
                                participants.with(|ps| {
                                    ps.iter()
                                        .find(|pp| pp.id == p_id_for_e2ee)
                                        .map(|pp| pp.e2ee_enabled)
                                        .unwrap_or(false)
                                })
                            });

                            let p_id_for_initial = p.id.clone();
                            let initial_char = Signal::derive(move || {
                                participants.with(|ps| {
                                    ps.iter()
                                        .find(|pp| pp.id == p_id_for_initial)
                                        .and_then(|pp| pp.name.chars().next())
                                        .unwrap_or('?')
                                        .to_uppercase()
                                        .to_string()
                                })
                            });

                            let is_speaking_memo = create_memo(move |_| speaking_peers.get().contains(&id_clone));
                            let (audio_level_sig, set_audio_level_sig) = create_signal(0.0f64);

                            create_effect(move |_| {
                                if is_speaking_memo.get() {
                                    let window = web_sys::window().unwrap();
                                    let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                        // Use a wider range starting near 0.0 to ensure the outer dots actually fade in and out.
                                        let random_val = js_sys::Math::random() * 0.8;
                                        set_audio_level_sig.set(random_val);
                                    }) as Box<dyn FnMut()>);
                                    let interval_id = window.set_interval_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 200).unwrap();

                                    on_cleanup(move || {
                                        let window = web_sys::window().unwrap();
                                        window.clear_interval_with_handle(interval_id);
                                        drop(cb);
                                        // Do not reset to 0.0 here, only in the else branch, to prevent flashing
                                    });
                                } else {
                                    set_audio_level_sig.set(0.0);
                                }
                            });

                            // Remote Stream Logic
                            let remote_video_ref = create_node_ref::<html::Video>();
                            let stream_signal = Signal::derive(move || {
                                // Performance: Use .with() to avoid cloning the entire HashMap of streams
                                remote_streams.with(|map| {
                                    if let Some(streams) = map.get(&id_clone_2) {
                                        if is_screen {
                                            // Bug 4: Stream disambiguation relies on displaySurface (not universally supported).
                                            // Fallback assumes second stream is screen share.

                                            // Try to find a stream with displaySurface (best effort)
                                            let screen_stream = streams.iter().find(|s| {
                                            let tracks = s.get_video_tracks();
                                            for i in 0..tracks.length() {
                                                let track_val = tracks.get(i);
                                                if let Ok(track) = track_val.dyn_into::<web_sys::MediaStreamTrack>() {
                                                    let settings = track.get_settings();
                                                    // use Reflect to check displaySurface prop safely
                                                    if let Ok(val) = js_sys::Reflect::get(&settings, &"displaySurface".into()) {
                                                        if !val.is_undefined() {
                                                            return true;
                                                        }
                                                    }
                                                }
                                            }
                                            false
                                        });

                                        if let Some(s) = screen_stream {
                                            return Some(s.clone());
                                        }

                                        // Fallback: use second stream if available (assuming order: camera, screen)
                                        if streams.len() > 1 {
                                            Some(streams[1].clone())
                                        } else {
                                            streams.first().cloned()
                                        }
                                    } else {
                                        // User card: use the first stream (or one that isn't screen)
                                         let camera_stream = streams.iter().find(|s| {
                                            let tracks = s.get_video_tracks();
                                            for i in 0..tracks.length() {
                                                let track_val = tracks.get(i);
                                                if let Ok(track) = track_val.dyn_into::<web_sys::MediaStreamTrack>() {
                                                    let settings = track.get_settings();
                                                    if let Ok(val) = js_sys::Reflect::get(&settings, &"displaySurface".into()) {
                                                        if !val.is_undefined() {
                                                            return false; // This is a screen
                                                        }
                                                    }
                                                }
                                            }
                                            true // Assume camera if no displaySurface
                                        });

                                        if let Some(s) = camera_stream {
                                            Some(s.clone())
                                        } else {
                                            streams.first().cloned()
                                        }
                                        }
                                    } else {
                                        None
                                    }
                                })
                            });

                            create_effect(move |_| {
                                if let Some(video) = remote_video_ref.get() {
                                    if let Some(s) = stream_signal.get() {
                                        video.set_src_object(Some(&s));
                                        let _ = video.play();
                                    } else {
                                        video.set_src_object(None);
                                    }
                                }
                            });

                            view! {
                                <div class="video-card" style=move || format!("flex: 1 1 300px; max-width: 100%; height: 240px; background: #222; border-radius: 8px; position: relative; display: flex; align-items: center; justify-content: center; border: {} solid {}; overflow: hidden;", if is_speaking_memo.get() { "3px" } else { "1px" }, if is_speaking_memo.get() { "#28a745" } else { "#444" })>
                                    <Show when=move || stream_signal.get().is_some() fallback=move || {
                                        if is_screen {
                                            view! {
                                                <div class="screen-placeholder" style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; color: #aaa; background: #111;">
                                                    "Waiting for screen..."
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <div class="avatar" style="width: 80px; height: 80px; background: #555; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 32px; color: white;">
                                                    {initial_char.get()}
                                                </div>
                                            }.into_view()
                                        }
                                    }>
                                        <video
                                            node_ref=remote_video_ref
                                            autoplay
                                            playsinline
                                            style=move || if is_screen {
                                                "width: 100%; height: 100%; object-fit: contain;"
                                            } else {
                                                "width: 100%; height: 100%; object-fit: cover;"
                                            }
                                        />
                                        <button
                                            on:click=move |_| {
                                                if let Some(video) = remote_video_ref.get() {
                                                    let js_video: &wasm_bindgen::JsValue = video.as_ref();
                                                    let prop = wasm_bindgen::JsValue::from_str("requestPictureInPicture");
                                                    if let Ok(func) = js_sys::Reflect::get(js_video, &prop) {
                                                        if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                                                            let promise = func.call0(js_video);
                                                            let _ = promise;
                                                        }
                                                    }
                                                }
                                            }
                                            style="position: absolute; top: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; z-index: 10;"
                                            title="Picture-in-Picture"
                                        >
                                            "PiP"
                                        </button>
                                    </Show>

                                    <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">
                                        {move || p_name.get()}
                                    </div>


                                    <div class="status-icons" style="position: absolute; top: 10px; right: 10px; display: flex; gap: 5px;">
                                        <Show when=move || is_p_e2ee_enabled.get() && !is_screen>
                                            <span style="font-size: 20px;" title="End-to-End Encrypted">"🔒"</span>
                                        </Show>
                                        <Show when=move || is_hand_raised.get() && !is_screen>
                                            <span style="font-size: 20px;" title="Hand Raised">"✋"</span>
                                        </Show>
                                        <AudioLevelIndicator audio_level=audio_level_sig />
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
            is_muted: false,
            speaking_time: 0,
            presence: shared::PresenceStatus::Connected,
            is_visitor: false,
            e2ee_enabled: false, hand_raised_at: None,
        };

        let item_user = GridItem::User(p.clone());
        // Key format: id
        assert_eq!(item_user.unique_key(), "user1");
        assert!(!item_user.is_screen());

        let item_screen = GridItem::RemoteScreen(p.clone());
        // Key format: id_screen
        assert_eq!(item_screen.unique_key(), "user1_screen");
        assert!(item_screen.is_screen());

        let item_video = GridItem::SharedVideo("http://test".to_string());
        assert_eq!(item_video.unique_key(), "shared_video_http://test");
        assert!(item_video.is_shared_video());
    }
}
