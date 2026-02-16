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
- Pilotage runtime status-driven par polling HTTP contractuel (source de vérité).
- Les canaux push serveur-vers-client peuvent exister comme hints (WebSocket/SSE/webhook/push mobile), mais ne sont jamais autoritatifs métier.

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
- Les changements policy/feature-flags sont pris en compte au prochain polling; un push peut déclencher un poll mais ne valide jamais l'état final.

## Push Hints (v1.2)

- `PUSH_NOT_AUTHORITATIVE`: aucun push ne vaut décision métier finale.
- `PUSH_TRIGGERS_POLL`: un push valide déclenche un polling des endpoints contractuels.
- `PUSH_DEDUP_REQUIRED` + `PUSH_REPLAY_PROTECTION`: déduplication + TTL obligatoires.
- `NO_SENSITIVE_PUSH_PAYLOAD`: aucun token/secret/PII/transcription dans le payload push.
- Scope mobile: `FCM/APNs/EPNS` réservé au client UI mobile (Android/iOS), hors agent/MCP mobile.

## Normative References

- `../specs/workflows/AGENT-PROTOCOL.md`
- `../specs/api/API-CONTRACTS.md`
- `../specs/policies/AUTHZ-MATRIX.md`
