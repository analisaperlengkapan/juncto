use leptos::*;
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
    my_id: ReadSignal<Option<String>>,
    shared_video_url: ReadSignal<Option<String>>,
    speaking_peers: ReadSignal<HashSet<String>>,
    remote_streams: ReadSignal<HashMap<String, Vec<MediaStream>>>,
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
        <div class="flex flex-col w-full h-full relative bg-gray-900">
            <div class="absolute top-4 right-4 z-50">
                <button
                    on:click=move |_| set_layout.update(|l| *l = if *l == "grid" { "spotlight" } else { "grid" })
                    class="px-3 py-1.5 bg-gray-800 bg-opacity-70 hover:bg-opacity-90 text-white text-xs font-medium rounded shadow-sm border border-gray-600 transition duration-150 backdrop-blur-sm focus:outline-none"
                >
                    {move || if layout.get() == "grid" { "Spotlight View" } else { "Grid View" }}
                </button>
            </div>

            <div
                class=move || format!("w-full h-full p-4 overflow-y-auto {}",
                    if layout.get() == "grid" {
                        "grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 auto-rows-fr place-content-center place-items-center"
                    } else {
                        "flex flex-col gap-4"
                    })
            >

            // Local Screen Share
            <Show when=move || local_screen_stream.get().is_some()>
                <div class=move || format!("relative bg-black rounded-xl overflow-hidden shadow-lg border-2 border-green-500 group {}",
                    if layout.get() == "spotlight" { "w-full flex-1 min-h-0" } else { "w-full aspect-video max-h-[300px]" })
                >
                    <video
                        _ref=screen_ref
                        autoplay
                        playsinline
                        muted
                        class="w-full h-full object-contain"
                    />

                    // Controls overlay (hover)
                    <div class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-20 transition duration-200">
                        <button
                            on:click=move |_| {
                                if let Some(video) = screen_ref.get() {
                                    let js_video: &wasm_bindgen::JsValue = video.as_ref();
                                    let prop = wasm_bindgen::JsValue::from_str("requestPictureInPicture");
                                    if let Ok(func) = js_sys::Reflect::get(js_video, &prop) {
                                        if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                                            let _ = func.call0(js_video);
                                        }
                                    }
                                }
                            }
                            class="absolute top-3 left-3 bg-gray-900 bg-opacity-60 hover:bg-opacity-80 text-white text-xs px-2 py-1 rounded backdrop-blur-sm opacity-0 group-hover:opacity-100 transition duration-200 focus:outline-none"
                            title="Picture-in-Picture"
                        >
                            "PiP"
                        </button>
                    </div>

                    <div class="absolute bottom-3 left-3 bg-gray-900 bg-opacity-60 text-white text-xs font-medium px-2.5 py-1 rounded backdrop-blur-sm shadow-sm flex items-center space-x-1 border border-gray-700">
                        <svg class="w-3 h-3 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"></path></svg>
                        <span>"My Screen"</span>
                    </div>
                </div>
            </Show>

            // Local User Video
            <div class=move || format!("relative bg-gray-800 rounded-xl overflow-hidden shadow-lg border-2 border-blue-500 group {}",
                 if layout.get() == "spotlight" && local_screen_stream.get().is_none() { "w-full flex-1 min-h-0" } else { "w-full aspect-video max-h-[300px]" })
            >
                <Show when=move || {
                    local_stream.get()
                        .map(|s| s.get_video_tracks().length() > 0)
                        .unwrap_or(false)
                } fallback=move || view! {
                    <div class="w-full h-full flex flex-col items-center justify-center text-gray-400 bg-gray-800 space-y-2">
                        <svg class="w-12 h-12 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"></path></svg>
                        <span class="text-sm font-medium">"Camera Off"</span>
                    </div>
                }>
                    <video
                        _ref=video_ref
                        autoplay
                        playsinline
                        muted
                        class="w-full h-full object-cover transform scale-x-[-1]"
                    />

                    <div class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-20 transition duration-200">
                        <button
                            on:click=move |_| {
                                if let Some(video) = video_ref.get() {
                                    let js_video: &wasm_bindgen::JsValue = video.as_ref();
                                    let prop = wasm_bindgen::JsValue::from_str("requestPictureInPicture");
                                    if let Ok(func) = js_sys::Reflect::get(js_video, &prop) {
                                        if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                                            let _ = func.call0(js_video);
                                        }
                                    }
                                }
                            }
                            class="absolute top-3 left-3 bg-gray-900 bg-opacity-60 hover:bg-opacity-80 text-white text-xs px-2 py-1 rounded backdrop-blur-sm opacity-0 group-hover:opacity-100 transition duration-200 focus:outline-none"
                            title="Picture-in-Picture"
                        >
                            "PiP"
                        </button>
                    </div>
                </Show>

                <div class="absolute bottom-3 left-3 bg-gray-900 bg-opacity-60 text-white text-xs font-medium px-2.5 py-1 rounded backdrop-blur-sm shadow-sm flex items-center space-x-1 border border-gray-700">
                    <span>"Me"</span>
                </div>
            </div>

            // Remote Items
            <For
                each=move || grid_items.get()
                key=|item| item.unique_key()
                children=move |item| {
                    match item {
                        GridItem::SharedVideo(url) => {
                            let video_id = if url.contains("youtube.com") || url.contains("youtu.be") {
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
                                <div class="col-span-full relative bg-black rounded-xl overflow-hidden shadow-lg border-2 border-orange-500 w-full max-w-4xl mx-auto aspect-video">
                                    <iframe
                                        class="w-full h-full"
                                        src=embed_url
                                        frameborder="0"
                                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                                        allowfullscreen
                                    ></iframe>
                                    <div class="absolute top-3 left-3 bg-gray-900 bg-opacity-60 text-white text-xs font-medium px-2.5 py-1 rounded backdrop-blur-sm shadow-sm border border-gray-700">
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
                            let is_hand_raised = Signal::derive(move || {
                                participants.with(|ps| {
                                    ps.iter()
                                        .find(|pp| pp.id == p_id_for_hand)
                                        .map(|pp| pp.is_hand_raised)
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

                            let is_speaking = Signal::derive(move || speaking_peers.get().contains(&id_clone));

                            let remote_video_ref = create_node_ref::<html::Video>();
                            let stream_signal = Signal::derive(move || {
                                remote_streams.with(|map| {
                                    if let Some(streams) = map.get(&id_clone_2) {
                                        if is_screen {
                                            let screen_stream = streams.iter().find(|s| {
                                                let tracks = s.get_video_tracks();
                                                for i in 0..tracks.length() {
                                                    let track_val = tracks.get(i);
                                                    if let Ok(track) = track_val.dyn_into::<web_sys::MediaStreamTrack>() {
                                                        let settings = track.get_settings();
                                                        if let Ok(val) = js_sys::Reflect::get(&settings, &"displaySurface".into()) {
                                                            if !val.is_undefined() { return true; }
                                                        }
                                                    }
                                                }
                                                false
                                            });
                                            if let Some(s) = screen_stream { return Some(s.clone()); }
                                            if streams.len() > 1 { Some(streams[1].clone()) } else { streams.first().cloned() }
                                        } else {
                                            let camera_stream = streams.iter().find(|s| {
                                                let tracks = s.get_video_tracks();
                                                for i in 0..tracks.length() {
                                                    let track_val = tracks.get(i);
                                                    if let Ok(track) = track_val.dyn_into::<web_sys::MediaStreamTrack>() {
                                                        let settings = track.get_settings();
                                                        if let Ok(val) = js_sys::Reflect::get(&settings, &"displaySurface".into()) {
                                                            if !val.is_undefined() { return false; }
                                                        }
                                                    }
                                                }
                                                true
                                            });
                                            if let Some(s) = camera_stream { Some(s.clone()) } else { streams.first().cloned() }
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
                                <div class=move || format!("relative rounded-xl overflow-hidden shadow-lg group transition-all duration-200 border-2 {} {}",
                                    if is_speaking.get() { "border-green-500 bg-gray-800 ring-2 ring-green-500 ring-opacity-50" } else { "border-gray-700 bg-gray-800" },
                                    if layout.get() == "spotlight" { "w-full max-h-[300px] aspect-video" } else { "w-full aspect-video" }
                                )>
                                    <Show when=move || stream_signal.get().is_some() fallback=move || {
                                        if is_screen {
                                            view! {
                                                <div class="w-full h-full flex items-center justify-center text-gray-500 bg-gray-900">
                                                    <div class="flex flex-col items-center space-y-2">
                                                        <svg class="w-10 h-10 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path></svg>
                                                        <span class="text-sm">"Loading screen..."</span>
                                                    </div>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <div class="w-full h-full flex items-center justify-center bg-gray-800">
                                                    <div class="w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center text-3xl font-bold text-white shadow-inner">
                                                        {initial_char.get()}
                                                    </div>
                                                </div>
                                            }.into_view()
                                        }
                                    }>
                                        <video
                                            node_ref=remote_video_ref
                                            autoplay
                                            playsinline
                                            class=move || format!("w-full h-full {}", if is_screen { "object-contain bg-black" } else { "object-cover bg-gray-900" })
                                        />

                                        <div class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-20 transition duration-200">
                                            <button
                                                on:click=move |_| {
                                                    if let Some(video) = remote_video_ref.get() {
                                                        let js_video: &wasm_bindgen::JsValue = video.as_ref();
                                                        let prop = wasm_bindgen::JsValue::from_str("requestPictureInPicture");
                                                        if let Ok(func) = js_sys::Reflect::get(js_video, &prop) {
                                                            if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                                                                let _ = func.call0(js_video);
                                                            }
                                                        }
                                                    }
                                                }
                                                class="absolute top-3 left-3 bg-gray-900 bg-opacity-60 hover:bg-opacity-80 text-white text-xs px-2 py-1 rounded backdrop-blur-sm opacity-0 group-hover:opacity-100 transition duration-200 focus:outline-none"
                                                title="Picture-in-Picture"
                                            >
                                                "PiP"
                                            </button>
                                        </div>
                                    </Show>

                                    <div class="absolute bottom-3 left-3 flex items-center space-x-2">
                                        <div class="bg-gray-900 bg-opacity-60 text-white text-xs font-medium px-2.5 py-1 rounded backdrop-blur-sm shadow-sm border border-gray-700">
                                            {move || p_name.get()}
                                        </div>

                                        <Show when=move || is_speaking.get()>
                                            <div class="bg-green-500 rounded-full p-1 border-2 border-white shadow-sm animate-pulse">
                                                <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"></path></svg>
                                            </div>
                                        </Show>
                                    </div>

                                    <div class="absolute top-3 right-3 flex space-x-1">
                                        <Show when=move || is_hand_raised.get() && !is_screen>
                                            <div class="bg-yellow-400 bg-opacity-90 rounded p-1.5 shadow-sm transform transition hover:scale-110" title="Hand Raised">
                                                <svg class="w-4 h-4 text-yellow-900" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 11.5V14m0-2.5v-6a1.5 1.5 0 113 0m-3 6a1.5 1.5 0 00-3 0v2a7.5 7.5 0 0015 0v-5a1.5 1.5 0 00-3 0m-6-3V11m0-5.5v-1a1.5 1.5 0 013 0v1m0 0V11m0-5.5a1.5 1.5 0 013 0v3m0 0V11"></path></svg>
                                            </div>
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
            is_muted: false,
            speaking_time: 0,
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
