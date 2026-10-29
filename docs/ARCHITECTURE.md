# Architecture

RVNFlow is a modular monolith with one native window and two background services.

## Authority and transactions

`domain/timer.rs`, `task.rs`, and `settings.rs` contain pure, testable business rules.
Commands are typed IPC adapters. `Engine` serializes mutations through a mutex,
clones the current state, applies validation, persists the candidate, and only then
replaces live state. A failed save leaves tasks and settings unchanged in memory.
One aggregate document keeps task references, preferences, timer, and history consistent.

The command surface is `get_app_snapshot`, `timer_action`, `task_action`,
`settings_update`, `window_set_mode`, `window_close`, `audio_preview`, and
`audio_import_custom`. Action enums keep related operations together without dozens
of duplicate adapters. The Rust handler validates every identifier and value.

Snapshots carry a mutation revision and a boot-clock sampling timestamp. Svelte rejects
older revisions and older samples at an equal revision. Listeners are registered before
the initial fetch and cleaned up on unmount. Focus and visibility recovery fetch a new
snapshot. Events are `timer:changed`, `timer:completed`, `tasks:changed`,
`settings:changed`, `window:changed`, plus `app:error`.

## Timing

The Windows clock uses `GetTickCount64`, which includes suspended time and is independent
of wall-clock adjustments. Pure timer methods accept injected clock values. A running
session holds an absolute boot-clock deadline. No incremental decrement accumulates drift.
The worker uses a condition variable, waking at most once per elapsed second while
running and sleeping indefinitely while idle. It does not emit countdown events every second.

The frontend paints a projection from the last remaining duration and `performance.now()`.
It cannot complete, count, or advance a session. The worker emits a resynchronization
after a detected sleep gap or wall-clock discontinuity. Wall deadlines are persisted at
mutations for restart recovery, clamped to the saved remaining duration. Time changes
while the process is absent are inherently ambiguous and documented as such.

Completion is committed once before alarm/notification side effects. At most one missed
session completes after sleep or restart; the next requires Start. A skipped focus earns
no credit. The last 1,000 completion records are retained. An OS kill between durable
completion and playback can lose the alarm, but cannot duplicate a focus credit.

## Windows and audio

The single frameless opaque window resizes between logical dimensions. Native monitor
work areas use physical coordinates to clamp saved positions. Tray actions and window
close behavior are implemented in Rust. The single-instance plugin raises the existing
window. Startup defaults off and is changed through the Rust plugin. OS calls use narrow
native APIs; the webview only receives event-listening and drag permissions.

The audio worker decodes all short sounds once into shared sample buffers. It opens an
output stream on demand, replaces previous tick playback, discards stale tick requests,
and releases the stream after inactivity. Device errors invalidate the stream so a later
request reopens it; unavailable ticking output retries with a cooldown. Custom imports
use a native picker and bounded file/decode validation off the UI thread. WAV and MP3
are the only enabled decoder formats, with 20 MB, 30 seconds, two channels and 192 kHz limits.
Original sounds are embedded at compile time, so packaging cannot omit them.

## Security and future extension

No HTTP client, remote UI content, telemetry, shell plugin, broad filesystem capability,
or cloud backend exists. A restrictive CSP and local-only capabilities protect the webview.
There is no updater: future updates require signing keys, metadata, migration tests,
and explicit release design. Storage has a schema boundary for future migrations.

Version-sensitive APIs were checked against the official documentation:
[Tauri windows](https://v2.tauri.app/learn/window-customization/),
[Tauri tray](https://v2.tauri.app/learn/system-tray/),
[autostart](https://v2.tauri.app/plugin/autostart/),
[notifications](https://v2.tauri.app/plugin/notification/),
[Rodio](https://docs.rs/rodio/0.23.0/rodio/),
[Svelte testing](https://svelte.dev/docs/svelte/testing), and
[Tailwind Vite integration](https://tailwindcss.com/docs/installation/using-vite).
