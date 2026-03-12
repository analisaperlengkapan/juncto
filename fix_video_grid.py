with open('rust-app/frontend/src/components_ui/video_grid.rs', 'r') as f:
    content = f.read()

# Add the derived signal
derived_signal_code = """
                            let p_id_for_presence = p.id.clone();
                            let is_connected = Signal::derive(move || {
                                participants.with(|ps| {
                                    ps.iter()
                                        .find(|pp| pp.id == p_id_for_presence)
                                        .map(|pp| pp.presence == shared::PresenceStatus::Connected)
                                        .unwrap_or(false)
                                })
                            });
"""

# Find where to insert it (near p_name and is_hand_raised)
content = content.replace('let is_hand_raised = Signal::derive(move || {', derived_signal_code + '\n                            let is_hand_raised = Signal::derive(move || {')

# Replace the specific Show condition
content = content.replace('<Show when=move || p.presence == shared::PresenceStatus::Connected>', '<Show when=move || is_connected.get()>')

with open('rust-app/frontend/src/components_ui/video_grid.rs', 'w') as f:
    f.write(content)
