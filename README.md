# retaia-agent

Client agent Rust (CLI obligatoire, GUI optionnelle).

- Règles: `AGENT.md`
- Specs normatives: submodule `specs/`
- Docs: `docs/`

## UX Agent (menu système)

L'agent est prévu comme une app de menu système (menu bar/tray) type Docker Desktop ou Ollama:

- état visible (`running`, `paused`, `stopped`)
- actions directes: toggle `Play/Resume`/`Pause`, `Stop`, `Quit`
- regle toggle:
  - quand l'agent est `paused`, afficher `Play/Resume` et masquer `Pause`
  - quand l'agent est `running`, afficher `Pause` et masquer `Play/Resume`
- accès rapide aux logs/statut
- ouverture d'une fenetre minimale de statut job en cours:
  - progression `%`
  - action en cours (ex: `claim`, `transcode`, `upload`, `submit`)
  - identifiant job/asset et message de statut court
- notifications système:
  - notifier `New job received` sur arrivée d'un nouveau job (événement/transition), sans répétition sur polls suivants
  - notifier `All jobs done` une seule fois lors de la transition `has_running_jobs=true -> false`
  - ne pas répéter la notification à chaque cycle de poll tant que l'état reste `no running jobs`
- acces au panneau de configuration depuis:
  - menu de l'app
  - menu système/tray

Configuration minimale attendue:

- URL Core/Agent API
- URL Ollama
- mode d'auth (interactif/technique) et identifiants techniques
- parametres runtime (ex: concurrence/max jobs, niveau de log)

Contraintes:

- la CLI reste obligatoire et doit piloter le même moteur runtime
- la GUI/menu système est optionnelle mais, si présente, ne doit pas diverger de la CLI

## Branch rules and hooks

- CI gate: `branch-up-to-date` (branch must include latest `master` and keep linear history)
- CI gate: `commitlint` (PR commit range must follow Conventional Commits, fast regex check)
- CI test gates (blocking for PR merge):
  - `rust-build-cache`: pre-build des tests pour reduire les recompilations
  - `test-tdd`: tests bases sur le fonctionnement du code
  - `test-bdd`: scenarios bases sur la spec
  - `test-e2e`: parcours end-to-end bases sur la spec
  - `coverage-gate`: coverage minimale 80%
  - `ci-required`: aggregate required status
  - path filters: les jobs de tests lourds sont skips si aucun fichier applicatif pertinent n'a change
- Local hooks:
  - `pre-commit`: blocks commits on `master`
  - `commit-msg`: enforces Conventional Commits via `cargo-commitlint`
  - `pre-push`: blocks pushes on `master` and runs `cargo run --bin check_branch_up_to_date`
  - managed by `cargo-husky` from `.cargo-husky/hooks/`

Cargo commands used by CI checks:

- `cargo test --test tdd_runtime`
- `cargo test --test bdd_specs`
- `cargo test --test e2e_flow`
- `cargo llvm-cov --workspace --summary-only --json --output-path coverage/llvm-cov-summary.json`
- `cargo run --bin check_coverage -- --file coverage/llvm-cov-summary.json --min 80`

Setup:

```bash
cargo install cargo-commitlint
# Ensure git uses repository hooks from .git/hooks.
git config --unset core.hooksPath || true
# Force hook refresh after hook file updates.
cargo clean -p cargo-husky
cargo test
```
