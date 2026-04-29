use leptos::*;
use shared::{ClientMessage, RemoteControlAction};
use std::cell::Cell;
use std::rc::Rc;

/// Minimum interval (in milliseconds) between successive `MouseMove`
/// `RemoteControlAction` messages. mousemove events typically fire at
/// ~60Hz, which would overwhelm the server's broadcast channel
/// (capacity 100). Throttling to ~20Hz keeps the protocol responsive
/// while preventing `RecvError::Lagged` for other subscribers.
const MOUSE_MOVE_THROTTLE_MS: f64 = 50.0;

#[derive(Clone)]
pub struct RemoteControlService {
    pub send_signal: Callback<ClientMessage>,
    pub controlled_peer: RwSignal<Option<String>>,
}

impl RemoteControlService {
    pub fn new(send_signal: Callback<ClientMessage>) -> Self {
        Self {
            send_signal,
            controlled_peer: create_rw_signal(None),
        }
    }

    pub fn request_control(&self, target_id: String) {
        self.send_signal.call(ClientMessage::RequestRemoteControl(target_id));
    }

    pub fn stop_control(&self) {
        if let Some(peer_id) = self.controlled_peer.get_untracked() {
            self.send_signal.call(ClientMessage::StopRemoteControl(peer_id));
            self.controlled_peer.set(None);
        }
    }

    pub fn set_controlled_peer(&self, peer_id: Option<String>) {
        self.controlled_peer.set(peer_id);
    }

    pub fn send_action(&self, action: RemoteControlAction) {
        if let Some(target_id) = self.controlled_peer.get_untracked() {
            self.send_signal.call(ClientMessage::RemoteControlAction {
                target_id,
                action,
            });
        }
    }
}

pub fn provide_remote_control_context(send_signal: Callback<ClientMessage>) {
    provide_context(RemoteControlService::new(send_signal));
}

pub fn use_remote_control() -> RemoteControlService {
    use_context::<RemoteControlService>().expect("RemoteControlService not provided")
}

#[component]
pub fn RemoteControlLayer() -> impl IntoView {
    let rc = use_remote_control();
    // Tracks the timestamp (ms) of the last sent MouseMove for throttling.
    let last_mousemove_ms: Rc<Cell<f64>> = Rc::new(Cell::new(0.0));

    view! {
        <Show when={let rc = rc.clone(); move || rc.controlled_peer.get().is_some()}>
            <div
                style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 9999; cursor: crosshair; background: rgba(0,0,0,0.1);"
                on:mousemove={
                    let rc = rc.clone();
                    let last = last_mousemove_ms.clone();
                    move |ev: web_sys::MouseEvent| {
                        let now = js_sys::Date::now();
                        if now - last.get() < MOUSE_MOVE_THROTTLE_MS {
                            return;
                        }
                        last.set(now);
                        rc.send_action(RemoteControlAction::MouseMove {
                            x: ev.client_x() as f64,
                            y: ev.client_y() as f64,
                        });
                    }
                }
                on:mousedown={let rc = rc.clone(); move |ev: web_sys::MouseEvent| {
                    rc.send_action(RemoteControlAction::MouseDown {
                        button: ev.button() as u8,
                    });
                }}
                on:mouseup={let rc = rc.clone(); move |ev: web_sys::MouseEvent| {
                    rc.send_action(RemoteControlAction::MouseUp {
                        button: ev.button() as u8,
                    });
                }}
                on:keydown={let rc = rc.clone(); move |ev: web_sys::KeyboardEvent| {
                    rc.send_action(RemoteControlAction::KeyDown {
                        key: ev.key(),
                    });
                }}
                on:keyup={let rc = rc.clone(); move |ev: web_sys::KeyboardEvent| {
                    rc.send_action(RemoteControlAction::KeyUp {
                        key: ev.key(),
                    });
                }}
                tabindex="0"
            >
                <div style="position: absolute; top: 10px; left: 50%; transform: translateX(-50%); background: rgba(0,0,0,0.7); color: white; padding: 5px 15px; border-radius: 20px;">
                    "Controlling Remote Peer - Press ESC to stop"
                    <button
                        on:click={let rc = rc.clone(); move |_| rc.stop_control()}
                        style="margin-left: 10px; background: red; border: none; color: white; border-radius: 4px; cursor: pointer;"
                    >
                        "Stop"
                    </button>
                </div>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_control_service_logic() {
        let _runtime = create_runtime();
        let service = RemoteControlService::new(Callback::new(|_| {}));

        assert!(service.controlled_peer.get().is_none());
        service.set_controlled_peer(Some("target".to_string()));
        assert_eq!(service.controlled_peer.get(), Some("target".to_string()));
    }
}
