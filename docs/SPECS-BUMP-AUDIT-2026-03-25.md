# Specs Bump Audit 2026-03-25

Submodule target:

- previous: `ed86b95dd1409b65f347e85dc78af46258b01c44`
- new: `b6eb0447cf3c9d3bf3d4b9d2969ceda4cd38202a`

Upstream commit:

- `b6eb044` `docs: add shared asset projects contract (#120)`

## Summary

The spec delta introduces a shared human-facing `projects[]` contract on assets.

This affects shared asset read/write payloads in the OpenAPI v1 contract, but it is
explicitly documented as outside `AGENT` processing scope:

- `projects[]` is a human business attachment
- owned by `Core` / `UI_WEB` and later `MCP`
- not part of `facts_patch`
- not part of any agent job input/output

So the current agent runtime and processing pipeline do not need behavioral changes for
this bump.

## Normative Changes In Specs

Changed in `specs/`:

- `api/API-CONTRACTS.md`
- `api/openapi/v1.yaml`
- `contracts/openapi-v1.sha256`
- `change-management/FEATURE-FLAG-REGISTRY.md`
- `definitions/JOB-TYPES.md`
- `policies/AUTHZ-MATRIX.md`
- `policies/FEATURE-RESOLUTION-ENGINE.md`
- `tests/TEST-PLAN.md`

Relevant effective change for this repo:

- `GET /assets/{uuid}` / `AssetDetail` now expose `projects[]`
- `PATCH /assets/{uuid}` now accepts `projects[]`
- new OpenAPI schema `AssetProjectRef`
- prose now states explicitly that `projects[]` is not an `AGENT` processing concept

## Impact On Current Repo

### No agent runtime change required

No change is required in:

- job claiming
- derived uploads
- `extract_facts`
- preview/thumb/waveform generation
- policy/features resolution
- auth/device flow/runtime orchestration

Reason:

- the new field does not belong to agent job processing
- no new job type was introduced
- no existing processing contract was tightened for agent outputs

### Generated Core client is now behind the spec

The generated Rust client in `crates/retaia-core-client/` is not aligned with the new
OpenAPI contract yet.

Concrete drift observed:

- `crates/retaia-core-client/src/models/asset_detail.rs`
  - missing `projects`
- `crates/retaia-core-client/src/models/_assets__uuid__patch_request.rs`
  - missing `projects`
- missing generated model for `AssetProjectRef`
- local generated client still matches the previous hash, while `specs/contracts/openapi-v1.sha256`
  changed to `9e2c7c3c4c8ecd0b3e3a6a21e15a5d56d4c44abc9db281299d9359cab589c6f9`

This is a contract parity issue, not a current runtime breakage.

### Current code usage risk

Current agent code does not appear to rely on the new `projects[]` field:

- no local runtime code writes `projects[]`
- no local processing code reads `projects[]`
- no job/result mapping expects `projects[]`

So the repo should continue to compile and behave the same after the submodule bump, even
before regenerating the client.

## Recommended Actions

### P0

- bump the `specs` submodule to `b6eb044`

### P1

- regenerate `crates/retaia-core-client` from `specs/api/openapi/v1.yaml`
- keep the existing post-generation compile gate

### P2

- add or refresh an HTTP-level test proving that extra `projects[]` data on asset payloads is
  accepted by the updated generated client

This is low risk because the agent does not currently consume that field.

## Expected Repo State After This Bump

After bumping the submodule only:

- runtime behavior: unchanged
- tests: expected to remain green
- OpenAPI client parity: stale until regeneration

## Conclusion

For `retaia-agent`, this spec bump is mostly a shared contract drift update.

It does not require agent processing changes, but it does require a generated client refresh
to stay aligned with the normative OpenAPI contract.
