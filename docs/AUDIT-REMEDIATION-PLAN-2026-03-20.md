# Plan d'action priorisé suite à l'audit

Date: 2026-03-20

Source: `docs/AUDIT-SPECS-CODE-TESTS-2026-03-20.md`

## Priorité P0

- Implémenter le respect réel de `effective_feature_enabled`:
  - intégrer `GET /app/policy`
  - consommer la policy dans la boucle runtime
  - bloquer l'exécution des capacités/jobs désactivés par policy
  - ajouter les tests de refresh périodique et de floor 15s

## Priorité P1

- Corriger le stockage des secrets:
  - sortir `technical_auth.secret_key` du fichier TOML
  - brancher un secret store OS-native
  - prévoir migration depuis le format actuel
  - ajouter tests d'absence de secret en clair dans la config persistée

- Implémenter le bootstrap agent conforme:
  - device flow `start/poll/cancel`
  - ouverture browser vers `UI_WEB`
  - rotation de secret
  - tests CLI/daemon pour ces flows

- Corriger le backoff/polling:
  - base 429 à `2s`
  - suivi réel du nombre de tentatives
  - prise en compte de `Retry-After`
  - support de `server_policy.min_poll_interval_seconds`
  - brancher aussi `PollEndpoint::Policy` et `PollEndpoint::DeviceFlow`

## Priorité P2

- Corriger les contrats dérivés OpenAPI:
  - remplacer `If-Match: *` par une stratégie conforme au contrat Core
  - vérifier la concurrence optimistic sur `/derived/upload/*`
  - ajouter tests E2E sur `ETag`/`If-Match`

- Corriger les URLs/références de dérivés:
  - supprimer les références `agent://derived/...`
  - s'aligner sur les URLs Core stables/same-origin attendues par la spec

## Priorité P3

- Aligner la doc locale:
  - documenter seulement les flows réellement implémentés
  - compléter la doc auth interactive/browser/device flow quand l'implémentation existera

- Renforcer i18n:
  - ajouter une vraie validation CI de parité des clés `locales/en.json` vs `locales/fr.json`
  - transformer les dérives en échec CI explicite

- Requalifier la suite de tests:
  - renommer ou recadrer les suites `spec_compatible_*` si elles ne prouvent pas la compatibilité normative
  - compléter la couverture sur authz matrix, policy runtime, Accept-Language, anti-rejeu, lock errors

## Ordre d'exécution recommandé

1. Policy/effective features
2. Secret storage
3. Device flow/browser/rotation
4. Polling/backoff
5. Processing réel previews/thumbs/waveform/facts
6. Concurrence OpenAPI dérivés
7. Docs et CI i18n

## Découpage en lots de travail

- Lot 1: contrat et authz
  - tests associés

- Lot 2: policy et bootstrap
  - `/app/policy`
  - `effective_feature_enabled`
  - device flow
  - browser approval
  - rotation secret

- Lot 3: sécurité locale et polling
  - secret store OS-native
  - migration config
  - backoff 429
  - Retry-After
  - min poll interval

- Lot 4: processing réel
  - previews
  - thumbnails
  - waveform
  - facts extraction
  - tests d'intégration

- Lot 5: hardening final
  - ETag/If-Match dérivés
  - docs locales
  - CI i18n
  - renommage/recadrage des suites "spec_compatible"
