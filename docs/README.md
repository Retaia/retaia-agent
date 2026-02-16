# Agent Docs

Ce dossier contient la documentation d'implémentation de l'agent.

## Références
- `../AGENT.md`
- `../specs/api/API-CONTRACTS.md`
- `../specs/workflows/AGENT-PROTOCOL.md`
- `../specs/tests/TEST-PLAN.md`

## Politique tests locale (gates PR)
- `TDD` : tests fondes sur le comportement du code (unitaires/integration selon le besoin technique).
- `BDD` : tests fondes sur les scenarios derives de `../specs/tests/TEST-PLAN.md`.
- `E2E` : tests de parcours complets fondes sur les workflows/specs.
- `Coverage` : minimum `80%` (ligne) sur le repo.

## UX cible de l'agent (GUI optionnelle)
- Application de menu système (menu bar/tray), style Docker Desktop/Ollama.
- Actions minimales: `Play/Resume`, `Pause`, `Stop`, `Quit`.
- L'état runtime doit rester visible en permanence (`running`, `paused`, `stopped`).
- La GUI utilise strictement le meme moteur que la CLI (pas de logique parallèle).

En CI, ces checks sont bloquants pour merge PR :
- `test-tdd`
- `test-bdd`
- `test-e2e`
- `coverage-gate`
