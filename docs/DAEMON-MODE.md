# Daemon Mode

## Objectif

Le runtime agent doit pouvoir tourner en arrière-plan et démarrer au boot, avec un seul daemon pilotable depuis CLI ou GUI.

## Contrat

- Une instance de daemon partagée (pas de runtime parallèle CLI vs GUI).
- Gestion lifecycle unifiée: `install`, `start`, `stop`, `status`, `uninstall`.
- Publication des stats runtime par le daemon dans un store local (`daemon-stats.json`).
- Historique long-terme de debug dans SQLite (`daemon-history.sqlite3`).
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
cargo run --bin agentctl -- daemon stats
cargo run --bin agentctl -- daemon history --limit 200
cargo run --bin agentctl -- daemon cycles --limit 500
cargo run --bin agentctl -- daemon report --provider github --repo owner/repo
cargo run --bin agentctl -- daemon report --provider jira
cargo run --bin agentctl -- daemon report --provider github --repo owner/repo --no-copy
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

Historique persistant:

- `daemon-stats.json`: snapshot courant (état + job courant + dernier job).
- `daemon-history.sqlite3`: historique requêtable:
  - `completed_jobs`: fin de job + durée,
  - `daemon_cycles`: snapshots cycliques pour debug specs.

Garde-fous perfs:

- SQLite en mode `WAL` + `synchronous=NORMAL`.
- Écriture `daemon_cycles` échantillonnée:
  - sur changement d'état/job/stage/progress,
  - sur tick non-success (`throttled`/`degraded`),
  - heartbeat périodique (1/60 ticks).
- Compaction périodique côté daemon (conservation des dernières lignes cycles).

Bug report:

- `agentctl daemon report` agrège snapshot + historique et imprime un contenu prêt à copier-coller.
- copie dans le presse-papiers activée par défaut.
- option `--no-copy`: désactive la copie automatique.
- Aucun ticket n'est créé automatiquement (GitHub/Jira restent actionnés manuellement côté opérateur).

Auth bearer pour polling API (build avec feature `core-api-client`):

- env var optionnelle: `RETAIA_AGENT_BEARER_TOKEN`.

Le mode interactif local est volontairement désactivé: seul `agent-runtime -- daemon` exécute le runtime.
CLI et GUI jouent le rôle de clients de contrôle/observabilité du daemon.

## Notes d'intégration GUI

- La GUI doit appeler le même contrat applicatif de gestion daemon que la CLI.
- Les actions menu système (`start/stop/status`) doivent refléter l'état réel du daemon installé.
