use retaia_agent::{
    AgentRuntimeConfig, AuthMode, CompletedJobEntry, DaemonCurrentJobStats, DaemonCycleEntry,
    DaemonDiagnosticsSnapshot, DaemonLastJobStats, DaemonRuntimeStats, DaemonStatus, LogLevel,
    TechnicalAuthConfig, append_redacted_config_markdown, build_bug_report_markdown,
    daemon_status_as_label, redacted_runtime_config_from, render_daemon_inspect,
    render_daemon_inspect_json,
};

#[test]
fn tdd_daemon_diagnostics_status_label_maps_variants() {
    assert_eq!(
        daemon_status_as_label(Some(&DaemonStatus::Running)),
        "running"
    );
    assert_eq!(
        daemon_status_as_label(Some(&DaemonStatus::NotInstalled)),
        "not_installed"
    );
    assert_eq!(
        daemon_status_as_label(Some(&DaemonStatus::Stopped(Some("x".to_string())))),
        "stopped"
    );
    assert_eq!(daemon_status_as_label(None), "unknown");
}

#[test]
fn tdd_daemon_diagnostics_build_bug_report_with_stats_and_history() {
    let snapshot = DaemonDiagnosticsSnapshot {
        daemon_status: Some(DaemonStatus::Running),
        stats: Some(DaemonRuntimeStats {
            updated_at_unix_ms: 1700000000000,
            run_state: "running".to_string(),
            tick: 42,
            current_job: Some(DaemonCurrentJobStats {
                job_id: "job-42".to_string(),
                asset_uuid: "asset-42".to_string(),
                progress_percent: 67,
                stage: "proxy".to_string(),
                status: "running".to_string(),
                started_at_unix_ms: 1700000001000,
            }),
            last_job: Some(DaemonLastJobStats {
                job_id: "job-41".to_string(),
                duration_ms: 950,
                completed_at_unix_ms: 1700000000900,
            }),
        }),
        completed_jobs: vec![CompletedJobEntry {
            completed_at_unix_ms: 1700000000900,
            job_id: "job-41".to_string(),
            duration_ms: 950,
        }],
        cycles: vec![DaemonCycleEntry {
            ts_unix_ms: 1700000000800,
            tick: 42,
            outcome: "ok".to_string(),
            run_state: "running".to_string(),
            job_id: Some("job-42".to_string()),
            asset_uuid: Some("asset-42".to_string()),
            progress_percent: Some(67),
            stage: Some("proxy".to_string()),
            short_status: Some("running".to_string()),
        }],
    };

    let markdown = build_bug_report_markdown(
        &snapshot,
        Some("Custom bug title"),
        "daemon-stats.json",
        Some("/tmp/daemon-history.sqlite3"),
    );

    assert_eq!(markdown.title, "Custom bug title");
    assert!(markdown.body.contains("daemon_status: `running`"));
    assert!(markdown.body.contains("current_job_id: `job-42`"));
    assert!(markdown.body.contains("last_job_id: `job-41`"));
    assert!(
        markdown
            .body
            .contains("completed_at=1700000000900 job_id=`job-41` duration_ms=950")
    );
    assert!(
        markdown
            .body
            .contains("ts=1700000000800 tick=42 outcome=`ok` run_state=`running`")
    );
}

#[test]
fn tdd_daemon_diagnostics_render_inspect_without_stats() {
    let snapshot = DaemonDiagnosticsSnapshot {
        daemon_status: Some(DaemonStatus::Stopped(None)),
        stats: None,
        completed_jobs: Vec::new(),
        cycles: Vec::new(),
    };

    let rendered = render_daemon_inspect(&snapshot, None);

    assert!(rendered.contains("daemon_status=stopped"));
    assert!(rendered.contains("history_db_path=unavailable"));
    assert!(rendered.contains("stats=unavailable"));
    assert!(rendered.contains("completed_jobs_count=0"));
    assert!(rendered.contains("cycles_count=0"));
}

#[test]
fn tdd_daemon_diagnostics_render_inspect_json_contains_redacted_config() {
    let snapshot = DaemonDiagnosticsSnapshot {
        daemon_status: Some(DaemonStatus::Running),
        stats: None,
        completed_jobs: Vec::new(),
        cycles: Vec::new(),
    };
    let config = redacted_runtime_config_from(&AgentRuntimeConfig {
        core_api_url: "https://core.example".to_string(),
        ollama_url: "http://localhost:11434".to_string(),
        auth_mode: AuthMode::Technical,
        technical_auth: Some(TechnicalAuthConfig {
            client_id: "client-id".to_string(),
            secret_key: "secret".to_string(),
        }),
        max_parallel_jobs: 3,
        log_level: LogLevel::Info,
    });
    let rendered =
        render_daemon_inspect_json(&snapshot, Some("/tmp/history.sqlite3"), Some(&config));
    assert!(rendered.contains("\"daemon_status\": \"running\""));
    assert!(rendered.contains("\"history_db_path\": \"/tmp/history.sqlite3\""));
    assert!(rendered.contains("\"redacted_config\""));
    assert!(rendered.contains("\"technical_secret_key_set\": true"));
}

#[test]
fn tdd_daemon_diagnostics_append_redacted_config_markdown_outputs_section() {
    let mut body = String::new();
    append_redacted_config_markdown(&mut body, None);
    assert!(body.contains("## Redacted Runtime Config"));
    assert!(body.contains("- unavailable"));
}
