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

    let msg_reaction = ServerMessage::Reaction { sender_id: "123".to_string(), emoji: "👍".to_string() };
    let json_reaction = serde_json::to_string(&msg_reaction).unwrap();
    let deserialized_reaction: ServerMessage = serde_json::from_str(&json_reaction).unwrap();
    assert_eq!(msg_reaction, deserialized_reaction);
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

    let msg_reaction = ClientMessage::Reaction("👍".to_string());
    let json_reaction = serde_json::to_string(&msg_reaction).unwrap();
    let deserialized_reaction: ClientMessage = serde_json::from_str(&json_reaction).unwrap();
    assert_eq!(msg_reaction, deserialized_reaction);
}

#[test]
fn test_room_config_serialization() {
    let config = RoomConfig::default();
    assert_eq!(config.is_recording, false);
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: RoomConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_poll_serialization() {
    let poll = Poll {
        id: "poll1".to_string(),
        question: "Color?".to_string(),
        options: vec![
            PollOption { id: 0, text: "Red".to_string(), votes: 0 },
            PollOption { id: 1, text: "Blue".to_string(), votes: 5 },
        ],
    };
    let msg = ClientMessage::CreatePoll(poll.clone());
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}
