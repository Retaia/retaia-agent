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
        "retaia-agent-rotate-secret-store-{}.json",
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
fn e2e_agentctl_rotate_secret_updates_local_technical_auth() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("local addr").port();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0_u8; 4096];
        let size = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..size]);
        let first_line = request.lines().next().unwrap_or_default().to_string();
        assert!(
            first_line.starts_with("POST /api/v1/auth/clients/agent-current/rotate-secret "),
            "unexpected request line: {first_line}"
        );

        let body = r#"{"client_id":"agent-current","secret_key":"rotated-secret","rotated_at":"2026-03-21T13:00:00Z"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
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
        "--auth-mode",
        "technical",
        "--client-id",
        "agent-current",
        "--secret-key",
        "old-secret",
    ]);
    assert!(init.status.success(), "init failed: {init:?}");

    let rotate = run_agentctl(&["config", "rotate-secret", "--config", &config_path_str]);
    assert!(
        rotate.status.success(),
        "rotate-secret failed: {}",
        String::from_utf8_lossy(&rotate.stderr)
    );
    let rotate_stdout = String::from_utf8_lossy(&rotate.stdout);
    assert!(rotate_stdout.contains("rotate_secret=ok"));
    assert!(rotate_stdout.contains("client_id=agent-current"));
    assert!(rotate_stdout.contains("rotated_at=2026-03-21T13:00:00Z"));

    let show = run_agentctl(&["config", "show", "--config", &config_path_str]);
    assert!(show.status.success(), "show failed: {show:?}");
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("auth_mode=technical"));
    assert!(show_stdout.contains("technical_client_id=agent-current"));
    assert!(show_stdout.contains("technical_secret_key_set=true"));
    assert!(!show_stdout.contains("rotated-secret"));

    let raw = fs::read_to_string(&config_path).expect("config file should exist");
    assert!(raw.contains("client_id = \"agent-current\""));
    assert!(!raw.contains("rotated-secret"));
    assert!(!raw.contains("secret_key"));

    server.join().expect("server thread");
}
