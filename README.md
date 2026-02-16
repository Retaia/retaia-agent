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

## Quick start

```bash
git submodule update --init --recursive
cargo install cargo-commitlint
cargo test
```

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
