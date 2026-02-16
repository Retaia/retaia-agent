use std::fs;
use std::process::Command;

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
    ]);
    assert!(init.status.success(), "init failed: {init:?}");

    let show = run_agentctl(&["config", "show", "--config", &config_path_str]);
    assert!(show.status.success(), "show failed: {show:?}");
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("core_api_url=https://core.retaia.local"));
    assert!(show_stdout.contains("technical_client_id=agent-e2e"));
    assert!(show_stdout.contains("technical_secret_key_set=true"));
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
    ]);
    assert!(set.status.success(), "set failed: {set:?}");

    let raw = fs::read_to_string(&config_path).expect("config file should exist");
    assert!(raw.contains("max_parallel_jobs = 6"));
    assert!(raw.contains("log_level = \"warn\""));
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
    assert!(stderr.contains("Unable to load current config for set"));
}
