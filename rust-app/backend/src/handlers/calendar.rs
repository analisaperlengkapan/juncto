use shared::ServerMessage;

pub fn handle_fetch_calendar() -> ServerMessage {
    let mock_events = vec![
        "Team Standup - 10:00 AM".to_string(),
        "Project Sync - 1:00 PM".to_string(),
        "1:1 with Manager - 3:30 PM".to_string(),
        "Release Planning - 4:00 PM".to_string(),
    ];
    ServerMessage::CalendarEvents(mock_events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_calendar() {
        let msg = handle_fetch_calendar();
        if let ServerMessage::CalendarEvents(events) = msg {
            assert!(events.len() >= 3);
            assert!(events.iter().any(|e| e.contains("Standup")));
        } else {
            panic!("Expected CalendarEvents message");
        }
    }
}
