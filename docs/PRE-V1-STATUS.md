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
  - Port applicatif `CoreApiGateway` + projection `poll_runtime_snapshot(...)` pour intégrer le polling jobs réel sans coupler le domaine au transport HTTP.
  - Port applicatif `DerivedProcessingGateway` (claim/heartbeat/submit + upload init/part/complete) + validation v1 des couples `derived kind`/`content_type`.
  - Socle capabilities v1: déclaration agent `media.facts@1`, `media.proxies.*@1`, `media.thumbnails@1`, `audio.waveform@1` + garde défensive de compatibilité (`required_capabilities ⊆ capabilities déclarées`) avant projection runtime.
  - Use-case d'enregistrement agent (`register_agent`) + port DDD `AgentRegistrationGateway` + adapter OpenAPI `POST /agents/register` publiant explicitement les capabilities déclarées.
- In progress:
  - Intégration shell GUI réelle sur cette base (menu/tray + fenêtre statut).
  - Contrat applicatif shell GUI minimal implémenté (`runtime_gui_shell`): actions menu (`play/pause/stop/open status/open settings`), rendu statut/settings, contrôle daemon (`start/stop/status`) via port partagé `DaemonManager`.
  - Contrôleur desktop applicatif (`DesktopShellController`) implémenté pour relier bridge toolkit GUI et moteur partagé (`RuntimeSession`) avec orchestration menu/statut/settings/quit.
  - Shell desktop minimal réel ajouté sous feature `desktop-shell` (`agent-desktop-shell`) via fenêtre native + raccourcis clavier + fenêtres status/settings.
  - Shell runtime CLI interactif (`agent-runtime`) ajouté comme miroir headless du menu système; UI desktop finale encore pending.
  - Boucle daemon branchée sur cycle poll runtime (`run_runtime_poll_cycle`) avec projection d'état dégradé (unauthorized/connectivité) et notifications associées.
- Pending:
  - Intégration tray/menu système native complète (au-delà du shell desktop minimal fenêtre + raccourcis).

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
  - Bridge applicatif de dispatch (`NotificationSink` + `dispatch_notifications`) + adaptateurs `SystemNotificationSink` (OS si supporté, résultat strict OK/NOK), `StdoutNotificationSink` et `TauriNotificationSink` (feature `tauri-notifications`).
  - Politique de sélection runtime des adapters (`notification_sink_profile_for_target` + `select_notification_sink`) pour router headless vs desktop sans logique dupliquée.
  - Intégration façade runtime: `RuntimeSession::update_snapshot_and_dispatch(...)`.
- Pending:
  - Brancher `TauriNotificationSink` dans le shell GUI toolkit final (policy headless/desktop déjà branchée côté runtime).

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
    - `daemon install/start/stop/status/uninstall` (service manager natif via port partagé)
  - Parsing CLI migré vers `clap`.
- Pending:
  - Écran/panneau GUI branché sur les mêmes services en production app.

### Test Strategy & CI Gates

- Done:
  - Suites séparées et non monolithiques:
    - `tests/tdd_capabilities.rs`
    - `tests/tdd_configuration.rs`
    - `tests/tdd_runtime_core.rs`
    - `tests/bdd_capabilities_authz.rs`
    - `tests/bdd_configuration_and_infra.rs`
    - `tests/bdd_runtime_behavior.rs`
    - `tests/e2e_authz_capabilities.rs`
    - `tests/e2e_configuration.rs`
    - `tests/e2e_runtime_behavior.rs`
  - CI bloquante avec jobs dédiés + gate coverage >= 80%.
  - Coverage mesurée: 89.90% (dernier run local).
- In progress:
  - Optimisations de temps CI itératives (cache, filtres, prebuild).
  - Ajouter des fixtures RAW réelles (Canon `CR2/CR3`, Nikon `NEF/NRW`, Sony `ARW`) dans les suites TDD/BDD/E2E photo proxy pour valider la compatibilité preview pre-v1.

### Engineering Baseline

- Done:
  - Stratégie lib-first sur ce repo: `clap` (CLI parsing) + `thiserror` (types d’erreurs).
  - Client API généré OpenAPI (`reqwest-trait`) branché via adapter infra `OpenApiJobsGateway` (feature `core-api-client`).
  - Adapter infra OpenAPI pour processing dérivés (`OpenApiDerivedProcessingGateway`) sur jobs + derived APIs.

## Remaining Pre-v1 Work (Priority)

1. Shell GUI minimal branché sur le runtime partagé:
   - menu système (controller app prêt, intégration toolkit desktop pending),
   - fenêtre statut job en cours (controller app prêt, intégration toolkit desktop pending),
   - accès settings (controller app prêt, intégration toolkit desktop pending).
2. Intégration shell GUI finale des adapters de notification selon cible (desktop/headless).
3. Hardening opérationnel (observabilité runtime et erreurs d’intégration API réelles).
   - Partiellement démarré: logs structurés par cycle daemon + corrélation `job_id/asset_uuid` quand disponible.
4. Ajouter une matrice de tests avec fixtures RAW réelles pour photo proxy preview (Canon/Nikon/Sony), avec résultats attendus documentés (supporté/non supporté).
5. Revue finale de conformité v1 contre `specs/` avant freeze.

## Operational Reference

- Docs de base:
  - `docs/RUNTIME-CONSTRAINTS.md`
  - `docs/UX-SYSTEM-TRAY.md`
  - `docs/NOTIFICATIONS.md`
  - `docs/CONFIGURATION-PANEL.md`
  - `docs/CI-QUALITY-GATES.md`
- Source normative:
  - `specs/`
