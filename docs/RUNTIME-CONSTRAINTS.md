# Runtime Constraints

## Scope

- Agent Rust.
- CLI obligatoire (Linux headless supporté).
- GUI optionnelle.
- Même moteur runtime pour CLI et GUI.

## Protocol Rules

- Bearer-only.
- Respect strict de `effective_feature_enabled`.
- L'agent n'est pas décideur métier.
- Aucun traitement MCP dans ce repo.

## Auth

- Mode interactif: `POST /auth/login`.
- Mode technique: `client_id + secret_key` via `POST /auth/clients/token` (ou OAuth2 client credentials).
- Invariant: 1 token actif par `client_id`.

## Security

- Aucun token/secret en clair dans logs/traces/crash reports.
- Secret storage OS-native (Linux/macOS/Windows).
- Rotation de secret supportée sans réinstallation.

## Processing Boundaries

- Jobs agents: discovery/claim/heartbeat/submit/fail sur `/jobs/*` selon scope.
- Actions destructives (move/purge) hors périmètre agent.
- MCP ne peut pas `claim/heartbeat/submit`.

## Normative References

- `../specs/workflows/AGENT-PROTOCOL.md`
- `../specs/api/API-CONTRACTS.md`
- `../specs/policies/AUTHZ-MATRIX.md`
