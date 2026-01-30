use super::*;
use serde_json;

#[test]
fn test_chat_message_serialization() {
    let msg = ChatMessage {
        user_id: "user1".to_string(),
        content: "Hello Rust".to_string(),
        timestamp: 1627840000,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}

#[test]
fn test_server_message_serialization() {
    let p = Participant {
        id: "123".to_string(),
        name: "Alice".to_string(),
    };
    let msg = ServerMessage::ParticipantJoined(p.clone());
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);

    let msg_update = ServerMessage::ParticipantUpdated(p.clone());
    let json_update = serde_json::to_string(&msg_update).unwrap();
    let deserialized_update: ServerMessage = serde_json::from_str(&json_update).unwrap();
    assert_eq!(msg_update, deserialized_update);
}

#[test]
fn test_client_message_serialization() {
    let msg = ClientMessage::ToggleRoomLock;
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);

    let msg_prof = ClientMessage::UpdateProfile("Bob".to_string());
    let json_prof = serde_json::to_string(&msg_prof).unwrap();
    let deserialized_prof: ClientMessage = serde_json::from_str(&json_prof).unwrap();
    assert_eq!(msg_prof, deserialized_prof);
}
