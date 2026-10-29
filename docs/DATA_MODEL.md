# Data model and recovery

Version 0.2.1 migrates schema 1/2 to schema 3, preserving the original in
`app-state.v1.json` or `app-state.v2.json`. Tasks, paused/running timers and explicitly
stored notification preferences survive migration. Repeating rounds default on; the
previous separate auto-start flags are replaced by automatic focus/rest transitions.
Long breaks have their own enable flag and cadence. Per-round progress and the number
of successful focus periods since the last long break are stored separately from lifetime
focus credits. Previously completed sessions are never counted again. Unsupported future
schemas remain untouched.
`diagnostics.log` and `diagnostics.previous.log` contain bounded fixed-code diagnostics.

Location: Tauri's Windows app data directory, normally `%APPDATA%\com.rvnflow.desktop`.

| File                            | Purpose                                                                |
| ------------------------------- | ---------------------------------------------------------------------- |
| `app-state.json`                | Versioned aggregate of settings, task lists, timer, window and history |
| `app-state.backup.json`         | Previous validated aggregate                                           |
| `app-state.corrupt-<uuid>.json` | Preserved malformed primary after successful backup recovery           |
| `custom-audio`                  | Validated local WAV/MP3 bytes, decoded from content                    |

Schema version 3 stores UUID list/task IDs. Tasks have `id`, `listId`, `title`,
`completed`, `createdAt`, `updatedAt`, `completedAt`, and `order`. Timestamps are Unix
milliseconds. Titles are trimmed and limited to 200 Unicode characters. The last list
cannot be deleted. Reordering requires an exact permutation of the list's task IDs.
Deleting a current task or its list clears the timer association in the same transaction.

Timer status is `idle`, `running`, `paused`, or `completed`; session is `focus`,
`shortBreak`, or `longBreak`. Saved duration/remaining values are integer milliseconds.
The boot deadline is runtime-only. `wallDeadlineMs` supports restart recovery.
Preferences validate ranges and finite volume values. Active timers retain their original
duration after preference edits; idle and newly selected sessions use the latest values.

Writes use a temporary file in the same directory, flush/sync, and atomic replacement.
The previous validated state is saved before replacement. A mutex prevents concurrent
writers. Timer ticks never write to disk. Drag position is held in memory and saved on
focus loss, close, quit, or another mutation.

A malformed primary recovers from a valid backup and preserves the damaged original.
If both copies are invalid, startup shows the folder path and stops without overwriting
anything. Unknown schemas are also refused. To recover manually, close the app, copy
the entire folder elsewhere, and inspect the preserved JSON. Restore a valid backup
as `app-state.json` only after retaining both originals. Never delete the folder to fix
a timer issue unless you intentionally want to remove your tasks.

Atomic replacement reduces torn writes; it is not a guarantee against hardware loss,
filesystem corruption, or external programs editing live data. Keep independent backups.
