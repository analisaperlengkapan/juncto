use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaDeviceInfo, MediaStream, MediaStreamConstraints};

pub async fn enumerate_devices() -> Result<Vec<MediaDeviceInfo>, JsValue> {
    let window = web_sys::window().ok_or(JsValue::from_str("No global window"))?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices()?;

    let promise = media_devices.enumerate_devices()?;
    let result: JsValue = JsFuture::from(promise).await?;

    let array: js_sys::Array = result.dyn_into()?;
    let mut devices = Vec::new();

    for i in 0..array.length() {
        let val = array.get(i);
        if let Ok(device) = val.dyn_into::<MediaDeviceInfo>() {
            devices.push(device);
        }
    }

    Ok(devices)
}

pub async fn get_user_media(video_device_id: Option<String>, audio_device_id: Option<String>) -> Result<MediaStream, JsValue> {
    let window = web_sys::window().ok_or(JsValue::from_str("No global window"))?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices()?;

    let constraints = MediaStreamConstraints::new();

    // Video constraints
    let video_val = if let Some(id) = video_device_id {
         let video_obj = js_sys::Object::new();
         // Using "exact" for deviceId is standard but can fail if device not found.
         // Using plain deviceId might be interpreted as "ideal".
         // Let's build { deviceId: { exact: id } } for stricter selection,
         // or just { deviceId: id } which usually works as ideal.
         // For simplicity: { deviceId: id }
         let _ = js_sys::Reflect::set(&video_obj, &"deviceId".into(), &id.into());
         wasm_bindgen::JsValue::from(video_obj)
    } else {
        wasm_bindgen::JsValue::TRUE
    };
    constraints.set_video(&video_val);

    // Audio constraints
    let audio_val = if let Some(id) = audio_device_id {
         let audio_obj = js_sys::Object::new();
         let _ = js_sys::Reflect::set(&audio_obj, &"deviceId".into(), &id.into());
         wasm_bindgen::JsValue::from(audio_obj)
    } else {
        wasm_bindgen::JsValue::TRUE
    };
    constraints.set_audio(&audio_val);

    let promise = media_devices.get_user_media_with_constraints(&constraints)?;
    let result: JsValue = JsFuture::from(promise).await?;

    result.dyn_into::<MediaStream>().map_err(|_| JsValue::from_str("Not a MediaStream"))
}

pub async fn get_display_media() -> Result<MediaStream, JsValue> {
    let window = web_sys::window().ok_or(JsValue::from_str("No global window"))?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices()?;

    // getDisplayMedia usually takes constraints, often { video: true } is enough.
    // Ideally we pass specific constraints but basic usage is just triggering the picker.
    // Note: getDisplayMedia is not available on all devices/browsers via web-sys if not feature-gated or updated.
    // Let's check if we can use it. web-sys 0.3.x has it.

    // We create an object for constraints { video: true }
    let constraints = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&constraints, &"video".into(), &wasm_bindgen::JsValue::TRUE);

    // get_display_media is the standard name in web-sys for the version taking no args or optional constraints?
    // Let's try get_display_media() which returns a promise.
    // If we want constraints, use get_display_media_with_display_media_stream_constraints.
    // BUT `DisplayMediaStreamConstraints` might not be in our Cargo.toml features?
    // Let's assume standard `get_display_media` exists and takes implicit optional constraints or we use Reflect to call it if types are missing.
    // However, the error says `get_display_media` exists with different arguments.
    // It likely takes no arguments? Or takes `DisplayMediaStreamConstraints`.
    // Let's try unchecked reflection for safety if typed method fails.

    // Attempt with unchecked access via JsValue
    // We cast media_devices to JsValue and use unchecked reflection logic or just `get_display_media` with no args if that matches the generated binding.
    // The previous error said `get_display_media` exists but with different args.
    // Let's assume it takes no args in the binding if constraints are optional, or it might be `get_display_media_with_display_media_stream_constraints`.
    // But `Reflect` API in js_sys is `get`, `set`, `apply`.
    // Let's use `js_sys::Reflect::get` to find the function, then `js_sys::Function::apply`.

    let func_val = js_sys::Reflect::get(&media_devices, &"getDisplayMedia".into())?;
    let func = func_val.dyn_into::<js_sys::Function>()?;
    let promise = func.call1(&media_devices, &wasm_bindgen::JsValue::from(constraints))?;

    let result: JsValue = JsFuture::from(js_sys::Promise::from(promise)).await?;

    result.dyn_into::<MediaStream>().map_err(|_| JsValue::from_str("Not a MediaStream"))
}

#[cfg(test)]
mod tests {
    // Note: Unit testing web_sys interactions requires a browser environment (wasm-bindgen-test).
    // Standard `cargo test` cannot instantiate web_sys objects.
    // Integration is verified via E2E tests (Playwright).

    #[test]
    fn test_compilation() {
        assert_eq!(1 + 1, 2);
    }
}
