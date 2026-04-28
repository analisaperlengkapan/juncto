use crate::AppState;
use shared::{ServerMessage, ClientMessage};
use std::sync::Arc;

pub fn handle_remote_control(uid: &str, msg: ClientMessage, _state: &Arc<AppState>) -> Vec<ServerMessage> {
    match msg {
        ClientMessage::RequestRemoteControl(target_id) => {
            vec![ServerMessage::RemoteControlRequest {
                requester_id: uid.to_string(),
                target_id,
            }]
        }
        ClientMessage::GrantRemoteControl(requester_id) => {
            vec![ServerMessage::RemoteControlAllowed {
                requester_id,
                target_id: uid.to_string(),
                allowed: true,
            }]
        }
        ClientMessage::DenyRemoteControl(requester_id) => {
            vec![ServerMessage::RemoteControlAllowed {
                requester_id,
                target_id: uid.to_string(),
                allowed: false,
            }]
        }
        ClientMessage::StopRemoteControl(peer_id) => {
            vec![ServerMessage::RemoteControlStopped {
                sender_id: uid.to_string(),
                peer_id,
            }]
        }
        ClientMessage::RemoteControlAction { target_id, action } => {
            vec![ServerMessage::RemoteControlAction {
                requester_id: uid.to_string(),
                target_id,
                action,
            }]
        }
        _ => vec![],
    }
}
