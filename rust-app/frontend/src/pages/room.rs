use leptos::*;
use leptos_router::*;
use crate::chat::Chat;
use crate::participants::ParticipantsList;
use crate::toolbox::Toolbox;
use crate::components_ui::prejoin::PrejoinScreen;
use crate::components_ui::lobby::LobbyScreen;
use crate::components_ui::breakout::BreakoutRooms;
use crate::components_ui::video_grid::VideoGrid;
use crate::settings::SettingsDialog;
use crate::reactions::ReactionDisplay;
use crate::polls::PollsDialog;
use crate::whiteboard::Whiteboard;
use crate::state::{use_room_state, RoomConnectionState};

#[component]
pub fn Room() -> impl IntoView {
    let params = use_params_map();
    let room_id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());

    let state = use_room_state();

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
                        <ParticipantsList
                            participants=state.participants
                            knocking_participants=state.knocking_participants
                            on_allow=state.grant_access
                            on_deny=state.deny_access
                            on_kick=state.kick_participant
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
                                        <h2>"Meeting Room: " {room_id}</h2>
                                        <Show when=move || state.current_room_id.get().is_some()>
                                            <h4 style="color: #17a2b8;">" (In Breakout Room)"</h4>
                                        </Show>
                                        <Show when=move || state.is_recording.get()>
                                            <div style="background: red; color: white; padding: 5px; border-radius: 4px; display: inline-block; margin-bottom: 10px;">
                                                "REC"
                                            </div>
                                        </Show>
                                        <VideoGrid
                                            participants=state.participants
                                            local_stream=state.local_stream
                                            my_id=state.my_id
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
                                on_settings=Callback::new(move |_| state.set_show_settings.set(true))
                                on_polls=Callback::new(move |_| state.set_show_polls.set(true))
                                on_raise_hand=state.toggle_raise_hand
                                on_screen_share=state.toggle_screen_share
                                on_whiteboard=Callback::new(move |_| state.set_show_whiteboard.update(|v| *v = !*v))
                                on_reaction=state.send_reaction
                                on_toggle_camera=state.toggle_camera
                            />
                        </div>
                        <Chat
                            messages=state.messages
                            typing_users=state.typing_users
                            participants=state.participants
                            on_send=state.send_message
                            on_typing=state.set_is_typing
                            is_connected=state.is_connected
                            my_id=state.my_id
                        />
                        <SettingsDialog
                            show=state.show_settings
                            on_close=Callback::new(move |_| state.set_show_settings.set(false))
                            on_save_profile=state.save_profile
                        />
                        <PollsDialog
                            show=state.show_polls
                            polls=state.polls
                            on_close=Callback::new(move |_| state.set_show_polls.set(false))
                            on_create_poll=state.create_poll
                            on_vote=state.vote_poll
                        />
                    </div>
                }.into_view()
            }}
        </div>
    }
}
