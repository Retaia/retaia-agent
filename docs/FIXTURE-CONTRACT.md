# External Fixture Contract (Pre-v1 Freeze)

## Goal

Standardiser l'onboarding des fixtures externes (RAW/vidéo/audio) avec un manifest checksum vérifiable avant freeze v1.

## Where to put files

Place les médias dans `fixtures/external/`:

- RAW photo:
  - `fixtures/external/raw/canon/` (`.cr2`, `.cr3`)
  - `fixtures/external/raw/nikon/` (`.nef`, `.nrw`)
  - `fixtures/external/raw/sony/` (`.arw`)
- Audio:
  - `fixtures/external/audio/wav/` (`.wav`)
  - `fixtures/external/audio/flac/` (`.flac`)
  - `fixtures/external/audio/mp3/` (`.mp3`)
  - `fixtures/external/audio/aac/` (`.aac`, `.m4a`)
- Video:
  - `fixtures/external/video/h264/` (`.mp4`, H264)
  - `fixtures/external/video/h265/` (`.mp4`, H265)

## Manifest

Déclare chaque fichier dans `fixtures/external/manifest.tsv`:

columns:

- `relative_path` (relatif à `fixtures/external/`)
- `sha256`
- `kind` (`raw_photo|proxy_video|proxy_audio`)
- `expected` (`supported|unsupported|negative`)
- `notes` (optionnel)

## Validation command

```bash
./scripts/validate_external_fixtures.sh
```

Optionnel avec manifest alternatif:

```bash
./scripts/validate_external_fixtures.sh /path/to/manifest.tsv
```

Le script échoue sur:

- fichier manquant,
- checksum invalide,
- `kind` invalide,
- `expected` invalide.
