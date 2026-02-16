# Configuration Panel

## Access

- Accessible depuis le menu de l'app.
- Accessible aussi depuis le menu système/tray.
- Accessible en CLI-only (SSH, serveur Linux, Raspberry Pi sans GUI) avec le même contrat de config.
- Le daemon runtime est unique et doit être pilotable via CLI ou GUI (même instance de service).

## Minimal Fields

- URL Core/Agent API
- URL Ollama
- Mode d'auth (`interactif` / `technique`)
- Identifiants techniques (si mode technique)
- Paramètres runtime (ex: concurrence/max jobs, niveau de log)

Normalisation URL Core:

- `https://host` est normalisé en `https://host/api/v1`.
- `https://host/api/v1/` est normalisé en `https://host/api/v1`.

## UX Rules

- Validation explicite des champs.
- Message de succès à la sauvegarde (`Settings saved`).
- Message d'erreur sur config invalide (`Settings invalid`).

## Parity Rule (GUI/CLI)

- Les champs supportés sont identiques en GUI et CLI.
- La validation est identique en GUI et CLI (mêmes erreurs, mêmes invariants).
- Cible d'exécution: Linux/macOS/Windows, y compris environnements headless.
- Persistance système et convention de chemin: voir `CONFIG-STORAGE.md`.

## CLI-Only Commands (headless)

- `agentctl config path`: affiche le chemin de config résolu.
- `agentctl config show`: affiche la config active (secret masqué).
- `agentctl config validate`: valide la config active.
- `agentctl config validate --check-respond`: valide la compatibilité API côté Core/Ollama.
  - Core: probe `GET /jobs` (statuts compatibles attendus + payload JSON).
  - Ollama: probe `POST /v1/chat/completions` via `genai` (endpoint OpenAI-compatible).
- `agentctl config init ...`: initialise la config (première installation).
- `agentctl config set ...`: met à jour la config existante.
- `agentctl daemon install/start/stop/status/uninstall`: lifecycle du daemon partagé.

Exemple:

```bash
cargo run --bin agentctl -- config init \
  --core-api-url https://core.retaia.local \
  --ollama-url http://127.0.0.1:11434 \
  --auth-mode technical \
  --client-id agent-prod \
  --secret-key '$RETAIA_AGENT_SECRET'
```
