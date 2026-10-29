# Automatic routines and minimal window — 0.2.1

Start the first focus session once. Focus and break countdowns then start automatically,
and rounds repeat until Pause or Stop is used. The old separate auto-start switches have
been removed. Stop resets the round and leaves the timer idle; Pause preserves its place.

The session count is the number of focus periods in one round. Long-break scheduling is
independent: enable **Include long breaks**, choose a duration, and set **Long break every**
to 2, 4, or another supported interval. A long break replaces the short break at that
boundary. It does not add a second break. Successful focus completions drive this count;
skipping focus never earns completion credit.

Example: four 50-minute focus sessions, 10-minute short breaks, and a 30-minute long
break every two focus sessions:

```text
50 focus → 10 short → 50 focus → 30 long
50 focus → 10 short → 50 focus → 30 long → repeat
```

Repeat is enabled by default, including when older settings are migrated. Turning it
off intentionally makes the current round finish after its configured focus count.
Long-break cadence continues across repeating rounds. Starting a new routine or pressing
Stop resets its counters, while lifetime focus history remains intact.

Rust owns all transitions, deadlines, completion records, alarms and notifications. A
sleep/wake or restart completes at most one overdue phase and starts the next phase
from the current time; it never consumes a backlog of missed rounds.

## Window and interface

The native shadow is disabled. The native window, WebView background, HTML and body
are transparent, and CSS draws a single rounded lavender surface. No opaque rectangle
or second shadow is painted behind the outline. The same configuration is exercised by
the native smoke build. Rendered PNG corner pixels were inspected and have alpha zero.

The focus/short-break/long-break tabs have been removed. A small phase label and round
counter describe the current phase. The expanded widget keeps the timer prominent,
reduces separators and decorative labels, and retains a fixed task composer and a
scrolling task list. The settings panel separates focus length, short breaks, focus
count, and optional long-break cadence.

## Migration and verification

Schema 3 adds per-round progress, an independent long-break counter, and routine settings.
Schema 1/2 files are backed up as `app-state.v1.json` or `app-state.v2.json`. Tasks,
remaining time, volumes, notification preferences and window preferences are preserved.
The old false `autoStartBreaks`/`autoStartFocus` flags no longer stop automatic transitions.
Existing long-break cadence is preserved when the new explicit option is absent.
An older automatically prepared idle break is resumed during migration when its history
identifies it as the scheduled next phase. Explicitly paused timers remain paused.

Verification covers the exact 4 × 50/10 schedule, long breaks every 2/4, disabling long
breaks, continuous repetition across rounds, optional single-round completion, migration,
and duplicate/sleep recovery boundaries. The native debug harness accelerates the real
engine through a complete repeating 4 × 50/10 round with long breaks every two sessions,
checks eight completion records, and verifies that the next focus countdown is running.
It also checks minimize/restore, tray hide/show, relaunch and saved-data restart.

Browser tests exercise 0/1/10/50 tasks and four display scales, check transparent document
backgrounds and the absence of a CSS widget shadow, and capture the real frontend with
mocked IPC. They do not establish visual results on every physical Windows display.

Build: `pnpm tauri build --bundles nsis`.
Output: `src-tauri/target/release/bundle/nsis/RVNFlow_0.2.1_x64-setup.exe`.

Local checks passed: 28 Rust tests, 20 frontend tests, 6 browser scenarios, and the two
native launch/restart scenarios. Type checks, formatting and Clippy with warnings denied
also passed. Browser PNG corners were inspected as RGBA `(0, 0, 0, 0)`; physical
display appearance across all Windows configurations is not claimed as manually tested.
