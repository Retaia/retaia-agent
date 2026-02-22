# CI & Quality Gates

## Required CI Checks

- `branch-up-to-date`
- `commitlint`
- `rust-build-cache`
- `test-tdd`
- `test-bdd`
- `test-e2e`
- `coverage-gate`
- `ci-required`

## Test Policy

- `TDD`: basé sur le comportement du code.
- `BDD`: basé sur les scénarios issus des specs.
- `E2E`: basé sur les parcours complets.
- Coverage minimal bloquant: `80%` (line coverage) sur la **couverture globale agrégée** des suites TDD+BDD+E2E.
- Les résumés de coverage par suite (`TDD`, `BDD`, `E2E`) restent générés et publiés comme artefacts CI pour diagnostiquer les zones faibles, mais ne bloquent plus individuellement.

## Local Hooks

- `pre-commit`: bloque commit sur `master`.
- `commit-msg`: impose Conventional Commits (`cargo-commitlint`).
- `pre-push`: bloque push sur `master` et vérifie fraîcheur/linéarité.

## Local Setup

```bash
cargo install cargo-commitlint
git config --unset core.hooksPath || true
cargo clean -p cargo-husky
cargo test
```
