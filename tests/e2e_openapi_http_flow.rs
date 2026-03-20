#![cfg(feature = "core-api-client")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Once;
use std::thread;

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
    assert_eq!(error, CoreApiGatewayError::Throttled);

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
            assert!(message.contains("invalid type") || message.contains("expected"))
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
