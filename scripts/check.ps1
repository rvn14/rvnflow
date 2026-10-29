$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
pnpm check
pnpm lint
pnpm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
pnpm build
node scripts/check-version.mjs
