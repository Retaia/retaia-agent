use std::collections::BTreeMap;

use retaia_agent::{
    AgentUiRuntime, ClientKind, JobStage, JobStatus, RuntimeSnapshot, SystemNotification,
    can_issue_client_token, can_process_jobs, resolve_effective_features,
};

#[test]
fn e2e_agent_service_mode_keeps_processing_authorized() {
    let app = BTreeMap::from([(String::from("features.ai"), true)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);
    let ai_enabled = *effective.get("features.ai").unwrap_or(&false);

    assert!(can_issue_client_token(ClientKind::Agent, ai_enabled));
    assert!(can_process_jobs(ClientKind::Agent));
}

#[test]
fn e2e_mcp_can_orchestrate_but_never_process_jobs() {
    let app = BTreeMap::from([(String::from("features.ai"), true)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);
    let ai_enabled = *effective.get("features.ai").unwrap_or(&false);

    assert!(can_issue_client_token(ClientKind::Mcp, ai_enabled));
    assert!(!can_process_jobs(ClientKind::Mcp));
}

#[test]
fn e2e_mcp_global_ai_off_blocks_client_token_flow() {
    let app = BTreeMap::from([(String::from("features.ai"), false)]);
    let user = BTreeMap::new();
    let deps = BTreeMap::new();
    let escalation = BTreeMap::new();

    let effective = resolve_effective_features(&app, &user, &deps, &escalation);
    let ai_enabled = *effective.get("features.ai").unwrap_or(&true);

    assert!(!can_issue_client_token(ClientKind::Mcp, ai_enabled));
    assert!(!can_process_jobs(ClientKind::Mcp));
}

#[test]
fn e2e_status_window_and_notifications_work_across_poll_transitions() {
    let mut runtime = AgentUiRuntime::new();
    let mut first = RuntimeSnapshot::default();
    first.known_job_ids.insert("job-100".to_string());
    first.running_job_ids.insert("job-100".to_string());
    first.current_job = Some(JobStatus {
        job_id: "job-100".to_string(),
        asset_uuid: "asset-9".to_string(),
        progress_percent: 37,
        stage: JobStage::Processing,
        short_status: "transcoding".to_string(),
    });

    let first_notifs = runtime.update_snapshot(first.clone());
    assert_eq!(
        first_notifs,
        vec![SystemNotification::NewJobReceived {
            job_id: "job-100".to_string()
        }]
    );
    let current = AgentUiRuntime::status_window_job(&first).expect("current job missing");
    assert_eq!(current.progress_percent, 37);
    assert_eq!(current.short_status, "transcoding");

    let second = RuntimeSnapshot::default();
    let second_notifs = runtime.update_snapshot(second);
    assert_eq!(second_notifs, vec![SystemNotification::AllJobsDone]);
}
