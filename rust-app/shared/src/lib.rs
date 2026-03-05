use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feedback {
    pub stars: u8, // 1-5
    pub comment: String,
    pub user_id: Option<String>,
}

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
    #[serde(default)]
    pub e2ee_enabled: bool,
    #[serde(default)]
    pub is_subtitles_enabled: bool,
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
            e2ee_enabled: false,
            is_subtitles_enabled: false,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum PresenceStatus {
    #[default]
    Connected,
    Disconnected,
    Busy,
    Calling,
    Ringing,
    Rejected,
    Ignored,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub is_hand_raised: bool,
    pub is_sharing_screen: bool,
    #[serde(default)]
    pub is_muted: bool,
    #[serde(default)]
    pub speaking_time: u64, // Total milliseconds spoken
    #[serde(default)]
    pub presence: PresenceStatus,
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
    Vote {
        poll_id: String,
        option_id: u32,
    },
    Join(String), // Display Name
    Chat {
        content: String,
        recipient_id: Option<String>,
        attachment: Option<FileAttachment>,
    },
    ToggleRoomLock,
    ToggleRecording,
    UpdateProfile(String), // New Name
    Reaction(String),      // Emoji
    ToggleRaiseHand,
    ToggleScreenShare,
    ToggleLobby,
    GrantAccess(String),
    DenyAccess(String),
    KickParticipant(String), // Target ID
    MuteParticipant(String), // Target ID
    TransferHost(String),    // Target ID
    SetMuteStatus(bool),
    EndMeeting,
    SetPresence(PresenceStatus),
    CreateBreakoutRoom(String),       // Room Name
    JoinBreakoutRoom(Option<String>), // Room ID (None for Main)
    Draw(DrawAction),
    ToggleSubtitles,
    Typing(bool),
    StartShareVideo(String), // URL
    StopShareVideo,
    Speaking(bool),
    Ping,
    // WebRTC Signaling
    Offer {
        target_id: String,
        sdp: String,
    },
    Answer {
        target_id: String,
        sdp: String,
    },
    IceCandidate {
        target_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BreakoutRoom {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    Chat {
        message: ChatMessage,
        room_id: Option<String>,
    },
    PeerTyping {
        user_id: String,
        is_typing: bool,
        room_id: Option<String>,
    },
    Kicked { target_id: String, room_id: Option<String> },
    MutedByHost(String), // Target ID (Broadcasted, filtered by client)
    BreakoutRoomsList(Vec<BreakoutRoom>),
    ParticipantJoined(Participant),
    ParticipantLeft {
        id: String,
        room_id: Option<String>,
    },
    ParticipantList(Vec<Participant>),
    KnockingParticipant(Participant),
    KnockingParticipantLeft(String), // ID
    RoomUpdated(RoomConfig),
    ParticipantUpdated(Participant),
    Reaction {
        sender_id: String,
        emoji: String,
    },
    PollCreated(Poll),
    PollUpdated(Poll),
    PollsList(Vec<Poll>),
    Draw(DrawAction),
    WhiteboardHistory(Vec<DrawAction>),
    ChatHistory(Vec<ChatMessage>),
    Welcome {
        id: String,
    },
    Knocking,
    AccessDenied,
    RoomEnded,
    VideoShared(String), // URL
    VideoStopped,
    PeerSpeaking {
        user_id: String,
        speaking: bool,
    },
    Pong {
        timestamp: u64,
    },
    // WebRTC Signaling
    Offer {
        source_id: String,
        target_id: String,
        sdp: String,
    },
    Answer {
        source_id: String,
        target_id: String,
        sdp: String,
    },
    IceCandidate {
        source_id: String,
        target_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
    Error(String),
}

#[cfg(test)]
mod tests;
