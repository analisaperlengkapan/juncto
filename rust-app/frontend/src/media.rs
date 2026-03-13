use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AnalyserNode, AudioContext, MediaDeviceInfo, MediaDeviceKind, MediaStream,
    MediaStreamConstraints,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub label: String,
    pub kind: String,
}

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

pub async fn get_video_input_devices() -> Result<Vec<DeviceInfo>, JsValue> {
    let devices = enumerate_devices().await?;
    let mut result = Vec::new();
    for device in devices {
        if device.kind() == MediaDeviceKind::Videoinput {
            result.push(DeviceInfo {
                device_id: device.device_id(),
                label: device.label(),
                kind: "videoinput".to_string(),
            });
        }
    }
    Ok(result)
}

pub async fn get_audio_input_devices() -> Result<Vec<DeviceInfo>, JsValue> {
    let devices = enumerate_devices().await?;
    let mut result = Vec::new();
    for device in devices {
        if device.kind() == MediaDeviceKind::Audioinput {
            result.push(DeviceInfo {
                device_id: device.device_id(),
                label: device.label(),
                kind: "audioinput".to_string(),
            });
        }
    }
    Ok(result)
}

pub async fn get_user_media(
    enable_video: bool,
    enable_audio: bool,
    video_device_id: Option<String>,
    audio_device_id: Option<String>,
    video_resolution: Option<&str>,
) -> Result<MediaStream, JsValue> {
    let window = web_sys::window().ok_or(JsValue::from_str("No global window"))?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices()?;

    let constraints = MediaStreamConstraints::new();

    // Video constraints
    let video_val = if enable_video {
        let video_obj = js_sys::Object::new();
        if let Some(id) = video_device_id {
            let _ = js_sys::Reflect::set(&video_obj, &"deviceId".into(), &id.into());
        }

        if let Some(res) = video_resolution {
            if res == "hd" {
                let _ = js_sys::Reflect::set(&video_obj, &"width".into(), &1280.into());
                let _ = js_sys::Reflect::set(&video_obj, &"height".into(), &720.into());
            } else if res == "sd" {
                let _ = js_sys::Reflect::set(&video_obj, &"width".into(), &640.into());
                let _ = js_sys::Reflect::set(&video_obj, &"height".into(), &360.into());
            }
        }
        wasm_bindgen::JsValue::from(video_obj)
    } else {
        wasm_bindgen::JsValue::FALSE
    };

    constraints.set_video(&video_val);

    // Audio constraints
    let audio_val = if enable_audio {
        if let Some(id) = audio_device_id {
            let audio_obj = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&audio_obj, &"deviceId".into(), &id.into());
            wasm_bindgen::JsValue::from(audio_obj)
        } else {
            wasm_bindgen::JsValue::TRUE
        }
    } else {
        wasm_bindgen::JsValue::FALSE
    };
    constraints.set_audio(&audio_val);

    let promise = media_devices.get_user_media_with_constraints(&constraints)?;
    let result: JsValue = JsFuture::from(promise).await?;

    result
        .dyn_into::<MediaStream>()
        .map_err(|_| JsValue::from_str("Not a MediaStream"))
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

    result
        .dyn_into::<MediaStream>()
        .map_err(|_| JsValue::from_str("Not a MediaStream"))
}

pub struct AudioMonitor {
    context: AudioContext,
    #[allow(dead_code)]
    analyser: AnalyserNode,
    _source: web_sys::MediaStreamAudioSourceNode,
    _closure: Closure<dyn FnMut()>,
    interval_id: i32,
    is_muted: std::rc::Rc<std::cell::RefCell<bool>>,
}

impl AudioMonitor {
    pub fn new(stream: &MediaStream, on_talking: Box<dyn FnMut(bool)>, mut on_no_audio: Option<Box<dyn FnMut()>>) -> Result<Self, JsValue> {
        // Bug 1 Fix: The original `stream`'s tracks are disabled when the user mutes themselves,
        // which sends silence to the Web Audio API. To detect talking while muted, we must
        // clone the audio track and keep it enabled, feeding the cloned stream to the AnalyserNode.
        let audio_tracks = stream.get_audio_tracks();
        let isolated_stream = MediaStream::new()?;

        for i in 0..audio_tracks.length() {
            let track_val = audio_tracks.get(i);
            if let Ok(track) = track_val.dyn_into::<web_sys::MediaStreamTrack>() {
                let cloned_track = track.clone();
                // Ensure the cloned track remains enabled for local analysis even if original is disabled
                cloned_track.set_enabled(true);
                isolated_stream.add_track(&cloned_track);
            }
        }

        let context = AudioContext::new()?;
        let source = context.create_media_stream_source(&isolated_stream)?;
        let analyser = context.create_analyser()?;
        analyser.set_fft_size(256);
        source.connect_with_audio_node(&analyser)?;

        let mut callback = on_talking;
        let mut was_talking = false;
        let buffer_len = analyser.frequency_bin_count() as usize;
        let data_array = vec![0u8; buffer_len];

        let analyser_clone = analyser.clone();

        let is_muted = std::rc::Rc::new(std::cell::RefCell::new(false));
        let is_muted_clone = is_muted.clone();

        let mut silence_counter = 0;
        let mut no_audio_triggered = false;
        let mut has_ever_talked = false;
        let mut talk_while_muted_counter = 0;

        let closure = Closure::wrap(Box::new(move || {
            let mut array = data_array.clone();
            analyser_clone.get_byte_frequency_data(&mut array);

            let sum: u32 = array.iter().map(|&x| x as u32).sum();
            let avg = sum as f64 / array.len() as f64;

            // Threshold for talking
            let is_talking = avg > 20.0;

            // Do not analyze or trigger callbacks while explicitly muted
            if *is_muted_clone.borrow() {
                // Talk While Muted feature
                if is_talking {
                    talk_while_muted_counter += 1;
                    // Trigger a toast if speaking while muted for > 1 second (10 * 100ms)
                    if talk_while_muted_counter == 10 {
                        // For closures that can't easily access leptos context, we can dispatch a custom event
                        // or rely on a passed-in callback. We'll fire a global custom event.
                        if let Some(window) = web_sys::window() {
                            if let Ok(event) = web_sys::CustomEvent::new("talk_while_muted") {
                                let _ = window.dispatch_event(&event);
                            }
                        }
                    }
                } else {
                    talk_while_muted_counter = 0;
                }

                // If we are muted, we don't count silence towards the broken mic timeout.
                // We also ensure the "was_talking" state is cleanly suppressed.
                if was_talking {
                    was_talking = false;
                    callback(false);
                }
                return;
            } else {
                talk_while_muted_counter = 0;
            }

            if is_talking {
                has_ever_talked = true;
            }

            if is_talking != was_talking {
                was_talking = is_talking;
                callback(is_talking);
            }

            // If audio level is practically zero for a long time, and we haven't triggered yet
            if avg < 1.0 && !has_ever_talked {
                silence_counter += 1;
                if silence_counter > 100 && !no_audio_triggered { // 100 * 100ms = 10 seconds
                    no_audio_triggered = true;
                    if let Some(cb) = on_no_audio.as_mut() {
                        cb();
                    }
                }
            } else {
                silence_counter = 0;
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
            is_muted,
        })
    }

    pub fn set_muted(&self, muted: bool) {
        *self.is_muted.borrow_mut() = muted;
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
    use super::*;

    #[test]
    fn test_device_info_serialization() {
        let device = DeviceInfo {
            device_id: "test-id".to_string(),
            label: "Test Device".to_string(),
            kind: "videoinput".to_string(),
        };

        let json = serde_json::to_string(&device).expect("Failed to serialize");
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Device"));
        assert!(json.contains("videoinput"));

        let deserialized: DeviceInfo = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized, device);
    }
}

#[cfg(test)]
mod tests_media_muted {
    use super::*;

    #[test]
    fn test_audio_monitor_compiles() {
        assert!(true); // Cannot truly test without browser/WASM bindings
    }
}
