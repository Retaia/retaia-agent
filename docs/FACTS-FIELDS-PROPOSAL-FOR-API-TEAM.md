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

Le cadrage retenu pour cette proposition est volontairement limité:

- pas de télémétrie time-series dans `FactsPatch`
- seulement des facts agrégés au niveau asset
- pour le GPS drone, conserver uniquement le premier fix GPS fiable

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

Exemple réel observé sur un `CR2` Canon EOS 5D Mark IV:

- `camera_make = Canon`
- `camera_model = Canon EOS 5D Mark IV`
- `captured_at_original = 2025-12-01T15:28:02`
- `exposure_time_s = 1/60 = 0.016666...`
- `aperture_f_number = 5.0`
- `iso = 100`
- `focal_length_mm = 24`
- `lens_model = EF24-70mm f/2.8L II USM`
- `orientation = 1`
- `width = 6720`
- `height = 4480`
- pas de GPS détecté dans cet exemple précis

Ce fichier confirme donc qu'un lot photo utile et réaliste peut être extrait localement sans heuristique exotique.

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
- `sample_rate_hz`
  - type suggéré: `integer`
  - unité: Hz
- `channel_count`
  - type suggéré: `integer`
- `timecode_start`
  - type suggéré: `string`

Pour les sources DJI, la recommandation côté agent est:

- préférer le sidecar `SRT` quand il est disponible pour les facts enrichis
- ne pas dépendre à ce stade du parsing des pistes data propriétaires embarquées
- éventuellement exposer la présence de pistes metadata DJI comme signal technique, sans interprétation métier

Exemple réel observé sur un `MOV` Canon EOS 5D Mark IV:

- `captured_at_original = 2026-03-16T15:11:29Z`
- `camera_make = Canon`
- `camera_model = Canon EOS 5D Mark IV`
- `media_format = mov`
- `duration_ms = 17680`
- `video_codec = mjpeg`
- `audio_codec = pcm_s16le`
- `width = 4096`
- `height = 2160`
- `fps = 25`
- `video_bitrate_kbps = 522866`
- `audio_bitrate_kbps = 1536`
- `sample_rate_hz = 48000`
- `channel_count = 2`
- `timecode_start = 01:53:08:03`
- pas de GPS détecté, ce qui est normal pour ce type de `MOV` boîtier seul

Ce fichier confirme qu'un lot vidéo utile peut être extrait depuis le conteneur QuickTime sans dépendre d'un sidecar.

Exemple réel observé sur un `MP4` DJI Air 3:

- `captured_at_original = 2024-04-12T14:00:17Z`
- `media_format = mp4`
- `duration_ms = 118560`
- `video_codec = hevc`
- `width = 3840`
- `height = 2160`
- `fps = 25`
- `video_bitrate_kbps = 90002`
- `pixel_format = yuv420p10le`
- `color_range = tv`
- `color_space = bt709`
- `color_transfer = bt709`
- `color_primaries = bt709`
- `encoder = DJI Air3`
- `has_dji_metadata_track = true`
  - pistes observées: `djmd`, `dbgi`
- pas de GPS standard lisible directement via `ffprobe`, ce qui est normal ici

Ce fichier confirme qu'un `MP4` DJI peut signaler la présence de metadata propriétaire embarquée, mais que le `SRT` reste la source la plus simple et la plus lisible pour les facts enrichis.

## Drone / sidecar `SRT`

Un `SRT` réel DJI Air 3 montre déjà des champs parseables de manière fiable:

- timestamp original de capture
- `iso`
- `shutter`
- `fnum`
- `ev`
- `focal_len`
- `latitude`
- `longitude`
- `rel_alt`
- `abs_alt`
- `ct`
- `color_md`

Pour `FactsPatch`, la recommandation n'est pas d'exposer la télémétrie complète frame par frame. La bonne granularité ici est asset-level:

- premier fix GPS fiable
- première valeur fiable
- dernière valeur fiable
- ou valeur constante si stable sur tout le clip

Ces champs restent utiles à cadrer au niveau API avant extraction.

- `gps_latitude`
  - type suggéré: `number`
  - unité: degrés décimaux
  - sensible: oui
- `gps_longitude`
  - type suggéré: `number`
  - unité: degrés décimaux
  - sensible: oui
- `gps_altitude_relative_m`
  - type suggéré: `number`
  - unité: mètres
  - sensible: oui
- `gps_altitude_absolute_m`
  - type suggéré: `number`
  - unité: mètres
  - sensible: oui
- `exposure_compensation_ev`
  - type suggéré: `number`
- `color_mode`
  - type suggéré: `string`
- `color_temperature_k`
  - type suggéré: `integer`
  - unité: kelvin

Règles d'agrégation suggérées pour les champs issus du `SRT`:

- `gps_latitude`, `gps_longitude`, `gps_altitude_relative_m`, `gps_altitude_absolute_m`
  - conserver le premier fix GPS fiable
- `captured_at_original`
  - conserver le premier timestamp fiable
- `iso`, `exposure_time_s`, `aperture_f_number`, `focal_length_mm`, `exposure_compensation_ev`, `color_mode`, `color_temperature_k`
  - conserver la valeur constante si stable sur tout le clip
  - sinon exposer `first_*` et `last_*` uniquement si l'API veut vraiment représenter la dérive intra-clip

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
3. Pour les sidecars `SRT`, veut-on autoriser seulement un agrégat asset-level et exclure explicitement toute télémétrie détaillée du contrat?
4. Veut-on modéliser la provenance du champ:
   - `source = media | exif | xmp | srt`
5. Veut-on accepter les champs partiels sans garantir leur présence sur tous les médias?
6. Quel format exact retenir pour les champs numériques photo:
   - `exposure_time_s` en décimal
   - `aperture_f_number` en décimal
   - `iso` en entier
7. Faut-il normaliser certains champs en enums:
   - `orientation`
   - `media_format`
   - `color_mode`
8. Le contrat doit-il prévoir des champs temporels "originaux" séparés des dates d'ingestion côté Core?
9. Si une valeur varie dans un `SRT`, veut-on:
   - ne conserver que la première valeur fiable
   - ne conserver que la dernière valeur fiable
   - ou accepter un couple `first_*` / `last_*` pour certains champs

## Recommandation minimale pour un premier lot API

Si l'équipe API veut ouvrir le chantier avec un lot restreint et immédiatement utile, le plus rentable semble être:

- `exposure_time_s`
- `aperture_f_number`
- `iso`
- `captured_at_original`
- `gps_latitude`
- `gps_longitude`
- `gps_altitude_relative_m`
- `gps_altitude_absolute_m`

Ce lot couvre:

- le besoin photo demandé immédiatement
- une base de géolocalisation commune photo/vidéo drone
- des champs photo réellement observés dans un `CR2` Canon
- des champs vidéo réellement observés dans un `MOV` Canon
- les champs réellement observés dans un `SRT` DJI Air 3
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
