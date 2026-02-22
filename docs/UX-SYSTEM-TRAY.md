# UX System Tray

## Target

- App menu système (menu bar/tray) type Docker Desktop/Ollama.

## Runtime State

- État visible en permanence via stats daemon publiées (`running`, `paused`, `stopped`).
- Accès rapide au statut et logs.

## Actions

- Toggle `Start/Stop Daemon` (selon état service)
- `Refresh Daemon Status`
- `Open Window`
- `Open Status`
- `Open Preferences`
- `Quit`

## Minimal Status Window

- Visible depuis la GUI.
- Fermeture fenêtre: doit masquer la fenêtre et laisser le tray actif (`hide to tray`).
- Doit afficher le job en cours:
  - progression `%`
  - étape active (`claim`, `processing`, `upload`, `submit`)
  - `job_id` / `asset_uuid`
  - message de statut court

## Control Center Window

- Fenêtre desktop interactive (au-delà d'un simple popup statut) avec:
  - boutons cliquables alignés sur le tray (toggle daemon, `Refresh Daemon Status`),
  - actions rapides (`Open Status`, `Open Preferences`, `Hide to Tray`, `Quit`),
  - bloc stats runtime lues depuis le store daemon:
    - job courant (`job_id`, `asset_uuid`, `%`, `stage`, `status`),
    - dernier job observé,
    - durée du dernier job observé,
    - uptime de l'application.

## Window Controls

- La fenêtre desktop expose les mêmes contrôles runtime que le tray (équivalent raccourcis):
  - start/stop/refresh daemon,
  - open status, open preferences,
  - quit.

## Architecture Rule

- `CLI === GUI` pour le périmètre opérateur:
  - mêmes capacités exposées: contrôle daemon + consultation stats daemon,
  - aucune exécution runtime métier locale côté CLI/GUI.
- La GUI et la CLI ne doivent jamais implémenter une logique runtime parallèle.
- Le moteur de processing est unique et partagé.
- Le daemon `agent-runtime -- daemon` est l'unique exécuteur runtime.
- GUI/CLI exposent uniquement:
  - contrôle daemon (`start/stop/status/refresh`),
  - consultation des stats daemon persistées (`daemon-stats.json`).
- Le daemon runtime (boot + arrière-plan) est unique; son lifecycle est pilotable depuis CLI et GUI.
