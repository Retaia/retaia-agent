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
- Runtime diagnostics files in same app directory:
  - `daemon-stats.json` (snapshot courant),
  - `daemon-history.sqlite3` (historique cycles + jobs complétés, avec compaction périodique).

### Storage Mount Mapping (Agent-side)

Le Core peut retourner des chemins relatifs (`INBOX/...`).  
L'agent résout ces chemins via `storage_mounts` dans `AgentRuntimeConfig`:

```toml
[storage_mounts]
nas-main = "/Volumes/NAS-01/retaia"
nas-archive = "/Volumes/NAS-01/archive"
```

Contraintes:
- clé `storage_id` non vide,
- path absolu uniquement,
- trailing slash normalisé côté agent (pas imposé côté Core),
- champ optionnel/backward compatible (config legacy sans `storage_mounts` reste valide).

### Storage Marker Contract (`/.retaia`)

Pour chaque mount déclaré dans `storage_mounts`, l'agent lit et valide un marker JSON `/.retaia` à la racine du mount.

Règles opératoires:
- le marker est créé et maintenu par Core,
- l'agent ne crée pas, ne modifie pas, ne répare pas ce marker,
- `source.storage_id` doit matcher strictement `/.retaia.storage_id`,
- `paths.inbox|archive|rejects` doivent être des chemins relatifs sûrs (pas de `..`, pas d'absolu, pas de byte nul),
- marker absent/invalide/incohérent => échec explicite de résolution du path source.

Politique de roots appliquée:
- marker `version=1` => seul `INBOX/...` est autorisé,
- marker `version>=2` => `INBOX/...`, `ARCHIVE/...`, `REJECTS/...` sont autorisés.

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
