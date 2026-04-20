import sys

filepath = 'rust-app/frontend/src/state.rs'
with open(filepath, 'r') as f:
    lines = f.readlines()

# Add import
if 'use crate::state_handlers::{handle_server_message, HandlerContext};' not in lines:
    lines.insert(1, 'use crate::state_handlers::{handle_server_message, HandlerContext};\n')

start_line = -1
for i, line in enumerate(lines):
    if 'match server_msg {' in line and i > 600:
        start_line = i
        break

if start_line == -1:
    print("Could not find start of match block")
    sys.exit(1)

# Find the end of the match block.
end_line = -1
for i in range(start_line, len(lines)):
    if 'ServerMessage::GiphyShared' in lines[i]:
        # Found the last variant. Now look for the closing braces of the match.
        # Check a few lines ahead for the pattern.
        for j in range(i, i + 10):
            if '}' in lines[j] and '}' in lines[j+1] and '}' in lines[j+2]:
                 # match, if, closure
                 end_line = j
                 break
        if end_line != -1:
            break

if end_line == -1:
    print("Could not find end of match block")
    sys.exit(1)

new_code = """                        let ctx = HandlerContext {
                            set_my_id,
                            set_current_state,
                            analytics: analytics.clone(),
                            start_media_on_join,
                            initial_cam_on,
                            start_media_stream,
                            set_start_media_on_join,
                            is_muted,
                            ws,
                            local_stream,
                            raw_local_stream,
                            add_toast: Callback::new(move |(msg, t)| add_toast(msg, t)),
                            set_is_camera_off,
                            room_config,
                            set_show_etherpad,
                            set_is_locked,
                            set_is_e2ee_enabled,
                            is_recording,
                            set_is_recording,
                            set_is_lobby_enabled,
                            is_subtitles_enabled,
                            set_subtitles,
                            set_room_config,
                            current_room_id,
                            set_messages,
                            set_is_connected,
                            set_knocking_participants,
                            set_participants,
                            my_id,
                            webrtc_manager: webrtc_manager.clone(),
                            set_typing_users,
                            set_speaking_peers,
                            set_power_statuses,
                            set_remote_streams,
                            is_recording_locally,
                            local_recorder: local_recorder_for_cleanup.clone(),
                            pending_recorders: pending_recorders_for_cleanup.clone(),
                            recording_stream_id: recording_stream_id_for_cleanup.clone(),
                            set_is_recording_locally,
                            set_is_muted,
                            set_audio_monitor,
                            participants,
                            set_last_reaction,
                            set_breakout_rooms,
                            set_polls,
                            set_grid_layout_sig,
                            set_whiteboard_history,
                            set_last_draw_action,
                            set_shared_video_url,
                            last_ping_time,
                            set_rtt,
                            set_is_authenticated,
                            set_show_login_dialog,
                            set_auth_error,
                            set_calendar_events,
                            set_lobby_announcement,
                        };
                        handle_server_message(server_msg, &ctx);
"""

new_lines = lines[:start_line] + [new_code] + lines[end_line+1:]

with open(filepath, 'w') as f:
    f.writelines(new_lines)

print(f"Successfully replaced lines {start_line+1} to {end_line+1}")
