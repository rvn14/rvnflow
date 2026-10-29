# Releasing RVNFlow

## Branches and versions

`main` is the integration branch. `feature/v0.1.0` records the initial implementation.
`release/0.1` is the maintenance line. Use short-lived feature/fix branches thereafter.
Version 0.2.1 is synchronized in package.json, Cargo.toml, Cargo.lock and tauri.conf.json.
`node scripts/check-version.mjs v0.2.1` verifies a proposed stable tag.

## CI

`ci.yml` runs on main pushes, pull requests and manual dispatch on Windows. It installs
pinned pnpm, Node 24 and Rust stable MSVC, caches dependencies, installs with frozen
lockfiles, checks frontend types/format/tests, checks Rust formatting/Clippy/tests,
builds the frontend, tests layouts and builds an NSIS installer. It uploads
`RVNFlow-Windows-x64`; failures are not masked. Artifacts expire after 14 days.

## First stable release

1. Review the [native checklist](VERIFICATION.md) and resolve release-blocking findings.
2. Ensure main CI passes. Synchronize application versions and lockfiles if changing them.
3. Explicitly choose to publish and create the annotated tag:

   ```powershell
   git switch main
   git pull --ff-only
   node scripts/check-version.mjs v0.2.1
   git tag -a v0.2.1 -m "RVNFlow 0.2.1"
   git push origin v0.2.1
   ```

4. `release.yml` verifies the tag, repeats quality checks, builds the installer and
   SHA-256 file, then publishes only if the build job succeeds. Only the publish job
   gets `contents: write`; the standard `GITHUB_TOKEN` is sufficient. No PAT is needed.
5. Verify assets at https://github.com/rvn14/rvnflow/releases/latest.

Do not tag a release merely to test CI. The implementation does not create a release tag.

## Public development builds

Run **Development release** manually to publish the installer from the most recent
successful main/push CI run. Optional automatic publication requires setting the
repository Actions variable `PUBLISH_DEVELOPMENT` to `true`. It is off by default.

The workflow downloads a successful trusted build without executing artifact contents,
generates checksums, and creates `dev-<run-number>-<short-sha>` as a prerelease. Each
development version has a distinct tag; no stable tags or release history are rewritten.
Concurrency serializes publication. If the artifact expired, rerun main CI first.
The canonical discovery location, newest first, is:
https://github.com/rvn14/rvnflow/releases?q=dev-&expanded=true

Published release assets do not depend on temporary Actions artifacts. Public anonymous
downloads require the repository to be public. Development packages retain the application
version of their source commit; their release title and source SHA distinguish builds.

## Packaging and signing

NSIS uses `currentUser`, includes the compiled sounds and app icon, and installs
`RVNFlow.exe`. Updates preserve app data. The generated uninstaller may offer an optional
app-data removal choice; users should leave it unchecked to retain tasks.

No signing keys are configured, and no output should be described as signed. Windows
may display SmartScreen warnings. A future signed updater must include key management,
signed metadata and migration validation before being enabled.
