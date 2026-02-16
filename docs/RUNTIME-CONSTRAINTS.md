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
- Pilotage runtime `pull-only` (polling HTTP contractuel).
- Aucune dépendance à un canal push serveur-vers-client (WebSocket/SSE/webhook) pour la cohérence runtime.

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
- Les actions mutatrices ne partent qu'après lecture d'un état compatible par polling.

## Polling Rules

- Les boucles de polling jobs/policy/device-flow respectent les intervalles contractuels renvoyés par Core.
- Sur `429` (`SLOW_DOWN` / `TOO_MANY_ATTEMPTS`), appliquer backoff + jitter avant nouvelle tentative.
- Les changements policy/feature-flags sont pris en compte au prochain polling, sans attente d'un signal push.

## Normative References

- `../specs/workflows/AGENT-PROTOCOL.md`
- `../specs/api/API-CONTRACTS.md`
- `../specs/policies/AUTHZ-MATRIX.md`
