use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use shared::PowerStatus;
use gloo_timers::callback::Interval;
use std::cell::Cell;
use std::rc::Rc;

#[component]
pub fn PowerMonitor(
    on_update: Callback<PowerStatus>,
) -> impl IntoView {
    create_effect(move |_| {
        // Track last sent values so we only send updates when something changes.
        let last_level: Rc<Cell<Option<i32>>> = Rc::new(Cell::new(None));
        let last_charging: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        let last_level_clone = last_level.clone();
        let last_charging_clone = last_charging.clone();
        let update_battery = move || {
            let on_update = on_update;
            let last_level = last_level_clone.clone();
            let last_charging = last_charging_clone.clone();
            spawn_local(async move {
                if let Some(window) = web_sys::window() {
                    let navigator = window.navigator();
                    let nav_val: &JsValue = navigator.as_ref();
                    let get_battery_prop = JsValue::from_str("getBattery");

                    if let Ok(func) = js_sys::Reflect::get(nav_val, &get_battery_prop) {
                        if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                            if let Ok(promise) = func.call0(nav_val) {
                                let promise = js_sys::Promise::from(promise);
                                if let Ok(battery_val) = wasm_bindgen_futures::JsFuture::from(promise).await {
                                    let level = js_sys::Reflect::get(&battery_val, &JsValue::from_str("level"))
                                        .and_then(|v| v.as_f64().ok_or(JsValue::UNDEFINED))
                                        .unwrap_or(1.0);
                                    let charging = js_sys::Reflect::get(&battery_val, &JsValue::from_str("charging"))
                                        .and_then(|v| v.as_bool().ok_or(JsValue::UNDEFINED))
                                        .unwrap_or(true);

                                    // Only send an update if the value actually changed.
                                    // Compare battery level as integer percentage to avoid
                                    // floating-point noise triggering spurious updates.
                                    let level_pct = (level * 100.0) as i32;
                                    let changed = last_level.get() != Some(level_pct)
                                        || last_charging.get() != Some(charging);

                                    if changed {
                                        last_level.set(Some(level_pct));
                                        last_charging.set(Some(charging));
                                        on_update.call(PowerStatus {
                                            battery_level: level,
                                            is_charging: charging,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            });
        };

        // Initial check
        update_battery();

        // Periodic check every 60 seconds
        let handle = Interval::new(60_000, update_battery);

        on_cleanup(move || {
            drop(handle);
        });
    });

    view! { <span style="display:none;" /> }
}
