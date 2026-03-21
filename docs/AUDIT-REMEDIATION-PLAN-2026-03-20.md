# Plan d'action priorisé suite à l'audit

Date: 2026-03-20

Source: `docs/AUDIT-SPECS-CODE-TESTS-2026-03-20.md`

## Priorité P0

- Compléter la couverture policy runtime:

## Priorité P1

- Implémenter le bootstrap agent conforme:
  - bootstrap daemon désormais câblé

- Corriger le backoff/polling:
  - rien de bloquant restant sur ce bloc

## Priorité P2

- Corriger les URLs/références de dérivés:
  - supprimer les références `agent://derived/...`
  - s'aligner sur les URLs Core stables/same-origin attendues par la spec

## Priorité P3

- Aligner la doc locale:
  - documenter seulement les flows réellement implémentés
  - compléter la doc auth interactive/browser/device flow quand l'implémentation existera

- Renforcer i18n:
  - couvert

- Requalifier la suite de tests:
  - renommage/recadrage des anciennes suites `spec_compatible_*`: couvert
- compléter ce qui peut l'être côté agent sur les flows d'approbation humain complet restants

## Ordre d'exécution recommandé

1. Policy/effective features
2. Device flow daemon
3. Polling/backoff
4. Processing réel previews/thumbs/waveform/facts
5. Concurrence OpenAPI dérivés
6. Docs et CI i18n

## Découpage en lots de travail

- Lot 1: contrat et authz
  - tests associés

- Lot 2: policy et bootstrap
  - couvert

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
