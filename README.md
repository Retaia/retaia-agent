# retaia-agent

Rust agent client for the Retaia platform.

## Why this project

`retaia-agent` is the execution client responsible for processing workloads defined by Retaia Core contracts.

- CLI-first design for Linux headless environments.
- Optional GUI mode using the same runtime engine as CLI.
- Strict contract alignment with `specs/` (submodule to `retaia-docs`).

## Features

- Contract-driven runtime behavior.
- Capability-driven scheduling guard (`media.facts@1`, `media.proxies.*@1`, `media.thumbnails@1`, `audio.waveform@1`).
- Derived-processing v1 runtime support (`claim/heartbeat/submit` + `/derived/upload/*` gateway).
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
- `ffmpeg` (required for audio/video proxy generation)
- Git
- Optional GUI notification adapter: `tauri` + `tauri-plugin-notification` via feature `tauri-notifications`
- Optional generated Core API client: feature `core-api-client` (`crates/retaia-core-client`)

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
cargo run --bin agentctl -- config validate --check-respond
```

`--check-respond` validates API compatibility (`Core /jobs`, `Ollama OpenAI-compatible /v1/chat/completions via genai`), not just TCP reachability.

`agentctl` is powered by `clap` and uses the same validation contract as GUI/runtime services.

Interactive runtime shell (CLI-only environments):

```bash
cargo run --bin agent-runtime
```

Supported commands: `menu`, `status`, `settings`, `play`, `pause`, `stop`, `quit`.

Daemon management (shared service for CLI/GUI):

```bash
# install user-level daemon with autostart at boot
cargo run --bin agentctl -- daemon install

# control lifecycle
cargo run --bin agentctl -- daemon start
cargo run --bin agentctl -- daemon status
cargo run --bin agentctl -- daemon stop
cargo run --bin agentctl -- daemon uninstall
```

Disable autostart at boot with `--no-autostart`.

Daemon runtime loop (foreground service mode):

```bash
cargo run --bin agent-runtime -- daemon --tick-ms 5000
```

With `core-api-client` enabled, daemon polling uses `GET /jobs` and can attach bearer auth from `RETAIA_AGENT_BEARER_TOKEN`.

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

## OpenAPI client

The Rust HTTP client for Core v1 is generated from `specs/api/openapi/v1.yaml`:

```bash
./scripts/generate_core_api_client.sh
```

To compile agent integration helpers with this generated client:

```bash
cargo test --features core-api-client
```

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
