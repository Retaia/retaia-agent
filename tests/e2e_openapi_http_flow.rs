#![cfg(feature = "core-api-client")]

use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;
use std::thread;

use chrono::{Duration, Utc};
use reqwest::header::CONTENT_TYPE;
use retaia_agent::infrastructure::agent_identity::AgentIdentity;
use retaia_agent::infrastructure::signed_core_http::{
    SignedRequest, absolute_url, apply_signed_headers, signature_payload,
};
use retaia_agent::{
    AgentRegistrationCommand, AgentRegistrationError, AgentRegistrationGateway, AgentRuntimeConfig,
    AuthMode, CoreApiGateway, CoreApiGatewayError, DerivedJobType, DerivedKind,
    DerivedManifestItem, DerivedProcessingError, DerivedProcessingGateway, DerivedUploadComplete,
    DerivedUploadInit, DerivedUploadPart, LogLevel, OpenApiAgentRegistrationGateway,
    OpenApiDerivedProcessingGateway, OpenApiJobsGateway, SubmitDerivedPayload,
    build_core_api_client,
};
use retaia_core_client::apis::assets_api::{AssetsApi, AssetsApiClient};
use tempfile::NamedTempFile;

struct MockExchange {
    method: &'static str,
    path: &'static str,
    status: u16,
    content_type: &'static str,
    body: &'static str,
}

struct MockExchangeWithHeaders {
    method: &'static str,
    path: &'static str,
    status: u16,
    content_type: &'static str,
    headers: Vec<&'static str>,
    body: &'static str,
}

fn runtime_config(base_url: &str) -> AgentRuntimeConfig {
    init_test_identity_env();
    AgentRuntimeConfig {
        core_api_url: base_url.to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts: std::collections::BTreeMap::new(),
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

fn init_test_identity_env() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let identity_path = std::env::temp_dir().join(format!(
            "retaia-agent-e2e-identity-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&identity_path);
        unsafe {
            std::env::set_var("RETAIA_AGENT_IDENTITY_PATH", identity_path);
            std::env::set_var("RETAIA_AGENT_SECRET_STORE_BACKEND", "memory");
        }
    });
}

fn spawn_mock_server(exchanges: Vec<MockExchange>) -> (thread::JoinHandle<()>, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");

    let handle = thread::spawn(move || {
        for exchange in exchanges {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0_u8; 4096];
            let size = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..size]);
            let first_line = request.lines().next().unwrap_or_default().to_string();
            assert!(
                first_line.starts_with(&format!("{} {}", exchange.method, exchange.path)),
                "unexpected request line: {first_line}"
            );

            let response = format!(
                "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                exchange.status,
                reason_phrase(exchange.status),
                exchange.content_type,
                exchange.body.len(),
                exchange.body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        }
    });

    (handle, base_url)
}

fn spawn_mock_server_with_headers(
    exchanges: Vec<MockExchangeWithHeaders>,
) -> (thread::JoinHandle<()>, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");

    let handle = thread::spawn(move || {
        for exchange in exchanges {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0_u8; 4096];
            let size = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..size]);
            let first_line = request.lines().next().unwrap_or_default().to_string();
            assert!(
                first_line.starts_with(&format!("{} {}", exchange.method, exchange.path)),
                "unexpected request line: {first_line}"
            );

            let extra_headers = if exchange.headers.is_empty() {
                String::new()
            } else {
                format!("{}\r\n", exchange.headers.join("\r\n"))
            };
            let response = format!(
                "HTTP/1.1 {} {}\r\nContent-Type: {}\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                exchange.status,
                reason_phrase(exchange.status),
                exchange.content_type,
                extra_headers,
                exchange.body.len(),
                exchange.body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        }
    });

    (handle, base_url)
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        401 => "Unauthorized",
        422 => "Unprocessable Entity",
        426 => "Upgrade Required",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Status",
    }
}

fn assert_request_contains_header(request: &str, header_line: &str) {
    let expected = header_line.to_ascii_lowercase();
    assert!(
        request
            .lines()
            .map(|line| line.to_ascii_lowercase())
            .any(|line| line == expected),
        "missing header `{header_line}` in request:\n{request}"
    );
}

fn request_header_value(request: &str, name: &str) -> String {
    let needle = format!("{}:", name.to_ascii_lowercase());
    request
        .lines()
        .find_map(|line| {
            let lower = line.to_ascii_lowercase();
            lower
                .starts_with(&needle)
                .then(|| {
                    line.split_once(':')
                        .map(|(_, value)| value.trim().to_string())
                })
                .flatten()
        })
        .unwrap_or_else(|| panic!("missing header `{name}` in request:\n{request}"))
}

fn build_signed_request(
    identity: &AgentIdentity,
    method: reqwest::Method,
    path: &str,
    timestamp: &str,
    nonce: &str,
    body: &[u8],
) -> SignedRequest {
    let payload = signature_payload(method, path, &identity.agent_id, timestamp, nonce, body);
    let signature = identity
        .detached_signature_http_header_value(payload.as_bytes())
        .expect("signature must be generated");
    SignedRequest {
        timestamp: timestamp.to_string(),
        nonce: nonce.to_string(),
        signature,
    }
}

fn send_signed_json_request(
    base_path: &str,
    path: &str,
    identity: &AgentIdentity,
    signed: &SignedRequest,
    body: &[u8],
) -> reqwest::blocking::Response {
    let client = reqwest::blocking::Client::new();
    let url = absolute_url(base_path, path).expect("absolute url");
    apply_signed_headers(
        client
            .request(reqwest::Method::POST, url)
            .header(CONTENT_TYPE, "application/json")
            .body(body.to_vec()),
        identity,
        signed,
        None,
        None,
    )
    .send()
    .expect("signed request should be sent")
}

static LANG_ENV_GUARD: Mutex<()> = Mutex::new(());

#[test]
fn e2e_openapi_jobs_gateway_maps_422_from_real_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "GET",
        path: "/api/v1/jobs",
        status: 422,
        content_type: "application/json",
        body: r#"{"code":"INVALID_QUERY"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiJobsGateway::new(client);
    let error = gateway.poll_jobs().expect_err("must fail on 422");
    assert_eq!(error, CoreApiGatewayError::UnexpectedStatus(422));

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_jobs_gateway_maps_401_from_real_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "GET",
        path: "/api/v1/jobs",
        status: 401,
        content_type: "application/json",
        body: r#"{"code":"UNAUTHORIZED"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiJobsGateway::new(client);
    let error = gateway.poll_jobs().expect_err("must fail on 401");
    assert_eq!(error, CoreApiGatewayError::Unauthorized);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_jobs_gateway_maps_429_from_real_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "GET",
        path: "/api/v1/jobs",
        status: 429,
        content_type: "application/json",
        body: r#"{"code":"TOO_MANY_REQUESTS"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiJobsGateway::new(client);
    let error = gateway.poll_jobs().expect_err("must fail on 429");
    assert_eq!(
        error,
        CoreApiGatewayError::Throttled {
            retry_after_ms: None,
        }
    );

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_jobs_gateway_maps_retry_after_from_real_http_response() {
    let (server, base_url) = spawn_mock_server_with_headers(vec![MockExchangeWithHeaders {
        method: "GET",
        path: "/api/v1/jobs",
        status: 429,
        content_type: "application/json",
        headers: vec!["Retry-After: 7"],
        body: r#"{"code":"TOO_MANY_REQUESTS"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiJobsGateway::new(client);
    let error = gateway.poll_jobs().expect_err("must fail on 429");
    assert_eq!(
        error,
        CoreApiGatewayError::Throttled {
            retry_after_ms: Some(7_000),
        }
    );

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_jobs_gateway_maps_invalid_payload_to_transport_error() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "GET",
        path: "/api/v1/jobs",
        status: 200,
        content_type: "application/json",
        body: r#"{"not":"a jobs array"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiJobsGateway::new(client);
    let error = gateway
        .poll_jobs()
        .expect_err("must fail on invalid payload");
    match error {
        CoreApiGatewayError::Transport(message) => {
            assert!(
                message.contains("invalid type")
                    || message.contains("expected")
                    || message.contains("error decoding response body")
            )
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_jobs_gateway_maps_text_success_payload_to_transport_error() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "GET",
        path: "/api/v1/jobs",
        status: 200,
        content_type: "text/plain",
        body: "ok-but-not-json",
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiJobsGateway::new(client);
    let error = gateway
        .poll_jobs()
        .expect_err("text payload should be rejected");
    match error {
        CoreApiGatewayError::Transport(message) => assert!(message.contains("text/plain")),
        other => panic!("unexpected error variant: {other:?}"),
    }

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_jobs_gateway_sends_accept_language_header() {
    let _guard = LANG_ENV_GUARD.lock().expect("lang env guard");
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0_u8; 4096];
        let size = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..size]).to_string();
        let first_line = request.lines().next().unwrap_or_default().to_string();
        assert!(
            first_line.starts_with("GET /api/v1/jobs"),
            "unexpected request line: {first_line}"
        );
        assert_request_contains_header(&request, "Accept-Language: fr");

        let body = "[]";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    unsafe {
        std::env::set_var("RETAIA_AGENT_LANG", "fr_BE");
    }
    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiJobsGateway::new(client);
    let jobs = gateway.poll_jobs().expect("jobs should succeed");
    assert!(jobs.is_empty());
    unsafe {
        std::env::remove_var("RETAIA_AGENT_LANG");
    }

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_assets_get_parses_asset_summary_name_and_updated_at() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "GET",
        path: "/api/v1/assets?limit=1",
        status: 200,
        content_type: "application/json",
        body: r#"{"items":[{"uuid":"asset-1","name":"IMG_0001.JPG","media_type":"PHOTO","state":"DECISION_PENDING","created_at":"2026-02-26T00:00:00Z","updated_at":"2026-02-26T01:00:00Z","revision_etag":"rev-1"}],"next_cursor":null}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let api = AssetsApiClient::new(Arc::new(client));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let response = runtime
        .block_on(api.assets_get(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(1),
            None,
            None,
        ))
        .expect("assets_get should succeed");

    let items = response.items.expect("items");
    assert_eq!(items.len(), 1);
    let first = &items[0];
    assert_eq!(first.name.as_deref(), Some("IMG_0001.JPG"));
    assert_eq!(first.updated_at, "2026-02-26T01:00:00Z");

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_claim_rejects_missing_lock_token_from_http_payload() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-1/claim",
        status: 200,
        content_type: "application/json",
        body: r#"{"job_id":"job-1","job_type":"generate_preview","status":"claimed","asset_uuid":"asset-1","source":{"storage_id":"nas-main","original_relative":"INBOX/a.mov"},"required_capabilities":["media.previews.photo@1"]}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let error = gateway
        .claim_job("job-1")
        .expect_err("must fail when lock token is missing");
    assert_eq!(error, DerivedProcessingError::MissingLockToken);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_claim_maps_401_from_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-401/claim",
        status: 401,
        content_type: "application/json",
        body: r#"{"code":"UNAUTHORIZED"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let error = gateway
        .claim_job("job-401")
        .expect_err("claim must fail on 401");
    assert_eq!(error, DerivedProcessingError::Unauthorized);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_claim_maps_optional_sidecars_from_http_payload() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-sidecars/claim",
        status: 200,
        content_type: "application/json",
        body: r#"{"job_id":"job-sidecars","job_type":"generate_preview","status":"claimed","asset_uuid":"asset-sidecars","lock_token":"lock-sidecars","fencing_token":1,"source":{"storage_id":"nas-main","original_relative":"INBOX/a.mov","sidecars_relative":["INBOX/a.xmp","INBOX/a.srt"]},"required_capabilities":["media.previews.video@1"]}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let claimed = gateway
        .claim_job("job-sidecars")
        .expect("claim with sidecars");
    assert_eq!(claimed.job_id, "job-sidecars");
    assert_eq!(claimed.source_storage_id, "nas-main");
    assert_eq!(claimed.source_original_relative, "INBOX/a.mov");
    assert_eq!(
        claimed.source_sidecars_relative,
        vec!["INBOX/a.xmp".to_string(), "INBOX/a.srt".to_string()]
    );

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_claim_accepts_extract_facts_job_type_from_http_payload() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-nd/claim",
        status: 200,
        content_type: "application/json",
        body: r#"{"job_id":"job-nd","job_type":"extract_facts","status":"claimed","asset_uuid":"asset-nd","source":{"storage_id":"nas-main","original_relative":"INBOX/a.mov"},"required_capabilities":["media.facts@1"],"lock_token":"lock-nd","fencing_token":1}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let claimed = gateway
        .claim_job("job-nd")
        .expect("extract_facts claim must pass");
    assert_eq!(claimed.job_id, "job-nd");
    assert_eq!(claimed.job_type, DerivedJobType::ExtractFacts);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_upload_init_maps_422_from_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/assets/asset-1/derived/upload/init",
        status: 422,
        content_type: "application/json",
        body: r#"{"code":"INVALID_DERIVED_UPLOAD"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let request = DerivedUploadInit {
        asset_uuid: "asset-1".to_string(),
        revision_etag: "\"asset-rev-1\"".to_string(),
        kind: DerivedKind::PreviewPhoto,
        content_type: "image/jpeg".to_string(),
        size_bytes: 64,
        sha256: None,
        idempotency_key: "idem-1".to_string(),
    };
    let error = gateway
        .upload_init(&request)
        .expect_err("must fail on unexpected 422");
    assert_eq!(error, DerivedProcessingError::UnexpectedStatus(422));

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_fetch_asset_revision_etag_reads_http_etag_header() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0_u8; 4096];
        let size = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..size]);
        let first_line = request.lines().next().unwrap_or_default().to_string();
        assert!(first_line.starts_with("GET /api/v1/assets/asset-9"));

        let response = concat!(
            "HTTP/1.1 200 OK\r\n",
            "Content-Type: application/json\r\n",
            "ETag: \"asset-rev-9\"\r\n",
            "Content-Length: 2\r\n",
            "Connection: close\r\n",
            "\r\n",
            "{}"
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let etag = gateway
        .fetch_asset_revision_etag("asset-9")
        .expect("etag fetch should succeed");
    assert_eq!(etag, "\"asset-rev-9\"");

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_agent_registration_gateway_maps_426_from_real_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/agents/register",
        status: 426,
        content_type: "application/json",
        body: r#"{"code":"UPGRADE_REQUIRED"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiAgentRegistrationGateway::new(client);
    let command = AgentRegistrationCommand {
        agent_id: "550e8400-e29b-41d4-a716-446655440010".to_string(),
        agent_name: "retaia-agent".to_string(),
        agent_version: "1.0.0".to_string(),
        os_name: "macos".to_string(),
        os_version: "15.3".to_string(),
        arch: "arm64".to_string(),
        capabilities: vec!["media.facts@1".to_string()],
        client_feature_flags_contract_version: Some("v1".to_string()),
        max_parallel_jobs: Some(2),
    };
    let error = gateway
        .register_agent(&command)
        .expect_err("must fail on 426");
    assert_eq!(error, AgentRegistrationError::UpgradeRequired);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_agent_registration_gateway_maps_401_from_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/agents/register",
        status: 401,
        content_type: "application/json",
        body: r#"{"code":"UNAUTHORIZED"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiAgentRegistrationGateway::new(client);
    let command = AgentRegistrationCommand {
        agent_id: "550e8400-e29b-41d4-a716-446655440011".to_string(),
        agent_name: "retaia-agent".to_string(),
        agent_version: "1.0.0".to_string(),
        os_name: "linux".to_string(),
        os_version: "6.8".to_string(),
        arch: "x86_64".to_string(),
        capabilities: vec!["media.facts@1".to_string()],
        client_feature_flags_contract_version: Some("v1".to_string()),
        max_parallel_jobs: Some(2),
    };
    let error = gateway
        .register_agent(&command)
        .expect_err("must fail on 401");
    assert_eq!(error, AgentRegistrationError::Unauthorized);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_agent_registration_gateway_maps_500_from_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/agents/register",
        status: 500,
        content_type: "application/json",
        body: r#"{"code":"INTERNAL_ERROR"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiAgentRegistrationGateway::new(client);
    let command = AgentRegistrationCommand {
        agent_id: "550e8400-e29b-41d4-a716-446655440012".to_string(),
        agent_name: "retaia-agent".to_string(),
        agent_version: "1.0.0".to_string(),
        os_name: "linux".to_string(),
        os_version: "6.8".to_string(),
        arch: "x86_64".to_string(),
        capabilities: vec!["media.facts@1".to_string()],
        client_feature_flags_contract_version: Some("v1".to_string()),
        max_parallel_jobs: Some(2),
    };
    let error = gateway
        .register_agent(&command)
        .expect_err("must fail on 500");
    assert_eq!(error, AgentRegistrationError::UnexpectedStatus(500));

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_heartbeat_maps_500_from_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-2/heartbeat",
        status: 500,
        content_type: "application/json",
        body: r#"{"code":"INTERNAL_ERROR"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let error = gateway
        .heartbeat("job-2", "lock-2", 1)
        .expect_err("must fail on 500");
    assert_eq!(error, DerivedProcessingError::UnexpectedStatus(500));

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_heartbeat_maps_invalid_success_payload_to_transport_error() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-hb/heartbeat",
        status: 200,
        content_type: "application/json",
        body: r#"{"locked_until":"unterminated""#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let error = gateway
        .heartbeat("job-hb", "lock-hb", 1)
        .expect_err("invalid payload must fail");
    match error {
        DerivedProcessingError::Transport(message) => assert!(!message.is_empty()),
        other => panic!("unexpected error variant: {other:?}"),
    }

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_heartbeat_maps_lock_required_error() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-lock-required/heartbeat",
        status: 409,
        content_type: "application/json",
        body: r#"{"code":"LOCK_REQUIRED"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let error = gateway
        .heartbeat("job-lock-required", "lock-required", 1)
        .expect_err("must fail on LOCK_REQUIRED");
    assert_eq!(error, DerivedProcessingError::LockRequired);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_submit_maps_401_from_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-3/submit",
        status: 401,
        content_type: "application/json",
        body: r#"{"code":"UNAUTHORIZED"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let payload = SubmitDerivedPayload {
        job_type: DerivedJobType::GeneratePreview,
        manifest: vec![DerivedManifestItem {
            kind: DerivedKind::PreviewPhoto,
            reference: "s3://bucket/proxy.webp".to_string(),
            size_bytes: Some(12),
            sha256: None,
        }],
        facts_patch: None,
        warnings: None,
        metrics: None,
    };
    let error = gateway
        .submit_derived("job-3", "lock-3", 1, "idem-3", &payload)
        .expect_err("must fail on 401");
    assert_eq!(error, DerivedProcessingError::Unauthorized);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_submit_maps_lock_invalid_error() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-lock-invalid/submit",
        status: 409,
        content_type: "application/json",
        body: r#"{"code":"LOCK_INVALID"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let payload = SubmitDerivedPayload {
        job_type: DerivedJobType::GeneratePreview,
        manifest: vec![DerivedManifestItem {
            kind: DerivedKind::PreviewPhoto,
            reference: "s3://bucket/proxy.webp".to_string(),
            size_bytes: Some(12),
            sha256: None,
        }],
        facts_patch: None,
        warnings: None,
        metrics: None,
    };
    let error = gateway
        .submit_derived(
            "job-lock-invalid",
            "lock-invalid",
            1,
            "idem-lock-invalid",
            &payload,
        )
        .expect_err("must fail on LOCK_INVALID");
    assert_eq!(error, DerivedProcessingError::LockInvalid);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_upload_part_maps_429_from_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/assets/asset-2/derived/upload/part",
        status: 429,
        content_type: "application/json",
        body: r#"{"code":"TOO_MANY_REQUESTS"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let chunk = NamedTempFile::new().expect("temp chunk");
    let request = DerivedUploadPart {
        asset_uuid: "asset-2".to_string(),
        revision_etag: "\"asset-rev-2\"".to_string(),
        upload_id: "upload-2".to_string(),
        part_number: 1,
        chunk_path: chunk.path().to_path_buf(),
    };
    let error = gateway
        .upload_part(&request)
        .expect_err("must fail on throttling");
    assert_eq!(error, DerivedProcessingError::Throttled);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_upload_init_sends_request_revision_etag_in_if_match_header() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0_u8; 4096];
        let size = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..size]);
        let first_line = request.lines().next().unwrap_or_default().to_string();
        assert!(first_line.starts_with("POST /api/v1/assets/asset-3/derived/upload/init"));
        assert_request_contains_header(&request, "If-Match: \"asset-rev-3\"");

        let response = concat!(
            "HTTP/1.1 204 No Content\r\n",
            "Content-Length: 0\r\n",
            "Connection: close\r\n",
            "\r\n"
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    gateway
        .upload_init(&DerivedUploadInit {
            asset_uuid: "asset-3".to_string(),
            revision_etag: "\"asset-rev-3\"".to_string(),
            kind: DerivedKind::PreviewPhoto,
            content_type: "image/jpeg".to_string(),
            size_bytes: 64,
            sha256: None,
            idempotency_key: "idem-3".to_string(),
        })
        .expect("upload init should succeed");

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_upload_complete_maps_500_from_http_response() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/assets/asset-2/derived/upload/complete",
        status: 500,
        content_type: "application/json",
        body: r#"{"code":"UPLOAD_STORE_DOWN"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let request = DerivedUploadComplete {
        asset_uuid: "asset-2".to_string(),
        revision_etag: "\"asset-rev-2\"".to_string(),
        upload_id: "upload-2".to_string(),
        idempotency_key: "idem-2".to_string(),
        parts: None,
    };
    let error = gateway
        .upload_complete(&request)
        .expect_err("must fail on 500");
    assert_eq!(error, DerivedProcessingError::UnexpectedStatus(500));

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_upload_complete_maps_stale_lock_token_error() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/assets/asset-stale/derived/upload/complete",
        status: 412,
        content_type: "application/json",
        body: r#"{"code":"STALE_LOCK_TOKEN"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let request = DerivedUploadComplete {
        asset_uuid: "asset-stale".to_string(),
        revision_etag: "\"asset-rev-stale\"".to_string(),
        upload_id: "upload-stale".to_string(),
        idempotency_key: "idem-stale".to_string(),
        parts: None,
    };
    let error = gateway
        .upload_complete(&request)
        .expect_err("must fail on STALE_LOCK_TOKEN");
    assert_eq!(error, DerivedProcessingError::StaleLockToken);

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_upload_part_sends_request_revision_etag_in_if_match_header() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0_u8; 8192];
        let size = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..size]);
        let first_line = request.lines().next().unwrap_or_default().to_string();
        assert!(first_line.starts_with("POST /api/v1/assets/asset-4/derived/upload/part"));
        assert_request_contains_header(&request, "If-Match: \"asset-rev-4\"");

        let body = r#"{"part_etag":"part-etag-1"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let chunk = NamedTempFile::new().expect("temp chunk");
    std::fs::write(chunk.path(), b"chunk").expect("write chunk");
    let uploaded = gateway
        .upload_part(&DerivedUploadPart {
            asset_uuid: "asset-4".to_string(),
            revision_etag: "\"asset-rev-4\"".to_string(),
            upload_id: "upload-4".to_string(),
            part_number: 1,
            chunk_path: chunk.path().to_path_buf(),
        })
        .expect("upload part should succeed");
    assert_eq!(uploaded.part_etag, "part-etag-1");

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_derived_gateway_upload_complete_sends_request_revision_etag_in_if_match_header() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0_u8; 4096];
        let size = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..size]);
        let first_line = request.lines().next().unwrap_or_default().to_string();
        assert!(first_line.starts_with("POST /api/v1/assets/asset-5/derived/upload/complete"));
        assert_request_contains_header(&request, "If-Match: \"asset-rev-5\"");

        let response = concat!(
            "HTTP/1.1 204 No Content\r\n",
            "Content-Length: 0\r\n",
            "Connection: close\r\n",
            "\r\n"
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    gateway
        .upload_complete(&DerivedUploadComplete {
            asset_uuid: "asset-5".to_string(),
            revision_etag: "\"asset-rev-5\"".to_string(),
            upload_id: "upload-5".to_string(),
            idempotency_key: "idem-5".to_string(),
            parts: Some(vec![retaia_agent::UploadedDerivedPart {
                part_number: 1,
                part_etag: "part-etag-1".to_string(),
            }]),
        })
        .expect("upload complete should succeed");

    server.join().expect("server thread");
}

#[test]
fn e2e_openapi_agent_registration_gateway_maps_invalid_success_payload_to_transport_error() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/agents/register",
        status: 200,
        content_type: "application/json",
        body: r#"["unexpected","array"]"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiAgentRegistrationGateway::new(client);
    let command = AgentRegistrationCommand {
        agent_id: "550e8400-e29b-41d4-a716-446655440013".to_string(),
        agent_name: "retaia-agent".to_string(),
        agent_version: "1.0.0".to_string(),
        os_name: "linux".to_string(),
        os_version: "6.8".to_string(),
        arch: "x86_64".to_string(),
        capabilities: vec!["media.facts@1".to_string()],
        client_feature_flags_contract_version: Some("v1".to_string()),
        max_parallel_jobs: Some(2),
    };
    let error = gateway
        .register_agent(&command)
        .expect_err("must fail on invalid payload");
    match error {
        AgentRegistrationError::Transport(message) => assert!(!message.trim().is_empty()),
        other => panic!("unexpected error variant: {other:?}"),
    }

    server.join().expect("server thread");
}

#[test]
fn e2e_signed_core_http_replay_nonce_is_rejected_by_mock_core_validator() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");
    let base_path = build_core_api_client(&runtime_config(&base_url)).base_path;
    let seen_nonces = Arc::new(Mutex::new(HashSet::<String>::new()));
    let seen_nonces_server = Arc::clone(&seen_nonces);

    let server = thread::spawn(move || {
        for expected_status in [200_u16, 401_u16] {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0_u8; 4096];
            let size = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..size]).to_string();
            let first_line = request.lines().next().unwrap_or_default().to_string();
            assert!(
                first_line.starts_with("POST /api/v1/agents/register"),
                "unexpected request line: {first_line}"
            );

            let nonce = request_header_value(&request, "X-Retaia-Signature-Nonce");
            let mut nonces = seen_nonces_server.lock().expect("nonce set");
            let accepted = nonces.insert(nonce);
            drop(nonces);

            let (status, body) = if accepted {
                (
                    200,
                    r#"{"agent_id":"550e8400-e29b-41d4-a716-446655440021","effective_capabilities":[],"capability_warnings":[]}"#,
                )
            } else {
                (401, r#"{"code":"REPLAYED_SIGNATURE_NONCE"}"#)
            };
            assert_eq!(status, expected_status, "unexpected server status order");

            let response = format!(
                "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                reason_phrase(status),
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        }
    });

    let identity = AgentIdentity::generate_ephemeral(Some("550e8400-e29b-41d4-a716-446655440021"))
        .expect("identity");
    let body = br#"{"agent_name":"retaia-agent"}"#;
    let path = "/agents/register";
    let signed = build_signed_request(
        &identity,
        reqwest::Method::POST,
        "/api/v1/agents/register",
        &Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "nonce-replay-test",
        body,
    );

    let first = send_signed_json_request(&base_path, path, &identity, &signed, body);
    assert_eq!(first.status(), 200);

    let second = send_signed_json_request(&base_path, path, &identity, &signed, body);
    assert_eq!(second.status(), 401);

    server.join().expect("server thread");
}

#[test]
fn e2e_signed_core_http_stale_timestamp_is_rejected_by_mock_core_validator() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let base_url = format!("http://127.0.0.1:{port}");
    let base_path = build_core_api_client(&runtime_config(&base_url)).base_path;

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0_u8; 4096];
        let size = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..size]).to_string();
        let first_line = request.lines().next().unwrap_or_default().to_string();
        assert!(
            first_line.starts_with("POST /api/v1/agents/register"),
            "unexpected request line: {first_line}"
        );

        let timestamp = request_header_value(&request, "X-Retaia-Signature-Timestamp");
        let parsed = chrono::DateTime::parse_from_rfc3339(&timestamp)
            .expect("timestamp should be rfc3339")
            .with_timezone(&Utc);
        assert!(
            Utc::now() - parsed > Duration::seconds(60),
            "request must be stale for validator"
        );

        let body = r#"{"code":"STALE_SIGNATURE_TIMESTAMP"}"#;
        let response = format!(
            "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    let identity = AgentIdentity::generate_ephemeral(Some("550e8400-e29b-41d4-a716-446655440022"))
        .expect("identity");
    let body = br#"{"agent_name":"retaia-agent"}"#;
    let signed = build_signed_request(
        &identity,
        reqwest::Method::POST,
        "/api/v1/agents/register",
        &(Utc::now() - Duration::seconds(61)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "nonce-stale-test",
        body,
    );

    let response =
        send_signed_json_request(&base_path, "/agents/register", &identity, &signed, body);
    assert_eq!(response.status(), 401);

    server.join().expect("server thread");
}
