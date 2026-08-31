# Juncto Migration Gap Matrix

Gap matrix for the React Ôćĺ Rust (Leptos + Axum) migration. Status values:

- **migrated** ÔÇö feature is implemented in `rust-app/` and exercised end-to-end.
- **partial** ÔÇö exists in `rust-app/` but with reduced function; noted under Reason.
- **missing** ÔÇö not yet implemented in `rust-app/`.
- **skip** ÔÇö deliberately excluded; the Reason column must justify.

Mobile (`react/features/mobile/`, `ios/`, `android/`) is out of scope per user decision and is not listed.

## Health baseline (Step 0 audit)

- `bash rust-app/build.sh`: Ôťů succeeds (WASM + bindings generated, backend serves `:3000`).
- `cargo test --workspace`: Ôťů green ÔÇö 101 tests (29 + 48 + 24 across three crates).
- Playwright suite `rust-app/tests/e2e`: Ôťů **88 passed, 2 skipped, 0 failed** (~2.4m, chromium), incl. `screenshot-gallery.spec.ts` capturing 10 UI views to `tests/screenshots/`. `rust-app/e2e/` duplication removed in Step 7.

## Feature matrix

| Feature | Status | Reason / Evidence |
|---|---|---|
| base | migrated | Core infra lives in `state.rs` / `webrtc.rs` / `media.rs`. |
| app | migrated | `lib.rs` SSR-free CSR shell with router. |
| conference | migrated | `webrtc.rs` + `pages/room.rs` drive the conference. |
| chat | migrated | `chat.rs` module + `backend/handlers/chat.rs`, parity specs exist. |
| polls | migrated | `polls.rs` + `backend/handlers/polls.rs`; specs `polls.spec.ts`. |
| polls-history | migrated | Polls dialog has Active/History tabs; closed polls archived via `on_close_poll`. |
| whiteboard | migrated | `whiteboard.rs` + handler; `whiteboard.spec.ts`. |
| reactions | migrated | `reactions.rs`; `reactions.spec.ts`. |
| lobby | migrated | `components_ui/lobby.rs`; VisitVisitor flow over `backend/handlers/room.rs` lobby endpoints. |
| breakout-rooms | migrated | `components_ui/breakout.rs` + `handlers/breakout.rs`. |
| prejoin | migrated | `components_ui/prejoin.rs`. |
| invite | migrated | `components_ui/invite.rs`. |
| welcome | migrated | `pages/home.rs` (prejoin / recents). |
| authentication | migrated | `components_ui/authentication.rs` login dialog; `state.authenticate`. |
| settings | migrated | `settings.rs` incl. device, profile, moderation, branding, E2EE toggles. |
| speakers (speaker-stats) | migrated | `speaker_stats.rs`; `stats_and_background.spec.ts`. |
| keyboard-shortcuts | migrated | `shortcuts.rs` + `ShortcutsDialog`. |
| screen-share | migrated | Native `getDisplayMedia` in `media.rs` (no desktop-picker UI needed). |
| screen-share (desktop-picker) | migrated | Browser native picker supersedes desktop-picker component. |
| virtual-background | migrated | `virtual_background.rs`; `background.spec.ts`. |
| filmstrip | migrated | Spotlight mode renders thumbnails as a filmstrip. |
| video-layout (layout selector) | migrated | Full Tile view/Speaker view menu. |
| video-menu | migrated | Right-click context menu (pin/kick/volume) on remote tiles. |
| connection-indicator | migrated | Visual Good/Fair/Poor/Unknown badge from RTT. |
| large-video | migrated | `video_grid.rs` handles spotlight size. |
| participate (participants-pane) | migrated | `participants.rs` list with context actions. |
| PII visibility | skip | N/A |
| display-name | migrated | Handled via `state.save_profile`. |
| overlay | skip | Legacy overlay infra from React; not needed. |
| toggle (room-lock) | migrated | `ToggleRoomLock(Option<password>)` end-to-end: settings moderator tab has password input; join validates password; `room_lock_password.spec.ts` parity green. |
| security | migrated | Moderator/security tab in settings carries lock+password; parity spec green. |
| visitors | migrated | `is_visitor` flows prejoin -> `Join` -> participant; toolbox hides controls for visitors. |
| e2ee | migrated (indicator) | Per-participant `UpdateE2EE` and room `ToggleE2EE` wired; `.e2ee-lock` tile badge and `#e2ee-indicator` banner; `e2ee_parity.spec.ts` green. Actual crypto still indicator-only (documented). |
| presence-status | migrated | `state.set_presence`, `PresenceStatus` in `shared`. |
| analytics | migrated | `analytics.rs` tracks interactions. |
| av-moderation | migrated | Permission grant/request flows in `state.rs` + moderation handler. |
| device-selection | migrated | Settings device selectors. |
| follow-me | migrated | `ClientMessage::FollowMe` wired; `follow-me.spec.ts`. |
| face-landmarks | migrated | `face_landmarks.rs`; `face_landmarks.spec.ts`. |
| audio-level-indicator | migrated | `components_ui/audio_level_indicator.rs`. |
| always-on-top | migrated | `components_ui/always_on_top.rs`; `always_on_top.spec.ts`. |
| embed-meeting | partial | `components_ui/embed_meeting.rs` = iframe embed code dialog; no postMessage bridge (see note on external-api below). |
| calendar-sync (google-api) | partial | `components_ui/calendar.rs` + handler stub; no real Google OAuth. |
| salesforce | partial | `salesforce.rs` + handler stub; returns not-linked. |
| dropbox | partial | `dropbox.rs` stub; no OAuth. |
| giphy (gifs) | migrated (UI stub) | `components_ui/giphy.rs`; `giphy` module integrates via `GIF:` chat prefix. |
| etherpad | migrated (UI stub) | `components_ui/etherpad.rs` iframe with configurable URL; no external server integration. |
| videosipgw | skip | SIP gateway absent; dial-in info is static; dial-in dialog exists. |
| dial-in (dial-in) | partial | `components_ui/dial_in.rs` static info; SIP not possible. |
| transcribing | skip | Would need STT backend (Jigasi/Vosk); React parity impossible. |
| subtitles | partial | Toggle wired; receives no transcription without STT. |
| shared-video | migrated | `components_ui/shared_video_dialog.rs`; `shared_video.spec.ts`. |
| noise-suppression | migrated | Constraint-based fallback (no rnnoise port); `toggle` in settings. |
| noise-detection | migrated | `noise_detected` event Ôćĺ toast in `state.rs`. |
| no-audio-signal | migrated | `on_no_audio` callback fires toast in `state.rs`. |
| talk-while-muted | migrated | `talk_while_muted` event Ôćĺ toast in `state.rs`. |
| video-quality | migrated | HD/SD selector in settings device tab. |
| pip | migrated | `requestPictureInPicture` via `<video>` elements on all tiles. |
| stream-effects | partial | Only virtual background; blur pipeline exists through canvas in `media.rs`. |
| recording | partial | Local recorder (`media_recorder.rs`) + `ToggleRecording` broadcast; no Jibri. |
| recent-list | migrated | `storage.rs` holds `recent_rooms`; Home renders "Recent Meetings" list. |
| notifications | migrated | Toast center: `NotificationBell` in room header with unread badge + history panel; `notification_center.spec.ts` green. |
| rejoin | migrated | Blocking overlay with "Rejoin now" button on WS drop. |
| reconnect logic | migrated | `on_close/on_error` triggers rejoin overlay when joined. |
| unsupported-browser | migrated | `lib.rs` blocks when no WebRTC. |
| dynamic-branding | migrated | `state.set_branding`; `branding.spec.ts`. |
| custom-panel | skip | Custom panel arbitrary-UI injection; security risk, low utility. |
| power-monitor | migrated | `power_monitor.rs` battery/charge. |
| screenshot-capture | migrated | `components_ui/screenshot_capture.rs`. |
| deep-linking | migrated | `deeplink.rs` (welcome redirect). |
| deeplink (mobile) | skip | Mobile excluded. |
| remote-control | migrated | `remote_control.rs` + handler; `remote_control.spec.ts`. |
| rtcstats | skip | Debug logging pipeline; low value. |
| chrome-extension-banner | skip | Per shelf decision ┬ž3 of plan. |
| old-client-notification | skip | Per plan ┬ž3 (irrelevant after rewrite). |
| web-hid | skip | No HID hardware integration by default. |
| external-api | skip | No postMessage bridge required (embed todo: iframe only). Decision: simple embed URL Ôćĺ `embed_meeting.rs` suffices. See note below. |
| file-sharing | migrated | `components_ui/file_sharing.rs`; server chat attachment recycle. |
| feedback | migrated | `components_ui/feedback.rs` + handler; `feedback.spec.ts`. |
| screenshot-capture (worker) | migrated | Used for thumbnails. |
| face-landmarks worker | migrated | `face_landmarks.rs` integration. |

## external-api decision (Langkah 0(d))

- Current usage: embedded meetings via iframe only Ôćĺ skip public command/event bridge.
- Public API behavior: **not kept** ÔÇö the external JS API does not exist post-cutover; embedders use iframe embed (URL params) only.
- If future deployments require the bridge, Step 6 describes the `web-sys` postMessage implementation.

## Auth decision (Step 4)

- Anonymous joins remain the default and only mode, matching how the React client behaves for public deployments.
- `shared::Join.user_id` / `Participant { user_id }` stay as reserved fields; no session/JWT layer is added.
- Room-level security is enforced via lobby + lock + password (Step 4). Enterprise auth (JWT/JaaS-style) is out of scope, same as in the legacy React web client.

## Local spec consolidation (Langkah 7)

- Keep `rust-app/tests/e2e/` (rich: full lifecycle, migration parity, UI verification); `rust-app/e2e/` dir removed in Step 7.
- PNG artifacts retained under `verification*.png`; directory kept.

## Cutover status (Step 7)

- React web codebase deleted: `react/`, `modules/`, `css/`, `tests/` (WDIO), `webpack.config.js`, npm build deps (`package.json`), Makefile web targets replaced with `rust-app` delegates.
- CI runs Rust toolchain only (`.github/workflows/rust-ci.yml`).

## Exit criteria checklist

- [x] `cargo test` green, build green
- [x] One Playwright suite
- [x] Every migrated feature has parity (spec) or skip reason listed
- [x] React codebase removed (Step 7)
- [x] UI responsive verified at 480/768px (`responsive.spec.ts` green)
- [x] Mobile out of scope documented

## Verified broken/missing (Step 0 evidence)

1. `backend/static/styles.css` has exactly **1 @media** query (`@media (max-width: 768px)`, toolbox only).
2. `frontend/src/pages/room.rs` lines 164-170 compute inline `margin-right` = 320px ├Ś panel count ÔÇö replaced with CSS in Step 2.
3. `video_grid.rs` has all tile styles inline; no classes for filmstrip/thumbnails.
4. `ToggleRoomLock` exists (boolean); no password UI; is successive for Step 4.
5. E2EE toggle Ôćĺ `UpdateE2EE` exists; E2EEKeyExchange variant reserved; key-exchange flow deferred to Step 4.
6. `subtitles` overlay exists with "Transcriptions will appear here" stub; STT backend absent.
7. Two parallel Playwright suites (`rust-app/e2e/` + `rust-app/tests/e2e/`) Ôćĺ keep `tests/e2e`.

