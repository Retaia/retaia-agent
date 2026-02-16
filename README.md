# retaia-agent

Point d'entrée humain du projet.

L'agent Retaia est un client Rust: CLI obligatoire, GUI optionnelle.

## Où lire

- Règles IA: `AGENT.md`
- Hub doc par sujets: `docs/README.md`
- Source de vérité normative: `specs/`

## Démarrage local

```bash
cargo install cargo-commitlint
git config --unset core.hooksPath || true
cargo clean -p cargo-husky
cargo test
```
