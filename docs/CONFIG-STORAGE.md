# Configuration Storage

## Objective

Define a single, recognized, cross-platform storage strategy for agent runtime configuration.

## Library Choice

- Path resolution: [`directories`](https://crates.io/crates/directories)
- Serialization format: TOML via [`toml`](https://crates.io/crates/toml)
- Encoding/decoding: [`serde`](https://crates.io/crates/serde)

These libraries are widely used in Rust projects and stable for system config persistence.

## File Format

- File name: `config.toml`
- Runtime model persisted: `AgentRuntimeConfig`
- Validation before save and after load: same domain rules (`validate_config`)

## System Location

Default path is resolved with `ProjectDirs::from("io", "Retaia", "retaia-agent")`:

- Linux: `$XDG_CONFIG_HOME/retaia-agent/config.toml` (or `~/.config/retaia-agent/config.toml`)
- macOS: `~/Library/Application Support/io.Retaia.retaia-agent/config.toml`
- Windows: `%APPDATA%\\Retaia\\retaia-agent\\config\\config.toml`

## Override for Ops / Headless

- Env var: `RETAIA_AGENT_CONFIG_PATH`
- If set to a non-empty value, this path is used directly.
- Useful for SSH/CLI-only deployments, containers, and custom filesystem layouts.

## Invariants

- GUI and CLI use the same persisted contract.
- No duplicated validation logic between channels.
- Invalid config is never persisted.

## DDD Boundary

- Application port: `ConfigRepository`
- Infrastructure adapters:
  - `SystemConfigRepository` (system path)
  - `FileConfigRepository` (explicit path, tests/ops tooling)
- `AgentRuntimeApp` depends on the port, not on filesystem/TOML details.
