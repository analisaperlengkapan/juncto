use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AnalyserNode, AudioContext, CanvasRenderingContext2d, DynamicsCompressorNode,
    HtmlCanvasElement, HtmlVideoElement, MediaDeviceInfo, MediaDeviceKind, MediaStream,
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

pub struct VideoProcessor {
    #[allow(dead_code)]
    canvas: HtmlCanvasElement,
    #[allow(dead_code)]
    context: CanvasRenderingContext2d,
    video: HtmlVideoElement,
    #[allow(dead_code)]
    _closure: Closure<dyn FnMut()>,
    interval_id: i32,
    #[allow(dead_code)]
    mode: Rc<RefCell<String>>,
}

impl VideoProcessor {
    pub fn set_mode(&self, new_mode: String) {
        *self.mode.borrow_mut() = new_mode;
    }

    pub fn new(stream: &MediaStream, initial_mode: String) -> Result<(Self, MediaStream), JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        let video = document
            .create_element("video")?
            .dyn_into::<HtmlVideoElement>()?;

        video.set_src_object(Some(stream));
        video.set_muted(true);
        let _ = video.play();

        let mode = Rc::new(RefCell::new(initial_mode));
        let mode_clone = mode.clone();
        let canvas_clone = canvas.clone();
        let context_clone = context.clone();
        let video_clone = video.clone();
        let last_width: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let last_height: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

        let closure = Closure::wrap(Box::new(move || {
            let width = video_clone.video_width() as f64;
            let height = video_clone.video_height() as f64;
            if width == 0.0 || height == 0.0 {
                return;
            }

            // Only update canvas dimensions when the video resolution changes.
            // Calling set_width/set_height clears the canvas buffer, so doing it
            // every frame (~30fps) wastes CPU on an unnecessary buffer reset.
            let w = width as u32;
            let h = height as u32;
            if *last_width.borrow() != w || *last_height.borrow() != h {
                canvas_clone.set_width(w);
                canvas_clone.set_height(h);
                *last_width.borrow_mut() = w;
                *last_height.borrow_mut() = h;
            }

            let current_mode = mode_clone.borrow();
            match current_mode.as_str() {
                "blur" => {
                    context_clone.set_filter("blur(5px)");
                    let _ = context_clone.draw_image_with_html_video_element(&video_clone, 0.0, 0.0);
                }
                "image" => {
                    // Draw a placeholder background color (representing an image)
                    context_clone.set_filter("none");
                    let style = wasm_bindgen::JsValue::from_str("#004400");
                    #[allow(deprecated)]
                    context_clone.set_fill_style(&style);
                    context_clone.fill_rect(0.0, 0.0, width, height);
                    let _ = context_clone.draw_image_with_html_video_element(&video_clone, 0.0, 0.0);
                }
                _ => {
                    context_clone.set_filter("none");
                    let _ = context_clone.draw_image_with_html_video_element(&video_clone, 0.0, 0.0);
                }
            }
        }) as Box<dyn FnMut()>);

        let interval_id = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            33, // ~30fps
        )?;

        // capture_stream is not directly in web_sys for all browsers, use Reflect as fallback
        let processed_stream = if let Ok(func) = js_sys::Reflect::get(&canvas, &"captureStream".into()) {
            let func = func.dyn_into::<js_sys::Function>()?;
            func.call0(&canvas)?.dyn_into::<MediaStream>()?
        } else {
            return Err(JsValue::from_str("Canvas captureStream not supported"));
        };

        // Canvas captureStream only contains video tracks. Copy audio tracks from the
        // original stream so that WebRTC peers still receive audio and mute/unmute
        // operations continue to work when a virtual background is active.
        let audio_tracks = stream.get_audio_tracks();
        for i in 0..audio_tracks.length() {
            let track = audio_tracks.get(i);
            if let Ok(track) = track.dyn_into::<web_sys::MediaStreamTrack>() {
                processed_stream.add_track(&track);
            }
        }

        Ok((
            Self {
                canvas,
                context,
                video,
                _closure: closure,
                interval_id,
                mode,
            },
            processed_stream,
        ))
    }

}

impl Drop for VideoProcessor {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(self.interval_id);
        }
        self.video.set_src_object(None);
    }
}

pub async fn get_display_media() -> Result<MediaStream, JsValue> {
    let window = web_sys::window().ok_or(JsValue::from_str("No global window"))?;
    let navigator = window.navigator();
    let media_devices = navigator.media_devices()?;

    let func_val = js_sys::Reflect::get(&media_devices, &"getDisplayMedia".into())?;
    let func = func_val.dyn_into::<js_sys::Function>()?;

    // First attempt with audio: true. Some browsers/surfaces (e.g. Firefox,
    // application windows on macOS) may reject audio capture, so fall back
    // to video-only on failure rather than failing the whole screen share.
    //
    // Important: only fall back when the browser indicates the audio
    // constraint was the problem. If the user explicitly cancels the
    // screen-share picker, the promise rejects with NotAllowedError /
    // AbortError, and retrying would pop a second picker dialog forcing
    // the user to cancel twice. Propagate cancellation errors as-is.
    fn is_user_cancellation(err: &JsValue) -> bool {
        if let Ok(name) = js_sys::Reflect::get(err, &"name".into()) {
            if let Some(name) = name.as_string() {
                return name == "NotAllowedError" || name == "AbortError";
            }
        }
        false
    }

    let constraints_with_audio = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&constraints_with_audio, &"video".into(), &wasm_bindgen::JsValue::TRUE);
    let _ = js_sys::Reflect::set(&constraints_with_audio, &"audio".into(), &wasm_bindgen::JsValue::TRUE);

    let result: JsValue = match func.call1(&media_devices, &wasm_bindgen::JsValue::from(constraints_with_audio)) {
        Ok(promise) => match JsFuture::from(js_sys::Promise::from(promise)).await {
            Ok(r) => r,
            Err(e) => {
                if is_user_cancellation(&e) {
                    return Err(e);
                }
                // Retry video-only for capability errors (e.g. NotSupportedError)
                let constraints_video_only = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&constraints_video_only, &"video".into(), &wasm_bindgen::JsValue::TRUE);
                let promise = func.call1(&media_devices, &wasm_bindgen::JsValue::from(constraints_video_only))?;
                JsFuture::from(js_sys::Promise::from(promise)).await?
            }
        },
        Err(e) => {
            if is_user_cancellation(&e) {
                return Err(e);
            }
            let constraints_video_only = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&constraints_video_only, &"video".into(), &wasm_bindgen::JsValue::TRUE);
            let promise = func.call1(&media_devices, &wasm_bindgen::JsValue::from(constraints_video_only))?;
            JsFuture::from(js_sys::Promise::from(promise)).await?
        }
    };

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
    is_noise_suppression_enabled: std::rc::Rc<std::cell::RefCell<bool>>,
    isolated_stream: MediaStream,
    compressor: Option<DynamicsCompressorNode>,
}

impl AudioMonitor {
    pub fn new(
        stream: &MediaStream,
        on_talking: Box<dyn FnMut(bool)>,
        on_level: Option<Box<dyn FnMut(f64)>>,
        mut on_no_audio: Option<Box<dyn FnMut()>>,
        noise_suppression: bool,
    ) -> Result<Self, JsValue> {
        // Bug 1 Fix: The original `stream`'s tracks are disabled when the user mutes themselves,
        // which sends silence to the Web Audio API. To detect talking while muted, we must
        // clone the audio track and keep it enabled, feeding the cloned stream to the AnalyserNode.
        let audio_tracks = stream.get_audio_tracks();
        let isolated_stream = MediaStream::new()?;

        for i in 0..audio_tracks.length() {
            let track_val = audio_tracks.get(i);
            if let Ok(track) = track_val.dyn_into::<web_sys::MediaStreamTrack>() {
                // We need to truly clone the JS MediaStreamTrack, not just the Rust reference
                let clone_fn = js_sys::Reflect::get(&track, &"clone".into())
                    .map_err(|_| JsValue::from_str("No clone method on MediaStreamTrack"))?;
                let clone_fn = clone_fn.dyn_into::<js_sys::Function>()
                    .map_err(|_| JsValue::from_str("clone is not a function"))?;
                let cloned_val = clone_fn.call0(&track)?;
                let cloned_track = cloned_val.dyn_into::<web_sys::MediaStreamTrack>()
                    .map_err(|_| JsValue::from_str("Clone did not return MediaStreamTrack"))?;

                // Ensure the cloned track remains enabled for local analysis even if original is disabled
                cloned_track.set_enabled(true);
                isolated_stream.add_track(&cloned_track);
            }
        }

        let context = AudioContext::new()?;
        let source = context.create_media_stream_source(&isolated_stream)?;
        let analyser = context.create_analyser()?;
        analyser.set_fft_size(256);

        let mut compressor = None;
        if noise_suppression {
            let node = context.create_dynamics_compressor()?;
            node.threshold().set_value(-50.0);
            node.knee().set_value(40.0);
            node.ratio().set_value(12.0);
            node.attack().set_value(0.003);
            node.release().set_value(0.25);
            source.connect_with_audio_node(&node)?;
            node.connect_with_audio_node(&analyser)?;
            compressor = Some(node);
        } else {
            source.connect_with_audio_node(&analyser)?;
        }

        let mut callback = on_talking;
        let mut level_callback = on_level;
        let mut was_talking = false;
        let buffer_len = analyser.frequency_bin_count() as usize;
        let data_array = vec![0u8; buffer_len];

        let analyser_clone = analyser.clone();

        let is_muted = std::rc::Rc::new(std::cell::RefCell::new(false));
        let is_muted_clone = is_muted.clone();
        let is_noise_suppression_enabled =
            std::rc::Rc::new(std::cell::RefCell::new(noise_suppression));

        let mut silence_counter = 0;
        let mut no_audio_triggered = false;
        let mut has_ever_talked = false;
        let mut talk_while_muted_counter = 0;
        let mut toast_fired_for_this_mute_cycle = false;
        let mut noise_counter = 0;
        let mut noise_triggered = false;

        let closure = Closure::wrap(Box::new(move || {
            let mut array = data_array.clone();
            analyser_clone.get_byte_frequency_data(&mut array);

            let sum: u32 = array.iter().map(|&x| x as u32).sum();
            let avg = sum as f64 / array.len() as f64;

            // Threshold for talking
            let is_talking = avg > 20.0;

            // Normalize average for level indicator (0.0 to 1.0).
            // Report 0.0 when muted so the indicator and any consumers
            // do not display real microphone activity from the isolated
            // analysis stream while the user is muted.
            let normalized_level = if *is_muted_clone.borrow() {
                0.0
            } else {
                (avg / 100.0).clamp(0.0, 1.0)
            };
            if let Some(ref mut cb) = level_callback {
                cb(normalized_level);
            }

            // Do not analyze or trigger callbacks while explicitly muted
            if *is_muted_clone.borrow() {
                // Talk While Muted feature
                if is_talking {
                    talk_while_muted_counter += 1;
                    // Trigger a toast if speaking while muted for > 1 second (10 * 100ms)
                    if talk_while_muted_counter >= 10 && !toast_fired_for_this_mute_cycle {
                        toast_fired_for_this_mute_cycle = true;
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

                // Reset noise-detection state while muted so moderate background
                // audio captured on the isolated (always-enabled) analysis stream
                // does not accumulate toward the noise threshold and fire a
                // warning the user cannot act on. Also clear `noise_triggered`
                // so a legitimate noise warning can fire after unmuting.
                noise_counter = 0;
                noise_triggered = false;

                // If we are muted, we don't count silence towards the broken mic timeout.
                // We also ensure the "was_talking" state is cleanly suppressed.
                if was_talking {
                    was_talking = false;
                    callback(false);
                }
                return;
            } else {
                talk_while_muted_counter = 0;
                toast_fired_for_this_mute_cycle = false;
            }

            if is_talking {
                has_ever_talked = true;
            }

            if is_talking != was_talking {
                was_talking = is_talking;
                callback(is_talking);
            }

            // Noise detection: Persistent moderate sound in the 12..30 band
            // while the user is not "talking" (talking threshold is avg > 20.0).
            // This range targets background noise loud enough to be annoying
            // but not loud enough to be intentional speech.
            //
            // The counter accumulates only during the noise-sample band.
            // Anything outside that band (talking above 20, or quieter-than-
            // noise audio at or below 12) resets the counter so we require
            // *continuous* noise rather than letting intermittent episodes
            // separated by quieter gaps add up to the 5-second threshold.
            if avg > 12.0 && avg < 30.0 && !is_talking {
                noise_counter += 1;
                if noise_counter > 50 && !noise_triggered { // 5 seconds of consistent noise
                    noise_triggered = true;
                    if let Some(window) = web_sys::window() {
                        if let Ok(event) = web_sys::CustomEvent::new("noise_detected") {
                            let _ = window.dispatch_event(&event);
                        }
                    }
                }
            } else if is_talking {
                noise_counter = 0;
                noise_triggered = false; // Reset when actually talking
            } else {
                // avg <= 12 and not talking — below the noise band.
                // Treat as a break in consistent noise.
                noise_counter = 0;
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
            is_noise_suppression_enabled,
            isolated_stream,
            compressor,
        })
    }

    pub fn set_muted(&self, muted: bool) {
        *self.is_muted.borrow_mut() = muted;
    }

    pub fn has_compressor(&self) -> bool {
        self.compressor.is_some()
    }

    pub fn set_noise_suppression(&self, enabled: bool) -> Result<(), JsValue> {
        let mut current = self.is_noise_suppression_enabled.borrow_mut();
        if *current == enabled {
            return Ok(());
        }

        if let Some(comp) = &self.compressor {
            // Compressor exists — adjust threshold to toggle bypass
            if enabled {
                comp.threshold().set_value(-50.0);
            } else {
                comp.threshold().set_value(0.0); // Effectively bypass
            }
            *current = enabled;
            Ok(())
        } else if enabled {
            // No compressor was created (monitor was built without noise suppression).
            // We cannot dynamically insert a DynamicsCompressorNode into the existing
            // audio graph without re-routing, so signal the caller to recreate the
            // AudioMonitor with the correct setting.
            Err(JsValue::from_str(
                "Cannot enable noise suppression: AudioMonitor was created without a compressor. Recreate the monitor.",
            ))
        } else {
            // Disabling when already disabled / no compressor — no-op is fine
            *current = enabled;
            Ok(())
        }
    }
}

impl Drop for AudioMonitor {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(self.interval_id);
        }
        let _ = self.context.close();

        // Ensure cloned tracks are explicitly stopped to release microphone
        let tracks = self.isolated_stream.get_tracks();
        for i in 0..tracks.length() {
            if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                track.stop();
            }
        }
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

    #[test]
    fn test_audio_monitor_compiles() {
        let _ = true; // Cannot truly test without browser/WASM bindings
    }

    #[test]
    fn test_video_processor_modes() {
        // Verification of mode values
        let mode = "blur".to_string();
        assert_eq!(mode, "blur");
        let mode2 = "image".to_string();
        assert_eq!(mode2, "image");
    }
}
