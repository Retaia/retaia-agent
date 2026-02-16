# retaia-agent

Client agent Rust (CLI obligatoire, GUI optionnelle).

- Règles: `AGENT.md`
- Specs normatives: submodule `specs/`
- Docs: `docs/`

## Branch rules and hooks

- CI gate: `branch-up-to-date` (branch must include latest `master` and keep linear history)
- CI test gates (blocking for PR merge):
  - `test-tdd`: tests bases sur le fonctionnement du code
  - `test-bdd`: scenarios bases sur la spec
  - `test-e2e`: parcours end-to-end bases sur la spec
  - `coverage-gate`: coverage minimale 80%
- Local hooks:
  - `pre-commit`: blocks commits on `master`
  - `pre-push`: blocks pushes on `master` and runs `npm run check:branch-up-to-date`

Required npm scripts (implemented by the codebase):

- `test:tdd`
- `test:bdd`
- `test:e2e`
- `test:coverage` (must generate `coverage/coverage-summary.json` with `total.lines.pct`)

Setup:

```bash
npm ci
```
