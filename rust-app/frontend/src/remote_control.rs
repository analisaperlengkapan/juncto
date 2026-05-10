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
    /// The peer currently controlling *us* (i.e. we are the controlled party).
    /// Set when the user grants an incoming `RemoteControlRequest`; cleared
    /// when either side stops the session, when the controller leaves the
    /// room, or when the user clicks "Stop". Drives a small banner in
    /// `RemoteControlLayer` so the controlled party has a visible indicator
    /// and an explicit way to end the session — without this, the target had
    /// no UI feedback that someone was sending input on their behalf.
    pub controlling_peer: RwSignal<Option<String>>,
    /// Pending incoming remote-control request: (requester_id, requester_name).
    /// When `Some`, the `RemoteControlLayer` renders a non-blocking in-app
    /// modal asking the user to Allow or Deny. Using a Leptos signal instead
    /// of `window.confirm()` avoids blocking the JS event loop, which would
    /// otherwise freeze WebSocket message processing (chat, signaling,
    /// heartbeats) for as long as the dialog is open.
    pub pending_incoming_request: RwSignal<Option<(String, String)>>,
}

impl RemoteControlService {
    pub fn new(send_signal: Callback<ClientMessage>) -> Self {
        Self {
            send_signal,
            controlled_peer: create_rw_signal(None),
            controlling_peer: create_rw_signal(None),
            pending_incoming_request: create_rw_signal(None),
        }
    }

    pub fn request_control(&self, target_id: String) {
        self.send_signal
            .call(ClientMessage::RequestRemoteControl(target_id));
    }

    pub fn stop_control(&self) {
        if let Some(peer_id) = self.controlled_peer.get_untracked() {
            self.send_signal
                .call(ClientMessage::StopRemoteControl(peer_id));
            self.controlled_peer.set(None);
        }
    }

    /// Stop a session in which we are the controlled party. Sends
    /// `StopRemoteControl` naming the controller, then clears the local
    /// `controlling_peer` signal so the banner disappears immediately.
    pub fn stop_being_controlled(&self) {
        if let Some(peer_id) = self.controlling_peer.get_untracked() {
            self.send_signal
                .call(ClientMessage::StopRemoteControl(peer_id));
            self.controlling_peer.set(None);
        }
    }

    pub fn set_controlled_peer(&self, peer_id: Option<String>) {
        self.controlled_peer.set(peer_id);
    }

    pub fn set_controlling_peer(&self, peer_id: Option<String>) {
        self.controlling_peer.set(peer_id);
    }

    pub fn send_action(&self, action: RemoteControlAction) {
        if let Some(target_id) = self.controlled_peer.get_untracked() {
            self.send_signal
                .call(ClientMessage::RemoteControlAction { target_id, action });
        }
    }

    /// Queue an incoming `RemoteControlRequest` for user consent. The
    /// `RemoteControlLayer` modal will read this signal and render Allow/Deny
    /// buttons.
    ///
    /// If a pending request is already on screen, the new request is
    /// auto-denied instead of overwriting the modal. Without this, the first
    /// requester's prompt would silently disappear (the user never sees it),
    /// and their server-side entry in `pending_remote_control_requests` would
    /// leak until they disconnect — `GrantRemoteControl`/`DenyRemoteControl`
    /// from the target only consume the entry whose `requester_id` matches
    /// the modal's current value.
    pub fn set_pending_incoming_request(&self, requester_id: String, requester_name: String) {
        if self.pending_incoming_request.get_untracked().is_some() {
            // Auto-deny the new request so the server clears its pending
            // entry. The original requester will receive a `RemoteControlAllowed { allowed: false }`.
            self.send_signal
                .call(ClientMessage::DenyRemoteControl(requester_id));
            return;
        }
        self.pending_incoming_request
            .set(Some((requester_id, requester_name)));
    }

    /// Respond to a pending incoming request and clear the signal. The
    /// banner (`controlling_peer`) is set reactively when the server echoes
    /// `RemoteControlAllowed { allowed: true }` back to us, not optimistically
    /// here — this avoids a stuck banner when the server rejects the grant
    /// (e.g. because another controller already holds a session against us).
    pub fn respond_to_incoming_request(&self, granted: bool) {
        if let Some((requester_id, _)) = self.pending_incoming_request.get_untracked() {
            let msg = if granted {
                ClientMessage::GrantRemoteControl(requester_id)
            } else {
                ClientMessage::DenyRemoteControl(requester_id)
            };
            self.send_signal.call(msg);
            self.pending_incoming_request.set(None);
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
    let overlay_ref = create_node_ref::<leptos::html::Div>();

    // Programmatically focus the overlay whenever a session becomes active so
    // keyboard events (including ESC to stop) are received without requiring
    // the user to click the overlay first.
    create_effect({
        let rc = rc.clone();
        move |_| {
            if rc.controlled_peer.get().is_some() {
                if let Some(el) = overlay_ref.get() {
                    let _ = el.focus();
                }
            }
        }
    });

    // Pre-clone `rc` for each `<Show>`. The `view!` macro wraps each Show's
    // children block in a `move` closure, which would otherwise capture the
    // outer `rc` by move on the first Show and leave subsequent Shows
    // referring to a moved value.
    let rc_pending_when = rc.clone();
    let rc_pending_children = rc.clone();
    let rc_controlling_when = rc.clone();
    let rc_controlling_children = rc.clone();
    let rc_controlled_when = rc.clone();
    let rc_controlled_children = rc.clone();
    view! {
        <Show when=move || rc_pending_when.pending_incoming_request.get().is_some()>
            {
                let rc = rc_pending_children.clone();
                let rc_for_name = rc.clone();
                let rc_allow = rc.clone();
                let rc_deny = rc.clone();
                view! {
                    <div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.5); z-index: 10000; display: flex; align-items: center; justify-content: center;">
                        <div style="background: white; color: black; padding: 20px; border-radius: 8px; max-width: 400px; box-shadow: 0 4px 20px rgba(0,0,0,0.3);">
                            <h3 style="margin-top: 0;">"Remote Control Request"</h3>
                            <p>
                                {move || {
                                    let name = rc_for_name
                                        .pending_incoming_request
                                        .get()
                                        .map(|(_, n)| n)
                                        .unwrap_or_default();
                                    format!("{} is requesting remote control of your session. Allow?", name)
                                }}
                            </p>
                            <div style="display: flex; gap: 10px; justify-content: flex-end;">
                                <button
                                    on:click=move |_| rc_deny.respond_to_incoming_request(false)
                                    style="padding: 8px 16px; border: 1px solid #ccc; background: white; cursor: pointer; border-radius: 4px;"
                                >
                                    "Deny"
                                </button>
                                <button
                                    on:click=move |_| rc_allow.respond_to_incoming_request(true)
                                    style="padding: 8px 16px; border: none; background: #007bff; color: white; cursor: pointer; border-radius: 4px;"
                                >
                                    "Allow"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            }
        </Show>
        <Show when=move || rc_controlling_when.controlling_peer.get().is_some()>
            {
                let rc_stop = rc_controlling_children.clone();
                view! {
                    // Non-modal banner shown to the controlled party so they
                    // know remote input is being injected and can stop the
                    // session at any time. Pinned to the top of the viewport
                    // with a high z-index but no input capture, so the user
                    // can still interact with the rest of the page.
                    <div style="position: fixed; top: 10px; left: 50%; transform: translateX(-50%); background: rgba(180, 0, 0, 0.9); color: white; padding: 8px 16px; border-radius: 20px; z-index: 9998; display: flex; align-items: center; gap: 10px; box-shadow: 0 2px 8px rgba(0,0,0,0.3);">
                        <span>"You are being remotely controlled"</span>
                        <button
                            on:click=move |_| rc_stop.stop_being_controlled()
                            style="background: white; color: #b40000; border: none; padding: 4px 10px; cursor: pointer; border-radius: 4px; font-weight: bold;"
                        >
                            "Stop"
                        </button>
                    </div>
                }
            }
        </Show>
        <Show when=move || rc_controlled_when.controlled_peer.get().is_some()>
            {
            let rc = rc_controlled_children.clone();
            view! {
            <div
                node_ref=overlay_ref
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
                    if ev.key() == "Escape" {
                        ev.prevent_default();
                        rc.stop_control();
                        return;
                    }
                    // Suppress browser-default behavior for keys that would
                    // steal focus from the overlay (Tab) or trigger destructive
                    // navigation/reload (F5, Ctrl+R, Ctrl+W, Alt+Left/Right).
                    // Other keys (printable characters, arrows, etc.) are
                    // forwarded to the controlled peer *and* allowed to fire
                    // locally so the controller's browser remains usable.
                    let key = ev.key();
                    let suppress = key == "Tab"
                        || key == "F5"
                        || (ev.ctrl_key() && (key == "r" || key == "R" || key == "w" || key == "W"))
                        || (ev.alt_key() && (key == "ArrowLeft" || key == "ArrowRight"));
                    if suppress {
                        ev.prevent_default();
                    }
                    rc.send_action(RemoteControlAction::KeyDown {
                        key,
                    });
                }}
                on:keyup={let rc = rc.clone(); move |ev: web_sys::KeyboardEvent| {
                    let key = ev.key();
                    let suppress = key == "Tab"
                        || key == "F5"
                        || (ev.ctrl_key() && (key == "r" || key == "R" || key == "w" || key == "W"))
                        || (ev.alt_key() && (key == "ArrowLeft" || key == "ArrowRight"));
                    if suppress {
                        ev.prevent_default();
                    }
                    rc.send_action(RemoteControlAction::KeyUp {
                        key,
                    });
                }}
                tabindex="0"
            >
                <div
                    style="position: absolute; top: 10px; left: 50%; transform: translateX(-50%); background: rgba(0,0,0,0.7); color: white; padding: 5px 15px; border-radius: 20px;"
                    // Stop mouse events from bubbling to the overlay's
                    // `on:mousedown`/`on:mouseup`/`on:mousemove` handlers
                    // — otherwise clicks on the Stop button would be
                    // forwarded as `RemoteControlAction` mouse events to
                    // the controlled peer right before the session ends.
                    on:mousedown=|ev: web_sys::MouseEvent| ev.stop_propagation()
                    on:mouseup=|ev: web_sys::MouseEvent| ev.stop_propagation()
                    on:mousemove=|ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    "Controlling Remote Peer - Press ESC to stop"
                    <button
                        on:click={let rc = rc.clone(); move |_| rc.stop_control()}
                        style="margin-left: 10px; background: red; border: none; color: white; border-radius: 4px; cursor: pointer;"
                    >
                        "Stop"
                    </button>
                </div>
            </div>
            }
            }
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
