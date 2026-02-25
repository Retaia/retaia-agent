use std::sync::{Arc, Mutex};

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClaimedDerivedJob, CoreApiGateway, CoreApiGatewayError,
    CoreJobState, CoreJobView, DerivedJobType, DerivedProcessingError, DerivedProcessingGateway,
    DerivedUploadComplete, DerivedUploadInit, DerivedUploadPart, HeartbeatReceipt, LogLevel,
    RuntimeDerivedPlanner, RuntimeSession, SubmitDerivedPayload, process_next_pending_job,
};

#[derive(Debug)]
struct SinglePendingGateway;

impl CoreApiGateway for SinglePendingGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError> {
        Ok(vec![CoreJobView {
            job_id: "job-1".to_string(),
            asset_uuid: "asset-1".to_string(),
            state: CoreJobState::Pending,
            required_capabilities: vec!["media.proxies.photo@1".to_string()],
        }])
    }
}

#[derive(Debug, Clone, Default)]
struct RecordingDerivedGateway {
    calls: Arc<Mutex<Vec<String>>>,
}

impl RecordingDerivedGateway {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("calls mutex").clone()
    }
}

impl DerivedProcessingGateway for RecordingDerivedGateway {
    fn claim_job(&self, job_id: &str) -> Result<ClaimedDerivedJob, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls mutex")
            .push(format!("claim:{job_id}"));
        Ok(ClaimedDerivedJob {
            job_id: job_id.to_string(),
            asset_uuid: "asset-1".to_string(),
            lock_token: "lock-1".to_string(),
            job_type: DerivedJobType::GenerateProxy,
            source_storage_id: "nas-main".to_string(),
            source_original_relative: "INBOX/asset.jpg".to_string(),
            source_sidecars_relative: Vec::new(),
        })
    }

    fn heartbeat(
        &self,
        job_id: &str,
        _lock_token: &str,
    ) -> Result<HeartbeatReceipt, DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls mutex")
            .push(format!("heartbeat:{job_id}"));
        Ok(HeartbeatReceipt { locked_until: None })
    }

    fn submit_derived(
        &self,
        job_id: &str,
        _lock_token: &str,
        _idempotency_key: &str,
        _payload: &SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        self.calls
            .lock()
            .expect("calls mutex")
            .push(format!("submit:{job_id}"));
        Ok(())
    }

    fn upload_init(&self, _request: &DerivedUploadInit) -> Result<(), DerivedProcessingError> {
        Ok(())
    }

    fn upload_part(&self, _request: &DerivedUploadPart) -> Result<(), DerivedProcessingError> {
        Ok(())
    }

    fn upload_complete(
        &self,
        _request: &DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        Ok(())
    }
}

#[test]
fn tdd_runtime_job_worker_processes_first_pending_job_with_source_staging() {
    let source_root = tempfile::tempdir().expect("source root");
    let source_path = source_root.path().join("INBOX/asset.jpg");
    std::fs::create_dir_all(source_path.parent().expect("parent")).expect("create dirs");
    std::fs::write(&source_path, b"fixture").expect("write source");

    let mut mounts = std::collections::BTreeMap::new();
    mounts.insert(
        "nas-main".to_string(),
        source_root.path().display().to_string(),
    );
    let settings = AgentRuntimeConfig {
        core_api_url: "http://localhost:3000/api/v1".to_string(),
        ollama_url: "http://localhost:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        storage_mounts: mounts,
        max_parallel_jobs: 1,
        log_level: LogLevel::Info,
    };
    let mut session =
        RuntimeSession::new(retaia_agent::ClientRuntimeTarget::Agent, settings).expect("session");
    let _ = session.on_poll_success(retaia_agent::PollEndpoint::Jobs, 5_000, true);

    let core = SinglePendingGateway;
    let derived = RecordingDerivedGateway::default();
    let planner = RuntimeDerivedPlanner;

    let report = process_next_pending_job(&session, &core, &derived, &planner)
        .expect("worker")
        .expect("job should be processed");
    assert_eq!(report.job_id, "job-1");
    assert_eq!(report.asset_uuid, "asset-1");
    let calls = derived.calls();
    assert_eq!(
        calls.first().map(std::string::String::as_str),
        Some("claim:job-1")
    );
    assert_eq!(
        calls.last().map(std::string::String::as_str),
        Some("submit:job-1")
    );
    assert!(
        calls
            .iter()
            .skip(1)
            .take(calls.len().saturating_sub(2))
            .all(|call| call == "heartbeat:job-1"),
        "expected only heartbeats between claim and submit, got: {calls:?}"
    );
    assert!(
        calls.iter().any(|call| call == "heartbeat:job-1"),
        "expected at least one heartbeat call, got: {calls:?}"
    );
}
