with open('rust-app/frontend/src/media.rs', 'r') as f:
    content = f.read()

# Add no_audio callback
content = content.replace('pub fn new(stream: &MediaStream, on_talking: Box<dyn FnMut(bool)>) -> Result<Self, JsValue> {', 'pub fn new(stream: &MediaStream, on_talking: Box<dyn FnMut(bool)>, mut on_no_audio: Option<Box<dyn FnMut()>>) -> Result<Self, JsValue> {')

# Add counter for no_audio inside closure
closure_patch = """
        let mut silence_counter = 0;
        let mut no_audio_triggered = false;

        let closure = Closure::wrap(Box::new(move || {
            let mut array = data_array.clone();

            analyser_clone.get_byte_frequency_data(&mut array);

            let sum: u32 = array.iter().map(|&x| x as u32).sum();
            let avg = sum as f64 / array.len() as f64;

            // Threshold for talking
            let is_talking = avg > 20.0;

            if is_talking != was_talking {
                was_talking = is_talking;
                callback(is_talking);
            }

            // If audio level is practically zero for a long time, and we haven't triggered yet
            if avg < 1.0 {
                silence_counter += 1;
                if silence_counter > 50 && !no_audio_triggered { // 50 * 100ms = 5 seconds
                    no_audio_triggered = true;
                    if let Some(cb) = on_no_audio.as_mut() {
                        cb();
                    }
                }
            } else {
                silence_counter = 0;
            }
"""

content = content.replace("""        let closure = Closure::wrap(Box::new(move || {
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
            }""", closure_patch)

with open('rust-app/frontend/src/media.rs', 'w') as f:
    f.write(content)
