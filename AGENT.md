# AGENT Rules (Entry Point)

Point d'entrée pour les agents IA.

## Source de vérité

- Specs normatives: `specs/`
- Hub doc local par sujets: `docs/README.md`

## Lecture minimale obligatoire (ordre)

- `docs/RUNTIME-CONSTRAINTS.md`
- `docs/UX-SYSTEM-TRAY.md`
- `docs/NOTIFICATIONS.md`
- `docs/CONFIGURATION-PANEL.md`
- `docs/CI-QUALITY-GATES.md`

## Règles d'exécution

- Ne jamais implémenter une logique qui contredit `specs/`.
- Garder CLI obligatoire; GUI optionnelle et même moteur runtime.
- Respect strict de `effective_feature_enabled`.
- Aucun traitement MCP dans ce repo.
- Aucun commit/push direct sur `master`.
