use leptos::*;

#[component]
pub fn Toolbox(
    is_locked: ReadSignal<bool>,
    is_host: Signal<bool>,
    is_lobby_enabled: ReadSignal<bool>,
    is_recording: ReadSignal<bool>,
    on_toggle_lock: Callback<()>,
    on_toggle_lobby: Callback<()>,
    on_toggle_recording: Callback<()>,
    on_invite: Callback<()>,
    on_toggle_chat: Callback<()>,
    on_settings: Callback<()>,
    on_polls: Callback<()>,
    on_shortcuts: Callback<()>,
    on_speaker_stats: Callback<()>,
    on_virtual_background: Callback<()>,
    on_feedback: Callback<()>,
    on_raise_hand: Callback<()>,
    on_screen_share: Callback<()>,
    on_share_video: Callback<()>,
    on_stop_share_video: Callback<()>,
    is_sharing_video: Signal<bool>,
    on_whiteboard: Callback<()>,
    on_reaction: Callback<String>,
    on_toggle_camera: Callback<()>,
    on_toggle_mic: Callback<()>,
    is_muted: ReadSignal<bool>,
    #[prop(optional)] on_leave: Option<Callback<()>>,
    #[prop(optional)] on_end_meeting: Option<Callback<()>>,
    #[prop(optional)] class: &'static str,
    #[prop(optional)] style: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!("toolbox bg-gray-900 border-t border-gray-800 p-3 flex flex-wrap items-center justify-center gap-2 sm:gap-4 {}", class) style=style>

            // Primary Controls (Left)
            <div class="flex items-center space-x-2 mr-auto bg-gray-800 px-3 py-1.5 rounded-full border border-gray-700">
                <button
                    on:click=move |_| on_toggle_mic.call(())
                    class=move || format!("p-2.5 rounded-full flex items-center justify-center transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900 {}",
                        if is_muted.get() { "bg-red-500 hover:bg-red-600 text-white shadow-md shadow-red-900/50" } else { "bg-gray-700 hover:bg-gray-600 text-gray-200" })
                    title=move || if is_muted.get() { "Unmute" } else { "Mute" }
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"></path></svg>
                </button>
                <button
                    on:click=move |_| on_toggle_camera.call(())
                    class="p-2.5 rounded-full bg-gray-700 hover:bg-gray-600 text-gray-200 flex items-center justify-center transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900"
                    title="Toggle Camera"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"></path></svg>
                </button>
                <button
                    on:click=move |_| on_screen_share.call(())
                    class="p-2.5 rounded-full bg-blue-600 hover:bg-blue-700 text-white shadow-md shadow-blue-900/50 flex items-center justify-center transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900"
                    title="Share Screen"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"></path></svg>
                </button>
                <button
                    on:click=move |_| on_raise_hand.call(())
                    class="p-2.5 rounded-full bg-yellow-500 hover:bg-yellow-600 text-yellow-50 shadow-md shadow-yellow-900/30 flex items-center justify-center transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900"
                    title="Raise Hand"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 11.5V14m0-2.5v-6a1.5 1.5 0 113 0m-3 6a1.5 1.5 0 00-3 0v2a7.5 7.5 0 0015 0v-5a1.5 1.5 0 00-3 0m-6-3V11m0-5.5v-1a1.5 1.5 0 013 0v1m0 0V11m0-5.5a1.5 1.5 0 013 0v3m0 0V11"></path></svg>
                </button>
            </div>

            // Secondary Controls (Center)
            <div class="flex items-center space-x-2">
                <button
                    on:click=move |_| on_toggle_chat.call(())
                    class="px-3 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300 border border-gray-700 hover:border-gray-600 transition-colors duration-200 text-sm font-medium focus:outline-none"
                >
                    "Chat"
                </button>
                <button
                    on:click=move |_| on_invite.call(())
                    class="px-3 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300 border border-gray-700 hover:border-gray-600 transition-colors duration-200 text-sm font-medium focus:outline-none hidden sm:block"
                >
                    "Invite"
                </button>
                <button
                    on:click=move |_| on_whiteboard.call(())
                    class="px-3 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300 border border-gray-700 hover:border-gray-600 transition-colors duration-200 text-sm font-medium focus:outline-none hidden md:block"
                >
                    "Whiteboard"
                </button>

                // Grouping less frequent actions into a visually distinct block
                <div class="flex bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
                    <button on:click=move |_| on_polls.call(()) class="px-3 py-2 hover:bg-gray-700 text-gray-300 transition-colors border-r border-gray-700 text-sm font-medium hidden md:block">"Polls"</button>
                    <button on:click=move |_| on_speaker_stats.call(()) class="px-3 py-2 hover:bg-gray-700 text-gray-300 transition-colors text-sm font-medium hidden lg:block border-r border-gray-700">"Stats"</button>
                    <button on:click=move |_| on_virtual_background.call(()) class="px-3 py-2 hover:bg-gray-700 text-gray-300 transition-colors text-sm font-medium hidden lg:block border-r border-gray-700" title="Virtual Background">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path></svg>
                    </button>
                    <button on:click=move |_| on_shortcuts.call(()) class="px-3 py-2 hover:bg-gray-700 text-gray-300 transition-colors text-sm font-medium hidden md:block border-r border-gray-700" title="Keyboard Shortcuts">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                    </button>
                    <button on:click=move |_| on_feedback.call(()) class="px-3 py-2 hover:bg-gray-700 text-gray-300 transition-colors text-sm font-medium hidden md:block border-r border-gray-700" title="Feedback">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z"></path></svg>
                    </button>
                    <button on:click=move |_| on_settings.call(()) class="px-3 py-2 hover:bg-gray-700 text-gray-300 transition-colors text-sm font-medium" title="Settings">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
                    </button>
                </div>

                // Host specific controls
                <Show when=move || is_host.get() fallback=move || view! {
                    <div class="flex items-center space-x-1 ml-2 pl-2 border-l border-gray-700">
                        <div class=move || format!("p-2 rounded {}", if is_locked.get() { "text-red-400" } else { "text-green-400" }) title=move || if is_locked.get() { "Room Locked" } else { "Room Unlocked" }>
                            <Show when=move || is_locked.get() fallback=|| view! { <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z"></path></svg> }>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path></svg>
                            </Show>
                        </div>
                    </div>
                }>
                    <div class="flex items-center space-x-1 ml-2 pl-2 border-l border-gray-700">
                        <button
                            on:click=move |_| {
                                if is_sharing_video.get() {
                                    on_stop_share_video.call(());
                                } else {
                                    on_share_video.call(());
                                }
                            }
                            class=move || format!("p-2 rounded hover:bg-gray-700 transition-colors {}", if is_sharing_video.get() { "text-orange-500" } else { "text-gray-400" })
                            title=move || if is_sharing_video.get() { "Stop Video" } else { "Share Video" }
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                        </button>
                        <button
                            on:click=move |_| on_toggle_recording.call(())
                            class=move || format!("p-2 rounded hover:bg-gray-700 transition-colors {}", if is_recording.get() { "text-red-400" } else { "text-gray-400" })
                            title=move || if is_recording.get() { "Stop Recording" } else { "Start Recording" }
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"></path></svg>
                        </button>
                        <button
                            on:click=move |_| on_toggle_lobby.call(())
                            class=move || format!("p-2 rounded hover:bg-gray-700 transition-colors {}", if is_lobby_enabled.get() { "text-teal-400" } else { "text-gray-400" })
                            title=move || if is_lobby_enabled.get() { "Disable Lobby" } else { "Enable Lobby" }
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path></svg>
                        </button>
                        <button
                            on:click=move |_| on_toggle_lock.call(())
                            class=move || format!("p-2 rounded hover:bg-gray-700 transition-colors {}", if is_locked.get() { "text-red-400" } else { "text-green-400" })
                            title=move || if is_locked.get() { "Unlock Room" } else { "Lock Room" }
                        >
                            <Show when=move || is_locked.get() fallback=|| view! { <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z"></path></svg> }>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path></svg>
                            </Show>
                        </button>
                    </div>
                </Show>
            </div>

            // Reactions (Right)
            <div class="flex items-center space-x-1 bg-gray-800 rounded-full px-2 py-1 border border-gray-700 hidden lg:flex">
                <button on:click=move |_| on_reaction.call("👍".to_string()) class="p-1.5 hover:bg-gray-700 rounded-full text-lg transition-transform hover:scale-125 focus:outline-none">"👍"</button>
                <button on:click=move |_| on_reaction.call("👏".to_string()) class="p-1.5 hover:bg-gray-700 rounded-full text-lg transition-transform hover:scale-125 focus:outline-none">"👏"</button>
                <button on:click=move |_| on_reaction.call("😂".to_string()) class="p-1.5 hover:bg-gray-700 rounded-full text-lg transition-transform hover:scale-125 focus:outline-none">"😂"</button>
                <button on:click=move |_| on_reaction.call("❤️".to_string()) class="p-1.5 hover:bg-gray-700 rounded-full text-lg transition-transform hover:scale-125 focus:outline-none">"❤️"</button>
            </div>

            // End/Leave Controls (Far Right)
            <div class="ml-auto flex items-center space-x-2">
                <button
                    on:click=move |_| {
                        if let Some(cb) = on_leave {
                            cb.call(());
                        }
                    }
                    class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium shadow-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900"
                >
                    "Leave"
                </button>
                <Show when=move || is_host.get() fallback=|| ()>
                    <button
                        on:click=move |_| {
                            if let Some(cb) = on_end_meeting {
                                cb.call(());
                            }
                        }
                        class="px-4 py-2 bg-red-900 hover:bg-red-800 text-red-100 rounded-lg font-medium border border-red-700 shadow-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-900"
                    >
                        "End Meeting"
                    </button>
                </Show>
            </div>
        </div>
    }
}
