# UX System Tray

## Target

- App menu système (menu bar/tray) type Docker Desktop/Ollama.

## Runtime State

- État visible en permanence: `running`, `paused`, `stopped`.
- Accès rapide au statut et logs.

## Actions

- Toggle `Play/Resume` / `Pause`
- `Stop`
- `Start/Stop Daemon` (selon état service)
- `Refresh Daemon Status`
- `Open Window`
- `Open Status`
- `Open Preferences`
- `Quit`

Règle toggle:

- si état `paused`: afficher `Play/Resume`, masquer `Pause`
- si état `running`: afficher `Pause`, masquer `Play/Resume`

## Minimal Status Window

- Visible depuis la GUI.
- Fermeture fenêtre: doit masquer la fenêtre et laisser le tray actif (`hide to tray`).
- Doit afficher le job en cours:
  - progression `%`
  - étape active (`claim`, `processing`, `upload`, `submit`)
  - `job_id` / `asset_uuid`
  - message de statut court

## Window Controls

- La fenêtre desktop expose les mêmes contrôles runtime que le tray (équivalent raccourcis):
  - play/resume, pause, stop,
  - start/stop/refresh daemon,
  - open status, open preferences,
  - quit.

## Architecture Rule

- La GUI ne doit jamais implémenter une logique runtime parallèle à la CLI.
- Le moteur de processing est unique et partagé.
- Contrat applicatif GUI ajouté: `runtime_gui_shell` (actions menu, contenu fenêtre statut, contenu panneau settings, contrôle daemon via port `DaemonManager`).
- Contrôleur desktop concret ajouté: `DesktopShellController` + `DesktopShellBridge` pour brancher un toolkit GUI (tray/menu/window) sans dupliquer la logique runtime.
- En environnement headless, un shell CLI (`agent-runtime`) expose le même contrat minimal
  (`menu/status/settings/play/pause/stop/quit`) en miroir fonctionnel du menu système.
- Le daemon runtime (boot + arrière-plan) est unique; son lifecycle est pilotable depuis CLI et GUI.
