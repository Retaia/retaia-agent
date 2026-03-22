# Proposition de champs `facts` pour l'API Team

Date: 2026-03-22

## Objet

Ce document prépare une évolution du contrat `FactsPatch` côté Core/API.

L'agent sait déjà:

- stager la source locale
- copier les sidecars déclarés par le job
- extraire les facts techniques minimaux déjà prévus par le contrat OpenAPI v1

En revanche, le contrat exécutable actuel ne permet pas encore de soumettre des facts enrichis issus de:

- EXIF photo
- XMP photo/vidéo
- sidecars `SRT` de captation drone
- métadonnées utiles déjà présentes dans certains conteneurs audio/vidéo

L'objectif ici n'est pas d'implémenter ces extractions dans l'agent immédiatement, mais de proposer à l'équipe API une liste de champs à modéliser avant implémentation.

## Contrat actuel

Le `FactsPatch` actuellement généré pour l'agent transporte uniquement:

- `duration_ms`
- `media_format`
- `video_codec`
- `audio_codec`
- `width`
- `height`
- `fps`

Ces champs couvrent le minimum technique v1, mais pas les métadonnées enrichies mentionnées par les specs prose.

## Décisions de cadrage proposées

- garder les champs techniques minimaux actuels inchangés
- ajouter les nouveaux champs de manière typée, avec unité explicite
- distinguer la provenance des données:
  - média principal
  - EXIF
  - XMP
  - sidecar `SRT`
- marquer les champs sensibles, surtout GPS/localisation
- éviter les noms ambigus ou dépendants d'un constructeur
- rester sur des facts agrégés au niveau asset
- ne pas exposer de télémétrie time-series dans `FactsPatch`

## Règles d'agrégation proposées

Règle générale:

- `FactsPatch` doit rester un patch asset-level
- pas de série temporelle détaillée
- `captured_at_original` est accepté quand la source est fiable, y compris après correction device-spécifique explicitement documentée

Pour les sidecars `SRT` DJI:

- `gps_latitude`, `gps_longitude`, `gps_altitude_relative_m`, `gps_altitude_absolute_m`
  - conserver le premier fix GPS fiable
- `captured_at_original`
  - conserver le premier timestamp fiable
- `iso`, `exposure_time_s`, `aperture_f_number`, `focal_length_mm`, `exposure_compensation_ev`, `color_mode`, `color_temperature_k`
  - conserver la valeur constante si stable sur tout le clip
  - sinon exposer `first_*` / `last_*` uniquement si l'API décide explicitement de supporter cette dérive intra-clip

Pour les sources DJI en général:

- préférer le sidecar `SRT` quand il est disponible pour les facts enrichis
- ne pas dépendre à ce stade du parsing des pistes data propriétaires embarquées
- éventuellement exposer la présence de pistes metadata DJI comme simple signal technique, sans interprétation métier

## Proposition de champs

## Lot minimal recommandé

Si l'équipe API veut ouvrir le chantier avec un lot restreint et immédiatement utile, le plus rentable semble être:

- `captured_at_original`
- `exposure_time_s`
- `aperture_f_number`
- `iso`
- `focal_length_mm`
- `gps_latitude`
- `gps_longitude`
- `gps_altitude_relative_m`
- `gps_altitude_absolute_m`

Ce lot couvre:

- le besoin photo demandé immédiatement
- une base de géolocalisation commune photo/vidéo drone
- des champs réellement observés sur des fichiers photo et vidéo réels

## Champs photo

Champs photo prioritaires:

- `captured_at_original`
  - type suggéré: `string` date-time
  - source typique: EXIF `DateTimeOriginal`, XMP équivalent
- `exposure_time_s`
  - type suggéré: `number`
  - unité: secondes
  - source typique: EXIF `ExposureTime`
- `aperture_f_number`
  - type suggéré: `number`
  - unité: f-number
  - source typique: EXIF `FNumber`
- `iso`
  - type suggéré: `integer`
  - source typique: EXIF `PhotographicSensitivity` / `ISOSpeedRatings`
- `focal_length_mm`
  - type suggéré: `number`
  - unité: millimètres
- `camera_make`
  - type suggéré: `string`
- `camera_model`
  - type suggéré: `string`
- `lens_model`
  - type suggéré: `string`
- `orientation`
  - type suggéré: `integer` ou enum normalisée

Champs photo optionnels si présents:

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

## Champs vidéo

Champs vidéo utiles au-delà du minimum déjà présent:

- `captured_at_original`
  - type suggéré: `string` date-time
- `camera_make`
  - type suggéré: `string`
- `camera_model`
  - type suggéré: `string`
- `bitrate_kbps`
  - type suggéré: `integer`
  - unité: kbps
- `rotation_deg`
  - type suggéré: `integer`
  - unité: degrés
- `sample_rate_hz`
  - type suggéré: `integer`
  - unité: Hz
- `channel_count`
  - type suggéré: `integer`
- `timecode_start`
  - type suggéré: `string`
- `pixel_format`
  - type suggéré: `string`
- `color_range`
  - type suggéré: `string`
- `color_space`
  - type suggéré: `string`
- `color_transfer`
  - type suggéré: `string`
- `color_primaries`
  - type suggéré: `string`

## Champs audio

Champs audio utiles:

- `bitrate_kbps`
  - type suggéré: `integer`
  - unité: kbps
- `sample_rate_hz`
  - type suggéré: `integer`
  - unité: Hz
- `channel_count`
  - type suggéré: `integer`
- `bits_per_sample`
  - type suggéré: `integer`
- `recorder_model`
  - type suggéré: `string`

Note:

- `captured_at_original` n'est pas systématiquement fiable sur les exports audio device
- en revanche, une règle de correction device-spécifique peut le rendre acceptable si elle est déterministe et documentée

## Champs enrichis via sidecar `SRT` DJI

Champs observables et utiles:

- `captured_at_original`
- `gps_latitude`
- `gps_longitude`
- `gps_altitude_relative_m`
- `gps_altitude_absolute_m`
- `iso`
- `exposure_time_s`
- `aperture_f_number`
- `focal_length_mm`
- `exposure_compensation_ev`
- `color_mode`
- `color_temperature_k`

## Signaux techniques de conteneur

Ces champs ne sont pas des facts métier forts, mais il est proposé de les exposer quand même:

- `has_dji_metadata_track`
  - type suggéré: `boolean`
- `dji_metadata_track_types`
  - type suggéré: `string[]`
  - exemple: `["djmd", "dbgi"]`

Justification:

- si Core stocke déjà ces signaux dès l'ingestion initiale
- alors un futur parsing de `djmd` / `dbgi` pourra être relancé sur les assets déjà présents en base
- sans devoir redécouvrir a posteriori quels fichiers contenaient ces pistes

## Exemples observés

## Photo: `CR2` Canon EOS 5D Mark IV

Champs observés:

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

Conclusion:

- le lot photo proposé est réaliste et extractible localement sans heuristique exotique

## Vidéo: `MOV` Canon EOS 5D Mark IV

Champs observés:

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

Conclusion:

- un lot vidéo utile peut être extrait depuis le conteneur QuickTime sans dépendre d'un sidecar

## Vidéo: `MP4` DJI Air 3

Champs observés:

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
- `dji_metadata_track_types = ["djmd", "dbgi"]`
- pas de GPS standard lisible directement via `ffprobe`, ce qui est normal ici

Conclusion:

- le `MP4` DJI signale la présence de metadata propriétaire embarquée
- le `SRT` reste la source la plus simple et la plus lisible pour les facts enrichis

## Sidecar `SRT` DJI Air 3

Champs observés:

- `captured_at_original`
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

Conclusion:

- le `SRT` DJI suffit déjà à produire des facts enrichis asset-level utiles
- pas besoin de télémétrie détaillée dans `FactsPatch`

## Audio: `WAV` RODE Wireless PRO

Champs observés:

- `media_format = wav`
- `duration_ms = 240326`
- `audio_codec = pcm_f32le`
- `sample_rate_hz = 48000`
- `channel_count = 1`
- `bits_per_sample = 32`
- `audio_bitrate_kbps = 1536`
- `recorder_model = RODE Wireless PRO`
- métadonnées device additionnelles présentes:
  - `rFWVER = 2.0.8`
  - `rSPEED = 024.000-ND`
- `bext.origination_date = 0026-03-22`
- `bext.origination_time = 10:03:39`
- `iXML.TIMESTAMP_SAMPLES_SINCE_MIDNIGHT = 1738563538`
- `iXML.TIMESTAMP_SAMPLE_RATE = 48000`
- date filesystem observée: `2026-03-22`
- `captured_at_original = 2026-03-22T10:03:39` après correction device-spécifique RODE

Conclusion:

- un lot audio utile existe
- dans ce cas précis, `captured_at_original` peut être promu après correction documentée
- règle proposée pour RODE:
  - lire `bext.origination_time`
  - lire `iXML.TIMESTAMP_SAMPLES_SINCE_MIDNIGHT`
  - vérifier leur cohérence temporelle
  - remplacer l'année manifestement invalide de `bext.origination_date` par l'année fiable du fichier hôte quand le reste de la date est cohérent

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
9. Les signaux de conteneur comme `has_dji_metadata_track` et `dji_metadata_track_types` sont proposés pour stockage côté Core afin de permettre une relance future de jobs de parsing sur les assets déjà ingérés.
10. Les corrections device-spécifiques de date, comme le cas RODE `bext/iXML`, sont-elles acceptées pour alimenter directement `captured_at_original`?

## Recommandation d'implémentation après validation API

Une fois le contrat API figé:

1. Étendre l'OpenAPI Core.
2. Régénérer le client Rust `retaia-core-client`.
3. Implémenter l'extraction dans l'agent:
   - EXIF photo
   - métadonnées conteneur audio/vidéo
   - sidecars `SRT` si déclarés
4. Ajouter des fixtures et tests dédiés pour chaque famille de champs.
