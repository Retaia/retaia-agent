# Daemon Mode

## Objectif

Le runtime agent doit pouvoir tourner en arrière-plan et démarrer au boot, avec un seul daemon pilotable depuis CLI ou GUI.

## Contrat

- Une instance de daemon partagée (pas de runtime parallèle CLI vs GUI).
- Gestion lifecycle unifiée: `install`, `start`, `stop`, `status`, `uninstall`.
- Support Linux/macOS/Windows via service manager natif (lib `service-manager`).

## Commandes CLI

- Installer le daemon (niveau user, autostart activé par défaut):

```bash
cargo run --bin agentctl -- daemon install
```

- Installer en niveau système:

```bash
cargo run --bin agentctl -- daemon install --system
```

- Désactiver l'autostart au boot:

```bash
cargo run --bin agentctl -- daemon install --no-autostart
```

- Contrôle lifecycle:

```bash
cargo run --bin agentctl -- daemon start
cargo run --bin agentctl -- daemon status
cargo run --bin agentctl -- daemon stop
cargo run --bin agentctl -- daemon uninstall
```

## Runtime daemon

Le daemon exécute `agent-runtime` en mode service:

```bash
cargo run --bin agent-runtime -- daemon
```

Comportement runtime en mode daemon:

- cycle de polling `GET /jobs` à intervalle fixe (paramètre `--tick-ms`),
- projection runtime partagée (`RuntimeSession`) + dispatch notifications système,
- sélection du sink de notification par target runtime (`headless` => stdout, `desktop` => system notification),
- dégradation explicite en cas d'erreur API:
  - `401` -> état `auth_reauth_required`,
  - transport/status inattendu -> état `reconnecting`,
  - `429` -> backoff+jitter via règles domaine.

Logs/observabilité:

- logs structurés par cycle avec `run_state`,
- corrélation job (`job_id`, `asset_uuid`) quand un job courant existe,
- niveau de logs aligné sur la config runtime (`log_level`).

Auth bearer pour polling API (build avec feature `core-api-client`):

- env var optionnelle: `RETAIA_AGENT_BEARER_TOKEN`.

Le shell interactif reste disponible en mode foreground:

```bash
cargo run --bin agent-runtime
```

## Notes d'intégration GUI

- La GUI doit appeler le même contrat applicatif de gestion daemon que la CLI.
- Les actions menu système (`start/stop/status`) doivent refléter l'état réel du daemon installé.
