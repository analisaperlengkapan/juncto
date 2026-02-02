use leptos::*;
use leptos::html::Canvas;
use shared::DrawAction;
use wasm_bindgen::JsCast;

#[component]
pub fn Whiteboard(
    on_draw: Callback<DrawAction>,
    // Signal for incoming actions to draw
    incoming_action: ReadSignal<Option<DrawAction>>,
    history: ReadSignal<Vec<DrawAction>>,
) -> impl IntoView {
    let canvas_ref = create_node_ref::<Canvas>();
    let (is_drawing, set_is_drawing) = create_signal(false);
    let (last_pos, set_last_pos) = create_signal(None::<(f64, f64)>);

    // Draw an action on the canvas
    let draw_on_canvas = move |action: &DrawAction| {
        if let Some(canvas) = canvas_ref.get() {
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            ctx.begin_path();
            ctx.set_line_width(action.width);
            #[allow(deprecated)]
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(&action.color));
            ctx.move_to(action.start_x, action.start_y);
            ctx.line_to(action.end_x, action.end_y);
            ctx.stroke();
        }
    };

    // React to incoming actions
    create_effect(move |_| {
        if let Some(action) = incoming_action.get() {
            draw_on_canvas(&action);
        }
    });

    // React to history (initial load)
    create_effect(move |_| {
        let actions = history.get();
        for action in actions {
            draw_on_canvas(&action);
        }
    });

    let on_mousedown = move |ev: web_sys::MouseEvent| {
        set_is_drawing.set(true);
        set_last_pos.set(Some((ev.offset_x() as f64, ev.offset_y() as f64)));
    };

    let on_mouseup = move |_| {
        set_is_drawing.set(false);
        set_last_pos.set(None);
    };

    let on_mousemove = move |ev: web_sys::MouseEvent| {
        if is_drawing.get() {
            if let Some((start_x, start_y)) = last_pos.get() {
                let end_x = ev.offset_x() as f64;
                let end_y = ev.offset_y() as f64;

                let action = DrawAction {
                    color: "#000000".to_string(),
                    width: 2.0,
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                };

                // Draw locally immediately
                draw_on_canvas(&action);

                // Send to server
                on_draw.call(action);

                set_last_pos.set(Some((end_x, end_y)));
            }
        }
    };

    view! {
        <div class="whiteboard-container" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(255, 255, 255, 0.9);">
            <canvas
                _ref=canvas_ref
                width="640" // Matches video placeholder size for simplicity
                height="360"
                style="border: 1px solid black; cursor: crosshair;"
                on:mousedown=on_mousedown
                on:mouseup=on_mouseup
                on:mouseleave=on_mouseup
                on:mousemove=on_mousemove
            />
        </div>
    }
}
