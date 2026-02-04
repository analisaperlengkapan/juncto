use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileAttachment {
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub content_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomConfig {
    pub room_name: String,
    pub is_locked: bool,
    pub is_recording: bool,
    pub is_lobby_enabled: bool,
    pub max_participants: u32,
    pub host_id: Option<String>,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            room_name: "Default Room".to_string(),
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100,
            host_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserIdentity {
    pub id: String,
    pub display_name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrawAction {
    pub color: String,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub width: f64,
    #[serde(default)] // For backward compatibility if needed, though we are migrating fresh
    pub sender_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub user_id: String,
    pub content: String,
    pub recipient_id: Option<String>,
    pub timestamp: u64,
    #[serde(default)] // Default to None for backward compatibility during migration
    pub attachment: Option<FileAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub is_hand_raised: bool,
    pub is_sharing_screen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PollOption {
    pub id: u32,
    pub text: String,
    pub votes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Poll {
    pub id: String,
    pub question: String,
    pub options: Vec<PollOption>,
    #[serde(default)]
    pub voters: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    CreatePoll(Poll),
    Vote { poll_id: String, option_id: u32 },
    Join(String), // Display Name
    Chat { content: String, recipient_id: Option<String>, attachment: Option<FileAttachment> },
    ToggleRoomLock,
    ToggleRecording,
    UpdateProfile(String), // New Name
    Reaction(String), // Emoji
    ToggleRaiseHand,
    ToggleScreenShare,
    ToggleLobby,
    GrantAccess(String),
    DenyAccess(String),
    KickParticipant(String), // Target ID
    EndMeeting,
    CreateBreakoutRoom(String), // Room Name
    JoinBreakoutRoom(Option<String>), // Room ID (None for Main)
    Draw(DrawAction),
    Typing(bool),
    StartShareVideo(String), // URL
    StopShareVideo,
    Speaking(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BreakoutRoom {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    Chat { message: ChatMessage, room_id: Option<String> },
    PeerTyping { user_id: String, is_typing: bool, room_id: Option<String> },
    Kicked(String), // Target ID
    BreakoutRoomsList(Vec<BreakoutRoom>),
    ParticipantJoined(Participant),
    ParticipantLeft(String), // ID
    ParticipantList(Vec<Participant>),
    KnockingParticipant(Participant),
    KnockingParticipantLeft(String), // ID
    RoomUpdated(RoomConfig),
    ParticipantUpdated(Participant),
    Reaction { sender_id: String, emoji: String },
    PollCreated(Poll),
    PollUpdated(Poll),
    Draw(DrawAction),
    WhiteboardHistory(Vec<DrawAction>),
    ChatHistory(Vec<ChatMessage>),
    Welcome { id: String },
    Knocking,
    AccessGranted,
    AccessDenied,
    RoomEnded,
    VideoShared(String), // URL
    VideoStopped,
    PeerSpeaking { user_id: String, speaking: bool },
    Error(String),
}

#[cfg(test)]
mod tests;
