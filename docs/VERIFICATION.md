# Verification record and native checklist

For 0.2 results, see [V0.2_BUGS.md](V0.2_BUGS.md). The new isolated native harness has
executed lifecycle, timer completion and restart checks. The list below retains manual
acceptance checks, including physical DPI/multi-monitor and visible OS notification tests.

## Automated and build verification

Implementation verification uses Rust domain/persistence/audio tests, Vitest component
and utility tests, Svelte type checks, Prettier, Clippy with warnings denied, and Edge
layout tests at 100%, 125%, 150%, and 200% simulated scale. See CI for results against
the current commit. Local final results are recorded in `IMPLEMENTATION_REPORT.md`.

The native debug executable was launched on Windows and remained responsive with no
stderr startup error. Automated native UI inspection was unavailable because the
computer-use native pipe could not be opened. Browser mocks are not native smoke tests.

## Manual Windows smoke tests

Before a stable release, record OS build, display configuration, app version and result
for each item. The following checks remain manual unless a later report marks them done.

- [ ] Start, pause and resume; verify countdown against an independent stopwatch.
- [ ] Set a one-minute focus, finish once, hear one alarm and confirm the next session manually.
- [ ] Complete a full cycle and receive a long break; skip focus and verify no credit.
- [ ] Pause while ticking; confirm immediate silence. Hide in tray and finish a session.
- [ ] Preview all sounds; import WAV and MP3; remove the original file and preview again.
- [ ] Reject malformed, oversized and overlong audio. Unplug/reconnect the output device.
- [ ] Create, rename, reorder, complete and delete tasks/lists; restart and verify saved data.
- [ ] Expand/collapse while running without resetting time. Test keyboard-only navigation.
- [ ] Always-on-top on/off over ordinary apps; verify exclusive fullscreen is not disrupted.
- [ ] Move across monitors with different scaling; disconnect the saved monitor and restart.
- [ ] Verify 100%, 125%, 150%, and 200% actual Windows scaling, including live DPI changes.
- [ ] Drag toward all work-area edges; ensure recovery brings the widget into view.
- [ ] Close-to-tray, show/hide, tray start/pause, explicit Quit, and single-instance activation.
- [ ] Start with Windows opt-in and opt-out across sign-out/restart.
- [ ] Sleep/wake before and after deadline; no repeated completion or backlog of ticks.
- [ ] Adjust wall clock during a session; live remaining time should not jump.
- [ ] Quit while running, restart before/after expiry, and verify sensible recovery.
- [ ] Test notifications from an installed app with OS notifications enabled/disabled.
- [ ] Install per-user, upgrade with existing tasks, uninstall without removing data, reinstall.
- [ ] Run offline with WebView2 already installed. Verify no runtime network dependency.
- [ ] Measure idle CPU, total process-tree memory and startup as in DEVELOPMENT.md.

Known boundaries: Win10 may show square outer native corners; rounded inner surfaces
remain. Restart recovery cannot distinguish elapsed real time from wall-clock changes
made while RVNFlow was closed. Alarm delivery is best-effort after an OS kill at completion.
Performance numbers are targets and have not been certified.
