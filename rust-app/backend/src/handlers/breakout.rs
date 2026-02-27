use crate::AppState;
use shared::{BreakoutRoom, ServerMessage, Participant};
use std::sync::Arc;

pub fn create_breakout_room(
    user_id: &str,
    name: String,
    state: &Arc<AppState>,
) -> Result<ServerMessage, String> {
    let is_host = {
        let config = state.room_config.lock().unwrap();
        config.host_id.as_deref() == Some(user_id)
    };

    if !is_host {
        return Err("Only host can create breakout rooms".to_string());
    }

    let id = uuid::Uuid::new_v4().to_string();
    let room = shared::BreakoutRoom {
        id: id.clone(),
        name,
    };

    {
        let mut rooms = state.breakout_rooms.lock().unwrap();
        rooms.insert(id, room);
    }

    let all_rooms: Vec<shared::BreakoutRoom> = {
        let rooms = state.breakout_rooms.lock().unwrap();
        rooms.values().cloned().collect()
    };

    Ok(ServerMessage::BreakoutRoomsList(all_rooms))
}

pub fn join_breakout_room(
    user_id: &str,
    room_id: Option<String>,
    state: &Arc<AppState>,
) -> Result<(Option<String>, Vec<ServerMessage>), String> {
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

    // Send updated participant list for the new room context
    // Filter participants by room
    let participants: Vec<Participant> = {
        let all_participants = state.participants.lock().unwrap();
        let locations = state.participant_locations.lock().unwrap();

        all_participants.values().filter(|p| {
             let loc = locations.get(&p.id).cloned().flatten();
             loc == room_id
        }).cloned().collect()
    };
    messages.push(ServerMessage::ParticipantList(participants));

    Ok((room_id, messages))
}
