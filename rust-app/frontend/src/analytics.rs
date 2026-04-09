use leptos::*;
use shared::ClientMessage;
use wasm_bindgen::JsValue;

#[derive(Clone)]
pub struct AnalyticsService {
    send_signal: Callback<ClientMessage>,
}

impl AnalyticsService {
    pub fn new(send_signal: Callback<ClientMessage>) -> Self {
        Self { send_signal }
    }

    pub fn track_event(&self, name: &str, _properties: JsValue) {
        #[cfg(target_arch = "wasm32")]
        let props_str = js_sys::JSON::stringify(&_properties)
            .map(|s| String::from(s))
            .unwrap_or_else(|_| "{}".to_string());

        #[cfg(not(target_arch = "wasm32"))]
        let props_str = "{}".to_string();

        let msg = ClientMessage::AnalyticsEvent {
            name: name.to_string(),
            properties: props_str,
        };
        self.send_signal.call(msg);
    }

    pub fn track_join(&self, _room_id: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            let props = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&props, &JsValue::from_str("room_id"), &JsValue::from_str(_room_id));
            self.track_event("join_room", props.into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.track_event("join_room", JsValue::NULL);
    }

    pub fn track_toggle_media(&self, _media_type: &str, _enabled: bool) {
        #[cfg(target_arch = "wasm32")]
        {
            let props = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&props, &JsValue::from_str("type"), &JsValue::from_str(_media_type));
            let _ = js_sys::Reflect::set(&props, &JsValue::from_str("enabled"), &JsValue::from_bool(_enabled));
            self.track_event("toggle_media", props.into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.track_event("toggle_media", JsValue::NULL);
    }

    pub fn track_interaction(&self, action: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            let props = js_sys::Object::new();
            self.track_event(action, props.into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.track_event(action, JsValue::NULL);
    }
}

pub fn provide_analytics_context(send_signal: Callback<ClientMessage>) {
    provide_context(AnalyticsService::new(send_signal));
}

pub fn use_analytics() -> AnalyticsService {
    use_context::<AnalyticsService>().expect("AnalyticsService not provided")
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::ClientMessage;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn test_analytics_event_formatting() {
        let last_msg = Rc::new(RefCell::new(None::<ClientMessage>));
        let last_msg_clone = last_msg.clone();

        let service = AnalyticsService::new(Callback::new(move |msg| {
            *last_msg_clone.borrow_mut() = Some(msg);
        }));

        service.track_interaction("test_action");

        let msg = last_msg.borrow().clone();
        if let Some(ClientMessage::AnalyticsEvent { name, properties }) = msg {
            assert_eq!(name, "test_action");
            assert_eq!(properties, "{}");
        } else {
            panic!("Event not sent or wrong type");
        }
    }
}
