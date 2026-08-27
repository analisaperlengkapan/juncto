# Juncto Migration Gap Matrix

Gap matrix for the React → Rust (Leptos + Axum) migration. Status values:

- **migrated** — feature is implemented in `rust-app/` and exercised end-to-end.
- **partial** — exists in `rust-app/` but with reduced function; noted under Reason.
- **missing** — not yet implemented in `rust-app/`.
- **skip** — deliberately excluded; the Reason column must justify.

Mobile (`react/features/mobile/`, `ios/`, `android/`) is out of scope per user decision and is not listed.

## Health baseline (Step 0 audit)

- `bash rust-app/build.sh`: ✅ succeeds (WASM + bindings generated, backend serves `:3000`).
- `cargo test --workspace`: ✅ green — 101 tests (29 + 48 + 24 across three crates).
- Playwright suite `rust-app/tests/e2e`: ✅ **77 passed, 2 skipped, 0 failed** (~1.9m, chromium). This becomes the consolidated suite in Step 7; `rust-app/e2e/` duplication is scheduled for removal.

## Feature matrix

| Feature | Status | Reason / Evidence |
|---|---|---|
| base | migrated | Core infra lives in `state.rs` / `webrtc.rs` / `media.rs`. |
| app | migrated | `lib.rs` SSR-free CSR shell with router. |
| conference | migrated | `webrtc.rs` + `pages/room.rs` drive the conference. |
| chat | migrated | `chat.rs` module + `backend/handlers/chat.rs`, parity specs exist. |
| polls | migrated | `polls.rs` + `backend/handlers/polls.rs`; specs `polls.spec.ts`. |
| polls-history | missing | Only active polls live in `state.polls`. |
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
| toggle (room-lock) | partial | `ToggleRoomLock` boolean; password prompt per Step 4 pending. |
| security | partial | Settings dialog has moderation/branding; dedicated security dialog pending. |
| visitors | partial | `is_visitor` flag wired; role logic restricted by Step 4. |
| e2ee | partial | Toggle wired to proto `UpdateE2EE`; indicator shows "indicator only — actual E2EE is not yet implemented"; activation deferred to Step 4. |
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
| noise-detection | missing | Not implemented; Step 3 adds it. |
| no-audio-signal | partial | Audio level zero → "no signal" toast still pending; Step 3. |
| talk-while-muted | partial | Detected (`is_talking_while_muted`); Step 3 wires the toast end. |
| video-quality | missing | No resolution selector UI; Step 3. |
| pip | partial | PiP buttons on video cards; dedicated toggle + Document PiP via Step 3. |
| stream-effects | partial | Only virtual background; blur pipeline exists through canvas in `media.rs`. |
| recording | partial | Local recorder (`media_recorder.rs`) + `ToggleRecording` broadcast; no Jibri. |
| recent-list | partial | `storage.rs` holds `recent_rooms`; UI on Home pending; Step 5. |
| notifications | partial | `components_ui/toast.rs` exists; center queue pending; Step 5. |
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
| chrome-extension-banner | skip | Per shelf decision §3 of plan. |
| old-client-notification | skip | Per plan §3 (irrelevant after rewrite). |
| web-hid | skip | No HID hardware integration by default. |
| external-api | skip | No postMessage bridge required (embed todo: iframe only). Decision: simple embed URL → `embed_meeting.rs` suffices. See note below. |
| file-sharing | migrated | `components_ui/file_sharing.rs`; server chat attachment recycle. |
| feedback | migrated | `components_ui/feedback.rs` + handler; `feedback.spec.ts`. |
| screenshot-capture (worker) | migrated | Used for thumbnails. |
| face-landmarks worker | migrated | `face_landmarks.rs` integration. |

## external-api decision (Langkah 0(d))

- Current usage: embedded meetings via iframe only → skip public command/event bridge.
- Public API behavior: **not kept** — the external JS API does not exist post-cutover; embedders use iframe embed (URL params) only.
- If future deployments require the bridge, Step 6 describes the `web-sys` postMessage implementation.

## Local spec consolidation (Langkah 7)

- Keep `rust-app/tests/e2e/` (rich: full lifecycle, migration parity, UI verification) and drop `rust-app/e2e/` dir entirely.
- Consolidate PNG artifacts under a `screenshots/` folder if retained; else delete.

## Exit criteria checklist

- [ ] `cargo test` green, build green
- [ ] One Playwright suite
- [ ] Every migrated feature has parity (spec) or skip reason listed
- [ ] React codebase removed (Step 7)
- [ ] UI responsive verified at 360/768/1280px
- [x] Mobile out of scope documented

## Verified broken/missing (Step 0 evidence)

1. `backend/static/styles.css` has exactly **1 @media** query (`@media (max-width: 768px)`, toolbox only).
2. `frontend/src/pages/room.rs` lines 164-170 compute inline `margin-right` = 320px × panel count — replaced with CSS in Step 2.
3. `video_grid.rs` has all tile styles inline; no classes for filmstrip/thumbnails.
4. `ToggleRoomLock` exists (boolean); no password UI; is successive for Step 4.
5. E2EE toggle → `UpdateE2EE` exists; E2EEKeyExchange variant reserved; key-exchange flow deferred to Step 4.
6. `subtitles` overlay exists with "Transcriptions will appear here" stub; STT backend absent.
7. Two parallel Playwright suites (`rust-app/e2e/` + `rust-app/tests/e2e/`) → keep `tests/e2e`.

