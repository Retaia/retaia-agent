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

- `GET /app/policy` est désormais câblé via la gateway OpenAPI jobs/policy et consommé dans la boucle daemon.
- La boucle daemon recharge maintenant la policy toutes les `30s`, mais ce cadencement n'est pas encore couvert par un test dédié.
- Aucun respect du plancher 15s pour refresh anticipé n'est implémenté.
- Le runtime bloque désormais `can_process_jobs()` tant que `features.core.jobs.runtime` n'est pas activé dans la policy Core.
- `resolve_effective_features` prend désormais en compte `feature_flags` et `core_v1_global_features`, et traite correctement `feature_flags` absent comme `false`.
- `resolve_effective_features` ne modélise toujours pas `feature_governance`, `reason_code`, `tier` ni `user_can_disable`.

### 2.3 Auth technique, device flow et approval UI

- Aucune implémentation de `POST /auth/clients/device/start`, `POST /auth/clients/device/poll` ou `POST /auth/clients/device/cancel` n'est présente dans `src/`.
- Aucun code d'ouverture du browser vers `UI_WEB` pour l'approval humain n'est présent dans `src/`.
- Aucune implémentation de rotation `POST /auth/clients/{client_id}/rotate-secret` n'est présente dans le runtime/CLI.
- Le bootstrap réellement implémenté repose seulement sur `client_id + secret_key` déjà présents en config, pas sur le flow d'approbation décrit par la spec.

### 2.4 MCP et acteurs autorisés

- Le code applicatif agent ne contient plus de surface MCP hors client généré.
- Le point restant côté conformité n'est donc plus "présence de MCP dans l'agent", mais l'absence des flows agent attendus par les specs sur les surfaces conservées.

### 2.5 Polling et backoff

- `src/domain/runtime_orchestration.rs` applique maintenant une base canonique `2s` et garde bien le plafond `60s`.
- Le runtime suit désormais un compteur de tentatives 429 par endpoint dans le moteur de sync, avec reset après succès.
- Aucun support de `Retry-After` n'est implémenté.
- `src/bin/agent-runtime.rs` respecte désormais `max(5s, server_policy.min_poll_interval_seconds)` pour le polling `/jobs`.
- `PollEndpoint::Policy` est désormais câblé au daemon; `PollEndpoint::DeviceFlow` reste non implémenté.

### 2.6 Processing réel vs processing annoncé

- `src/domain/capabilities.rs` déclare `media.facts@1`, `media.thumbnails@1` et `audio.waveform@1` comme capacités disponibles par défaut.
- `src/application/runtime_job_worker.rs` n'utilise aucun générateur réel; il se contente d'appeler le planner puis le gateway.
- Les implémentations `FfmpegProxyGenerator` et `RustPhotoProxyGenerator` sont désormais branchées pour `generate_preview`, ce qui permet au planner de produire un vrai artefact preview local avant upload.
- Ce branchement reste partiel: `generate_thumbnails` produit désormais un thumb représentatif réel en `WEBP`, mais le mode `video_storyboard_v1` n'est pas implémenté.
- `src/application/runtime_derived_planner.rs` écrit désormais des références Core stables same-origin de la forme `/api/v1/assets/{uuid}/derived/{kind}` pour les dérivés runtime.
- Pour `extract_facts`, le planner produit désormais un `facts_patch` réel à partir du média source, sans upload, et le gateway OpenAPI soumet ce patch à `SubmitExtractFacts`.
- Pour `generate_audio_waveform`, le planner génère désormais un payload JSON réel (`duration_ms`, `bucket_count`, `samples[]`) avec `bucket_count=1000`, puis l'uploade comme dérivé `waveform`.
- Pour `generate_preview`, le moteur génère maintenant un fichier preview local à partir du média source avec un mapping explicite vers les profils canoniques v1 (`video_review_default_v1`, `audio_review_default_v1`, `photo_review_default_v1`) et une référence Core stable same-origin.
- Pour `generate_thumbnails`, le moteur produit maintenant un thumb principal réel avec le profil canonique local `video_representative_v1`, mais il n'implémente pas encore `video_storyboard_v1` ni la sélection temporelle fine basée sur la durée.
- La spec dit explicitement qu'une waveform requise doit être produite et qu'un asset audio ne doit pas dépasser `READY` sans `waveform_url`; l'executor local n'accepte plus une waveform vide et les références runtime sont désormais same-origin, mais la publication finale dépend encore du Core et du contrat `If-Match`/`ETag`.

### 2.7 Stockage des secrets et sécurité locale

- `technical_auth.secret_key` n'est plus persistée dans `config.toml`; `src/infrastructure/config_store.rs` sérialise seulement `client_id` et relit le secret depuis le secret store local.
- Le loader migre automatiquement les anciens fichiers TOML contenant encore `secret_key` inline vers le secret store, puis réécrit une version assainie du fichier.
- Le point restant côté conformité n'est plus le stockage en clair local, mais l'absence des flows normatifs de bootstrap/rotation décrits par la spec.

### 2.8 GUI/CLI parity et packaging

- Le shell desktop est derrière la feature Cargo `desktop-shell`; le build par défaut n'inclut pas la GUI.
- La parité GUI/CLI est surtout testée au niveau des chaînes de rendu et des actions de menu locales, pas au niveau des flows d'approval/auth/policy complets décrits par la spec.

### 2.9 i18n et garde-fous de validation

- `src/infrastructure/i18n.rs` panique sur JSON de locale invalide (`src/infrastructure/i18n.rs:59-60`), ce qui fournit un garde-fou binaire de chargement mais pas une validation structurée de compatibilité inter-locales.
- La détection de clés manquantes repose sur `debug_assert!` seulement (`src/infrastructure/i18n.rs:43-45`); en build non debug, une clé manquante peut tomber sur `""`.

### 2.10 API client OpenAPI

- La compilation `core-api-client` est désormais réparée et la CI de base la compile explicitement.
- Le mapping OpenAPI local est désormais aligné sur `GeneratePreview` / `Preview*`; le point restant est la sémantique effective des artefacts générés, pas leur nommage.

## 3. Ecarts tests vs specs

### 3.1 Les tests restent centrés sur un pipeline preview encore transport-only

- Le nommage de contrat a été aligné dans les tests (`media.previews.*`, `GeneratePreview`, `Preview*`).
- En revanche, plusieurs tests continuent de protéger un pipeline qui accepte surtout des manifests/artefacts transportés, sans exiger la génération effective des previews normatives.

### 3.2 Les tests n'autorisent plus une waveform vide, mais ne couvrent pas encore toute la conformité finale

- `tests/bdd_specs/derived_job_executor.rs`, `tests/tdd_runtime/derived_job_executor.rs` et `tests/e2e_flow/derived_job_executor_flow.rs` rejettent désormais un job `generate_audio_waveform` sans dérivé produit.
- Cela aligne l'executor local avec `specs/workflows/AGENT-PROTOCOL.md` et `specs/api/API-CONTRACTS.md` sur l'obligation de dérivé waveform.
- Les trous restants sont surtout la projection finale via URL Core stable et la validation fine du contenu rendu côté Core.

### 3.3 Les tests couvrent maintenant un `facts_patch` utile, mais pas encore toute la finesse métier

- `tests/tdd_runtime/runtime_derived_planner.rs` vérifie désormais qu'un `extract_facts` runtime remplit un `facts_patch` utile sans upload.
- `tests/tdd_runtime/derived_job_executor.rs` vérifie désormais qu'un flow runtime `extract_facts` soumet bien ce patch.
- Les trous restants sont surtout la validation fine des champs minimaux par type média sur de vrais fixtures audio/vidéo/photo et la projection finale côté Core.

### 3.4 Les tests ne couvrent pas des pans normatifs majeurs

- Aucun test d'intégration de refresh périodique des flags/policy à `30s` dans la boucle daemon.
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
- Aucun test bout-en-bout de récupération `GET /app/policy` dans le daemon.
- Aucun test de production réelle de preview/thumb/waveform via les générateurs du repo.
- Aucun test ne vérifie qu'un `extract_facts` produit un patch utile.
- Aucun test de flux browser/approval `UI_WEB`.

### 3.5 Les tests "spec_compatible" ne prouvent pas la compatibilité spec

- Les suites `spec_compatible_*` vérifient surtout des contrats locaux de session/menu/config/notifications.
- Elles ne valident pas les exigences normatives les plus structurantes: policy runtime, auth bootstrap, device flow, flags, authz matrice, URLs Core stables des dérivés, waveform obligatoire, MCP asymétrique.
- Le nom "spec_compatible" est donc plus large que ce que les assertions couvrent réellement.

## 4. Ecarts docs/test/code sur le runtime réel

- Le README annonce "Derived-processing v1 runtime support", mais le runtime ne fait ni génération effective de previews, ni thumbnails, ni waveform, ni facts extraction.
- Le README annonce "Derived-processing v1 runtime support"; la génération effective des previews est désormais branchée, mais les thumbnails, la waveform, les facts et les références Core stables restent incomplètes.
- Le README annonce "Strict contract alignment with specs/", mais le policy polling, la waveform obligatoire et la génération effective des previews/facts divergent encore au niveau du code et des tests.
- Le README annonce le même contrat de configuration GUI/CLI; en pratique le build par défaut ne livre pas la GUI.
- Le runtime reste partiellement générique via `ui-web` et `ui-mobile`, alors que les flows normatifs complets attendus côté agent ne sont pas encore implémentés.
- Les tests passent en mode par défaut, mais ce succès reflète surtout le contrat local actuel, pas la conformité aux specs normatives lues.

## 5. Synthèse courte

Le repo est partiellement structuré pour la spec v1, mais il n'est pas aligné sur plusieurs axes contractuels centraux:

- absence de policy runtime et de device flow
- backoff 429 non conforme
- runtime de processing surtout "transport/protocole", pas "processing" réel
- tests qui valident plusieurs comportements contraires à la spec
- couverture de tests absente sur plusieurs invariants normatifs
- voie OpenAPI recompilable, mais encore avec hypothèses de concurrence et de sémantique locales discutables

En l'état, `cargo test` qui passe n'est pas un signal suffisant de conformité aux specs/docs.
