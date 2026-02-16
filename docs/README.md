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
- Actions minimales: toggle `Play/Resume`/`Pause`, `Stop`, `Quit`.
- Règle toggle: si `paused`, afficher `Play/Resume` et masquer `Pause`; si `running`, afficher `Pause` et masquer `Play/Resume`.
- L'état runtime doit rester visible en permanence (`running`, `paused`, `stopped`).
- Une fenêtre minimale doit afficher le job en cours (`%`, étape active, job/asset, statut court).
- Notifications système:
  - `New job received`: émission à l'arrivée d'un nouveau job, sans répétition pour le même job.
  - `All jobs done`: une seule émission sur transition vers zéro job actif, sans répétition sur poll.
- Un panneau de configuration doit être accessible depuis le menu app et le menu système.
- Champs de config minimum: URL Core/Agent, URL Ollama, auth, paramètres runtime.
- La GUI utilise strictement le meme moteur que la CLI (pas de logique parallèle).

En CI, ces checks sont bloquants pour merge PR :
- `test-tdd`
- `test-bdd`
- `test-e2e`
- `coverage-gate`
