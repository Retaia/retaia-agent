# UX System Tray

## Target

- App menu système (menu bar/tray) type Docker Desktop/Ollama.

## Runtime State

- État visible en permanence: `running`, `paused`, `stopped`.
- Accès rapide au statut et logs.

## Actions

- Toggle `Play/Resume` / `Pause`
- `Stop`
- `Quit`

Règle toggle:

- si état `paused`: afficher `Play/Resume`, masquer `Pause`
- si état `running`: afficher `Pause`, masquer `Play/Resume`

## Minimal Status Window

- Visible depuis la GUI.
- Doit afficher le job en cours:
  - progression `%`
  - étape active (`claim`, `processing`, `upload`, `submit`)
  - `job_id` / `asset_uuid`
  - message de statut court

## Architecture Rule

- La GUI ne doit jamais implémenter une logique runtime parallèle à la CLI.
- Le moteur de processing est unique et partagé.
- En environnement headless, un shell CLI (`agent-runtime`) expose le même contrat minimal
  (`menu/status/settings/play/pause/stop/quit`) en miroir fonctionnel du menu système.
- Le daemon runtime (boot + arrière-plan) est unique; son lifecycle est pilotable depuis CLI et GUI.
