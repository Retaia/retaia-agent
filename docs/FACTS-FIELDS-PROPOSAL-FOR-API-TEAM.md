# Proposition de champs `facts` pour l'API Team

Date: 2026-03-22

## Objet

Ce document prépare une évolution du contrat `FactsPatch` côté Core/API.

Le runtime agent sait déjà:

- stager la source locale
- copier les sidecars déclarés par le job
- extraire les facts techniques minimaux déjà prévus par le contrat OpenAPI v1

En revanche, le contrat exécutable actuel ne permet pas encore de soumettre des facts enrichis issus de:

- EXIF photo
- XMP photo/vidéo
- sidecars `SRT` de captation drone

L'objectif ici n'est pas d'implémenter ces extractions dans l'agent immédiatement, mais de proposer à l'équipe API une liste de champs à modéliser avant implémentation.

## État actuel du contrat exécutable

Le `FactsPatch` actuellement généré pour l'agent transporte uniquement:

- `duration_ms`
- `media_format`
- `video_codec`
- `audio_codec`
- `width`
- `height`
- `fps`

Ces champs couvrent le minimum technique v1, mais pas les métadonnées enrichies mentionnées par les specs prose.

## Principes de modélisation proposés

- Garder les champs techniques minimaux actuels inchangés.
- Ajouter les nouveaux champs de manière typée, avec unité explicite.
- Distinguer clairement la provenance des données:
  - média principal
  - EXIF
  - XMP
  - sidecar `SRT`
- Marquer les champs sensibles, surtout GPS/localisation.
- Éviter les noms ambigus ou dépendants d'un constructeur.
- Préférer des champs déjà normalisés par l'écosystème photo/vidéo quand c'est possible.

## Champs candidats

## Photo: EXIF/XMP

Champs demandés en priorité:

- `exposure_time_s`
  - type suggéré: `number`
  - unité: secondes
  - exemple: `0.005`
  - source typique: EXIF `ExposureTime`
- `aperture_f_number`
  - type suggéré: `number`
  - unité: f-number
  - exemple: `2.8`
  - source typique: EXIF `FNumber`
- `iso`
  - type suggéré: `integer`
  - exemple: `400`
  - source typique: EXIF `PhotographicSensitivity` / `ISOSpeedRatings`

Autres champs photo probablement utiles à prévoir dans le même lot:

- `captured_at_original`
  - type suggéré: `string` date-time
  - source: EXIF `DateTimeOriginal`, XMP équivalent
- `camera_make`
  - type suggéré: `string`
- `camera_model`
  - type suggéré: `string`
- `lens_model`
  - type suggéré: `string`
- `focal_length_mm`
  - type suggéré: `number`
  - unité: millimètres
- `orientation`
  - type suggéré: `integer` ou enum normalisée
- `gps_latitude`
  - type suggéré: `number`
  - unité: degrés décimaux
  - sensible: oui
- `gps_longitude`
  - type suggéré: `number`
  - unité: degrés décimaux
  - sensible: oui
- `gps_altitude_m`
  - type suggéré: `number`
  - unité: mètres
  - sensible: oui

## Vidéo: média principal + XMP éventuel

Champs utiles au-delà du minimum déjà présent:

- `bitrate_kbps`
  - type suggéré: `integer`
  - unité: kbps
- `rotation_deg`
  - type suggéré: `integer`
  - unité: degrés
- `captured_at_original`
  - type suggéré: `string` date-time
- `camera_make`
  - type suggéré: `string`
- `camera_model`
  - type suggéré: `string`

## Drone / sidecar `SRT`

Ces champs dépendent fortement du format exact du sidecar et du constructeur. Ils restent utiles à cadrer au niveau API avant extraction.

- `gps_latitude`
  - type suggéré: `number`
  - unité: degrés décimaux
  - sensible: oui
- `gps_longitude`
  - type suggéré: `number`
  - unité: degrés décimaux
  - sensible: oui
- `gps_altitude_m`
  - type suggéré: `number`
  - unité: mètres
  - sensible: oui
- `gps_speed_mps`
  - type suggéré: `number`
  - unité: mètres/seconde
- `camera_heading_deg`
  - type suggéré: `number`
  - unité: degrés
- `gimbal_pitch_deg`
  - type suggéré: `number`
  - unité: degrés
- `gimbal_yaw_deg`
  - type suggéré: `number`
  - unité: degrés
- `drone_make`
  - type suggéré: `string`
- `drone_model`
  - type suggéré: `string`
- `telemetry_sample_count`
  - type suggéré: `integer`

## Audio

L'audio a moins de métadonnées réellement utiles dans le flux agent actuel, mais quelques champs peuvent valoir la peine si l'API veut les exposer:

- `bitrate_kbps`
  - type suggéré: `integer`
  - unité: kbps
- `sample_rate_hz`
  - type suggéré: `integer`
  - unité: Hz
- `channel_count`
  - type suggéré: `integer`

## Questions à trancher côté API

1. Veut-on étendre `FactsPatch` directement, ou introduire une sous-structure dédiée aux facts enrichis?
2. Les champs GPS doivent-ils vivre dans le même objet que les facts techniques, ou dans une zone explicitement sensible?
3. Veut-on modéliser la provenance du champ:
   - `source = media | exif | xmp | srt`
4. Veut-on accepter les champs partiels sans garantir leur présence sur tous les médias?
5. Quel format exact retenir pour les champs numériques photo:
   - `exposure_time_s` en décimal
   - `aperture_f_number` en décimal
   - `iso` en entier
6. Faut-il normaliser certains champs en enums:
   - `orientation`
   - éventuellement `media_format`
7. Le contrat doit-il prévoir des champs temporels "originaux" séparés des dates d'ingestion côté Core?

## Recommandation minimale pour un premier lot API

Si l'équipe API veut ouvrir le chantier avec un lot restreint et immédiatement utile, le plus rentable semble être:

- `exposure_time_s`
- `aperture_f_number`
- `iso`
- `captured_at_original`
- `gps_latitude`
- `gps_longitude`
- `gps_altitude_m`

Ce lot couvre:

- le besoin photo demandé immédiatement
- une base de géolocalisation commune photo/vidéo drone
- un minimum de valeur métier sans élargir trop vite le contrat

## Recommandation d'implémentation après validation API

Une fois le contrat API figé:

1. Étendre l'OpenAPI Core.
2. Régénérer le client Rust `retaia-core-client`.
3. Implémenter l'extraction dans l'agent:
   - EXIF photo
   - XMP si disponible
   - sidecars `SRT` si déclarés
4. Ajouter des fixtures et tests dédiés pour chaque famille de champs.
