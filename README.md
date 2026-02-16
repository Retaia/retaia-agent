# retaia-agent

Rust agent client for the Retaia platform.

## Why this project

`retaia-agent` is the execution client responsible for processing workloads defined by Retaia Core contracts.

- CLI-first design for Linux headless environments.
- Optional GUI mode using the same runtime engine as CLI.
- Strict contract alignment with `specs/` (submodule to `retaia-docs`).

## Features

- Contract-driven runtime behavior.
- CLI mandatory, GUI optional.
- Same runtime configuration contract in GUI and CLI-only environments (Linux/macOS/Windows, including SSH/headless).
- Branch protection workflow with linear-history enforcement.
- `cargo-husky` local guards (`pre-commit`, `pre-push`) to block direct work on `master`.

## Project structure

- `src/`: runtime code
- `tests/`: automated test suite
- `docs/`: implementation and operations docs
- `AGENT.md`: normative rules for agent implementation
- `specs/`: contract source of truth (git submodule)

## Requirements

- Rust (stable toolchain)
- `cargo-commitlint` (for local `commit-msg` hook)
- Git
- Optional GUI notification adapter: `tauri` + `tauri-plugin-notification` via feature `tauri-notifications`

## Quick start

```bash
git submodule update --init --recursive
cargo install cargo-commitlint
cargo test
```

Headless config (CLI):

```bash
cargo run --bin agentctl -- config init \
  --core-api-url https://core.retaia.local \
  --ollama-url http://127.0.0.1:11434
cargo run --bin agentctl -- config validate
```

`agentctl` is powered by `clap` and uses the same validation contract as GUI/runtime services.

## Development workflow

```bash
# create a feature branch
git checkout -b codex/my-feature

# run checks
cargo run --quiet --bin check_branch_up_to_date
cargo test
```

Rules:

- No commit on `master` (blocked by `cargo-husky` `pre-commit`)
- No push on `master` (blocked by `cargo-husky` `pre-push`)
- Rebase on latest `master` before merge
- Keep linear history (no merge commits in feature branch)

## CI checks

- `branch-up-to-date`

## Contributing

See:

- `CONTRIBUTING.md`
- `AGENT.md`
- `docs/README.md`
- `docs/CONFIG-STORAGE.md`

## Security

- Do not log tokens/secrets in clear text.
- Keep runtime behavior aligned with policies in `specs/policies/`.

## Roadmap

- Rust CLI baseline (v1)
- Optional GUI shell on top of the same engine
- Extended observability and operational hardening

## License

Licensed under the GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later).
See `LICENSE`.
