use gloo_timers::callback::Interval;
use leptos::*;
use shared::PowerStatus;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn PowerMonitor(on_update: Callback<PowerStatus>) -> impl IntoView {
    create_effect(move |_| {
        let update_battery = move || {
            let on_update = on_update;
            spawn_local(async move {
                if let Some(window) = web_sys::window() {
                    let navigator = window.navigator();
                    let nav_val: &JsValue = navigator.as_ref();
                    let get_battery_prop = JsValue::from_str("getBattery");

                    if let Ok(func) = js_sys::Reflect::get(nav_val, &get_battery_prop) {
                        if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                            if let Ok(promise) = func.call0(nav_val) {
                                let promise = js_sys::Promise::from(promise);
                                if let Ok(battery_val) =
                                    wasm_bindgen_futures::JsFuture::from(promise).await
                                {
                                    let level = js_sys::Reflect::get(
                                        &battery_val,
                                        &JsValue::from_str("level"),
                                    )
                                    .and_then(|v| v.as_f64().ok_or(JsValue::UNDEFINED))
                                    .unwrap_or(1.0);
                                    let charging = js_sys::Reflect::get(
                                        &battery_val,
                                        &JsValue::from_str("charging"),
                                    )
                                    .and_then(|v| v.as_bool().ok_or(JsValue::UNDEFINED))
                                    .unwrap_or(true);

                                    // Always send the current status so that
                                    // participants who just switched breakout
                                    // rooms receive fresh data. The 60-second
                                    // poll interval and server-side rate limiter
                                    // keep the overhead negligible.
                                    on_update.call(PowerStatus {
                                        battery_level: level,
                                        is_charging: charging,
                                    });
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_power_monitor_compiles() {
        let _ = true;
    }
}
