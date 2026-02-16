# AGENT Rules (v1)

## Scope
Ce repo implémente le client agent Retaia.
La source de vérité contrat/runtime est le submodule `specs/`.

## Normes obligatoires
- CLI obligatoire (Linux headless supporté)
- GUI optionnelle, même moteur que la CLI
- Si GUI présente: menu système type Docker Desktop/Ollama (play/resume, pause, stop, quit + statut)
- Play/Pause est un toggle:
  - état `paused` => `Play/Resume` visible, `Pause` masqué
  - état `running` => `Pause` visible, `Play/Resume` masqué
- Fenêtre minimale de statut job obligatoire si GUI présente:
  - progression `%` du job actif
  - étape active (`claim`, `processing`, `upload`, `submit`)
  - job/asset courant + message court
- Notification système `All jobs done` obligatoire:
  - émise une seule fois sur transition vers aucun job actif
  - interdiction de répétition sur les polls suivants tant que l'état reste inchangé
- Panneau de config accessible depuis menu app + menu système
- Config minimale exposée: URL Core/Agent, URL Ollama, auth, paramètres runtime
- Bearer-only
- Respect strict de `effective_feature_enabled`
- Aucun traitement MCP dans ce repo

## Auth
- Mode interactif: login utilisateur
- Mode technique: client credentials (`client_id` + `secret_key`)
- 1 token actif par `client_id`

## Sécurité
- Aucun token/secret en clair dans logs, traces, crash reports
- Secret storage OS-native (Linux/macOS/Windows)
- Rotation de secret supportée sans réinstallation

## Delivery
- PR atomiques
- Rebase sur `master`
- Pas de merge commits de synchronisation
- Historique linéaire obligatoire (branche à jour + aucun merge commit)
- Check CI bloquant `commitlint`: tous les commits PR en Conventional Commits
- Tests obligatoires en PR:
  - `TDD` base sur le fonctionnement du code
  - `BDD` base sur les scenarios des specs
  - `E2E` base sur les parcours specs/workflows
- Coverage minimale obligatoire en PR: `80%` (line coverage)
- Hook git `pre-commit` (via `cargo-husky`): interdit les commits sur `master`
- Hook git `commit-msg` (via `cargo-husky`): impose Conventional Commits via `cargo-commitlint`
- Hook git `pre-push` (via `cargo-husky`): interdit les pushes sur `master` + vérifie la fraîcheur/linéarité de branche
- Guard local: `core.hooksPath` ne DOIT PAS surcharger les hooks du repo (`git config --unset core.hooksPath`)
