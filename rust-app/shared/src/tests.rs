use super::*;
#[test]
fn test_chat_message_serialization() {
    let msg = ChatMessage {
        user_id: "user1".to_string(),
        content: "Hello Rust".to_string(),
        recipient_id: None,
        timestamp: 1627840000,
        attachment: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}

#[test]
fn test_chat_message_with_attachment_serialization() {
    let attachment = FileAttachment {
        filename: "test.txt".to_string(),
        mime_type: "text/plain".to_string(),
        size: 12,
        content_base64: "SGVsbG8gV29ybGQ=".to_string(),
    };
    let msg = ChatMessage {
        user_id: "user1".to_string(),
        content: "Here is a file".to_string(),
        recipient_id: None,
        timestamp: 1627840000,
        attachment: Some(attachment),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
    assert!(deserialized.attachment.is_some());
    assert_eq!(deserialized.attachment.unwrap().filename, "test.txt");
}

#[test]
fn test_server_message_serialization() {
    let p = Participant {
        id: "123".to_string(),
        name: "Alice".to_string(),
        is_hand_raised: false,
        is_sharing_screen: false,
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

    let msg_rec = ClientMessage::ToggleRecording;
    let json_rec = serde_json::to_string(&msg_rec).unwrap();
    let deserialized_rec: ClientMessage = serde_json::from_str(&json_rec).unwrap();
    assert_eq!(msg_rec, deserialized_rec);

    let msg_prof = ClientMessage::UpdateProfile("Bob".to_string());
    let json_prof = serde_json::to_string(&msg_prof).unwrap();
    let deserialized_prof: ClientMessage = serde_json::from_str(&json_prof).unwrap();
    assert_eq!(msg_prof, deserialized_prof);

    let msg_reaction = ClientMessage::Reaction("👍".to_string());
    let json_reaction = serde_json::to_string(&msg_reaction).unwrap();
    let deserialized_reaction: ClientMessage = serde_json::from_str(&json_reaction).unwrap();
    assert_eq!(msg_reaction, deserialized_reaction);

    let msg_hand = ClientMessage::ToggleRaiseHand;
    let json_hand = serde_json::to_string(&msg_hand).unwrap();
    let deserialized_hand: ClientMessage = serde_json::from_str(&json_hand).unwrap();
    assert_eq!(msg_hand, deserialized_hand);

    let msg_screen = ClientMessage::ToggleScreenShare;
    let json_screen = serde_json::to_string(&msg_screen).unwrap();
    let deserialized_screen: ClientMessage = serde_json::from_str(&json_screen).unwrap();
    assert_eq!(msg_screen, deserialized_screen);

    let msg_end = ClientMessage::EndMeeting;
    let json_end = serde_json::to_string(&msg_end).unwrap();
    let deserialized_end: ClientMessage = serde_json::from_str(&json_end).unwrap();
    assert_eq!(msg_end, deserialized_end);
}

#[test]
fn test_room_config_serialization() {
    let config = RoomConfig::default();
    assert!(!config.is_recording);
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
        voters: std::collections::HashSet::new(),
    };
    let msg = ClientMessage::CreatePoll(poll.clone());
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}

#[test]
fn test_shared_video_messages() {
    let msg = ClientMessage::StartShareVideo("https://youtu.be/test".to_string());
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);

    let msg = ClientMessage::StopShareVideo;
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);

    let msg = ServerMessage::VideoShared("https://youtu.be/test".to_string());
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}

#[test]
fn test_speaking_message() {
    let msg = ClientMessage::Speaking(true);
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);

    let msg = ServerMessage::PeerSpeaking { user_id: "u1".to_string(), speaking: true };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}

#[test]
fn test_draw_serialization() {
    let draw = DrawAction {
        color: "#000000".to_string(),
        start_x: 10.0,
        start_y: 20.0,
        end_x: 30.0,
        end_y: 40.0,
        width: 2.0,
        sender_id: "user1".to_string(),
    };
    let msg = ClientMessage::Draw(draw.clone());
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
}
