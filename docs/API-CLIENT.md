# API Client

Client HTTP Rust généré depuis l'OpenAPI v1:

- Spec source: `../specs/api/openapi/v1.yaml`
- Crate généré: `../crates/retaia-core-client`
- Docs API générées: `./api/`
- Générateur: `openapi-generator` (Rust, `reqwest-trait`)

## Regenerate

```bash
./scripts/generate_core_api_client.sh
```

## Integration in `retaia-agent`

Le crate généré est branché en dépendance optionnelle via feature:

- feature: `core-api-client`
- helpers d'infra:
  - `build_core_api_client(...)`
  - `with_bearer_token(...)`

Activation:

```bash
cargo test --features core-api-client
```
