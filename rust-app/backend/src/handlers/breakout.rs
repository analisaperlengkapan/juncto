use crate::AppState;
use shared::{ServerMessage, BreakoutRoom};
use std::sync::Arc;

pub fn create_breakout_room(user_id: &str, name: String, state: &Arc<AppState>) -> Result<ServerMessage, String> {
    let is_host = {
        let config = state.room_config.lock().unwrap();
        config.host_id.as_deref() == Some(user_id)
    };

    if !is_host {
        return Err("Only host can create breakout rooms".to_string());
    }

    let id = uuid::Uuid::new_v4().to_string();
    let room = BreakoutRoom {
        id: id.clone(),
        name,
    };

    {
        let mut rooms = state.breakout_rooms.lock().unwrap();
        rooms.insert(id, room);
    }

    let all_rooms: Vec<BreakoutRoom> = {
        let rooms = state.breakout_rooms.lock().unwrap();
        rooms.values().cloned().collect()
    };

    Ok(ServerMessage::BreakoutRoomsList(all_rooms))
}

pub fn join_breakout_room(user_id: &str, room_id: Option<String>, state: &Arc<AppState>) -> Result<(Option<String>, Vec<ServerMessage>), String> {
    // Check if room exists if not None
    if let Some(rid) = &room_id {
         let rooms = state.breakout_rooms.lock().unwrap();
         if !rooms.contains_key(rid) {
             return Err("Breakout room not found".to_string());
         }
    }

    {
        let mut locations = state.participant_locations.lock().unwrap();
        locations.insert(user_id.to_string(), room_id.clone());
    }

    let mut messages = Vec::new();

    // If joining Main Room (None), return chat history to be sent to self
    if room_id.is_none() {
        let history = {
            let history = state.chat_history.lock().unwrap();
            history.clone()
        };
        if !history.is_empty() {
            messages.push(ServerMessage::ChatHistory(history));
        }
    }

    Ok((room_id, messages))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::RoomConfig;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tokio::sync::broadcast;

    fn create_mock_state() -> Arc<AppState> {
        let (tx, _) = broadcast::channel(10);
        Arc::new(AppState {
            tx,
            participants: Arc::new(Mutex::new(HashMap::new())),
            knocking_participants: Arc::new(Mutex::new(HashMap::new())),
            room_config: Arc::new(Mutex::new(RoomConfig::default())),
            polls: Arc::new(Mutex::new(HashMap::new())),
            whiteboard: Arc::new(Mutex::new(Vec::new())),
            chat_history: Arc::new(Mutex::new(Vec::new())),
            breakout_rooms: Arc::new(Mutex::new(HashMap::new())),
            participant_locations: Arc::new(Mutex::new(HashMap::new())),
            shared_video_url: Arc::new(Mutex::new(None)),
            speaking_start_times: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    #[test]
    fn test_create_breakout_room_host() {
        let state = create_mock_state();
        let user_id = "host";
        {
            let mut config = state.room_config.lock().unwrap();
            config.host_id = Some(user_id.to_string());
        }

        let res = create_breakout_room(user_id, "Room A".to_string(), &state);
        assert!(res.is_ok());
        if let Ok(ServerMessage::BreakoutRoomsList(rooms)) = res {
            assert_eq!(rooms.len(), 1);
            assert_eq!(rooms[0].name, "Room A");
        } else {
            panic!("Wrong message type");
        }
    }

    #[test]
    fn test_join_breakout_room() {
        let state = create_mock_state();
        let user_id = "user1";
        let room_id = "room1";

        // Pre-populate room
        {
            let mut rooms = state.breakout_rooms.lock().unwrap();
            rooms.insert(room_id.to_string(), BreakoutRoom { id: room_id.to_string(), name: "A".to_string() });
        }

        let res = join_breakout_room(user_id, Some(room_id.to_string()), &state);
        assert!(res.is_ok());
        let (new_rid, msgs) = res.unwrap();
        assert_eq!(new_rid, Some(room_id.to_string()));
        assert!(msgs.is_empty()); // No chat history for breakout rooms

        // Verify location
        {
            let locs = state.participant_locations.lock().unwrap();
            assert_eq!(locs.get(user_id), Some(&Some(room_id.to_string())));
        }
    }

    #[test]
    fn test_join_main_room() {
        let state = create_mock_state();
        let user_id = "user1";

        // Add some chat history
        {
             let mut hist = state.chat_history.lock().unwrap();
             hist.push(shared::ChatMessage {
                 user_id: "other".to_string(),
                 content: "hi".to_string(),
                 recipient_id: None,
                 timestamp: 0,
                 attachment: None
             });
        }

        let res = join_breakout_room(user_id, None, &state);
        assert!(res.is_ok());
        let (new_rid, msgs) = res.unwrap();
        assert_eq!(new_rid, None);
        assert_eq!(msgs.len(), 1);
        if let ServerMessage::ChatHistory(h) = &msgs[0] {
            assert_eq!(h.len(), 1);
        } else {
            panic!("Expected ChatHistory");
        }
    }
}
