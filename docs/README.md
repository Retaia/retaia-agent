# Agent Docs

Hub documentaire local, organisé par sujet.

## Sujets

- Statut d'implémentation pré-v1: `PRE-V1-STATUS.md`
- Matrice de conformité v1 vs `specs/`: `V1-SPECS-CONFORMITY.md`
- Rapport final de freeze v1: `V1-FREEZE-REPORT.md`
- Runtime et contraintes: `RUNTIME-CONSTRAINTS.md`
- Client HTTP API (OpenAPI v1): `API-CLIENT.md`
- Mode daemon: implementation locale + commandes exactes dans `DAEMON-MODE.md`
- Configuration locale: stockage, commandes et details agent dans `CONFIGURATION-PANEL.md`
- Notifications locales: bridge runtime et adapters dans `NOTIFICATIONS.md`
- Desktop shell local: details tray/control center dans `UX-SYSTEM-TRAY.md`
- Persistance configuration système (lib + chemins OS): `CONFIG-STORAGE.md`
- Contrat des fixtures externes (RAW/AV + checksums): `FIXTURE-CONTRACT.md`
- Qualité/CI/Hooks: `CI-QUALITY-GATES.md`

## Cadrage fonctionnel migre vers `retaia-docs`

Les docs fonctionnelles cross-project du client agent sont centralisees dans `retaia-docs`:

- `agent/README.md`
- `agent/DAEMON-OPERATIONS.md`
- `agent/CONFIGURATION-UX.md`
- `agent/NOTIFICATIONS-UX.md`
- `agent/DESKTOP-SHELL.md`

Reference GitHub:

- [retaia-docs/agent](https://github.com/Retaia/retaia-docs/tree/master/agent)

## Références normatives

- `../specs/api/API-CONTRACTS.md`
- `../specs/workflows/AGENT-PROTOCOL.md`
- `../specs/tests/TEST-PLAN.md`
- `../specs/policies/AUTHZ-MATRIX.md`

## Docs API Generated

- Les docs générées depuis l'OpenAPI sont isolées dans `api/`.
