use leptos::leptos_dom::helpers::IntervalHandle;
use leptos::*;
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

        let send_signal = self.send_signal;
        let active = self.active;

        // Mock detection loop. Only flip `active` to true after the interval
        // is successfully created — otherwise a failed `set_interval_with_handle`
        // would leave `active=true` with no interval running, and the early
        // return at the top of `start()` would prevent recovery on retry.
        if let Ok(handle) = set_interval_with_handle(
            move || {
                if !active.get_untracked() {
                    return;
                }

                // Randomly send an expression every few seconds for demonstration
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
            self.active.set(true);
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
    let service = FaceLandmarksService::new(send_signal);
    // Ensure the interval is cleared if the owning scope is dropped (e.g.
    // navigation away from the room) without `stop()` being called first.
    // Without this, the `set_interval_with_handle` keeps firing for the
    // lifetime of the page even after the service is no longer reachable,
    // which sends `FaceExpression` messages on a possibly-closed WebSocket.
    let cleanup_interval = service.interval.clone();
    let cleanup_active = service.active;
    on_cleanup(move || {
        cleanup_active.set(false);
        if let Some(handle) = cleanup_interval.borrow_mut().take() {
            handle.clear();
        }
    });
    provide_context(service);
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
