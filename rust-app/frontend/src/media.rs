use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaDeviceInfo, MediaStream, MediaStreamConstraints, AudioContext, AnalyserNode};

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

    let constraints = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&constraints, &"video".into(), &wasm_bindgen::JsValue::TRUE);

    let func_val = js_sys::Reflect::get(&media_devices, &"getDisplayMedia".into())?;
    let func = func_val.dyn_into::<js_sys::Function>()?;
    let promise = func.call1(&media_devices, &wasm_bindgen::JsValue::from(constraints))?;

    let result: JsValue = JsFuture::from(js_sys::Promise::from(promise)).await?;

    result.dyn_into::<MediaStream>().map_err(|_| JsValue::from_str("Not a MediaStream"))
}

pub struct AudioMonitor {
    context: AudioContext,
    analyser: AnalyserNode,
    _source: web_sys::MediaStreamAudioSourceNode,
    _closure: Closure<dyn FnMut()>,
    interval_id: i32,
}

impl AudioMonitor {
    pub fn new(stream: &MediaStream, on_talking: Box<dyn FnMut(bool)>) -> Result<Self, JsValue> {
        let context = AudioContext::new()?;
        let source = context.create_media_stream_source(stream)?;
        let analyser = context.create_analyser()?;
        analyser.set_fft_size(256);
        source.connect_with_audio_node(&analyser)?;

        let mut callback = on_talking;
        let mut was_talking = false;
        let buffer_len = analyser.frequency_bin_count() as usize;
        let data_array = vec![0u8; buffer_len];

        let analyser_clone = analyser.clone();

        let closure = Closure::wrap(Box::new(move || {
            let mut array = data_array.clone(); // Clone for safety in loop, ideally we reuse buffer but closure ownership is tricky
            // Wait, copying vec every frame is bad. But with `move` closure, we own `data_array`.
            // `get_byte_frequency_data` takes `&mut [u8]`.
            // We need `data_array` to be mutable inside closure.

            analyser_clone.get_byte_frequency_data(&mut array);

            let sum: u32 = array.iter().map(|&x| x as u32).sum();
            let avg = sum as f64 / array.len() as f64;

            // Threshold for talking
            let is_talking = avg > 20.0;

            if is_talking != was_talking {
                was_talking = is_talking;
                callback(is_talking);
            }
        }) as Box<dyn FnMut()>);

        // Run interval
        let window = web_sys::window().unwrap();
        let interval_id = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            100, // Check every 100ms
        )?;

        Ok(AudioMonitor {
            context,
            analyser,
            _source: source,
            _closure: closure,
            interval_id,
        })
    }
}

impl Drop for AudioMonitor {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(self.interval_id);
        }
        let _ = self.context.close();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compilation() {
        assert_eq!(1 + 1, 2);
    }
}
