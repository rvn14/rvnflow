# Development

Requirements: Windows 10/11 x64, Node 24 LTS, pnpm 12.16.1, Rust stable MSVC,
Visual Studio C++ Build Tools and Windows SDK, and WebView2. The current compatible
TypeScript line is 6; svelte-check requires an additional compatibility setup for 7.

```powershell
pnpm install --frozen-lockfile
pnpm tauri dev
```

```powershell
pnpm format
cargo fmt --manifest-path src-tauri/Cargo.toml
./scripts/check.ps1
pnpm exec playwright test
pnpm tauri build --bundles nsis
```

The browser layout test uses installed Microsoft Edge. It mounts production components
with mocked native transport and checks both dimensions, long task lists and settings at
four device scales. It also regenerates the documented screenshots with synthetic data.
This does not test Windows DPI transitions, native IPC, alarms, tray behavior or installation.

`python scripts/generate-sounds.py` regenerates the original bundled sounds using only
Python's standard library. `pnpm tauri icon assets/icon.svg` regenerates the icon set.
These generated assets are committed; consumers do not need Python to build.

Use `feature/*` branches from main. Keep phase commits reviewable and keep meaningful
comments near recovery rules, safety bounds, clock semantics and native workarounds.
Stop the development executable before replacing its binary: Windows locks running EXEs.

## Performance measurements

Measure a release build, with devtools closed, after a 30-second warmup. Use Task Manager
to include RVNFlow and its WebView2 subprocesses. Record private working set, CPU over
five idle minutes, and CPU with ticking and a running timer. Repeat hidden in the tray.
Under 100 MB is a target, not a guarantee; WebView2 and Windows versions affect memory.

Use Windows Performance Recorder/Analyzer for startup and wake investigations. Measure
launch-to-first-widget on both warm and cold starts and record hardware/OS versions.
Use Process Monitor filtered to the app data directory to verify there are no per-second
writes. Idle timer waits and inactive audio should not produce repeated wakeups.
