import sys

def main():
    with open("rust-app/frontend/src/components_ui/video_grid.rs", "r") as f:
        content = f.read()

    # In Rust, building an entire reactive WebAudio analyser locally for a grid element is very verbose and causes scope issues.
    # The review stated: "If a placeholder is needed temporarily, revert to the previous fixed value (0.8) so behavior is at least deterministic and testable, or create a timer-based signal that periodically generates new random values to at least simulate animation."
    # Let's create a timer-based signal that periodically generates new random values if `is_speaking()` is true.
    # We can do this easily with `set_interval`.

    fix1 = """let is_speaking = move || speaking_peers.get().contains(&id_clone);
                            let audio_level_sig = Signal::derive(move || if is_speaking() { 0.8 } else { 0.0 });"""

    fix2 = """let is_speaking = move || speaking_peers.get().contains(&id_clone);
                            let (audio_level_sig, set_audio_level_sig) = create_signal(0.0f64);

                            create_effect({
                                let is_speaking_cloned = is_speaking.clone();
                                move |_| {
                                    if is_speaking_cloned() {
                                        // Set a quick interval to jitter the audio level
                                        let window = web_sys::window().unwrap();
                                        let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                                            let random_val = 0.5 + js_sys::Math::random() * 0.5;
                                            set_audio_level_sig.set(random_val);
                                        }) as Box<dyn FnMut()>);
                                        let interval_id = window.set_interval_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 200).unwrap();

                                        // Cleanup interval when effect reruns (e.g. stops speaking)
                                        on_cleanup(move || {
                                            let window = web_sys::window().unwrap();
                                            window.clear_interval_with_handle(interval_id);
                                            drop(cb); // drop closure
                                            set_audio_level_sig.set(0.0);
                                        });
                                    } else {
                                        set_audio_level_sig.set(0.0);
                                    }
                                }
                            });"""

    content = content.replace(fix1, fix2)

    with open("rust-app/frontend/src/components_ui/video_grid.rs", "w") as f:
        f.write(content)

if __name__ == "__main__":
    main()
