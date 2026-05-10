use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlVideoElement};

#[component]
pub fn ScreenshotCapture() -> impl IntoView {
    let capture = move |_| {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        // Target the main video grid area
        if let Some(capture_area) = document.get_element_by_id("capture-area") {
            let canvas = document
                .create_element("canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();

            let el = capture_area.dyn_ref::<web_sys::Element>().unwrap();
            let rect = el.get_bounding_client_rect();

            canvas.set_width(rect.width() as u32);
            canvas.set_height(rect.height() as u32);

            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            // Draw background
            #[allow(deprecated)]
            ctx.set_fill_style(&"black".into());
            ctx.fill_rect(0.0, 0.0, rect.width(), rect.height());

            // Find all video elements in the capture area
            let videos = capture_area.get_elements_by_tag_name("video");
            for i in 0..videos.length() {
                if let Some(video) = videos.item(i) {
                    let video_el = video.dyn_into::<HtmlVideoElement>().unwrap();
                    let v_rect = video_el.get_bounding_client_rect();

                    // Calculate relative position
                    let x = v_rect.left() - rect.left();
                    let y = v_rect.top() - rect.top();

                    let _ = ctx.draw_image_with_html_video_element_and_dw_and_dh(
                        &video_el,
                        x,
                        y,
                        v_rect.width(),
                        v_rect.height(),
                    );
                }
            }

            // Download the result
            let data_url = canvas.to_data_url().unwrap();
            let link = document
                .create_element("a")
                .unwrap()
                .dyn_into::<web_sys::HtmlAnchorElement>()
                .unwrap();
            link.set_href(&data_url);
            link.set_download("juncto-screenshot.png");
            link.click();
        }
    };

    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                capture(());
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback(
                "screenshot_trigger",
                closure.as_ref().unchecked_ref(),
            );

            let closure_for_cleanup: js_sys::Function =
                closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
            on_cleanup(move || {
                if let Some(win) = web_sys::window() {
                    let _ = win.remove_event_listener_with_callback(
                        "screenshot_trigger",
                        &closure_for_cleanup,
                    );
                }
            });
            closure.forget();
        }
    });

    view! {
        <div style="display: none;"></div>
    }
}
