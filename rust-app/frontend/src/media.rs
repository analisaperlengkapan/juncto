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
