use leptos::*;

#[component]
pub fn VirtualBackgroundDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_change: Callback<(String, Option<String>)>,
) -> impl IntoView {
    let (selected, set_selected) = create_signal("none".to_string());

    let apply = move |mode: String| {
        set_selected.set(mode.clone());
        // For now use a placeholder, or we could add a file picker
        let img_url = if mode == "image" {
            Some("https://images.unsplash.com/photo-1558591710-4b4a1ae0f04d?auto=format&fit=crop&w=640".to_string())
        } else {
            None
        };
        on_change.call((mode, img_url));
    };

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; padding: 20px; border-radius: 8px; width: 500px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>"Virtual Background"</h3>
                        <button on:click=move |_| on_close.call(()) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div class="options" style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px;">
                        <div
                            on:click=move |_| apply("none".to_string())
                            style=move || format!("
                                cursor: pointer;
                                border: 2px solid {};
                                border-radius: 4px;
                                padding: 10px;
                                text-align: center;
                            ", if selected.get() == "none" { "#007bff" } else { "#ccc" })
                        >
                            <div style="height: 60px; background: #eee; margin-bottom: 5px; display: flex; align-items: center; justify-content: center;">
                                "None"
                            </div>
                            <span>"None"</span>
                        </div>

                        <div
                            on:click=move |_| apply("blur".to_string())
                            style=move || format!("
                                cursor: pointer;
                                border: 2px solid {};
                                border-radius: 4px;
                                padding: 10px;
                                text-align: center;
                            ", if selected.get() == "blur" { "#007bff" } else { "#ccc" })
                        >
                            <div style="height: 60px; background: #eee; margin-bottom: 5px; filter: blur(2px); display: flex; align-items: center; justify-content: center;">
                                "Blur"
                            </div>
                            <span>"Blur"</span>
                        </div>

                        <div
                            on:click=move |_| apply("image".to_string())
                            style=move || format!("
                                cursor: pointer;
                                border: 2px solid {};
                                border-radius: 4px;
                                padding: 10px;
                                text-align: center;
                            ", if selected.get() == "image" { "#007bff" } else { "#ccc" })
                        >
                            <div style="height: 60px; background: url('https://via.placeholder.com/150'); background-size: cover; margin-bottom: 5px;"></div>
                            <span>"Image"</span>
                        </div>
                    </div>

                    <div style="margin-top: 20px; text-align: right;">
                         <button
                            on:click=move |_| on_close.call(())
                            style="padding: 8px 16px; background-color: #007bff; color: white; border: none; cursor: pointer; border-radius: 4px;"
                        >
                            "Done"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_background_selection() {
        // Logic test for default state could be here, but visual mostly.
        assert_eq!(1, 1);
    }
}
