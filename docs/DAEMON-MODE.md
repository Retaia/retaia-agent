# Daemon Mode (Agent local)

> Cadrage fonctionnel global: `retaia-docs/agent/DAEMON-OPERATIONS.md`

## Objectif

Documenter les choix d'implementation locaux du mode daemon dans `retaia-agent`.

## Details locaux

- stats courantes dans `daemon-stats.json`
- historique long-terme dans `daemon-history.sqlite3`
- support Linux/macOS/Windows via `service-manager`

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

## Runtime daemon local

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
- Compaction périodique côté daemon:
  - `daemon_cycles`: conservation des `250_000` dernières lignes,
  - `completed_jobs`: conservation des `150_000` dernières lignes.

Bug report:

- `agentctl daemon report` agrège snapshot + historique et imprime un contenu prêt à copier-coller.
- copie dans le presse-papiers activée par défaut.
- option `--no-copy`: désactive la copie automatique.
- Aucun ticket n'est créé automatiquement (GitHub/Jira restent actionnés manuellement côté opérateur).

Auth bearer pour polling API (build avec feature `core-api-client`):

- auth technique attendue: `technical_auth.client_id` + `technical_auth.secret_key`.
- le daemon génère/persiste une identité agent locale (`agent_id` + clé OpenPGP) et s’enregistre avant de traiter des jobs.

Le mode interactif local est desactive: seul `agent-runtime -- daemon` execute le runtime.
CLI et GUI jouent le role de clients de controle/observabilite du daemon.

## Notes d'integration GUI

- La GUI doit appeler le même contrat applicatif de gestion daemon que la CLI.
- Les actions menu système (`start/stop/status`) doivent refléter l'état réel du daemon installé.
