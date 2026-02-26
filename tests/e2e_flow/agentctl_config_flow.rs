use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use tempfile::tempdir;

fn run_agentctl(args: &[&str]) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_agentctl");
    Command::new(exe)
        .args(args)
        .output()
        .expect("agentctl must execute")
}

#[test]
fn e2e_agentctl_init_show_validate_set_flow() {
    let dir = tempdir().expect("temp dir");
    let config_path = dir.path().join("agent-config.toml");
    let config_path_str = config_path.to_string_lossy().to_string();

    let init = run_agentctl(&[
        "config",
        "init",
        "--config",
        &config_path_str,
        "--core-api-url",
        "https://core.retaia.local",
        "--ollama-url",
        "http://127.0.0.1:11434",
        "--auth-mode",
        "technical",
        "--client-id",
        "agent-e2e",
        "--secret-key",
        "super-secret",
        "--max-parallel-jobs",
        "3",
        "--log-level",
        "info",
        "--storage-mount",
        "nas-main=/mnt/nas/main/",
        "--storage-mount",
        "archive=/mnt/nas/archive",
    ]);
    assert!(init.status.success(), "init failed: {init:?}");

    let show = run_agentctl(&["config", "show", "--config", &config_path_str]);
    assert!(show.status.success(), "show failed: {show:?}");
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("core_api_url=https://core.retaia.local/api/v1"));
    assert!(show_stdout.contains("technical_client_id=agent-e2e"));
    assert!(show_stdout.contains("technical_secret_key_set=true"));
    assert!(show_stdout.contains("storage_mounts=archive=/mnt/nas/archive,nas-main=/mnt/nas/main"));
    assert!(!show_stdout.contains("super-secret"));

    let validate = run_agentctl(&["config", "validate", "--config", &config_path_str]);
    assert!(validate.status.success(), "validate failed: {validate:?}");

    let set = run_agentctl(&[
        "config",
        "set",
        "--config",
        &config_path_str,
        "--max-parallel-jobs",
        "6",
        "--log-level",
        "warn",
        "--storage-mount",
        "nas-main=/srv/nas/main/",
    ]);
    assert!(set.status.success(), "set failed: {set:?}");

    let raw = fs::read_to_string(&config_path).expect("config file should exist");
    assert!(raw.contains("max_parallel_jobs = 6"));
    assert!(raw.contains("log_level = \"warn\""));
    assert!(raw.contains("nas-main = \"/srv/nas/main\""));
}

#[test]
fn e2e_agentctl_set_requires_existing_config() {
    let dir = tempdir().expect("temp dir");
    let config_path = dir.path().join("missing.toml");
    let config_path_str = config_path.to_string_lossy().to_string();

    let set = run_agentctl(&[
        "config",
        "set",
        "--config",
        &config_path_str,
        "--log-level",
        "debug",
    ]);

    assert!(!set.status.success(), "set should fail on missing config");
    let stderr = String::from_utf8_lossy(&set.stderr);
    assert!(stderr.contains("unable to load current config for set"));
}

#[test]
fn e2e_agentctl_validate_check_respond_succeeds_when_core_and_ollama_endpoints_reply() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("local addr").port();

    let server = thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0_u8; 1024];
            let size = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..size]);
            let response = if request.starts_with("GET /api/v1/jobs ") {
                "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: 23\r\nConnection: close\r\n\r\n{\"code\":\"UNAUTHORIZED\"}".to_string()
            } else if request.starts_with("GET /api/v1/assets?captured_at_from=") {
                "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: 23\r\nConnection: close\r\n\r\n{\"code\":\"UNAUTHORIZED\"}".to_string()
            } else if request.starts_with("POST /v1/chat/completions ") {
                let body = "{\"error\":{\"message\":\"model not found\"}}";
                format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
            } else {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_string()
            };
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        }
    });

    let dir = tempdir().expect("temp dir");
    let config_path = dir.path().join("agent-config-check-respond.toml");
    let config_path_str = config_path.to_string_lossy().to_string();
    let base_url = format!("http://127.0.0.1:{port}");

    let init = run_agentctl(&[
        "config",
        "init",
        "--config",
        &config_path_str,
        "--core-api-url",
        &base_url,
        "--ollama-url",
        &base_url,
    ]);
    assert!(init.status.success(), "init failed: {init:?}");

    let validate = run_agentctl(&[
        "config",
        "validate",
        "--config",
        &config_path_str,
        "--check-respond",
    ]);
    assert!(
        validate.status.success(),
        "validate with check-respond failed: {validate:?}"
    );

    server.join().expect("server thread");
}

#[test]
fn e2e_agentctl_validate_check_respond_probes_assets_with_captured_at_filters() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("local addr").port();
    let seen_requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen_requests_server = Arc::clone(&seen_requests);

    let server = thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0_u8; 1024];
            let size = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..size]).to_string();
            seen_requests_server
                .lock()
                .expect("lock seen requests")
                .push(request.clone());
            let response = if request.starts_with("GET /api/v1/jobs ") {
                "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: 23\r\nConnection: close\r\n\r\n{\"code\":\"UNAUTHORIZED\"}".to_string()
            } else if request.starts_with("GET /api/v1/assets?captured_at_from=") {
                "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: 23\r\nConnection: close\r\n\r\n{\"code\":\"UNAUTHORIZED\"}".to_string()
            } else if request.starts_with("POST /v1/chat/completions ") {
                let body = "{\"error\":{\"message\":\"model not found\"}}";
                format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
            } else {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_string()
            };
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        }
    });

    let dir = tempdir().expect("temp dir");
    let config_path = dir.path().join("agent-config-check-respond-assets.toml");
    let config_path_str = config_path.to_string_lossy().to_string();
    let base_url = format!("http://127.0.0.1:{port}");

    let init = run_agentctl(&[
        "config",
        "init",
        "--config",
        &config_path_str,
        "--core-api-url",
        &base_url,
        "--ollama-url",
        &base_url,
    ]);
    assert!(init.status.success(), "init failed: {init:?}");

    let validate = run_agentctl(&[
        "config",
        "validate",
        "--config",
        &config_path_str,
        "--check-respond",
    ]);
    assert!(
        validate.status.success(),
        "validate with check-respond failed: {validate:?}"
    );

    server.join().expect("server thread");

    let requests = seen_requests.lock().expect("lock seen requests");
    let assets_probe_present = requests.iter().any(|request| {
        request.starts_with(
            "GET /api/v1/assets?captured_at_from=2024-01-01T00:00:00Z&captured_at_to=2024-12-31T23:59:59Z&sort=-captured_at&limit=1 ",
        )
    });
    assert!(
        assets_probe_present,
        "expected assets captured_at probe request, got: {:?}",
        requests
    );
}

#[test]
fn e2e_agentctl_validate_check_respond_fails_when_endpoints_reply_but_are_not_compatible() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("local addr").port();

    let server = thread::spawn(move || {
        for _ in 0..1 {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer);
            let response =
                b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            stream.write_all(response).expect("write response");
        }
    });

    let dir = tempdir().expect("temp dir");
    let config_path = dir.path().join("agent-config-incompatible.toml");
    let config_path_str = config_path.to_string_lossy().to_string();
    let base_url = format!("http://127.0.0.1:{port}");

    let init = run_agentctl(&[
        "config",
        "init",
        "--config",
        &config_path_str,
        "--core-api-url",
        &base_url,
        "--ollama-url",
        &base_url,
    ]);
    assert!(init.status.success(), "init failed: {init:?}");

    let validate = run_agentctl(&[
        "config",
        "validate",
        "--config",
        &config_path_str,
        "--check-respond",
    ]);
    assert!(
        !validate.status.success(),
        "validate should fail: {validate:?}"
    );
    let stderr = String::from_utf8_lossy(&validate.stderr);
    assert!(stderr.contains("config endpoint incompatible"));

    server.join().expect("server thread");
}
