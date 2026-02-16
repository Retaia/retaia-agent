# AGENT Rules (v1)

## Scope
Ce repo implémente le client agent Retaia.
La source de vérité contrat/runtime est le submodule `specs/`.

## Normes obligatoires
- CLI obligatoire (Linux headless supporté)
- GUI optionnelle, même moteur que la CLI
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
- Tests obligatoires en PR:
  - `TDD` base sur le fonctionnement du code
  - `BDD` base sur les scenarios des specs
  - `E2E` base sur les parcours specs/workflows
- Coverage minimale obligatoire en PR: `80%` (line coverage)
- Hook git `pre-commit` (via `cargo-husky`): interdit les commits sur `master`
- Hook git `commit-msg` (via `cargo-husky`): impose Conventional Commits
- Hook git `pre-push` (via `cargo-husky`): interdit les pushes sur `master` + vérifie la fraîcheur/linéarité de branche
