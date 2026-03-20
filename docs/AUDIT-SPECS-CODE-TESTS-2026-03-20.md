# Audit specs/docs vs code/tests

Date de l'audit: 2026-03-20

Périmètre lu:

- `README.md`
- `docs/README.md`
- `specs/README.md`
- `specs/DOCUMENT-INDEX.md`
- `specs/workflows/AGENT-PROTOCOL.md`
- `specs/api/API-CONTRACTS.md`
- `specs/tests/TEST-PLAN.md`
- `specs/agent/*.md`
- implémentation `src/`
- tests `tests/`

Vérifications exécutées:

- `cargo test` -> OK
- `cargo test --features core-api-client --tests --no-run` -> OK
- `cargo test --features core-api-client --test e2e_openapi_http_flow` -> OK

Historique notable sur `2026-03-20`:

- une erreur de génération OpenAPI dans `crates/retaia-core-client/src/apis/derived_api.rs:526` empêchait auparavant la compilation de la feature `core-api-client`
- ce point a été corrigé dans le repo et la CI de base force désormais aussi `cargo test --features core-api-client --tests --no-run`
- la présente mise à jour d'audit doit donc être lue comme un audit de conformité fonctionnelle/normative, plus comme un audit d'une feature OpenAPI cassée à la compilation

## 1. Ecarts README/docs locales vs repo réel

- `README.md` indique `cargo test --features core-api-client` comme commande de validation des helpers OpenAPI; cette commande recompile désormais, mais ce point a dérivé suffisamment récemment pour montrer que la voie OpenAPI n'était pas protégée par la gate de base avant le correctif CI ajouté le `2026-03-20`.
- `docs/RUNTIME-CONSTRAINTS.md` annonce un "Respect strict de effective_feature_enabled" (`docs/RUNTIME-CONSTRAINTS.md:13`) qui n'est pas observé dans l'implémentation.

## 2. Ecarts code vs specs normatives

### 2.1 Capabilities et noms contractuels

- Le nommage contractuel est maintenant aligné sur `media.previews.*@1` et `generate_preview`.
- Le point restant n'est plus un drift de nommage, mais un drift d'implémentation: le pipeline runtime de preview ne produit pas encore les outputs structurants conformes aux profils canoniques attendus.

### 2.2 Runtime feature flags / policy

- Aucune implémentation applicative de `GET /app/policy` n'est présente dans `src/`.
- Aucune boucle de refresh périodique des `feature_flags` toutes les 30s n'est implémentée.
- Aucun respect du plancher 15s pour refresh anticipé n'est implémenté.
- Aucune consommation de `effective_feature_enabled` n'est implémentée côté agent runtime.
- `resolve_effective_features` ne prend en compte ni `feature_flags`, ni `core_v1_global_features`, ni `feature_governance`, ni `reason_code`, ni `tier`, ni `user_can_disable`.
- `resolve_effective_features` traite implicitement l'absence de clé applicative comme `true`, alors que la spec impose `flag absent = false` pour les `feature_flags`.

### 2.3 Auth technique, device flow et approval UI

- Aucune implémentation de `POST /auth/clients/device/start`, `POST /auth/clients/device/poll` ou `POST /auth/clients/device/cancel` n'est présente dans `src/`.
- Aucun code d'ouverture du browser vers `UI_WEB` pour l'approval humain n'est présent dans `src/`.
- Aucune implémentation de rotation `POST /auth/clients/{client_id}/rotate-secret` n'est présente dans le runtime/CLI.
- Le bootstrap réellement implémenté repose seulement sur `client_id + secret_key` déjà présents en config, pas sur le flow d'approbation décrit par la spec.

### 2.4 MCP et acteurs autorisés

- Le code applicatif agent ne contient plus de surface MCP hors client généré.
- Le point restant côté conformité n'est donc plus "présence de MCP dans l'agent", mais l'absence des flows agent attendus par les specs sur les surfaces conservées.

### 2.5 Polling et backoff

- `src/domain/runtime_orchestration.rs` fixe `BASE_BACKOFF_MS = 500`, alors que la spec impose une base de `2s`.
- Le plafond `60s` est respecté, mais le profil complet canonique n'est pas respecté à cause de la base.
- `src/application/runtime_poll_cycle.rs` appelle toujours `session.on_poll_throttled(..., attempt = 1, ...)`; le nombre de tentatives n'est pas suivi au fil des 429.
- Aucun support de `Retry-After` n'est implémenté.
- `src/bin/agent-runtime.rs` repasse `tick_ms.max(100)` comme intervalle contractuel pour `/jobs`; cela ne modélise ni la cadence canonique `5s`, ni `max(5, server_policy.min_poll_interval_seconds)`.
- Le runtime ne poll que `PollEndpoint::Jobs`; `PollEndpoint::Policy` et `PollEndpoint::DeviceFlow` existent dans les enums mais ne sont pas câblés au daemon.

### 2.6 Processing réel vs processing annoncé

- `src/domain/capabilities.rs` déclare `media.facts@1`, `media.thumbnails@1` et `audio.waveform@1` comme capacités disponibles par défaut.
- `src/application/runtime_job_worker.rs` n'utilise aucun générateur réel; il se contente d'appeler le planner puis le gateway.
- Les implémentations `FfmpegProxyGenerator` et `RustPhotoProxyGenerator` sont désormais branchées pour `generate_preview`, ce qui permet au planner de produire un vrai artefact preview local avant upload.
- Ce branchement reste partiel: `generate_thumbnails`, `generate_audio_waveform` et `extract_facts` ne passent toujours pas par une génération/extraction réelle conforme.
- `src/application/runtime_derived_planner.rs` écrit des références `agent://derived/...`, alors que la spec impose des URLs Core stables et same-origin pour les dérivés exposés par Core.
- Pour `extract_facts`, le planner produit un `manifest` vide et aucun upload; le gateway OpenAPI soumet ensuite un `FactsPatch::new()` vide. Il n'y a pas d'extraction de faits réelle.
- Pour `generate_audio_waveform`, le planner ne calcule aucune waveform; il marque juste un item de manifest de kind `Waveform` et peut uploader le fichier source brut.
- Pour `generate_preview`, le moteur génère maintenant un fichier preview local à partir du média source avec un mapping explicite vers les profils canoniques v1 (`video_review_default_v1`, `audio_review_default_v1`, `photo_review_default_v1`). Les écarts restants sont surtout l'absence de références Core stables et le fait que `generate_thumbnails` reste séparé et non implémenté.
- La spec dit explicitement qu'une waveform requise doit être produite et qu'un asset audio ne doit pas dépasser `READY` sans `waveform_url`; l'implémentation courante ne garantit rien de cela.

### 2.7 Stockage des secrets et sécurité locale

- `src/infrastructure/config_store.rs` persiste `technical_auth.secret_key` en clair dans `StoredTechnicalAuthConfig` (`src/infrastructure/config_store.rs:68-72`, `src/infrastructure/config_store.rs:137-142`).
- La conversion `AgentRuntimeConfig <-> StoredAgentRuntimeConfig` recopie ce secret tel quel dans le TOML de configuration (`src/infrastructure/config_store.rs:146-171`).
- Cela contredit la contrainte locale explicitement rappelée dans `docs/RUNTIME-CONSTRAINTS.md:27-29` et dans les specs de configuration, qui demandent un secret storage OS-native.
- Aucun test ne protège un comportement de stockage sécurisé des secrets; au contraire, le modèle de config actuel normalise le stockage en clair.

### 2.8 GUI/CLI parity et packaging

- Le shell desktop est derrière la feature Cargo `desktop-shell`; le build par défaut n'inclut pas la GUI.
- La parité GUI/CLI est surtout testée au niveau des chaînes de rendu et des actions de menu locales, pas au niveau des flows d'approval/auth/policy complets décrits par la spec.

### 2.9 i18n et garde-fous de validation

- `src/infrastructure/i18n.rs` panique sur JSON de locale invalide (`src/infrastructure/i18n.rs:59-60`), ce qui fournit un garde-fou binaire de chargement mais pas une validation structurée de compatibilité inter-locales.
- La détection de clés manquantes repose sur `debug_assert!` seulement (`src/infrastructure/i18n.rs:43-45`); en build non debug, une clé manquante peut tomber sur `""`.

### 2.10 API client OpenAPI

- La compilation `core-api-client` est désormais réparée et la CI de base la compile explicitement.
- En revanche, `src/infrastructure/openapi_derived_processing_gateway.rs` envoie encore `If-Match: *` comme placeholder fixe pour `upload/init`, `upload/part` et `upload/complete` (`src/infrastructure/openapi_derived_processing_gateway.rs:25`, `src/infrastructure/openapi_derived_processing_gateway.rs:199-215`, `src/infrastructure/openapi_derived_processing_gateway.rs:223-240`, `src/infrastructure/openapi_derived_processing_gateway.rs:273-290`).
- Cette valeur n'est pas dérivée d'un `ETag`/d'une révision réellement lue côté Core; si le contrat serveur durcit la concurrence optimistic, cette implémentation risque de diverger.
- Le mapping OpenAPI local est désormais aligné sur `GeneratePreview` / `Preview*`; le point restant est la sémantique effective des artefacts générés, pas leur nommage.

## 3. Ecarts tests vs specs

### 3.1 Les tests restent centrés sur un pipeline preview encore transport-only

- Le nommage de contrat a été aligné dans les tests (`media.previews.*`, `GeneratePreview`, `Preview*`).
- En revanche, plusieurs tests continuent de protéger un pipeline qui accepte surtout des manifests/artefacts transportés, sans exiger la génération effective des previews normatives.

### 3.2 Les tests valident une waveform vide alors que la spec impose un dérivé

- `tests/bdd_specs/derived_job_executor.rs` considère valide un job `generate_audio_waveform` sans upload ni manifest waveform.
- Des tests TDD/E2E reprennent la même hypothèse.
- Références directes: `tests/bdd_specs/derived_job_executor.rs:209-218`, `tests/tdd_runtime/derived_job_executor.rs:667-670`, `tests/e2e_flow/derived_job_executor_flow.rs:319-322`.
- Cela contredit explicitement `specs/workflows/AGENT-PROTOCOL.md` et `specs/api/API-CONTRACTS.md`, qui imposent une waveform quand le profil la requiert.

### 3.3 Les tests de planner/executor acceptent une extraction de faits vide

- `tests/tdd_runtime/runtime_derived_planner.rs:57-79` vérifie surtout que `extract_facts` reste sans upload et avec `manifest` vide.
- `tests/tdd_runtime/derived_job_executor.rs:778-812` considère valide un flow `extract_facts` sans vérifier la production d'un patch de faits utile.
- Ces tests verrouillent une implémentation "transport-only" du job `extract_facts`, alors que la spec attend une extraction de faits effective.

### 3.4 Les tests ne couvrent pas des pans normatifs majeurs

- Aucun test de `GET /app/policy` côté runtime agent.
- Aucun test de refresh périodique des flags/policy à 30s.
- Aucun test de floor 15s sur refresh anticipé.
- Aucun test de `POST /auth/clients/device/start`.
- Aucun test de `POST /auth/clients/device/poll`.
- Aucun test de `POST /auth/clients/device/cancel`.
- Aucun test de rotation `POST /auth/clients/{client_id}/rotate-secret`.
- Aucun test de flow browser vers `UI_WEB`.
- Aucun test de support `Accept-Language` sur les appels REST du runtime agent.
- Aucun test de prise en compte `Retry-After` sur 429.
- Aucun test d'anti-rejeu, de fenêtre de fraîcheur `<= 60s` ou de gestion de nonce côté signatures.
- Aucun test de refus explicite `LOCK_REQUIRED`, `LOCK_INVALID`, `STALE_LOCK_TOKEN`.
- Aucun test de `server_policy.min_poll_interval_seconds`.
- Aucun test de `effective_feature_enabled` bloquant réellement l'exécution.
- Aucun test de stockage OS-native de `secret_key` ou d'absence de secret en clair dans la config persistée.
- Aucun test de production réelle de preview/thumb/waveform via les générateurs du repo.
- Aucun test ne vérifie qu'un `extract_facts` produit un patch utile.
- Aucun test de flux browser/approval `UI_WEB`.
- Aucun test de concurrence `If-Match`/`ETag` réellement alimentée depuis Core sur `/derived/upload/*`.

### 3.5 Les tests "spec_compatible" ne prouvent pas la compatibilité spec

- Les suites `spec_compatible_*` vérifient surtout des contrats locaux de session/menu/config/notifications.
- Elles ne valident pas les exigences normatives les plus structurantes: policy runtime, auth bootstrap, device flow, flags, authz matrice, URLs Core stables des dérivés, waveform obligatoire, MCP asymétrique.
- Le nom "spec_compatible" est donc plus large que ce que les assertions couvrent réellement.

## 4. Ecarts docs/test/code sur le runtime réel

- Le README annonce "Derived-processing v1 runtime support", mais le runtime ne fait ni génération effective de previews, ni thumbnails, ni waveform, ni facts extraction.
- Le README annonce "Derived-processing v1 runtime support"; la génération effective des previews est désormais branchée, mais les thumbnails, la waveform, les facts et les références Core stables restent incomplètes.
- Le README annonce "Strict contract alignment with specs/", mais le policy polling, la waveform obligatoire et la génération effective des previews/facts divergent encore au niveau du code et des tests.
- Le README annonce le même contrat de configuration GUI/CLI; en pratique le build par défaut ne livre pas la GUI.
- Les docs locales de contraintes runtime annoncent un stockage OS-native des secrets, mais la config persistée garde toujours `secret_key` en clair.
- Le runtime reste partiellement générique via `ui-web` et `ui-mobile`, alors que les flows normatifs complets attendus côté agent ne sont pas encore implémentés.
- Les tests passent en mode par défaut, mais ce succès reflète surtout le contrat local actuel, pas la conformité aux specs normatives lues.

## 5. Synthèse courte

Le repo est partiellement structuré pour la spec v1, mais il n'est pas aligné sur plusieurs axes contractuels centraux:

- absence de policy runtime et de device flow
- secret technique persisté en clair
- backoff 429 non conforme
- runtime de processing surtout "transport/protocole", pas "processing" réel
- tests qui valident plusieurs comportements contraires à la spec
- couverture de tests absente sur plusieurs invariants normatifs
- voie OpenAPI recompilable, mais encore avec hypothèses de concurrence et de sémantique locales discutables

En l'état, `cargo test` qui passe n'est pas un signal suffisant de conformité aux specs/docs.
