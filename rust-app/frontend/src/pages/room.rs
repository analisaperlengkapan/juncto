use crate::chat::Chat;
use crate::components_ui::always_on_top::AlwaysOnTop;
use crate::components_ui::breakout::BreakoutRooms;
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

#[component]
pub fn Room() -> impl IntoView {
    let params = use_params_map();
    let room_id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());

    let state = use_room_state();
    let (show_shared_video_dialog, set_show_shared_video_dialog) = create_signal(false);
    let (show_invite, set_show_invite) = create_signal(false);
    let (show_chat, set_show_chat) = create_signal(true);

    let invite_url = Signal::derive(move || {
        if let Some(window) = web_sys::window() {
            window.location().href().unwrap_or_default()
        } else {
            "".to_string()
        }
    });

    let leave_room = Callback::new(move |_| {
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
                    <PrejoinScreen on_join=state.join_meeting />
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
                            on_transfer_host=state.transfer_host
                        />
                        <div class="main-content" style="flex: 1; display: flex; flex-direction: column; background: #333; color: white;">
                            <BreakoutRooms
                                breakout_rooms=state.breakout_rooms
                                current_room_id=state.current_room_id
                                is_host=state.is_host
                                on_create=state.create_breakout_room
                                on_join=state.join_breakout_room
                            />
                            <div style="position: relative; flex: 1; width: 100%; height: 100%;">
                                <div class="video-container" style="display: flex; justify-content: center; align-items: center; height: 100%;">
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
                                            <div style="background: red; color: white; padding: 5px; border-radius: 4px; display: inline-block; margin-bottom: 10px;">
                                                "REC"
                                            </div>
                                        </Show>
                                        <AlwaysOnTop
                                            is_video_muted=Signal::derive(move || state.local_stream.get().is_none_or(|s| s.get_video_tracks().length() == 0))
                                            is_audio_muted=Signal::derive(move || state.is_muted.get())
                                            on_toggle_video=state.toggle_camera
                                            on_toggle_audio=state.toggle_mic
                                            on_leave=leave_room
                                        />
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
                                <Show when=move || state.show_whiteboard.get()>
                                    <Whiteboard
                                        on_draw=state.send_draw
                                        history=state.whiteboard_history
                                        my_id=state.my_id
                                    />
                                </Show>
                            </div>
                            <Toolbox
                                is_locked=state.is_locked
                                is_host=state.is_host
                                is_lobby_enabled=state.is_lobby_enabled
                                class="room-toolbox"
                                style="position: relative; z-index: 20;" // Ensure toolbox is above whiteboard
                                is_recording=state.is_recording
                                on_toggle_lock=state.toggle_lock
                                on_toggle_lobby=state.toggle_lobby
                                on_toggle_recording=state.toggle_recording
                                on_invite=Callback::new(move |_| set_show_invite.set(true))
                                on_toggle_chat=Callback::new(move |_| set_show_chat.update(|v| *v = !*v))
                                on_settings=Callback::new(move |_| state.set_show_settings.set(true))
                                on_polls=Callback::new(move |_| state.set_show_polls.set(true))
                                on_shortcuts=Callback::new(move |_| state.set_show_shortcuts.set(true))
                                on_speaker_stats=Callback::new(move |_| state.set_show_speaker_stats.set(true))
                                on_virtual_background=Callback::new(move |_| state.set_show_virtual_background.set(true))
                                on_feedback=Callback::new(move |_| state.set_show_feedback.set(true))
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
                                on_leave=leave_room
                                on_end_meeting=end_meeting_and_leave
                            />
                        </div>
                        <div style=move || if show_chat.get() { "display: block;" } else { "display: none;" }>
                            <Chat
                                messages=state.messages
                                typing_users=state.typing_users
                                participants=state.participants
                                on_send=state.send_message
                                on_typing=state.set_is_typing
                                is_connected=state.is_connected
                                my_id=state.my_id
                            />
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
                            on_change=Callback::new(move |mode| {
                                web_sys::console::log_1(&format!("Background changed to: {}", mode).into());
                            })
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
