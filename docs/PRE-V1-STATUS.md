# Pre-v1 Implementation Status

Last updated: 2026-02-16

## Purpose

Ce document sert de référence de suivi pré-v1 (implémentation + qualité) pour aligner les prochains incréments.

## Scope v1 (normatif)

- Runtime status-driven par polling contractuel.
- Push traité comme hint non autoritatif.
- CLI obligatoire, GUI optionnelle, même moteur runtime.
- Parité de contrat de configuration GUI/CLI, y compris headless.
- Gates CI bloquants: TDD, BDD, E2E, coverage >= 80%.

## Status Summary

### Domain/Application Runtime

- Done:
  - Orchestration runtime (polling contractuel, backoff+jitter 429, push dedup/TTL).
  - Gating mutation après état compatible lu par polling.
  - Runtime controls (`play/pause/stop`) + règles toggle menu.
  - Façade applicative `RuntimeSession` pour composer UI runtime + loop sync.
  - Projection domaine `RuntimeStatusTracker` pour alimenter la fenêtre statut (`job_id`, `asset_uuid`, `%`, `stage`, message) sans logique dupliquée.
- In progress:
  - Intégration shell GUI réelle sur cette base (menu/tray + fenêtre statut).
- Pending:
  - Implémentation UI desktop concrète (au-delà du modèle de domaine/app).

### Notifications

- Done:
  - `New job received` (dédupliqué),
  - `All jobs done` (transition unique),
  - `Job failed` (dédupliqué),
  - `Agent disconnected/reconnecting` (sur transition),
  - `Auth expired/re-auth required`,
  - `Settings saved`,
  - `Settings invalid` (dédupliqué),
  - `Updates available` (version unique).
  - Bridge applicatif de dispatch (`NotificationSink` + `dispatch_notifications`) + adaptateur `StdoutNotificationSink`.
- Pending:
  - Adaptateur notification OS natif (GUI).

### Configuration (GUI/CLI parity + headless)

- Done:
  - Contrat de config unifié (`AgentRuntimeConfig` + validation commune).
  - Persistance système via `directories` + override `RETAIA_AGENT_CONFIG_PATH`.
  - Repository infra (`FileConfigRepository`, `SystemConfigRepository`).
  - CLI headless `agentctl`:
    - `config path`
    - `config show`
    - `config validate`
    - `config init`
    - `config set`
- Pending:
  - Écran/panneau GUI branché sur les mêmes services en production app.

### Test Strategy & CI Gates

- Done:
  - Suites séparées et non monolithiques:
    - `tests/tdd_runtime.rs`
    - `tests/bdd_specs.rs`
    - `tests/e2e_flow.rs`
  - CI bloquante avec jobs dédiés + gate coverage >= 80%.
  - Coverage mesurée: 89.90% (dernier run local).
- In progress:
  - Optimisations de temps CI itératives (cache, filtres, prebuild).

## Remaining Pre-v1 Work (Priority)

1. Shell GUI minimal branché sur le runtime partagé:
   - menu système,
   - fenêtre statut job en cours (%, stage, job_id/asset_uuid, message),
   - accès settings.
2. Bridge notifications OS (émission unique par transition déjà gérée côté domaine).
3. Hardening opérationnel (observabilité runtime et erreurs d’intégration API réelles).
4. Revue finale de conformité v1 contre `specs/` avant freeze.

## Operational Reference

- Docs de base:
  - `docs/RUNTIME-CONSTRAINTS.md`
  - `docs/UX-SYSTEM-TRAY.md`
  - `docs/NOTIFICATIONS.md`
  - `docs/CONFIGURATION-PANEL.md`
  - `docs/CI-QUALITY-GATES.md`
- Source normative:
  - `specs/`
