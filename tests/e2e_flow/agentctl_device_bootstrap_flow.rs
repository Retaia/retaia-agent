#![cfg(feature = "core-api-client")]

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use tempfile::tempdir;

fn run_agentctl(args: &[&str]) -> std::process::Output {
    run_agentctl_with_env(args, &[])
}

fn run_agentctl_with_env(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_agentctl");
    let store_file = std::env::temp_dir().join(format!(
        "retaia-agent-device-bootstrap-store-{}.json",
        std::process::id()
    ));
    let mut command = Command::new(exe);
    command
        .args(args)
        .env("RETAIA_AGENT_SECRET_STORE_BACKEND", "memory")
        .env("RETAIA_AGENT_SECRET_STORE_FILE", store_file);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("agentctl must execute")
}

fn spawn_agentctl_with_env(args: &[&str], envs: &[(&str, &str)]) -> std::process::Child {
    let exe = env!("CARGO_BIN_EXE_agentctl");
    let store_file = std::env::temp_dir().join(format!(
        "retaia-agent-device-bootstrap-store-{}-spawn.json",
        std::process::id()
    ));
    let mut command = Command::new(exe);
    command
        .args(args)
        .env("RETAIA_AGENT_SECRET_STORE_BACKEND", "memory")
        .env("RETAIA_AGENT_SECRET_STORE_FILE", store_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }
    command.spawn().expect("agentctl must spawn")
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

#[test]
fn e2e_agentctl_bootstrap_device_opens_browser_with_verification_uri_complete() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("local addr").port();

    let server = thread::spawn(move || {
        for (index, expected_path) in [
            "POST /api/v1/auth/clients/device/start ",
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
                    r#"{"device_code":"dev-browser","user_code":"BROW-SE12","verification_uri":"https://ui.retaia.local/device","verification_uri_complete":"https://ui.retaia.local/device?user_code=BROW-SE12","expires_in":900,"interval":1}"#
                }
                _ => {
                    r#"{"status":"APPROVED","client_id":"agent-browser","client_kind":"AGENT","secret_key":"browser-secret","approved_at":"2026-03-21T12:05:00Z","approved_by_user_id":"11111111-1111-1111-1111-111111111111"}"#
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
    let capture_path = dir.path().join("browser-url.txt");
    let browser_script = dir.path().join("browser-open.sh");
    fs::write(
        &browser_script,
        format!(
            "#!/bin/sh\nprintf '%s' \"$1\" > \"{}\"\n",
            capture_path.display()
        ),
    )
    .expect("write browser script");
    let chmod_status = Command::new("chmod")
        .args(["+x", browser_script.to_string_lossy().as_ref()])
        .status()
        .expect("chmod browser script");
    assert!(chmod_status.success());

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

    let bootstrap = run_agentctl_with_env(
        &[
            "config",
            "bootstrap-device",
            "--config",
            &config_path_str,
            "--client-label",
            "Browser Test",
        ],
        &[(
            "RETAIA_AGENT_BROWSER_OPEN_COMMAND",
            browser_script.to_string_lossy().as_ref(),
        )],
    );
    assert!(
        bootstrap.status.success(),
        "bootstrap failed: {}",
        String::from_utf8_lossy(&bootstrap.stderr)
    );

    let captured = fs::read_to_string(&capture_path).expect("captured browser URL");
    assert_eq!(
        captured,
        "https://ui.retaia.local/device?user_code=BROW-SE12"
    );

    server.join().expect("server thread");
}

#[cfg(unix)]
#[test]
fn e2e_agentctl_bootstrap_device_cancels_flow_on_sigint() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let port = listener.local_addr().expect("local addr").port();

    let server = thread::spawn(move || {
        for (index, expected_path) in [
            "POST /api/v1/auth/clients/device/start ",
            "POST /api/v1/auth/clients/device/poll ",
            "POST /api/v1/auth/clients/device/cancel ",
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
                    r#"{"device_code":"dev-cancel","user_code":"CANC-EL12","verification_uri":"https://ui.retaia.local/device","verification_uri_complete":"https://ui.retaia.local/device?user_code=CANC-EL12","expires_in":900,"interval":1}"#
                }
                1 => r#"{"status":"PENDING","interval":1}"#,
                _ => r#"{"canceled":true}"#,
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

    let child = spawn_agentctl_with_env(
        &[
            "config",
            "bootstrap-device",
            "--config",
            &config_path_str,
            "--client-label",
            "Cancel Test",
            "--no-browser",
        ],
        &[],
    );
    thread::sleep(Duration::from_millis(250));
    let signal_status = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("send SIGINT");
    assert!(signal_status.success(), "SIGINT should be delivered");

    let output = child.wait_with_output().expect("wait bootstrap child");
    assert!(!output.status.success(), "bootstrap cancel should fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("user_code=CANC-EL12"));
    assert!(stderr.contains("device flow canceled by user"));

    server.join().expect("server thread");
}
