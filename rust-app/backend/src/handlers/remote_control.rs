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
        ClientMessage::GrantRemoteControl(_requester_id) => {
            vec![ServerMessage::RemoteControlAllowed {
                target_id: uid.to_string(),
                allowed: true,
            }]
        }
        ClientMessage::DenyRemoteControl(_requester_id) => {
            vec![ServerMessage::RemoteControlAllowed {
                target_id: uid.to_string(),
                allowed: false,
            }]
        }
        ClientMessage::StopRemoteControl(_peer_id) => {
            vec![ServerMessage::RemoteControlStopped {
                sender_id: uid.to_string(),
            }]
        }
        ClientMessage::RemoteControlAction { target_id: _, action } => {
            vec![ServerMessage::RemoteControlAction {
                requester_id: uid.to_string(),
                action,
            }]
        }
        _ => vec![],
    }
}
