use leptos::*;
use shared::{ClientMessage, FaceExpression};

#[derive(Clone)]
pub struct FaceLandmarksService {
    send_signal: Callback<ClientMessage>,
    pub active: RwSignal<bool>,
}

impl FaceLandmarksService {
    pub fn new(send_signal: Callback<ClientMessage>) -> Self {
        Self {
            send_signal,
            active: create_rw_signal(false),
        }
    }

    pub fn start(&self) {
        if self.active.get_untracked() {
            return;
        }
        self.active.set(true);

        let send_signal = self.send_signal;
        let active = self.active;

        // Mock detection loop
        set_interval(
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
        );
    }

    pub fn stop(&self) {
        self.active.set(false);
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
