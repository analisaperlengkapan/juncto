# 1. OBJECTIVE

Menyelesaikan migrasi aplikasi **web** Juncto dari React/TypeScript ke stack Rust yang sudah dirintis di `rust-app/` (frontend Leptos WASM + backend Axum). Sasaran akhir: seluruh fitur web `react/features/` termigrasi atau secara sadar diputuskan untuk di-skip, UI dipoles hingga rapi/responsive (saat ini masih berantakan — hanya 1 media query), dan codebase React lama dihapus. **Mobile apps (iOS/Android) dikecualikan dari scope sesuai keputusan user.**

# 2. CONTEXT SUMMARY

**Status migrasi saat ini (sudah ada di `rust-app/`):**
- **Workspace Cargo**: `backend` (Axum 0.7, WebSocket `/ws/chat`), `frontend` (Leptos 0.6 CSR → WASM), `shared` (tipe protokol `ClientMessage`/`ServerMessage`).
- **Frontend**: ~27 modul fitur (webrtc, media, chat, polls, whiteboard, reactions, settings, toolbox, participants, speaker_stats, connection_stats, analytics, i18n, storage, dll) + 19 komponen UI (prejoin, lobby, breakout, invite, feedback, file_sharing, embed_meeting, dsb).
- **Backend**: handler breakout, calendar, chat, dropbox, feedback, moderation, polls, remote_control, room, salesforce, whiteboard, ws. Media: WebRTC mesh; state in-memory (Arc<Mutex>).
- **Pengujian**: **dua** suite Playwright paralel (`rust-app/e2e/` dan `rust-app/tests/e2e/`, ~60 spec) — perlu konsolidasi; plus `cargo test`.
- **Build**: `build.sh` (cargo build wasm32 + wasm-bindgen).

**Hasil uji/verifikasi kode (apa yang rusak/belum ada):**
- **Migrated & berfungsi**: screen share (via `get_display_media`/`getDisplayMedia` native di `media.rs` → browser native prompt, tidak butuh desktop-picker UI), E2EE toggle (di SettingsDialog, wired ke `state.toggle_e2ee`, `toggle_participant_e2ee`), lobby, breakout room, polls, chat, whiteboard, reactions, virtual background, media recorder lokal, auth dialog, speaker stats, keyboard shortcuts.
- **Belum ada (verified via grep)**: `filmstrip` (thumbnail strip), `pip` (picture-in-picture), video-menu kontekstual (klik kanan/pin/volume dari menu), layout selector (tile/speaker), rejoin flow, connection-indicator visual badge (hanya ping/RTT mentah di `connection_stats.rs`).
- **UI berantakan (verified)**: `backend/static/styles.css` hanya berisi **1 blok `@media`** (toolbox di ≤768px). Layout panel samping memakai inline style `margin-right` hard-coded 320px × jumlah panel (`room.rs` baris 164-170) yang berisiko rusak di layar kecil. Banyak komponen pakai inline style → sulit konsistensikan ke best practice.
- **Duplikasi struktur**: 2 direktori e2e paralel + 2 pageobject set — hambat parity audit.

**Mobile**: `react/features/mobile/`, `ios/`, `android/` — **EXCLUDED** (keputusan user).

**Ketergantungan infrastruktur yang perlu diperhatikan saat memilih skip**: fitur berikut di versi React bergantung pada service eksternal (XMPP/Prosody, Jicofo, Jigasi/SIP, Google API, Salesforce API, Etherpad server). Di backend Rust baru semua itu hilang → fitur hanya bisa jadi stub UI atau perlu di-skip.


# 3. APPROACH OVERVIEW

Migrasi bertahap per-fitur dengan **gap matrix** di `rust-app/MIGRATION.md` sebagai sumber kebenaran. Setiap fitur dipetakan ke: (a) modul frontend Leptos, (b) komponen `components_ui`, (c) handler backend, (d) varian protokol `shared`. Ada fase khusus **UI lintas-resolusi/responsif** (Langkah 2) sebelum fitur tersier; dan fase cutover penghapusan codebase React.

**Rekomendasi skip** — fitur dengan utilitas kecil atau yang butuh infrastruktur eksternal:

1. **`chrome-extension-banner`** — banner promo ekstensi; utilitas kecil → **skip**.
2. **`old-client-notification`** — peringatan versi client usang; tidak relevan setelah rewrite → **skip**.
3. **`web-hid`** — integrasi perangkat HID (headset); hanya bila ada hardware kontrol → **default skip**.
4. **`salesforce`** — integrasi CRM niche; tanpa backend nyata hanya jadi stub → **skip**.
5. **`giphy`** — picker GIF; **dapat skip** bila tidak terpakai.
6. **`dial-in`/`videosipgw`** — PSTN/SIP membutuhkan Jigasi/gateway; di Rust hanya info statis → **skip** (kecuali deployment menyediakan SIP).
7. **`transcribing`/`subtitles`** — STT membutuhkan service (Jigasi/Vosk); protokol masih ada tapi tanpa backend STT hanya stub → **skip**.
8. **`etherpad`** — butuh server Etherpad eksternal → **skip bila tidak dipakai**.
9. **`google-api`/calendar-sync** — sudah termigrasi (`calendar.rs`); **skip hanya bila tidak dipakai**.
10. **`external-api`** — kondisional: jika embedder publik memakai command/event bridge (`api.executeCommand(...)`), **perlu implementasi**; jika hanya embed iframe sederhana, dialog `embed_meeting.rs` sudah cukup → **skip**. Keputusan final dibuat di Langkah 0 berdasarkan audit deployment.

Setiap skip di atas akan dicatat dalam gap matrix beserta alasannya, tanpa implementasi.

# 4. IMPLEMENTATION STEPS

**Langkah 0 — Gap matrix & audit status (uji apa yang rusak/belum ada)**
- Goal: inventory pasti per fitur: migrated / partial / missing / skip (alasan).
- Method: (a) jalankan `bash rust-app/build.sh` lalu `npx playwright test` pada salah satu suite (setelah install deps) untuk katalog kesehatan fitur; (b) per fitur `react/features/` → status via grep/view; (c) tulis ke `rust-app/MIGRATION.md` dengan kolom Status/Alasan (skip reason); (d) putuskan nasib external-api berdasarkan audit siapa yang meng-embed app ini; perilaku API publik didokumentasikan bila dipertahankan.
- Reference: `react/features/*`, `rust-app/*`, `MIGRATION.md`.

**Langkah 1 — Conference-core parity & video layout (tampilan dasar)**
- Goal: parity visual/fungsional tampilan conference + perbaikan tata letak viewport.
- Method: implementasikan `filmstrip` (thumbnails dengan fallback avatar), layout `tile/speaker` selector, video-menu kontekstual (pin, volume fader per partisipan, kick), spotlight pin (varian `PinParticipant` ada di `shared`), visual `connection-indicator` (kualitas dari `rtt` `connection_stats.rs`), rejoin prompt pasca putus. Semua pakai CSS class (bukan inline) — siapkan fondasi Langkah 2.
- Reference: `react/features/{filmstrip,large-video,video-menu,video-layout,connection-indicator,rejoin,pip}` → `components_ui/video_grid.rs` (refactor), `participants.rs`, `state.rs`, `styles.css`.

**Langkah 2 — UI/UX overhaul & responsive**
- Goal: tampilan yang saat ini berantakan menjadi rapi, responsive, dan mengikuti best practice (CSS modular, breakpoint strategis, tanpa inline style layout).
- Method:
  1. **CSS audit**: katalogkan class CSS di `styles.css`, revisi tiap komponen `components_ui` agar memakai class alih-alih inline `style=`. Ukuran lebar panel tidak lagi dihitung hard-coded di `room.rs` (inline `margin-right: 320px × panel`); pindahkan ke CSS (flexbox/grid, `clamp()`, viewport units).
  2. Tambahkan breakpoint strategis: `@media (max-width: 1024px)` (tablet) dan `@media (max-width: 640px)` (mobile web); chat/participants jadi overlay penuh atau bottom sheet di viewport kecil, bukan menekan video.
  3. Hapus komputasi margin inline ganda — satu aturan CSS, mis. lebar panel dengan `--side-panel-w: min(320px, 80vw)` atau layout `grid-area`.
  4. Verifikasi dengan **visual parity test** (perbandingan screenshot Playwright) sebagai bukti penataan ulang.
  5. Tambahkan design tokens (spacing, radius, tipografi) sebagai variabel CSS pada `:root`.
- Reference: `rust-app/backend/static/styles.css`, `frontend/src/components_ui/*`, `frontend/src/pages/room.rs` (hapus inline margins), `tests/e2e` (screenshot parity).

**Langkah 3 — Media pipeline & kualitas**
- Goal: parity audio/video in-call.
- Method: noise-suppression (fallback constraint-based jika port WASM rnnoise terlalu berat), noise-detection visual, `no-audio-signal` indicator, `talk-while-muted` toast, `video-quality` selector (res constraints `media.rs`), PiP dalam panggilan (document PiP API bila ada).
- Reference: `react/features/{noise-suppression,noise-detection,no-audio-signal,talk-while-muted,video-quality,pip}` → `frontend/src/media.rs`, `frontend/src/webrtc.rs`, `components_ui/`.

**Langkah 4 — Security & roles**
- Goal: melengkapi security yang belum atau masih parsial: room-lock dengan password (saat ini hanya toggle boolean), security dialog, role `visitor` (field reserved di `shared`, belum dipakai), kontrol permission; plus parity E2EE (flow pertukaran key bila didukung deployment).
- Method: wire `ToggleRoomLock` dengan prompt password; implementasikan logika role visitor; aktifkan flow `E2EEKeyExchange` per partisipan (varian protokol sudah ada di `shared`).
- Reference: `shared/src/lib.rs`, `react/features/{e2ee,room-lock,security,visitors}`.


**Langkah 5 — Notifications, riwayat, util**
- Goal: parity fitur sekunder.
- Method: upgrade `toast.rs` menjadi notification center (antrian notifikasi bertipe); recent-list room (riwayat di `storage.rs`); view `polls-history` (riwayat poll peserta); plus parity test tiap fitur.
- Reference: `react/features/{notifications,recent-list,polls-history}` → `components_ui/toast.rs`, `storage.rs`.

**Langkah 6 — Integrasi eksternal (kondisional per audit Langkah 0)**
- Goal: implementasikan `external-api` hanya jika audit deployment membutuhkan; jika tidak, tetap skip.
- Method (bila perlu): postMessage bridge via `web-sys` (`window.post_message`, event listener) untuk commands (`toggleAudio`, `toggleVideo`, dsb) dan events (`participantJoined`, dsb).
- Reference: `modules/API/external/*` → `components_ui/embed_meeting.rs` (opsional upgrade) / modul baru `frontend/src/external_api.rs`.

**Langkah 7 — Cutover**
- Goal: rust-app satu-satunya implementasi.
- Method: hapus `react/`, `modules/`, `css/`, `webpack.config.js`, `Makefile` web targets, `package.json` npm build deps yang tak dipakai, `tests/` (WDIO) lama; konsolidasikan e2e ke **satu** suite Playwright re-test (gunakan `rust-app/tests/e2e/` yang lebih lengkap); update CLAUDE.md/README; update CI (GitHub Actions) ke `cargo test` + `cargo build --target wasm32` + Playwright suite.
- Reference: root repo; `rust-app/build.sh`.

# 5. TESTING AND VALIDATION

- **Unit**: `cargo test` (workspace) hijau tiap langkah — handles `backend/api.rs::tests`, `shared/src/tests.rs`.
- **Build**: `bash rust-app/build.sh` sukses → men-generate `frontend/pkg/{frontend.js,frontend_bg.wasm,index.html}`; backend berjalan di `:3000`.
- **E2E parity**: satu suite Playwright yang terkonsolidasi; setiap fitur termigrasi harus punya spec parity (contoh pola: `migration_parity*.spec.ts`). Sukses: spec existing tetap hijau + semua fitur baru dimigrasi punya spec hijau.
- **UI**: visual parity via tangkapan layar Playwright pada setiap Langkah 2; viewport test pada 360px, 768px, 1280px — tidak ada tumpangtindih toolbox/panel, margin tidak terkomputasi inline, dialog-scrollbar dapat di-scroll.
- **Manual parity**: 2+ klien di satu room; verifikasi fitur per-langkah vs perilaku React lama (layout switching, mute/unmute, screen share, room-lock/E2EE, notifikasi, PiP).
- **Exit criteria**: gap matrix semua migrated / skip-disetujui (dengan alasan terdokumentasi), UI konsisten di viewport target, codebase React terhapus (Langkah 7), CI cargo+Playwright hijau, dan mobile apps secara resmi didokumentasikan sebagai di luar scope.
