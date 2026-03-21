#![cfg(feature = "core-api-client")]

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

use tempfile::tempdir;

fn run_agentctl(args: &[&str]) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_agentctl");
    let store_file = std::env::temp_dir().join(format!(
        "retaia-agent-device-bootstrap-store-{}.json",
        std::process::id()
    ));
    Command::new(exe)
        .args(args)
        .env("RETAIA_AGENT_SECRET_STORE_BACKEND", "memory")
        .env("RETAIA_AGENT_SECRET_STORE_FILE", store_file)
        .output()
        .expect("agentctl must execute")
}

#[test]
fn e2e_agentctl_bootstrap_device_flow_persists_approved_technical_auth() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("local addr").port();

    let server = thread::spawn(move || {
        for (index, expected_path) in [
            "POST /api/v1/auth/clients/device/start ",
            "POST /api/v1/auth/clients/device/poll ",
            "POST /api/v1/auth/clients/device/poll ",
        ]
        .into_iter()
        .enumerate()
        {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0_u8; 4096];
            let size = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..size]);
            let first_line = request.lines().next().unwrap_or_default().to_string();
            assert!(
                first_line.starts_with(expected_path),
                "unexpected request line: {first_line}"
            );

            let body = match index {
                0 => {
                    r#"{"device_code":"dev-123","user_code":"ABCD-EFGH","verification_uri":"https://ui.retaia.local/device","verification_uri_complete":"https://ui.retaia.local/device?user_code=ABCD-EFGH","expires_in":900,"interval":1}"#
                }
                1 => r#"{"status":"PENDING","interval":1}"#,
                _ => {
                    r#"{"status":"APPROVED","client_id":"agent-approved","client_kind":"AGENT","secret_key":"approved-secret","approved_at":"2026-03-21T12:00:00Z","approved_by_user_id":"11111111-1111-1111-1111-111111111111"}"#
                }
            };

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        }
    });

    let dir = tempdir().expect("temp dir");
    let config_path = dir.path().join("agent-config.toml");
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
        "http://127.0.0.1:11434",
    ]);
    assert!(init.status.success(), "init failed: {init:?}");

    let bootstrap = run_agentctl(&[
        "config",
        "bootstrap-device",
        "--config",
        &config_path_str,
        "--client-label",
        "Studio Mac",
        "--no-browser",
    ]);
    assert!(
        bootstrap.status.success(),
        "bootstrap failed: {}",
        String::from_utf8_lossy(&bootstrap.stderr)
    );
    let bootstrap_stdout = String::from_utf8_lossy(&bootstrap.stdout);
    assert!(bootstrap_stdout.contains("user_code=ABCD-EFGH"));
    assert!(bootstrap_stdout.contains("device_bootstrap=approved"));

    let show = run_agentctl(&["config", "show", "--config", &config_path_str]);
    assert!(show.status.success(), "show failed: {show:?}");
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("auth_mode=technical"));
    assert!(show_stdout.contains("technical_client_id=agent-approved"));
    assert!(show_stdout.contains("technical_secret_key_set=true"));
    assert!(!show_stdout.contains("approved-secret"));

    let raw = fs::read_to_string(&config_path).expect("config file should exist");
    assert!(raw.contains("auth_mode = \"technical\""));
    assert!(raw.contains("client_id = \"agent-approved\""));
    assert!(!raw.contains("approved-secret"));
    assert!(!raw.contains("secret_key"));

    server.join().expect("server thread");
}
