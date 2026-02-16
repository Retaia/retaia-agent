# retaia-agent

Client agent Rust (CLI obligatoire, GUI optionnelle).

- Règles: `AGENT.md`
- Specs normatives: submodule `specs/`
- Docs: `docs/`

## Branch rules and hooks

- CI gate: `branch-up-to-date` (branch must include latest `master` and keep linear history)
- Local hooks:
  - `pre-commit`: blocks commits on `master`
  - `pre-push`: blocks pushes on `master` and runs `npm run check:branch-up-to-date`

Setup:

```bash
npm ci
```
