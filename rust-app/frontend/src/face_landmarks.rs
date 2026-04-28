use leptos::*;
use leptos::leptos_dom::helpers::IntervalHandle;
use shared::{ClientMessage, FaceExpression};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct FaceLandmarksService {
    send_signal: Callback<ClientMessage>,
    pub active: RwSignal<bool>,
    interval: Rc<RefCell<Option<IntervalHandle>>>,
}

impl FaceLandmarksService {
    pub fn new(send_signal: Callback<ClientMessage>) -> Self {
        Self {
            send_signal,
            active: create_rw_signal(false),
            interval: Rc::new(RefCell::new(None)),
        }
    }

    pub fn start(&self) {
        if self.active.get_untracked() {
            return;
        }
        self.active.set(true);

        let send_signal = self.send_signal;

        // Mock detection loop
        if let Ok(handle) = set_interval_with_handle(
            move || {
                let expressions = ["happy", "sad", "surprised", "neutral"];
                let idx = (js_sys::Math::random() * expressions.len() as f64) as usize;

                let msg = ClientMessage::FaceExpression(FaceExpression {
                    expression: expressions[idx].to_string(),
                    timestamp: js_sys::Date::now() as u64,
                });
                send_signal.call(msg);
            },
            std::time::Duration::from_secs(5),
        ) {
            *self.interval.borrow_mut() = Some(handle);
        }
    }

    pub fn stop(&self) {
        self.active.set(false);
        if let Some(handle) = self.interval.borrow_mut().take() {
            handle.clear();
        }
    }
}

pub fn provide_face_landmarks_context(send_signal: Callback<ClientMessage>) {
    provide_context(FaceLandmarksService::new(send_signal));
}

pub fn use_face_landmarks() -> FaceLandmarksService {
    use_context::<FaceLandmarksService>().expect("FaceLandmarksService not provided")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_landmarks_service_logic() {
        let _runtime = create_runtime();
        let service = FaceLandmarksService::new(Callback::new(|_| {}));

        assert!(!service.active.get());
        service.active.set(true);
        assert!(service.active.get());
        service.stop();
        assert!(!service.active.get());
    }
}
