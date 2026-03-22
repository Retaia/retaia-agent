# Audit du bump `specs` du 2026-03-22

## Périmètre

Révision précédente du submodule `specs`:

- `9e30c1f14374b13102bde1307fee7b4e188ea0e2`

Révision courante visée:

- `ed86b95dd1409b65f347e85dc78af46258b01c44`

Commits inclus depuis le bump précédent:

- `dc87bbe` `chore(deps): bump github/codeql-action from 3 to 4 (#117)`
- `8ff7102` `chore(deps): bump actions/checkout from 4 to 6 (#118)`
- `ed86b95` `docs: enrich facts contract and prepare transcript rollout (#119)`

## Fichiers normatifs modifiés dans `specs`

Le commit `ed86b95` modifie les surfaces normatives suivantes:

- `api/API-CONTRACTS.md`
- `api/openapi/v1.yaml`
- `change-management/FEATURE-FLAG-REGISTRY.md`
- `contracts/openapi-v1.sha256`
- `definitions/JOB-TYPES.md`
- `policies/AUTHZ-MATRIX.md`
- `policies/FEATURE-RESOLUTION-ENGINE.md`
- `tests/TEST-PLAN.md`

## Changements à appliquer côté agent

## 1. `facts_patch` enrichi

Le contrat OpenAPI autorise désormais explicitement des champs enrichis dans `FactsPatch`, notamment:

- `captured_at`
- `exposure_time_s`
- `aperture_f_number`
- `iso`
- `focal_length_mm`
- `camera_make`
- `camera_model`
- `lens_model`
- `orientation`
- `bitrate_kbps`
- `sample_rate_hz`
- `channel_count`
- `bits_per_sample`
- `rotation_deg`
- `timecode_start`
- `pixel_format`
- `color_range`
- `color_space`
- `color_transfer`
- `color_primaries`
- `recorder_model`
- `gps_latitude`
- `gps_longitude`
- `gps_altitude_m`
- `gps_altitude_relative_m`
- `gps_altitude_absolute_m`
- `exposure_compensation_ev`
- `color_mode`
- `color_temperature_k`
- `has_dji_metadata_track`
- `dji_metadata_track_types[]`

Actions agent à appliquer:

- régénérer le client OpenAPI `core-api-client`
- étendre le mapping local `FactsPatchPayload`
- brancher l'extraction réelle de ces champs quand ils sont disponibles de façon déterministe
- garder les minima techniques existants obligatoires

## 2. `captured_at` et GPS dédiés

Le contrat ajoute des champs dédiés côté lecture détaillée:

- `summary.captured_at`
- `gps_latitude`
- `gps_longitude`
- `gps_altitude_m`
- `gps_altitude_relative_m`
- `gps_altitude_absolute_m`
- `location_country`
- `location_city`
- `location_label`

Conséquence pour l'agent:

- ne plus raisonner comme si ces données devaient rester dans `fields`
- préparer les `facts_patch` pour promotion Core vers ces champs typés

## 3. `transcribe_audio` entre dans le contrat partagé pré-release

Le job type `transcribe_audio` apparaît désormais dans:

- `Job.job_type`
- `SubmitJobResultRequest`
- `ProcessingResultPatch`
- `TranscriptPatch`

Le contrat `TranscriptPatch` ajoute:

- `status`
- `text`
- `text_preview`
- `language`
- `updated_at`

Actions agent à appliquer:

- régénérer le client OpenAPI
- ajouter le support de transport `transcribe_audio`
- ajouter le type local `transcript_patch`
- préparer l'executor/planner pour ce job sous `features.ai.transcribe_audio`
- ne pas le considérer comme bloquant pour la complétude `v1`

## 4. Nettoyage des anciennes feature keys runtime

Les anciennes clés assimilées au nominal sortent du contrat runtime actif:

- `features.core.auth`
- `features.core.assets.lifecycle`
- `features.core.jobs.runtime`
- `features.core.search.query`
- `features.core.policy.runtime`
- `features.core.derived.access`
- `features.core.clients.bootstrap`

Contraintes nouvelles:

- ces clés ne doivent plus apparaître dans `feature_flags`
- elles ne doivent plus apparaître dans `app_feature_enabled`
- elles ne doivent plus apparaître dans `user_feature_enabled`
- elles ne doivent plus apparaître dans `effective_feature_enabled`
- elles ne doivent plus apparaître dans `feature_governance`
- `core_v1_global_features` ne fait plus partie du contrat runtime partagé

Actions agent à appliquer:

- retirer tout branchement runtime restant sur ces anciennes clés
- supprimer la dépendance à `core_v1_global_features`
- adapter le moteur local de résolution des features au nouveau registre actif
- adapter les tests qui supposent encore l'émission de ces clés

## 5. Règles de validation des anciennes clés

Le contrat impose désormais que les anciennes clés `deprecated`:

- soient refusées avec `422 VALIDATION_FAILED` quand envoyées dans `PATCH /app/features`
- soient refusées avec `422 VALIDATION_FAILED` quand envoyées dans `PATCH /auth/me/features`

Impact agent:

- si l'agent émet encore ces clés dans des flows utilisateur/admin, c'est désormais non conforme
- les fixtures et tests HTTP doivent vérifier leur absence

## 6. Tests à ajouter ou corriger

Le plan de tests du submodule a évolué. Côté agent, il faut désormais couvrir explicitement:

- sérialisation OpenAPI des nouveaux champs `FactsPatch`
- absence des anciennes feature keys `deprecated` dans les payloads runtime
- non-utilisation de `core_v1_global_features`
- support transport `transcribe_audio`
- `TranscriptPatch` et projection HTTP associée
- payloads GPS/location enrichis quand présents
- signaux `has_dji_metadata_track` / `dji_metadata_track_types`

## Priorités recommandées

## P0

- bump effectif du submodule `specs` vers `ed86b95`
- régénération `core-api-client`
- correction des types générés et du mapping `FactsPatch`

## P1

- mise à jour du moteur local de feature resolution
- suppression des anciennes clés runtime `deprecated`
- suppression de `core_v1_global_features` dans la logique et les tests

## P2

- ajout du transport `transcribe_audio`
- ajout des tests HTTP et runtime associés

## P3

- implémentation progressive des extracteurs enrichis réels:
  - EXIF photo
  - conteneur vidéo/audio
  - sidecar `SRT` DJI
  - correction RODE `bext/iXML` pour `captured_at`

## Conclusion

Contrairement au bump intermédiaire vers `8ff7102`, le bump réel vers `ed86b95` introduit un delta normatif substantiel applicable à `retaia-agent`.

Les principaux travaux à mener sont:

- régénérer le client OpenAPI
- adopter le nouveau contrat `FactsPatch`
- intégrer `transcribe_audio`
- purger les anciennes feature keys runtime `deprecated`
