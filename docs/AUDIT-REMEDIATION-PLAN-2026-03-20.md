# Plan d'action priorisé suite à l'audit

Date: 2026-03-20

Source: `docs/AUDIT-SPECS-CODE-TESTS-2026-03-20.md`

## Priorité P0

- Rien de bloquant restant côté agent local.

## Priorité P1

- Clarifier le périmètre restant:
  - publication finale côté Core
  - approval humain complet côté `UI_WEB`
  - tests cross-app

## Priorité P2

- Aligner la doc locale finale:
  - documenter seulement les flows réellement portés par l'agent
  - marquer explicitement les dépendances à Core/UI

## Priorité P3

- Garder la suite agent propre:
  - maintenir la gate `core-api-client`
  - conserver les tests réels previews/thumbs/waveform/facts

## Ordre d'exécution recommandé

1. Clôture doc/audit côté agent
2. Validation cross-app Core/UI hors de ce repo

## Découpage en lots de travail

- Lot 1: clôture agent
  - audit
  - README/docs locales
  - suppression des surfaces runtime non-agent restantes

- Lot 2: système complet
  - Core
  - `UI_WEB`
  - tests bout-en-bout
