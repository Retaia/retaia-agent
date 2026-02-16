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
- Husky local guards (`pre-commit`, `pre-push`) to block direct work on `master`.

## Project structure

- `src/`: runtime code
- `tests/`: automated test suite
- `docs/`: implementation and operations docs
- `AGENT.md`: normative rules for agent implementation
- `specs/`: contract source of truth (git submodule)

## Requirements

- Rust (stable toolchain)
- Node.js 22+ (for CI/husky tooling)
- Git

## Quick start

```bash
git submodule update --init --recursive
npm ci
cargo test
```

## Development workflow

```bash
# create a feature branch
git checkout -b codex/my-feature

# run checks
npm run check:branch-up-to-date
cargo test
```

Rules:

- No commit on `master` (blocked by husky `pre-commit`)
- No push on `master` (blocked by husky `pre-push`)
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
