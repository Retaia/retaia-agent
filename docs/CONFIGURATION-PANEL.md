# Configuration Panel

## Access

- Accessible depuis le menu de l'app.
- Accessible aussi depuis le menu système/tray.
- Accessible en CLI-only (SSH, serveur Linux, Raspberry Pi sans GUI) avec le même contrat de config.

## Minimal Fields

- URL Core/Agent API
- URL Ollama
- Mode d'auth (`interactif` / `technique`)
- Identifiants techniques (si mode technique)
- Paramètres runtime (ex: concurrence/max jobs, niveau de log)

## UX Rules

- Validation explicite des champs.
- Message de succès à la sauvegarde (`Settings saved`).
- Message d'erreur sur config invalide (`Settings invalid`).

## Parity Rule (GUI/CLI)

- Les champs supportés sont identiques en GUI et CLI.
- La validation est identique en GUI et CLI (mêmes erreurs, mêmes invariants).
- Cible d'exécution: Linux/macOS/Windows, y compris environnements headless.
- Persistance système et convention de chemin: voir `CONFIG-STORAGE.md`.
