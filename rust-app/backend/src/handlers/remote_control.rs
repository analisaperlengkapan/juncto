use crate::AppState;
use shared::{ClientMessage, ServerMessage};
use std::sync::Arc;

/// Returns `true` iff `requester` and `target` are both connected and located
/// in the same room (Main Room or the same breakout). This prevents
/// cross-room remote-control requests/actions, mirroring the scoping used by
/// `MuteParticipant` and `RequestUnmute`.
fn same_room(state: &Arc<AppState>, requester: &str, target: &str) -> bool {
    // Drop the `participants` lock before acquiring `participant_locations`
    // to avoid holding two state locks simultaneously. This keeps the lock
    // ordering explicit and prevents deadlocks if any future code path
    // acquires these locks in the opposite order.
    {
        let participants = state.participants.lock().unwrap();
        if !participants.contains_key(requester) || !participants.contains_key(target) {
            return false;
        }
    }
    let locs = state.participant_locations.lock().unwrap();
    let req_loc = locs.get(requester).cloned().flatten();
    let tgt_loc = locs.get(target).cloned().flatten();
    req_loc == tgt_loc
}

pub fn handle_remote_control(
    uid: &str,
    msg: ClientMessage,
    state: &Arc<AppState>,
) -> Vec<ServerMessage> {
    match msg {
        ClientMessage::RequestRemoteControl(target_id) => {
            // Validate target exists and is in the same room before broadcasting.
            if uid == target_id || !same_room(state, uid, &target_id) {
                return vec![];
            }
            // Record the pending request so a subsequent `GrantRemoteControl`
            // from `target_id` can be authorized. Without this, any client
            // could fabricate a grant naming any `requester_id` and force
            // the overlay onto an unconsenting peer.
            state
                .pending_remote_control_requests
                .lock()
                .unwrap()
                .insert((uid.to_string(), target_id.clone()));
            vec![ServerMessage::RemoteControlRequest {
                requester_id: uid.to_string(),
                target_id,
            }]
        }
        ClientMessage::GrantRemoteControl(requester_id) => {
            // Authorize: a matching `RequestRemoteControl` must have been
            // issued by `requester_id` targeting `uid`.
            let was_pending = state
                .pending_remote_control_requests
                .lock()
                .unwrap()
                .remove(&(requester_id.clone(), uid.to_string()));
            if !was_pending {
                return vec![];
            }
            // If the requester has moved to a different room since the request
            // was issued, surface this to the requester as a denial rather than
            // silently dropping the grant. The pending entry has already been
            // consumed above, so without this branch neither side would learn
            // the outcome (the target's modal is gone, the requester is still
            // waiting on a response).
            if !same_room(state, uid, &requester_id) {
                return vec![ServerMessage::RemoteControlAllowed {
                    requester_id,
                    target_id: uid.to_string(),
                    allowed: false,
                }];
            }
            // Record the active session: requester controls `uid`. If the
            // requester already had an active session controlling a different
            // peer, tear it down explicitly and notify both parties via a
            // `RemoteControlStopped` message so the previously-controlled peer
            // (and the controller's overlay state on other clients) does not
            // think the old session is still active.
            let previous_controlled = state
                .remote_control_sessions
                .lock()
                .unwrap()
                .insert(requester_id.clone(), uid.to_string());
            let mut out = Vec::with_capacity(2);
            if let Some(prev) = previous_controlled {
                if prev != uid {
                    out.push(ServerMessage::RemoteControlStopped {
                        sender_id: requester_id.clone(),
                        peer_id: prev,
                    });
                }
            }
            out.push(ServerMessage::RemoteControlAllowed {
                requester_id,
                target_id: uid.to_string(),
                allowed: true,
            });
            out
        }
        ClientMessage::DenyRemoteControl(requester_id) => {
            // Only respond if there is a matching pending request, and only
            // remove that pending entry so unrelated participants cannot
            // emit spurious "denied" toasts.
            let was_pending = state
                .pending_remote_control_requests
                .lock()
                .unwrap()
                .remove(&(requester_id.clone(), uid.to_string()));
            if !was_pending {
                return vec![];
            }
            vec![ServerMessage::RemoteControlAllowed {
                requester_id,
                target_id: uid.to_string(),
                allowed: false,
            }]
        }
        ClientMessage::StopRemoteControl(peer_id) => {
            // Authorize: stop is valid only if `uid` is a party to a session
            // with `peer_id` (either side may end it).
            let mut sessions = state.remote_control_sessions.lock().unwrap();
            let is_controller = sessions.get(uid).map(|t| t == &peer_id).unwrap_or(false);
            let is_controlled = sessions.get(&peer_id).map(|t| t == uid).unwrap_or(false);
            if !is_controller && !is_controlled {
                return vec![];
            }
            if is_controller {
                sessions.remove(uid);
            } else {
                sessions.remove(&peer_id);
            }
            drop(sessions);
            vec![ServerMessage::RemoteControlStopped {
                sender_id: uid.to_string(),
                peer_id,
            }]
        }
        ClientMessage::RemoteControlAction { target_id, action } => {
            // Authorize: `uid` must have an active session controlling `target_id`.
            let authorized = state
                .remote_control_sessions
                .lock()
                .unwrap()
                .get(uid)
                .map(|t| t == &target_id)
                .unwrap_or(false);
            if !authorized {
                return vec![];
            }
            // Enforce same-room scoping: if the controller or controlled has
            // moved to a different breakout room since the session was granted,
            // drop the action rather than forward it across rooms. The session
            // entry is preserved so it resumes if both parties return to the
            // same room; callers can also explicitly `StopRemoteControl` to
            // tear it down.
            if !same_room(state, uid, &target_id) {
                return vec![];
            }
            vec![ServerMessage::RemoteControlAction {
                requester_id: uid.to_string(),
                target_id,
                action,
            }]
        }
        _ => vec![],
    }
}
