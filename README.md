# retaia-agent

Client agent Rust (CLI obligatoire, GUI optionnelle).

- Règles: `AGENT.md`
- Specs normatives: submodule `specs/`
- Docs: `docs/`

## Branch rules and hooks

- CI gate: `branch-up-to-date` (branch must include latest `master` and keep linear history)
- CI gate: `commitlint` (PR commit range must follow Conventional Commits)
- CI test gates (blocking for PR merge):
  - `test-and-coverage`: execute `tdd` + `bdd` + `e2e` en un seul job avec coverage minimale 80%
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
