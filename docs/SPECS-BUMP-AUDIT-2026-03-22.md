# Audit du bump `specs` du 2026-03-22

## Périmètre

Révision précédente du submodule `specs`:

- `9e30c1f14374b13102bde1307fee7b4e188ea0e2`

Révision bumpée:

- `8ff71021f5155425183cfad5fcaea6c1dea4cdaf`

## Fichiers modifiés dans `specs`

Entre ces deux révisions, seuls les fichiers suivants ont changé:

- `.github/workflows/ci.yml`
- `.github/workflows/security.yml`

## Nature des changements

Les changements observés sont uniquement des mises à jour de versions d'actions GitHub:

- `actions/checkout`
  - `v4` -> `v6`
- `github/codeql-action`
  - `v3` -> `v4`

Il n'y a pas de changement sur:

- `specs/api/`
- `specs/agent/`
- `specs/workflows/`
- `specs/definitions/`
- `specs/policies/`
- `specs/tests/`

## Impact sur `retaia-agent`

Pour ce bump précis, aucun changement d'implémentation n'est requis côté agent:

- pas de changement de contrat OpenAPI
- pas de changement de comportement runtime
- pas de changement de matrice authz
- pas de changement de jobs, dérivés, policy ou device flow
- pas de changement de tests produit à appliquer

## Actions à appliquer

Actions requises:

- aucune au niveau code agent
- aucune au niveau tests agent
- aucune au niveau docs produit agent

Action déjà réalisée:

- bump du SHA du submodule `specs` dans ce repo

## Conclusion

Le bump `specs` du 2026-03-22 n'introduit aucun delta normatif applicable à `retaia-agent`.

La seule action nécessaire dans ce repo est de pointer vers le nouveau SHA du submodule.
