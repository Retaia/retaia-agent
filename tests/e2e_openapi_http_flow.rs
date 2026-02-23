#![cfg(feature = "core-api-client")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use retaia_agent::{
    AgentRegistrationCommand, AgentRegistrationError, AgentRegistrationGateway, AgentRuntimeConfig,
    AuthMode, CoreApiGateway, CoreApiGatewayError, DerivedJobType, DerivedKind,
    DerivedManifestItem, DerivedProcessingError, DerivedProcessingGateway, DerivedUploadComplete,
    DerivedUploadInit, DerivedUploadPart, LogLevel, OpenApiAgentRegistrationGateway,
    OpenApiDerivedProcessingGateway, OpenApiJobsGateway, SubmitDerivedPayload,
    build_core_api_client,
};

struct MockExchange {
    method: &'static str,
    path: &'static str,
    status: u16,
    content_type: &'static str,
    body: &'static str,
}

fn runtime_config(base_url: &str) -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: base_url.to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
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
fn e2e_openapi_derived_gateway_claim_rejects_missing_lock_token_from_http_payload() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-1/claim",
        status: 200,
        content_type: "application/json",
        body: r#"{"job_id":"job-1","job_type":"generate_proxy","status":"claimed","asset_uuid":"asset-1","required_capabilities":["media.proxies.photo@1"]}"#,
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
fn e2e_openapi_derived_gateway_claim_rejects_non_derived_job_type_from_http_payload() {
    let (server, base_url) = spawn_mock_server(vec![MockExchange {
        method: "POST",
        path: "/api/v1/jobs/job-nd/claim",
        status: 200,
        content_type: "application/json",
        body: r#"{"job_id":"job-nd","job_type":"extract_facts","status":"claimed","asset_uuid":"asset-nd","required_capabilities":["media.facts@1"],"lock_token":"lock-nd"}"#,
    }]);

    let client = build_core_api_client(&runtime_config(&base_url));
    let gateway = OpenApiDerivedProcessingGateway::new(client);
    let error = gateway
        .claim_job("job-nd")
        .expect_err("non-derived job type must fail");
    assert_eq!(
        error,
        DerivedProcessingError::NotDerivedJobType("extract_facts".to_string())
    );

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
        kind: DerivedKind::ProxyPhoto,
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
        agent_name: "retaia-agent".to_string(),
        agent_version: "1.0.0".to_string(),
        platform: Some("macos".to_string()),
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
        .heartbeat("job-2", "lock-2")
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
        .heartbeat("job-hb", "lock-hb")
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
        job_type: DerivedJobType::GenerateProxy,
        manifest: vec![DerivedManifestItem {
            kind: DerivedKind::ProxyPhoto,
            reference: "s3://bucket/proxy.webp".to_string(),
            size_bytes: Some(12),
            sha256: None,
        }],
        warnings: None,
        metrics: None,
    };
    let error = gateway
        .submit_derived("job-3", "lock-3", "idem-3", &payload)
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
    let request = DerivedUploadPart {
        asset_uuid: "asset-2".to_string(),
        upload_id: "upload-2".to_string(),
        part_number: 1,
    };
    let error = gateway
        .upload_part(&request)
        .expect_err("must fail on throttling");
    assert_eq!(error, DerivedProcessingError::Throttled);

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
        agent_name: "retaia-agent".to_string(),
        agent_version: "1.0.0".to_string(),
        platform: Some("linux".to_string()),
        capabilities: vec!["media.facts@1".to_string()],
        client_feature_flags_contract_version: Some("v1".to_string()),
        max_parallel_jobs: Some(2),
    };
    let error = gateway
        .register_agent(&command)
        .expect_err("must fail on invalid payload");
    match error {
        AgentRegistrationError::Transport(message) => {
            assert!(message.contains("invalid type") || message.contains("expected"))
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    server.join().expect("server thread");
}
