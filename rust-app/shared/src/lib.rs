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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PowerStatus {
    pub battery_level: f64,
    pub is_charging: bool,
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
    #[serde(default)]
    pub etherpad_url: Option<String>,
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
            etherpad_url: None,
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
    #[serde(default)]
    pub room_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[derive(Eq, Hash)]
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
    /// Whether this participant joined as a visitor (read-only mode).
    /// Currently always `false`; reserved for future visitor-role support.
    #[serde(default)]
    pub is_visitor: bool,
    /// Whether this participant has end-to-end encryption enabled locally.
    /// Updated via `ClientMessage::UpdateE2EE`; the frontend does not yet
    /// expose a UI toggle for this field.
    #[serde(default)]
    pub e2ee_enabled: bool,
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
    #[serde(default)]
    pub is_closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    CreatePoll(Poll),
    Vote {
        poll_id: String,
        option_id: u32,
    },
    Join {
        name: String,
        #[serde(default)]
        is_visitor: bool,
    },
    Chat {
        content: String,
        recipient_id: Option<String>,
        attachment: Option<FileAttachment>,
    #[serde(default)]
    room_id: Option<String>,
    },
    ToggleRoomLock,
    ToggleRecording,
    UpdateProfile(String), // New Name
    Reaction(String),      // Emoji
    ToggleRaiseHand,
    ToggleScreenShare,
    ToggleLobby,
    ToggleE2EE,
    SetEtherpadUrl(Option<String>),
    GiphyShare(String),
    GrantAccess(String),
    DenyAccess(String),
    KickParticipant(String), // Target ID
    MuteParticipant(String), // Target ID
    TransferHost(String),    // Target ID
    SetMuteStatus(bool),
    EndMeeting,
    MuteCameraParticipant(String), // Target ID
    MuteCameraAll,
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
    MuteAll,
    UpdatePowerStatus(PowerStatus),
    RequestUnmute(String), // Target ID
    ToggleLocalRecording(bool),
    FollowMe(String), // Layout name (e.g., "grid", "spotlight")
    ClosePoll(String), // Poll ID
    /// Update this participant's per-user E2EE status. The server handler
    /// exists (`ws.rs: UpdateE2EE`) but no frontend UI sends this message yet.
    /// Kept for protocol completeness; wire up when the E2EE settings panel
    /// is migrated.
    UpdateE2EE(bool),
    BroadcastToLobby(String),
    PromoteVisitor(String),
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
    Authenticate {
        username: String,
        password: Option<String>,
    },
    FetchCalendar,
    AnalyticsEvent {
        name: String,
        properties: String,
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
    #[serde(default)]
    room_id: Option<String>,
    },
    PeerTyping {
        user_id: String,
        is_typing: bool,
    #[serde(default)]
    room_id: Option<String>,
    },
    Kicked { target_id: String, room_id: Option<String> },
    MutedByHost(String), // Target ID (Broadcasted, filtered by client)
    BreakoutRoomsList(Vec<BreakoutRoom>),
    ParticipantJoined(Participant),
    ParticipantLeft {
        id: String,
    #[serde(default)]
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
    EtherpadUrlUpdated { url: Option<String>, room_id: Option<String> },
    GiphyShared { url: String, sender_id: String, room_id: Option<String> },
    RoomEnded,
    VideoShared(String), // URL
    VideoStopped,
    PowerStatusUpdated {
        user_id: String,
        status: PowerStatus,
    },
    UnmuteRequested {
        requester_id: String,
        target_id: String,
    },
    RecordingStatusChanged {
        user_id: String,
        is_recording: bool,
    },
    PeerSpeaking {
        user_id: String,
        speaking: bool,
    },
    Pong {
        timestamp: u64,
    },
    Transcription {
        user_id: String,
        text: String,
        timestamp: u64,
    },
    FollowMe(String), // Layout name
    PollClosed(String), // Poll ID
    LobbyAnnouncement(String),
    CameraMutedByHost(String), // Target ID
    VisitorPromoted(String), // ID
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
    AuthenticationResult(bool),
    CalendarEvents(Vec<String>),
    Error(String),
}

/// Returns `true` if the given URL belongs to a known Giphy CDN domain.
/// Used on both the backend (chat validation, WS handler) and the frontend
/// (chat rendering) to enforce a consistent allowlist.
///
/// Recognised CDN hosts: `media.giphy.com`, `media0`–`media9.giphy.com`,
/// and `i.giphy.com`.  The check uses `starts_with` with a trailing `/`
/// to prevent authority-based bypasses (e.g. `https://media.giphy.com@evil.com`).
pub fn is_giphy_cdn_url(url: &str) -> bool {
    if let Some(rest) = url.strip_prefix("https://media") {
        // Matches "media.giphy.com/" or "media0.giphy.com/" … "media9.giphy.com/"
        if rest.starts_with(".giphy.com/") {
            return true;
        }
        if let Some(after_digit) = rest.strip_prefix(|c: char| c.is_ascii_digit()) {
            return after_digit.starts_with(".giphy.com/");
        }
        false
    } else {
        url.starts_with("https://i.giphy.com/")
    }
}

#[cfg(test)]
mod tests;
