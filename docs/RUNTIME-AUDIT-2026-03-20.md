# Runtime Audit — 2026-03-20

Scope: review of the current branch after the spec bump to `specs@9e30c1f14374b13102bde1307fee7b4e188ea0e2`, focused on runtime conformance for technical agent identity, signed writes, and daemon bootstrap.

Status: partial conformance. Major previous gaps are closed, but 2 findings remain before the implementation can be considered fully aligned with the current specs.

## Reviewed Areas

- specs bump and generated Core client refresh
- daemon bootstrap and technical bearer minting
- agent identity persistence
- `POST /agents/register`
- signed technical writes:
  - `POST /jobs/{job_id}/claim`
  - `POST /jobs/{job_id}/heartbeat`
  - `POST /jobs/{job_id}/submit`
  - `POST /assets/{uuid}/derived/upload/*`
- derived runtime updates already implemented:
  - `fencing_token`
  - multipart upload part
  - `part_etag`
  - `preview_*` mapping

## Findings

### P1. Signature header payload is transformed before transport

Files:
- [src/infrastructure/agent_identity.rs](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/src/infrastructure/agent_identity.rs:97)

Observed behavior:
- the detached OpenPGP ASCII-armored signature is generated correctly
- before returning it, the implementation rewrites line breaks:
  - `\r` removed
  - `\n` replaced by literal `\\n`

Why this matters:
- the spec requires `X-Retaia-Signature` to contain the detached ASCII-armored signature
- the current code sends a transformed representation, not the armored value itself
- unless Core explicitly reverses this escaping before verification, signed writes can be rejected despite correct signing material

Spec reference:
- [specs/workflows/AGENT-PROTOCOL.md](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/specs/workflows/AGENT-PROTOCOL.md:121)

Impact:
- possible hard runtime failure on every mutating technical request once Core enforces the documented format strictly

Recommended fix:
- define one explicit transport encoding shared with Core
- prefer a header-safe canonical representation documented in specs, or move the signature out of headers if the raw armored form is required verbatim
- add one end-to-end verifier test using the exact transmitted header value

### P2. Private OpenPGP key is persisted in plaintext app storage

Files:
- [src/infrastructure/agent_identity.rs](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/src/infrastructure/agent_identity.rs:154)

Observed behavior:
- `openpgp_private_key` is stored directly in `identity.json`
- Unix permissions are reduced to `0600`
- no equivalent hardening exists for non-Unix targets

Why this matters:
- the spec requires the private key to be persisted in the OS secret store or protected application storage
- current persistence is file-based plaintext
- theft of the config directory is enough to impersonate the agent

Spec reference:
- [specs/workflows/AGENT-PROTOCOL.md](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/specs/workflows/AGENT-PROTOCOL.md:82)

Impact:
- credential theft risk
- non-conformance with the key storage requirement

Recommended fix:
- move the private key into platform-specific secret storage
- keep only non-secret metadata on disk if needed:
  - `agent_id`
  - fingerprint
  - public key
- add migration from current plaintext file storage

## Closed Gaps

These previously known gaps are now addressed on this branch:

- runtime no longer uses placeholder `job_id` as `X-Retaia-Agent-Id`
- technical bearer can be minted from configured `client_id + secret_key`
- daemon registers the agent before processing jobs
- signed writes now use actual agent identity material
- `fencing_token` is propagated and refreshed correctly
- derived upload flow follows current multipart and `part_etag` contract

## Residual Risks

- no proof yet that Core accepts the exact header encoding currently emitted for `X-Retaia-Signature`
- current review did not uncover a runtime bug in `claim/heartbeat/submit/upload` sequencing after the conformance refactor
- generated client/doc noise is large; future reviews should isolate generated diff from runtime diff earlier

## Validation Basis

Executed on this branch:

- `cargo check`
- `cargo check --features core-api-client`
- `cargo test -q`
- `cargo test -q --features core-api-client`

Manual review focus:

- [src/infrastructure/agent_identity.rs](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/src/infrastructure/agent_identity.rs)
- [src/infrastructure/signed_core_http.rs](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/src/infrastructure/signed_core_http.rs)
- [src/infrastructure/openapi_agent_registration_gateway.rs](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/src/infrastructure/openapi_agent_registration_gateway.rs)
- [src/infrastructure/openapi_derived_processing_gateway.rs](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/src/infrastructure/openapi_derived_processing_gateway.rs)
- [src/bin/agent-runtime.rs](/Users/fullfrontend/Jobs/A%20-%20Full%20Front-End/retaia-workspace/retaia-agent/src/bin/agent-runtime.rs)
