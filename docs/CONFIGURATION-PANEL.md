# Configuration Panel (Agent local)

> Cadrage fonctionnel global: `retaia-docs/agent/CONFIGURATION-UX.md`

## Access

- Accessible depuis le menu de l'app.
- Accessible aussi depuis le menu système/tray.
- Accessible en CLI-only (SSH, serveur Linux, Raspberry Pi sans GUI) avec le même contrat de config.
- Le daemon runtime est unique et doit être pilotable via CLI ou GUI (même instance de service).

## Champs et commandes locales

- URL Core/Agent API
- URL Ollama
- Mode d'auth (`interactif` / `technique`)
- Identifiants techniques (si mode technique)
- Paramètres runtime (ex: concurrence/max jobs, niveau de log)
- Mapping des montages storage agent (`storage_mounts`) pour résoudre les chemins relatifs Core (`INBOX/...`) vers des chemins absolus locaux NAS.

Normalisation URL Core:

- `https://host` est normalisé en `https://host/api/v1`.
- `https://host/api/v1/` est normalisé en `https://host/api/v1`.

Déploiement NAS + workstations:

- si Core est privé dans Docker, utiliser l'URL gateway LAN exposée par UI/Caddy.
- exemple: `http://192.168.0.14:8080/api/v1`.
- ne pas configurer un hostname Docker interne côté agent (`core:9000`, `app-prod:9000`).
- profil de déploiement normatif: `specs/architecture/DEPLOYMENT-TOPOLOGY.md`.

## Details locaux

- persistance systeme et conventions de chemin: `CONFIG-STORAGE.md`
- la validation s'appuie sur le meme contrat d'application dans GUI et CLI

## Commandes CLI-only (headless)

- `agentctl config path`: affiche le chemin de config résolu.
- `agentctl config show`: affiche la config active (secret masqué).
- `agentctl config show`: inclut `storage_mounts` (`storage_id=/mount/absolu`).
- `agentctl config validate`: valide la config active.
- `agentctl config validate --check-respond`: valide la compatibilité API côté Core/Ollama.
  - Core: probe `GET /jobs` (statuts compatibles attendus + payload JSON).
  - Ollama: probe `POST /v1/chat/completions` via `genai` (endpoint OpenAI-compatible).
- `agentctl config init ...`: initialise la config (première installation).
- `agentctl config set ...`: met à jour la config existante.
- `--storage-mount <storage_id=/mount/absolu>`: ajout/remplacement du mapping (répétable).
- `--clear-storage-mounts`: supprime tous les mappings.
- `agentctl daemon install/start/stop/status/uninstall`: lifecycle du daemon partagé.

Exemple:

```bash
cargo run --bin agentctl -- config init \
  --core-api-url http://192.168.0.14:8080/api/v1 \
  --ollama-url http://127.0.0.1:11434 \
  --auth-mode technical \
  --client-id agent-prod \
  --secret-key '$RETAIA_AGENT_SECRET' \
  --storage-mount nas-main=/Volumes/NAS-01/retaia \
  --storage-mount nas-archive=/Volumes/NAS-01/archive
```
