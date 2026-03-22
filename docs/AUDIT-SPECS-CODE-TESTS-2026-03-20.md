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
- Les docs locales sont désormais globalement réalignées sur l'état réel de l'agent; le résiduel documentaire concerne surtout les étapes finales dépendantes de Core et de `UI_WEB`.

## 2. Ecarts code vs specs normatives

### 2.1 Capabilities et noms contractuels

- Le nommage contractuel est maintenant aligné sur `media.previews.*@1` et `generate_preview`.
- Le pipeline runtime de preview, thumbnails, waveform et facts est désormais branché sur des générateurs réels; le résiduel se situe surtout côté publication finale Core, pas dans le nommage agent.

### 2.2 Runtime feature flags / policy

- `GET /app/policy` est désormais câblé via la gateway OpenAPI jobs/policy et consommé dans la boucle daemon.
- La boucle daemon recharge maintenant la policy toutes les `30s`, et un test dédié du binaire daemon couvre désormais ce cadencement.
- Le daemon applique désormais un plancher `15s` sur les refresh policy anticipés et ce comportement est couvert par des tests dédiés.
- Le runtime bloque désormais `can_process_jobs()` tant que `features.core.jobs.runtime` n'est pas activé dans la policy Core.
- `resolve_effective_features` prend désormais en compte `feature_flags` et `core_v1_global_features`, et traite correctement `feature_flags` absent comme `false`.
- Le repo agent ne consomme pas encore la métadonnée descriptive complète (`feature_governance`, `reason_code`, `tier`, `user_can_disable`); ce résiduel est surtout informatif et cross-app.

### 2.3 Auth technique, device flow et approval UI

- `src/bin/agentctl.rs` et `src/infrastructure/technical_auth.rs` implémentent désormais le bootstrap device flow CLI via `POST /auth/clients/device/start`, `POST /auth/clients/device/poll` et `POST /auth/clients/device/cancel`, avec persistance du `client_id` en config et du `secret_key` dans le secret store local après approval.
- `src/bin/agentctl.rs` contient désormais un ouvreur de browser natif pour lancer l'approval humain vers `UI_WEB` via `verification_uri_complete`.
- `src/bin/agentctl.rs` et `src/infrastructure/technical_auth.rs` implémentent désormais la rotation CLI `POST /auth/clients/{client_id}/rotate-secret`, avec mise à jour du secret store local.
- `src/bin/agent-runtime.rs` câble désormais aussi `PollEndpoint::DeviceFlow` dans le daemon: démarrage du bootstrap browser-assisted en mode interactif, polling du `device_code`, persistance du `technical_auth` à l'approval, puis reprise du cycle runtime normal avec gateways reconstruits.

### 2.4 MCP et acteurs autorisés

- Le code applicatif agent ne contient plus de surface MCP hors client généré.
- Il ne reste pas d'écart agent-local notable sur ce bloc.

### 2.5 Polling et backoff

- `src/domain/runtime_orchestration.rs` applique maintenant une base canonique `2s` et garde bien le plafond `60s`.
- Le runtime suit désormais un compteur de tentatives 429 par endpoint dans le moteur de sync, avec reset après succès.
- La gateway HTTP jobs/policy lit désormais `Retry-After` sur `429` et le daemon réutilise ce `wait_ms` pour recalculer les prochains polls.
- `src/bin/agent-runtime.rs` respecte désormais `max(5s, server_policy.min_poll_interval_seconds)` pour le polling `/jobs`.
- `PollEndpoint::Policy` et `PollEndpoint::DeviceFlow` sont désormais câblés au daemon.

### 2.6 Processing réel vs processing annoncé

- `src/domain/capabilities.rs` déclare `media.facts@1`, `media.thumbnails@1` et `audio.waveform@1` comme capacités disponibles par défaut.
- `src/application/runtime_job_worker.rs` n'utilise aucun générateur réel; il se contente d'appeler le planner puis le gateway.
- Les implémentations `FfmpegProxyGenerator` et `RustPhotoProxyGenerator` sont désormais branchées pour `generate_preview`, ce qui permet au planner de produire un vrai artefact preview local avant upload.
- `src/application/runtime_derived_planner.rs` écrit désormais des références Core stables same-origin de la forme `/api/v1/assets/{uuid}/derived/{kind}` pour les dérivés runtime.
- Pour `extract_facts`, le planner produit désormais un `facts_patch` réel à partir du média source, sans upload, et le gateway OpenAPI soumet ce patch à `SubmitExtractFacts`.
- Pour `generate_audio_waveform`, le planner génère désormais un payload JSON réel (`duration_ms`, `bucket_count`, `samples[]`) avec `bucket_count=1000`, puis l'uploade comme dérivé `waveform`.
- Pour `generate_preview`, le moteur génère maintenant un fichier preview local à partir du média source avec un mapping explicite vers les profils canoniques v1 (`video_review_default_v1`, `audio_review_default_v1`, `photo_review_default_v1`) et une référence Core stable same-origin.
- Pour `generate_thumbnails`, le moteur produit désormais un storyboard vidéo réel par défaut avec plusieurs `thumb` same-origin distincts (`/derived/thumbs/{n}`) et le profil local `video_storyboard_v1`; il retombe sur un thumb représentatif unique `video_representative_v1` quand la durée n'est pas disponible.
- La spec dit explicitement qu'une waveform requise doit être produite et qu'un asset audio ne doit pas dépasser `READY` sans `waveform_url`; l'executor local n'accepte plus une waveform vide et les références runtime sont désormais same-origin, mais la publication finale dépend encore du Core et du contrat `If-Match`/`ETag`.

### 2.7 Stockage des secrets et sécurité locale

- `technical_auth.secret_key` n'est plus persistée dans `config.toml`; `src/infrastructure/config_store.rs` sérialise seulement `client_id` et relit le secret depuis le secret store local.
- Le loader migre automatiquement les anciens fichiers TOML contenant encore `secret_key` inline vers le secret store, puis réécrit une version assainie du fichier.
- Les flows agent de bootstrap et rotation sont désormais implémentés; le résiduel est le parcours humain complet côté `UI_WEB`.

### 2.8 GUI/CLI parity et packaging

- Le shell desktop est derrière la feature Cargo `desktop-shell`; le build par défaut n'inclut pas la GUI.
- La parité GUI/CLI est surtout testée au niveau des chaînes de rendu et des actions de menu locales, pas au niveau des flows d'approval/auth/policy complets décrits par la spec.

### 2.9 i18n et garde-fous de validation

- `src/infrastructure/i18n.rs` panique désormais aussi sur clé i18n absente dans tous les builds, au lieu de retomber silencieusement sur `""`.
- `scripts/validate_locales.py` et la job CI `validate-locales` valident désormais explicitement la parité de clés `locales/en.json` vs `locales/fr.json` et les valeurs non vides.

### 2.10 API client OpenAPI

- La compilation `core-api-client` est désormais réparée et la CI de base la compile explicitement.
- Le mapping OpenAPI local est désormais aligné sur `GeneratePreview` / `Preview*`; le point restant est la sémantique effective des artefacts générés, pas leur nommage.

## 3. Ecarts tests vs specs

### 3.1 Les tests ne couvrent pas encore toute la publication finale côté Core

- Le nommage de contrat a été aligné dans les tests (`media.previews.*`, `GeneratePreview`, `Preview*`).
- Le pipeline agent génère désormais réellement previews, thumbnails, waveform et facts; ce qui reste hors couverture locale est surtout la publication finale observée depuis Core.

### 3.2 Les tests n'autorisent plus une waveform vide, mais ne couvrent pas encore toute la conformité finale

- `tests/bdd_specs/derived_job_executor.rs`, `tests/tdd_runtime/derived_job_executor.rs` et `tests/e2e_flow/derived_job_executor_flow.rs` rejettent désormais un job `generate_audio_waveform` sans dérivé produit.
- Cela aligne l'executor local avec `specs/workflows/AGENT-PROTOCOL.md` et `specs/api/API-CONTRACTS.md` sur l'obligation de dérivé waveform.
- Les trous restants sont surtout la projection finale via URL Core stable et la validation fine du contenu rendu côté Core.

### 3.3 Les tests couvrent maintenant un `facts_patch` utile, mais pas encore toute la finesse métier

- `tests/tdd_runtime/runtime_derived_planner.rs` vérifie désormais qu'un `extract_facts` runtime remplit un `facts_patch` utile sans upload.
- `tests/tdd_runtime/derived_job_executor.rs` vérifie désormais qu'un flow runtime `extract_facts` soumet bien ce patch.
- Les suites fixtures externes couvrent désormais aussi l'extraction réelle des facts minimaux sur de vrais médias audio/vidéo/photo.
- Le trou restant est surtout la projection finale côté Core.

### 3.4 Les tests ne couvrent pas des pans normatifs majeurs

- Un test e2e `agentctl` couvre désormais `POST /auth/clients/device/start` puis `POST /auth/clients/device/poll` jusqu'à approval et persistance locale des credentials techniques.
- Un test e2e `agentctl` couvre désormais `POST /auth/clients/device/cancel` lors d'une interruption utilisateur du bootstrap.
- Un test e2e `agentctl` couvre désormais `POST /auth/clients/{client_id}/rotate-secret` et la mise à jour locale du secret technique.
- Un test e2e `agentctl` couvre désormais le chemin d'ouverture browser via une commande override injectée, avec vérification de `verification_uri_complete`.
- La gateway runtime `jobs/policy` envoie désormais `Accept-Language`, et un test HTTP dédié le couvre sur les appels REST agent.
- Les tests de `signed_core_http` couvrent désormais l'émission d'un nonce distinct par requête et une fenêtre de fraîcheur locale `<= 60s` sur le timestamp signé.
- La suite HTTP OpenAPI couvre désormais aussi un validateur Core mocké qui rejette un `nonce` rejoué et un timestamp signé trop vieux depuis une requête réellement envoyée par l'agent.
- Les suites `authz` couvrent désormais plus explicitement la matrice locale v1 modélisée par l'agent: `AGENT` seul peut mint un token technique et traiter des jobs, `UI_WEB`/`UI_MOBILE` sont refusés, et les flags `CORE_V1_GLOBAL` restent forcés à `true`.
- La gateway OpenAPI dérivés mappe désormais explicitement `LOCK_REQUIRED`, `LOCK_INVALID` et `STALE_LOCK_TOKEN`, avec tests HTTP dédiés.
- Un test dédié du binaire daemon couvre désormais le chemin de récupération et d'application de `GET /app/policy`.
- Il n'y a toujours pas de test d'approval humain complet côté `UI_WEB`; la couverture actuelle s'arrête à l'ouverture du navigateur depuis `agentctl`.

### 3.5 Les anciennes suites "spec_compatible" étaient sur-vendues

- Les suites concernées ont été recadrées sous des noms plus précis orientés "runtime contract/config contract".
- Le fond reste inchangé: elles vérifient surtout des contrats locaux de session/menu/config/notifications.
- Elles ne valident pas, à elles seules, la publication finale Core ni le parcours humain complet côté `UI_WEB`.

## 4. Ecarts docs/test/code sur le runtime réel

- Le principal résiduel n'est plus dans le code agent local mais dans la publication finale observée côté Core après `submit_derived`.
- Le parcours humain complet côté `UI_WEB` n'est toujours pas vérifiable depuis ce repo seul.
- Les tests qui passent ici sont désormais un bon signal de conformité agent locale, mais pas une preuve de conformité cross-app complète.

## 5. Synthèse courte

Le repo agent est désormais largement aligné sur la spec v1 pour son périmètre propre.

Les sujets restants sont surtout externes au repo:

- publication finale et projection des dérivés côté Core
- parcours humain complet côté `UI_WEB`
- validation cross-app bout-en-bout

En l'état, `cargo test` qui passe est un bon signal de conformité agent locale, mais pas une preuve de conformité système complète.
