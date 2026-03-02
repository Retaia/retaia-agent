# V1 Specs Conformity Matrix

Last reviewed: 2026-03-03

## Scope

Ce document trace la conformité v1 de `retaia-agent` contre les specs normatives de `specs/`.
Statuts:

- `Covered`: implémenté et couvert par tests.
- `Covered (doc/runtime)`: implémenté et documenté, sans endpoint direct côté agent.
- `Planned v1.1+`: explicitement hors périmètre v1.

## Matrix

| Normative source | Requirement (v1) | Status | Evidence |
|---|---|---|---|
| `specs/workflows/AGENT-PROTOCOL.md` | Polling status-driven, push hint non autoritatif, backoff+jitter sur `429` | Covered | `tests/tdd_runtime/runtime_orchestration.rs`, `tests/bdd_specs/runtime_orchestration.rs`, `tests/e2e_flow/runtime_specs_flow.rs`, `src/application/runtime_orchestration.rs` |
| `specs/workflows/AGENT-PROTOCOL.md` | Actions mutatrices seulement après état compatible lu par polling | Covered | `tests/tdd_runtime/runtime_orchestration.rs`, `tests/tdd_runtime/runtime_sync.rs`, `src/application/runtime_sync.rs` |
| `specs/workflows/AGENT-PROTOCOL.md` | Register agent + capabilities déclarées | Covered | `tests/tdd_capabilities/agent_registration.rs`, `tests/bdd_specs/agent_registration.rs`, `tests/e2e_flow/agent_registration_flow.rs`, `src/application/agent_registration.rs` |
| `specs/workflows/AGENT-PROTOCOL.md` + `specs/definitions/CAPABILITIES.md` | Matching strict `required_capabilities ⊆ capabilities` | Covered | `tests/tdd_capabilities/capabilities.rs`, `tests/bdd_specs/core_api_gateway.rs`, `tests/e2e_flow/core_api_gateway_flow.rs`, `src/application/core_api_gateway.rs` |
| `specs/workflows/AGENT-PROTOCOL.md` (`storage_mounts` + `/.retaia`) | Résolution source basée sur marker (`storage_id` match, roots validées, échec explicite si marker absent/invalide) | Covered | `tests/tdd_runtime/source_path_resolver.rs`, `tests/tdd_runtime/source_staging.rs`, `src/domain/configuration.rs`, `src/application/source_staging.rs` |
| `specs/workflows/AGENT-PROTOCOL.md` + `specs/policies/AUTHZ-MATRIX.md` | `MCP` ne traite jamais de jobs processing | Covered | `tests/tdd_capabilities/feature_flags.rs`, `tests/bdd_capabilities_authz.rs`, `tests/e2e_authz_capabilities.rs`, `src/domain/feature_flags.rs` |
| `specs/workflows/AGENT-PROTOCOL.md` | Claim atomique + heartbeat + submit/fail via gateways dédiés | Covered | `tests/tdd_runtime/derived_processing_gateway.rs`, `tests/e2e_flow/derived_processing_gateway_flow.rs`, `src/infrastructure/openapi_derived_processing_gateway.rs` |
| `specs/api/API-CONTRACTS.md` (`/jobs`, `/jobs/*`, `/derived/upload/*`, `/agents/register`) | Mapping erreurs HTTP transport (`401/429/422/5xx`) + payload invalides | Covered | `tests/e2e_openapi_http_flow.rs`, `src/infrastructure/openapi_jobs_gateway.rs`, `src/infrastructure/openapi_derived_processing_gateway.rs`, `src/infrastructure/openapi_agent_registration_gateway.rs` |
| `specs/tests/TEST-PLAN.md` (gates v1) | Suites TDD/BDD/E2E séparées + gate coverage | Covered | `tests/tdd_runtime_core.rs`, `tests/bdd_runtime_behavior.rs`, `tests/e2e_runtime_behavior.rs`, `docs/CI-QUALITY-GATES.md` |
| `specs/tests/TEST-PLAN.md` | CLI obligatoire, GUI optionnelle, même moteur runtime | Covered | `src/bin/agentctl.rs`, `src/bin/agent-runtime.rs`, `src/bin/agent-desktop-shell.rs`, `tests/e2e_flow/runtime_gui_shell_flow.rs`, `tests/e2e_flow/runtime_desktop_shell_controller_flow.rs` |
| `specs/policies/I18N-LOCALIZATION.md` | `en`/`fr`, fallback, parité des clés, pas de logique métier sur labels | Covered | `locales/en.json`, `locales/fr.json`, `tests/tdd_runtime/i18n.rs`, `src/infrastructure/i18n.rs` |
| `specs/api/OBSERVABILITY-CONTRACT.md` + `specs/workflows/AGENT-PROTOCOL.md` | Logs structurés + corrélation job + historique debug long-run | Covered (doc/runtime) | `src/bin/agent-runtime.rs`, `src/infrastructure/runtime_history_store.rs`, `src/infrastructure/daemon_diagnostics.rs`, `docs/DAEMON-MODE.md` |
| `specs/tests/TEST-PLAN.md` (derived format compliance - proxy photo) | Validation preview RAW multi-marques sur corpus réel | Covered | `fixtures/external/manifest.tsv`, `tests/bdd_specs/external_fixtures_photo_proxy.rs`, `tests/bdd_runtime_behavior.rs` |
| `specs/tests/TEST-PLAN.md` (derived format compliance - audio/video) | Validation AV réelle (H264/H265, WAV/MP3/AAC, VFR/CFR) | Covered | `fixtures/external/manifest.tsv`, `tests/e2e_flow/external_fixtures_av_flow.rs`, `tests/e2e_runtime_behavior.rs` |
| `specs/api/API-CONTRACTS.md` (rollout) | Features AI/transcription/providers | Planned v1.1+ | Hors scope v1 dans `specs/api/API-CONTRACTS.md` |
| `specs/workflows/AGENT-PROTOCOL.md` + `specs/api/API-CONTRACTS.md` | Push mobile (`FCM/APNs/EPNS`) | Planned v1.2 | Hors scope v1 dans `specs/api/API-CONTRACTS.md` |

## Conclusion v1

- Le socle v1 agent est conforme sur runtime, authz/capabilities, OpenAPI transport mapping, i18n, observabilité opératoire et tests gates.
- Le run final de freeze est validé et publié: `docs/V1-FREEZE-REPORT.md`.

## V1 Freeze Checklist

- [x] Runtime status-driven / polling contractuel validé en TDD/BDD/E2E.
- [x] Mapping authz/capabilities agent (`AGENT` vs `MCP`) validé.
- [x] Mapping transport OpenAPI (`401/429/422/5xx` + payloads invalides) validé.
- [x] Parité config + modèle daemon (`CLI === GUI`) validée.
- [x] i18n v1 (`en`/`fr`, fallback, parité de clés) validé.
- [x] Observabilité opérationnelle daemon (stats + history + bug report) validée.
- [x] Corpus RAW réel onboardé et validé (`CR2/CR3/NEF/NRW/ARW`).
- [x] Corpus vidéo/audio réel onboardé et validé (`H264/H265`, `WAV/MP3/AAC`, `CFR/VFR`).
- [x] Checksums + attentes du corpus externe versionnés dans le repo.
- [x] Re-run final des gates CI avec corpus externe et publication du rapport de freeze (`docs/V1-FREEZE-REPORT.md`).
