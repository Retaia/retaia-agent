use retaia_agent::{
    CompletedJobEntry, DaemonCurrentJobStats, DaemonCycleEntry, DaemonDiagnosticsSnapshot,
    DaemonLastJobStats, DaemonRuntimeStats, DaemonStatus, build_bug_report_markdown,
    daemon_status_as_label, render_daemon_inspect,
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
