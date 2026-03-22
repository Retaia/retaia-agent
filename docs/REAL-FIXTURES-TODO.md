# Real Fixtures TODO

Liste des fichiers reels utiles a ajouter pour durcir les tests d'extraction de facts enrichis cote agent.

## Priorite 1

- `DJI MP4 + SRT` associe
  - objectif: valider l'extraction drone depuis le sidecar `SRT`
  - champs attendus:
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
  - note: on prefere explicitement le `SRT` a `djmd/dbgi` pour le moment

## Priorite 2

- photo avec `GPS EXIF` reel
  - formats utiles: `JPEG`, `HEIC`, `DNG`, `CR2`, `CR3`
  - objectif: verrouiller:
    - `gps_latitude`
    - `gps_longitude`
    - `gps_altitude_m`

- photo avec `DateTimeOriginal + OffsetTimeOriginal`
  - objectif: valider un `captured_at` photo normalise en UTC
  - note: beaucoup de RAW exposent la date mais pas l'offset, donc ce cas vaut un fixture dedie

## Priorite 3

- `MOV` ou `MP4` camera avec `timecode` reel
  - objectif: verrouiller `timecode_start`

- audio recorder non-RODE avec metadata stable
  - exemples utiles: `Zoom`, `Sound Devices MixPre`
  - objectif: comparer:
    - `captured_at`
    - `recorder_model`
    - metadata audio container

## Regles de selection

- preferer des fichiers petits mais metadata-riches
- garder un fichier par cas metier principal, pas une collection redondante
- preferer des valeurs simples a verifier dans les tests
- si un fichier contient des donnees sensibles reelles, le sanitiser avant ajout au repo
