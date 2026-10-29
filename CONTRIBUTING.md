# Contributing

Create a branch from `main`, for example `feature/task-shortcuts` or `fix/timer-recovery`.
Keep changes focused and use meaningful Conventional Commit messages such as
`fix(timer): preserve remaining time across resume`. Comments should explain why a
boundary, recovery rule, or native workaround exists, rather than narrating obvious code.

Run `pnpm install --frozen-lockfile`, `./scripts/check.ps1`, and
`pnpm exec playwright test` before submitting a pull request. Describe behavior,
validation, and remaining Windows-specific checks. Add regression tests for domain
changes. Never introduce a second frontend-owned timer or direct frontend filesystem access.

Rust domain logic must remain independent of Tauri. Update TypeScript contracts when
changing IPC. Do not silently migrate or delete saved tasks. Schema migrations require
tests and a retained recovery copy. Commit Cargo.lock and pnpm-lock.yaml. Do not commit
build outputs, local app data, credentials, or personal settings.

Use `release/0.1` for reviewed fixes to 0.1. Patch releases need synchronized versions,
successful Windows CI, and native verification. Only maintainers publish release tags.
