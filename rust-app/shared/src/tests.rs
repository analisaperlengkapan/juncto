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
    // {"type":"ParticipantJoined","payload":{"id":"123","name":"Alice"}}
    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}
