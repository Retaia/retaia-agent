# Plan d'action priorisé suite à l'audit

Date: 2026-03-20

Source: `docs/AUDIT-SPECS-CODE-TESTS-2026-03-20.md`

## Priorité P0

- Compléter la couverture policy runtime:
  - ajouter les tests de refresh périodique `30s`
  - ajouter les tests de floor `15s` sur refresh anticipé
  - ajouter un test d'intégration daemon pour `GET /app/policy`

## Priorité P1

- Implémenter le bootstrap agent conforme:
  - device flow `start/poll/cancel`
  - ouverture browser vers `UI_WEB`
  - rotation de secret
  - tests CLI/daemon pour ces flows

- Corriger le backoff/polling:
  - brancher aussi `PollEndpoint::DeviceFlow`

## Priorité P2

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
2. Device flow/browser/rotation
3. Polling/backoff
4. Processing réel previews/thumbs/waveform/facts
5. Concurrence OpenAPI dérivés
6. Docs et CI i18n

## Découpage en lots de travail

- Lot 1: contrat et authz
  - tests associés

- Lot 2: policy et bootstrap
  - device flow
  - browser approval
  - rotation secret
  - tests daemon `/app/policy`
  - tests refresh `30s` et floor `15s`

- Lot 3: sécurité locale et polling
  - migration config
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
