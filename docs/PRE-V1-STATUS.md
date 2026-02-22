# Pre-v1 Implementation Status

Last updated: 2026-02-22

## Purpose

Ce document sert de référence de suivi pré-v1 (implémentation + qualité) pour aligner les prochains incréments.

## Scope v1 (normatif)

- Runtime status-driven par polling contractuel.
- Push traité comme hint non autoritatif.
- CLI obligatoire, GUI optionnelle, même moteur runtime.
- Parité de contrat de configuration GUI/CLI, y compris headless.
- Gates CI bloquants: TDD, BDD, E2E, coverage globale agrégée >= 80%.

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
  - Intégration shell GUI réelle sur cette base (menu/tray + fenêtre statut/préférences).
  - Contrat applicatif shell GUI minimal implémenté (`runtime_gui_shell`): actions menu (`play/pause/stop/open status/open settings`), rendu statut/settings, contrôle daemon (`start/stop/status`) via port partagé `DaemonManager`.
  - Contrôleur desktop applicatif (`DesktopShellController`) implémenté pour relier bridge toolkit GUI et moteur partagé (`RuntimeSession`) avec orchestration menu/statut/settings/quit.
  - Shell desktop réel ajouté sous feature `desktop-shell` (`agent-desktop-shell`) avec tray natif, fenêtre masquable (`hide to tray`), raccourcis clavier et actions runtime/daemon synchronisées.
  - Fenêtre desktop "control center" branchée (boutons cliquables + stats runtime + dernier job/durée observée) tout en gardant le tray comme point d'entrée principal.
  - Shell runtime CLI interactif (`agent-runtime`) ajouté comme miroir headless du menu système.
  - Boucle daemon branchée sur cycle poll runtime (`run_runtime_poll_cycle`) avec projection d'état dégradé (unauthorized/connectivité) et notifications associées.
- Pending:
  - Enrichissement UX desktop (fenêtre contrôles persistants + historique jobs/metrics détaillées).

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
  - CI bloquante avec jobs dédiés + gate coverage globale agrégée >= 80% (TDD+BDD+E2E).
  - Génération des rapports de coverage par suite (TDD/BDD/E2E) conservée pour diagnostic non bloquant.
  - Coverage mesurée: 89.90% (dernier run local).
- In progress:
  - Optimisations de temps CI itératives (cache, filtres, prebuild).
  - Scénarios sans fixtures externes ajoutés:
    - robustesse runtime multi-ticks (`success/throttle/unauthorized`) + dédup notifications,
    - renforcement executor dérivés pour `audio.waveform@1` (manifest compatible/incompatible),
    - cas négatifs photo proxy sans médias externes (inputs invalides, fallback decoder, conversions, write path).
  - Ajouter des fixtures RAW réelles (Canon `CR2/CR3`, Nikon `NEF/NRW`, Sony `ARW`) dans les suites TDD/BDD/E2E photo proxy pour valider la compatibilité preview pre-v1.
  - Préparer le corpus fixture externe versionné (checksums + attentes) pour valider la preview RAW réelle sans rendu complet.
  - Ajouter des scénarios photo proxy pre-v1 avec fixtures:
    - happy path RAW par marque/modèle (Canon/Nikon/Sony),
    - RAW non supporté (erreur contrôlée, sans panic),
    - RAW corrompu/tronqué (échec contrôlé),
    - incohérence extension/contenu (comportement déterministe),
    - lot mixte (`jpg/png/tiff/webp/raw`) avec rapport succès/échecs,
    - smoke perf preview sur RAW volumineux (borne temps large).
  - Ajouter/maintenir des tests photo proxy faisables sans fixtures externes:
    - source JPEG/PNG/TIFF/WEBP valides -> proxy JPEG/WEBP généré avec dimensions bornées,
    - fichier inexistant/illisible -> erreur contrôlée,
    - extension trompeuse (ex: `.cr2` avec contenu texte) -> échec déterministe,
    - fichier vide/tronqué -> échec contrôlé sans panic,
    - paramètres invalides (qualité/dimensions) -> validation explicite.
  - Ajouter des fixtures vidéo/audio réelles pour proxy generation:
    - vidéo: H264/H265, CFR/VFR, présence/absence de piste audio,
    - audio: WAV/MP3/AAC, sample rates atypiques, mono/stéréo.
  - Ajouter des scénarios waveform orientés capability `audio.waveform@1`:
    - waveform produite,
    - waveform absente mais job non bloquant,
    - format/manifest cohérent côté submit.
  - Ajouter des tests d’adapters OpenAPI avec payloads/réponses réelles:
    - mapping des erreurs HTTP (`401/429/422/5xx`),
    - payloads incomplets/invalides sur jobs et derived upload.
  - Ajouter des scénarios runtime de robustesse:
    - enchaînements multi-ticks success/throttle/unauthorized,
    - stabilité notifications (pas de répétition parasite),
    - cohérence des transitions état runtime en mode daemon.

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
4. Ajouter une matrice de tests avec fixtures RAW réelles pour photo proxy preview (Canon/Nikon/Sony), avec résultats attendus documentés (supporté/non supporté) et checksums.
5. Couvrir explicitement les cas négatifs et robustesse photo proxy (RAW non supporté, fichier corrompu, extension/contenu incohérents, batch mixte, smoke perf) sur corpus réel.
6. Ajouter une matrice de fixtures vidéo/audio pour `media.proxies.video@1` et `media.proxies.audio@1` (H264/H265, CFR/VFR, WAV/MP3/AAC, mono/stéréo, edge sample rates).
7. Renforcer la couverture capability `audio.waveform@1` (production waveform, absence non bloquante, cohérence submit/manifest).
8. Ajouter des tests de mapping d’erreurs et payloads pour les adapters OpenAPI (`jobs`, `derived upload`, `agent registration`) avec réponses HTTP réalistes.
9. Ajouter des tests de robustesse runtime daemon sur séquences longues (success/throttle/unauthorized) avec vérification de déduplication notifications.
10. Revue finale de conformité v1 contre `specs/` avant freeze.

## Fixture Roadmap (Pre-v1)

- Faisable sans fichiers externes (à implémenter/maintenir en priorité):
  - Cas unitaires et intégration sur formats déjà présents dans le repo (JPEG/PNG/TIFF/WEBP).
  - Cas d’erreur structurels (fichier absent, vide, tronqué, permissions, extension incohérente).
  - Validation des invariants de sortie (format proxy, bornes dimensions, erreurs stables).
- Dépendant de fixtures externes (à onboarder avant freeze v1):
  - RAW Canon (`CR2/CR3`), Nikon (`NEF/NRW`), Sony (`ARW`) pour vérifier extraction preview embarquée.
  - Vidéo/audio représentatifs pour `media.proxies.video@1`, `media.proxies.audio@1` et `audio.waveform@1`.
  - Matrice d’attendus documentée (succès/échec contrôlé, timings de smoke perf, checksums).

## Operational Reference

- Docs de base:
  - `docs/RUNTIME-CONSTRAINTS.md`
  - `docs/UX-SYSTEM-TRAY.md`
  - `docs/NOTIFICATIONS.md`
  - `docs/CONFIGURATION-PANEL.md`
  - `docs/CI-QUALITY-GATES.md`
- Source normative:
  - `specs/`
