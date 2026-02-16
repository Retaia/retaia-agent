# Contributing (retaia-agent)

## Source de vérité
Le comportement agent doit rester aligné sur `retaia-docs` (submodule `specs/`).
Toute divergence contractuelle doit d'abord être corrigée dans la spec.

## Workflow Git
- Branche depuis `master` avec préfixe `codex/`.
- Commits atomiques, PR atomiques.
- Rebase sur `master` avant merge.
- Pas de merge commit de synchronisation.
- Aucun commit/push direct sur `master` (bloqué par Husky).

## Exigences de PR
- Garder l'agent CLI obligatoire et GUI optionnelle.
- Respecter la séparation normative:
  - `capabilities` = capacités locales du client/agent.
  - `app_feature_enabled` / `user_feature_enabled` = gouvernance pilotée par Core.
- Ne jamais traiter l'IA dans MCP; l'agent exécute le runtime.
- Couvrir les changements par tests (unitaires/intégration/BDD selon impact).

## Règles de sécurité
- Ne jamais logger de token, secret, clé privée, ou PII en clair.
- Stocker les secrets via mécanismes sécurisés OS quand applicable.
- Respecter les politiques sécurité et RGPD définies dans `specs/policies/`.

## Pratiques d'implémentation
- Approche préférée: DDD.
- Validation recommandée par défaut: TDD + BDD.

## Licence des contributions
- Toute contribution est publiée sous `AGPL-3.0-or-later`.
- En soumettant une PR, vous acceptez que votre contribution soit distribuée sous cette licence.
