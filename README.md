# retaia-agent

Rust agent client for Retaia.

## Overview

`retaia-agent` is the processing client for the Retaia platform.

- CLI is mandatory (including Linux headless usage).
- GUI is optional.
- If GUI is present, it uses the exact same runtime engine as CLI.

Normative behavior is defined by the `specs/` submodule.

## Features

- Agent runtime aligned with Retaia Core contracts.
- System tray UX target (optional GUI).
- CI quality gates: branch freshness, commit message policy, test suites, coverage.
- Conventional Commits enforced via git hooks.

## Requirements

- Rust toolchain (stable)
- Git

Optional local tooling:

- `cargo-commitlint` (for local `commit-msg` hook)

## Getting Started

```bash
cargo install cargo-commitlint
git config --unset core.hooksPath || true
cargo clean -p cargo-husky
cargo test
```

## Quality Gates

Main CI checks:

- `branch-up-to-date`
- `commitlint`
- `test-tdd`
- `test-bdd`
- `test-e2e`
- `coverage-gate`
- `ci-required`

Coverage minimum: `80%`.

## Documentation

- Human docs hub: `docs/README.md`
- AI entry point: `AGENT.md`
- Normative specs: `specs/`

Topic docs:

- Runtime constraints: `docs/RUNTIME-CONSTRAINTS.md`
- System tray UX: `docs/UX-SYSTEM-TRAY.md`
- Notifications: `docs/NOTIFICATIONS.md`
- Configuration panel: `docs/CONFIGURATION-PANEL.md`
- CI and quality gates: `docs/CI-QUALITY-GATES.md`

## Contributing

- Use a feature branch (never commit directly on `master`).
- Keep history linear and branch up to date with `master`.
- Follow Conventional Commits.
- Keep implementation aligned with `specs/`.
