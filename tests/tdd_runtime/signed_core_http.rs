use chrono::{DateTime, Utc};
use reqwest::Method;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT_LANGUAGE, AUTHORIZATION, CONTENT_TYPE, IF_MATCH};
use retaia_agent::AgentIdentity;
use retaia_agent::infrastructure::signed_core_http::{
    SignedCoreHttpError, absolute_url, json_bytes, multipart_part_request, signed_empty_request,
    signed_json_request, signed_request,
};
use serde_json::json;

#[test]
fn tdd_signed_core_http_signed_json_request_sets_contract_headers_and_body() {
    let client = Client::new();
    let identity = AgentIdentity::generate_ephemeral(Some("550e8400-e29b-41d4-a716-446655440099"))
        .expect("identity");
    let payload = json_bytes(&json!({"ok": true})).expect("json bytes");

    let request = signed_json_request(
        &client,
        &identity,
        Some("token-1"),
        "https://core.example/api",
        Method::POST,
        "/v1/jobs/job-1/submit",
        &payload,
        Some("fr-BE"),
    )
    .expect("signed json request")
    .build()
    .expect("build request");

    assert_eq!(request.method(), Method::POST);
    assert_eq!(
        request.url().as_str(),
        "https://core.example/api/v1/jobs/job-1/submit"
    );
    assert_eq!(request.headers()[CONTENT_TYPE], "application/json");
    assert_eq!(request.headers()[AUTHORIZATION], "Bearer token-1");
    assert_eq!(request.headers()[ACCEPT_LANGUAGE], "fr-BE");
    assert_eq!(request.headers()["X-Retaia-Agent-Id"], identity.agent_id);
    assert_eq!(
        request.headers()["X-Retaia-OpenPGP-Fingerprint"],
        identity.openpgp_fingerprint
    );
    assert!(
        request.headers()["X-Retaia-Signature"]
            .to_str()
            .expect("signature")
            .contains("BEGIN PGP SIGNATURE")
    );
    assert!(
        !request.headers()["X-Retaia-Signature"]
            .to_str()
            .expect("signature")
            .contains('\n')
    );
    assert!(request.body().is_some());
}

#[test]
fn tdd_signed_core_http_signed_empty_request_hashes_empty_body_without_accept_language() {
    let client = Client::new();
    let identity = AgentIdentity::generate_ephemeral(None).expect("identity");

    let request = signed_empty_request(
        &client,
        &identity,
        None,
        "https://core.example",
        Method::POST,
        "/api/v1/jobs/job-1/claim",
        None,
    )
    .expect("signed empty request")
    .build()
    .expect("build request");

    assert!(request.headers().get(ACCEPT_LANGUAGE).is_none());
    assert!(request.headers().get(AUTHORIZATION).is_none());
    assert!(
        request
            .headers()
            .contains_key("X-Retaia-Signature-Timestamp")
    );
    assert!(request.headers().contains_key("X-Retaia-Signature-Nonce"));
}

#[test]
fn tdd_signed_core_http_multipart_request_sets_if_match_boundary_and_signed_headers() {
    let client = Client::new();
    let identity = AgentIdentity::generate_ephemeral(None).expect("identity");

    let request = multipart_part_request(
        &client,
        &identity,
        Some("token-2"),
        "https://core.example/api",
        "/v1/assets/asset-1/derived/upload/part",
        "\"rev-1\"",
        "upload-1",
        3,
        b"hello".to_vec(),
        Some("en-US"),
    )
    .expect("multipart request")
    .build()
    .expect("build request");

    assert_eq!(request.method(), Method::POST);
    assert_eq!(request.headers()[IF_MATCH], "\"rev-1\"");
    assert_eq!(request.headers()[AUTHORIZATION], "Bearer token-2");
    assert!(
        request.headers()[CONTENT_TYPE]
            .to_str()
            .expect("content type")
            .starts_with("multipart/form-data; boundary=retaia-agent-")
    );
    assert_eq!(request.headers()[ACCEPT_LANGUAGE], "en-US");
    assert!(request.headers().contains_key("X-Retaia-Signature"));
}

#[test]
fn tdd_signed_core_http_signed_request_returns_header_safe_signature_and_rfc3339_timestamp() {
    let identity = AgentIdentity::generate_ephemeral(None).expect("identity");
    let signed = signed_request(&identity, Method::PATCH, "/api/v1/assets/asset-1", br#"{}"#)
        .expect("signed request");

    assert!(!signed.signature.contains("\\n"));
    assert!(!signed.signature.contains('\n'));
    assert!(signed.signature.contains("BEGIN PGP SIGNATURE"));
    assert!(signed.timestamp.ends_with('Z'));
    assert_eq!(signed.nonce.len(), 36);
}

#[test]
fn tdd_signed_core_http_signed_request_timestamp_is_fresh_within_sixty_seconds() {
    let identity = AgentIdentity::generate_ephemeral(None).expect("identity");
    let before = Utc::now();
    let signed = signed_request(&identity, Method::POST, "/api/v1/jobs/job-1/claim", b"{}")
        .expect("signed request");
    let after = Utc::now();
    let parsed =
        DateTime::parse_from_rfc3339(&signed.timestamp).expect("timestamp should be rfc3339");
    let signed_at = parsed.with_timezone(&Utc);

    let skew_before = signed_at.signed_duration_since(before).num_seconds();
    let skew_after = after.signed_duration_since(signed_at).num_seconds();

    assert!(
        skew_before >= -60,
        "timestamp too old before generation window"
    );
    assert!(skew_after >= 0, "timestamp should not be in the future");
    assert!(
        skew_after <= 60,
        "timestamp too old after generation window"
    );
}

#[test]
fn tdd_signed_core_http_signed_request_generates_distinct_nonces() {
    let identity = AgentIdentity::generate_ephemeral(None).expect("identity");
    let first = signed_request(&identity, Method::POST, "/api/v1/jobs/job-1/claim", b"{}")
        .expect("first signed request");
    let second = signed_request(&identity, Method::POST, "/api/v1/jobs/job-1/claim", b"{}")
        .expect("second signed request");

    assert_ne!(first.nonce, second.nonce);
    assert_ne!(first.signature, second.signature);
}

#[test]
fn tdd_signed_core_http_absolute_url_rejects_invalid_base_url() {
    let error = absolute_url("://bad url", "/api/v1/jobs").expect_err("invalid base must fail");
    match error {
        SignedCoreHttpError::Url(message) => assert!(!message.is_empty()),
        other => panic!("unexpected error: {other:?}"),
    }
}
