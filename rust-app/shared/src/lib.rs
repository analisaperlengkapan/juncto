use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomConfig {
    pub room_name: String,
    pub is_locked: bool,
    pub is_recording: bool,
    pub max_participants: u32,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            room_name: "Default Room".to_string(),
            is_locked: false,
            is_recording: false,
            max_participants: 100,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub user_id: String,
    pub content: String,
    pub timestamp: u64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    CreatePoll(Poll),
    Vote { poll_id: String, option_id: u32 },
    Join(String), // Display Name
    Chat(String), // Content
    ToggleRoomLock,
    ToggleRecording,
    UpdateProfile(String), // New Name
    Reaction(String), // Emoji
    ToggleRaiseHand,
    ToggleScreenShare,
    Draw(DrawAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    Chat(ChatMessage),
    ParticipantJoined(Participant),
    ParticipantLeft(String), // ID
    ParticipantList(Vec<Participant>),
    RoomUpdated(RoomConfig),
    ParticipantUpdated(Participant),
    Reaction { sender_id: String, emoji: String },
    PollCreated(Poll),
    PollUpdated(Poll),
    Draw(DrawAction),
    WhiteboardHistory(Vec<DrawAction>),
}

#[cfg(test)]
mod tests;
