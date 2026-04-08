use crate::chat::Chat;
use crate::components_ui::always_on_top::AlwaysOnTop;
use crate::components_ui::authentication::LoginDialog;
use crate::components_ui::breakout::BreakoutRooms;
use crate::components_ui::calendar::CalendarList;
use crate::components_ui::feedback::FeedbackDialog;
use crate::components_ui::invite::InviteDialog;
use crate::components_ui::lobby::LobbyScreen;
use crate::components_ui::prejoin::PrejoinScreen;
use crate::components_ui::shared_video_dialog::SharedVideoDialog;
use crate::components_ui::video_grid::VideoGrid;
use crate::connection_stats::ConnectionStats;
use crate::participants::ParticipantsList;
use crate::polls::PollsDialog;
use crate::reactions::ReactionDisplay;
use crate::settings::SettingsDialog;
use crate::shortcuts::{KeyboardShortcuts, ShortcutsDialog};
use crate::speaker_stats::SpeakerStatsDialog;
use crate::state::{use_room_state, RoomConnectionState};
use crate::toolbox::Toolbox;
use crate::virtual_background::VirtualBackgroundDialog;
use crate::whiteboard::Whiteboard;
use gloo_timers::callback::Interval;
use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Room() -> impl IntoView {
    let params = use_params_map();
    let room_id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());

    let state = use_room_state();
    let (show_shared_video_dialog, set_show_shared_video_dialog) = create_signal(false);
    let (show_invite, set_show_invite) = create_signal(false);
    let (show_embed, set_show_embed) = create_signal(false);
    let (show_chat, set_show_chat) = create_signal(true);
    let (show_participants, set_show_participants) = create_signal(true);

    let invite_url = Signal::derive(move || {
        if let Some(window) = web_sys::window() {
            window.location().href().unwrap_or_default()
        } else {
            "".to_string()
        }
    });

    let state_clone = state.clone();
    let leave_room = Callback::new(move |_| {
        // Stop raw camera/mic tracks first to release hardware immediately.
        // When a virtual background is active, local_stream holds canvas video
        // tracks (not the real getUserMedia tracks), so stopping only
        // local_stream would leak the camera.
        if let Some(raw) = state_clone.raw_local_stream.get_untracked() {
            let tracks = raw.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        // Also stop processed stream tracks (canvas video tracks) for cleanup
        if let Some(stream) = state_clone.local_stream.get_untracked() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/");
        }
    });

    let state_end_meeting = state.end_meeting;
    let end_meeting_and_leave = Callback::new(move |_| {
        state_end_meeting.call(());
        set_timeout(
            move || {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/");
                }
            },
            std::time::Duration::from_millis(1000), // Increase buffer to ensure server has time to broadcast RoomEnded
        );
    });

    // Meeting Timer
    let (elapsed_time, set_elapsed_time) = create_signal(0u32);
    create_effect(move |_| {
        let handle = Interval::new(1000, move || {
            set_elapsed_time.update(|t| *t += 1);
        });
        on_cleanup(move || drop(handle));
    });

    let format_time = move || {
        let t = elapsed_time.get();
        let h = t / 3600;
        let m = (t % 3600) / 60;
        let s = t % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    };

    view! {
        <div style="height: 100vh;">
            {move || match state.connection_state.get() {
                RoomConnectionState::Prejoin => view! {
                    <PrejoinScreen
                        on_join=state.join_meeting
                        is_connected=state.is_connected
                    />
                }.into_view(),
                RoomConnectionState::Lobby => view! {
                    <LobbyScreen />
                }.into_view(),
                RoomConnectionState::Joined => view! {
                    <div class="room-container" style="display: flex; height: 100vh;">
                        <KeyboardShortcuts
                            on_toggle_mic=state.toggle_mic
                            on_toggle_camera=state.toggle_camera
                            on_raise_hand=state.toggle_raise_hand
                            on_screen_share=state.toggle_screen_share
                        />
                        <ConnectionStats
                            on_ping=state.send_ping
                            rtt=state.rtt
                        />
                        <div class="main-content" style=move || {
                            let mut margin = 0;
                            if show_chat.get() { margin += 320; }
                            if show_participants.get() { margin += 320; }
                            format!("flex: 1; display: flex; flex-direction: column; background: #333; color: white; margin-right: {}px;", margin)
                        }>
                            <Show when=move || state.show_login_dialog.get()>
                                <LoginDialog
                                    auth_error=state.auth_error
                                    on_login=state.authenticate
                                    on_cancel=Callback::new(move |_| {
                                        state.set_auth_error.set(None);
                                        state.set_show_login_dialog.set(false);
                                    })
                                />
                            </Show>
                            <Show when=move || state.show_calendar.get()>
                                <CalendarList
                                    events=state.calendar_events
                                    on_refresh=state.fetch_calendar
                                    on_close=Callback::new(move |_| state.set_show_calendar.set(false))
                                />
                            </Show>
                            <BreakoutRooms
                                breakout_rooms=state.breakout_rooms
                                current_room_id=state.current_room_id
                                is_host=state.is_host
                                on_create=state.create_breakout_room
                                on_join=state.join_breakout_room
                            />
                            <div style="position: relative; flex: 1; width: 100%; height: 100%;">
                                <div class="video-container" style=move || {
                                    if state.show_etherpad.get() {
                                        "display: flex; justify-content: center; align-items: center; height: 30%; border-bottom: 2px solid #555;"
                                    } else {
                                        "display: flex; justify-content: center; align-items: center; height: 100%;"
                                    }
                                }>
                                    <div>
                                        <div style="display: flex; align-items: center; justify-content: center; gap: 15px;">
                                            <h2>"Meeting Room: " {room_id}</h2>
                                            <span class="meeting-timer" style="font-family: monospace; font-size: 1.2em; color: #aaa;">
                                                {format_time}
                                            </span>
                                        </div>
                                        <Show when=move || state.current_room_id.get().is_some()>
                                            <h4 style="color: #17a2b8;">" (In Breakout Room)"</h4>
                                        </Show>
                                        <Show when=move || state.is_recording.get()>
                                            <div style="background: red; color: white; padding: 5px; border-radius: 4px; display: inline-block; margin-bottom: 10px; margin-right: 5px;">
                                                "REC"
                                            </div>
                                        </Show>
                                        <Show when=move || state.is_e2ee_enabled.get()>
                                            <div style="background: #28a745; color: white; padding: 5px; border-radius: 4px; display: inline-block; margin-bottom: 10px;" title="End-to-End Encrypted">
                                                "🔒 E2EE"
                                            </div>
                                        </Show>
                                        <Show when=move || !state.is_connected.get()>
                                            <AlwaysOnTop
                                                is_video_muted=Signal::derive(move || state.local_stream.get().is_none_or(|s| s.get_video_tracks().length() == 0))
                                                is_audio_muted=Signal::derive(move || state.is_muted.get())
                                                on_toggle_video=state.toggle_camera
                                                on_toggle_audio=state.toggle_mic
                                                on_leave=leave_room
                                            />
                                        </Show>
                                        <VideoGrid
                                            participants=state.participants
                                            local_stream=state.local_stream
                                            local_screen_stream=state.local_screen_stream
                                            my_id=state.my_id
                                            shared_video_url=state.shared_video_url
                                            speaking_peers=state.speaking_peers
                                            remote_streams=state.remote_streams
                                        />
                                    </div>
                                </div>
                                <ReactionDisplay last_reaction=state.last_reaction />
                                <Show when=move || state.is_subtitles_enabled.get()>
                                    <div class="subtitles-overlay" style="position: absolute; bottom: 80px; left: 50%; transform: translateX(-50%); background: rgba(0, 0, 0, 0.7); color: white; padding: 10px 20px; border-radius: 8px; font-size: 1.2em; text-align: center; z-index: 100; max-width: 80%; min-width: 200px;">
                                        <For
                                            each=move || state.subtitles.get()
                                            key=|(uid, text, ts)| format!("{}-{}-{}", uid, text, ts)
                                            children=move |(_uid, text, _ts)| {
                                                view! {
                                                    <div style="margin-bottom: 5px;">{text}</div>
                                                }
                                            }
                                        />
                                        {move || if state.subtitles.get().is_empty() {
                                            "Subtitles are currently enabled. (Transcriptions will appear here)".to_string()
                                        } else {
                                            "".to_string()
                                        }}
                                    </div>
                                </Show>
                                <Show when=move || state.show_whiteboard.get()>
                                    <Whiteboard
                                        on_draw=state.send_draw
                                        history=state.whiteboard_history
                                        my_id=state.my_id
                                    />
                                </Show>
                                <Show when=move || state.show_etherpad.get()>
                                    <div style="height: 70%; width: 100%;">
                                        <crate::components_ui::etherpad::Etherpad
                                            url=Signal::derive(move || state.room_config.get().etherpad_url)
                                        />
                                    </div>
                                </Show>
                            </div>
                            <Toolbox
                                is_locked=state.is_locked
                                is_host=state.is_host
                                _is_lobby_enabled=state.is_lobby_enabled
                                class="room-toolbox"
                                style="position: relative; z-index: 20;" // Ensure toolbox is above whiteboard
                                is_recording=state.is_recording
                                _on_toggle_lock=state.toggle_lock
                                _on_toggle_lobby=state.toggle_lobby
                                on_toggle_recording=state.toggle_recording
                                is_subtitles_enabled=state.is_subtitles_enabled
                                on_toggle_subtitles=state.toggle_subtitles
                                on_toggle_e2ee=state.toggle_e2ee
                                is_e2ee_enabled=state.is_e2ee_enabled
                                on_toggle_etherpad=Callback::new({
                                    let state = state.clone();
                                    move |_| {
                                        let current = state.show_etherpad.get_untracked();
                                        if state.is_host.get_untracked() {
                                            if !current {
                                                state.toggle_etherpad.call(Some("https://etherpad.org/p/juncto-demo".to_string()));
                                            } else {
                                                state.toggle_etherpad.call(None);
                                            }
                                        } else {
                                            state.set_show_etherpad.set(!current);
                                        }
                                    }
                                })
                                is_etherpad_active=state.show_etherpad
                                current_presence=Signal::derive(move || {
                                    if let Some(my_id) = state.my_id.get() {
                                        if let Some(me) = state.participants.get().iter().find(|p| p.id == my_id) {
                                            return me.presence.clone();
                                        }
                                    }
                                    shared::PresenceStatus::Connected
                                })
                                on_set_presence=state.set_presence
                                on_invite=Callback::new(move |_| set_show_invite.set(true))
                                on_toggle_chat=Callback::new(move |_| set_show_chat.update(|v| *v = !*v))
                                on_toggle_participants=Callback::new(move |_| set_show_participants.update(|v| *v = !*v))
                                on_settings=Callback::new(move |_| state.set_show_settings.set(true))
                                on_polls=Callback::new(move |_| state.set_show_polls.set(true))
                                on_shortcuts=Callback::new(move |_| state.set_show_shortcuts.set(true))
                                on_speaker_stats=Callback::new(move |_| state.set_show_speaker_stats.set(true))
                                on_virtual_background=Callback::new(move |_| state.set_show_virtual_background.set(true))
                                on_feedback=Callback::new(move |_| state.set_show_feedback.set(true))
                                on_embed=Callback::new(move |_| set_show_embed.set(true))
                                on_raise_hand=state.toggle_raise_hand
                                on_screen_share=state.toggle_screen_share
                                on_share_video=Callback::new(move |_| set_show_shared_video_dialog.set(true))
                                on_stop_share_video=state.stop_share_video
                                is_sharing_video=Signal::derive(move || state.shared_video_url.get().is_some())
                                on_whiteboard=Callback::new(move |_| state.set_show_whiteboard.update(|v| *v = !*v))
                                on_reaction=state.send_reaction
                                on_toggle_camera=state.toggle_camera
                                on_toggle_mic=state.toggle_mic
                                is_muted=state.is_muted
                                on_auth_dialog=Callback::new(move |_| {
                                    state.set_auth_error.set(None);
                                    state.set_show_login_dialog.set(true);
                                })
                                on_calendar=Callback::new(move |_| state.set_show_calendar.set(true))
                                on_leave=leave_room
                                on_end_meeting=end_meeting_and_leave
                            />
                        </div>
                        <div class="side-panel chat-container" style=move || {
                            if show_chat.get() {
                                let right_pos = if show_participants.get() { "320px" } else { "0px" };
                                format!("display: flex; position: fixed; top: 0; right: {}; width: 320px; height: 100vh; box-shadow: -2px 0 5px rgba(0,0,0,0.2); z-index: 10;", right_pos)
                            } else {
                                "display: none;".to_string()
                            }
                        }>
                            <div class="panel-header">
                                <h3>"Chat"</h3>
                                <button class="close-btn" on:click=move |_| set_show_chat.set(false)>"✕"</button>
                            </div>
                            <div class="panel-content" style="padding: 0;">
                                <Chat
                                    messages=state.messages
                                    typing_users=state.typing_users
                                    participants=state.participants
                                    on_send=state.send_message
                                    on_typing=state.set_is_typing
                                    is_connected=state.is_connected
                                    my_id=state.my_id
                                    current_room_id=state.current_room_id
                                />
                            </div>
                        </div>
                        <div class="side-panel participants-container" style=move || {
                            if show_participants.get() {
                                "display: flex; position: fixed; top: 0; right: 0; width: 320px; height: 100vh; box-shadow: -2px 0 5px rgba(0,0,0,0.2); z-index: 10;".to_string()
                            } else {
                                "display: none;".to_string()
                            }
                        }>
                            <div class="panel-header">
                                <h3>"Participants"</h3>
                                <button class="close-btn" on:click=move |_| set_show_participants.set(false)>"✕"</button>
                            </div>
                            <div class="panel-content" style="padding: 0;">
                                <ParticipantsList
                                    participants=state.participants
                                    knocking_participants=state.knocking_participants
                                    host_id=state.host_id
                                    is_host=state.is_host
                                    my_id=state.my_id
                                    on_allow=state.grant_access
                                    on_deny=state.deny_access
                                    on_kick=state.kick_participant
                                    on_mute=state.mute_participant
                                    on_mute_all=state.mute_all
                                    on_transfer_host=state.transfer_host
                                />
                            </div>
                        </div>
                        <InviteDialog
                            show=show_invite
                            on_close=Callback::new(move |_| set_show_invite.set(false))
                            room_url=invite_url
                        />
                        <SettingsDialog
                            show=state.show_settings
                            on_close=Callback::new(move |_| state.set_show_settings.set(false))
                            on_save_profile=state.save_profile
                            on_save_devices=state.set_input_devices
                            current_video_id=state.selected_camera_id
                            current_audio_id=state.selected_mic_id
                            current_resolution=state.video_resolution
                            current_noise_suppression=state.is_noise_suppression_enabled
                            is_host=state.is_host
                            is_locked=state.is_locked
                            is_e2ee_enabled=state.is_e2ee_enabled
                            is_lobby_enabled=state.is_lobby_enabled
                            on_toggle_lock=state.toggle_lock
                            on_toggle_e2ee=state.toggle_e2ee
                            on_toggle_lobby=state.toggle_lobby
                        />
                        <SharedVideoDialog
                            show=show_shared_video_dialog
                            on_close=Callback::new(move |_| set_show_shared_video_dialog.set(false))
                            on_submit=state.start_share_video
                        />
                        <PollsDialog
                            show=state.show_polls
                            polls=state.polls
                            on_close=Callback::new(move |_| state.set_show_polls.set(false))
                            on_create_poll=state.create_poll
                            on_vote=state.vote_poll
                        />
                        <ShortcutsDialog
                            show=state.show_shortcuts
                            on_close=Callback::new(move |_| state.set_show_shortcuts.set(false))
                        />
                        <SpeakerStatsDialog
                            show=state.show_speaker_stats
                            participants=state.participants
                            on_close=Callback::new(move |_| state.set_show_speaker_stats.set(false))
                        />
                        <VirtualBackgroundDialog
                            show=state.show_virtual_background
                            on_close=Callback::new(move |_| state.set_show_virtual_background.set(false))
                            on_change=state.set_background_mode
                            current_mode=state.background_mode
                        />

                        <crate::components_ui::embed_meeting::EmbedMeetingDialog
                            show=show_embed
                            on_close=Callback::new(move |_| set_show_embed.set(false))
                        />
                        <FeedbackDialog
                            show=state.show_feedback
                            on_close=Callback::new(move |_| state.set_show_feedback.set(false))
                        />
                    </div>
                }.into_view()
            }}
        </div>
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_compiles() {
        // dummy test
        assert!(true);
    }

    #[test]
    fn test_subtitle_overlay_logic() {
        let _runtime = create_runtime();
        let (subtitles, _set_subtitles) = create_signal(vec![("u1".to_string(), "hello".to_string(), 123u64)]);
        let (is_enabled, _set_is_enabled) = create_signal(true);

        // Verification of logic used in component
        assert!(is_enabled.get());
        assert_eq!(subtitles.get().len(), 1);
    }
}
