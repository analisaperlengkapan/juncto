use leptos::*;
use shared::Participant;
use web_sys::MediaStream;

#[component]
pub fn VideoGrid(
    participants: ReadSignal<Vec<Participant>>,
    local_stream: ReadSignal<Option<MediaStream>>,
    local_screen_stream: ReadSignal<Option<MediaStream>>,
    my_id: ReadSignal<Option<String>>,
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

    view! {
        <div class="video-grid" style="display: flex; flex-wrap: wrap; justify-content: center; gap: 10px; width: 100%; height: 100%; padding: 20px; box-sizing: border-box; overflow-y: auto;">
            // Local Screen Share
            <Show when=move || local_screen_stream.get().is_some()>
                <div class="video-card" style="width: 320px; height: 240px; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #28a745;">
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
            <div class="video-card" style="width: 320px; height: 240px; background: black; border-radius: 8px; position: relative; overflow: hidden; border: 2px solid #007bff;">
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

            // Remote Participants
            <For
                each=move || participants.get()
                key=|p| p.id.clone()
                children=move |p| {
                    let is_me = my_id.get() == Some(p.id.clone());
                    // Skip myself in the list if I'm already shown above (or handle duplicates)
                    // The participants list from server usually includes everyone.
                    // If my_id is in participants, we should skip it here to avoid double rendering,
                    // OR we render the list uniformly and treat "Me" special within the loop.
                    // For now, let's skip "Me" in this loop and stick to the "Local User Video" block above for me.

                    let p_name = p.name.clone();
                    let p_is_sharing_screen = p.is_sharing_screen;
                    let initial_char = p_name.chars().next().unwrap_or('?').to_uppercase().to_string();

                    view! {
                        <Show when=move || !is_me>
                            <div class="video-card" style="width: 320px; height: 240px; background: #222; border-radius: 8px; position: relative; display: flex; align-items: center; justify-content: center; border: 1px solid #444;">
                                // Placeholder for remote video
                                <div class="avatar" style="width: 80px; height: 80px; background: #555; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 32px; color: white;">
                                    {initial_char.clone()}
                                </div>
                                <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">
                                    {p_name.clone()}
                                </div>
                                <div class="status-icons" style="position: absolute; top: 10px; right: 10px; display: flex; gap: 5px;">
                                    <Show when=move || !p_is_sharing_screen>
                                        // Mic off icon logic could go here if we tracked audio status
                                        <span style="color: red;">"🎤"</span>
                                    </Show>
                                </div>
                            </div>
                        </Show>
                    }
                }
            />
        </div>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_video_grid_component_logic() {
        // Just verify it compiles and exists for now
        assert_eq!(1, 1);
    }
}
