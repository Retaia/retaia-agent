use retaia_agent::{
    AgentUiRuntime, CoreApiGateway, CoreApiGatewayError, CoreJobState, CoreJobView,
    dispatch_notifications, poll_runtime_snapshot,
};

struct MemoryGateway {
    jobs: Vec<CoreJobView>,
}

impl CoreApiGateway for MemoryGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError> {
        Ok(self.jobs.clone())
    }
}

#[derive(Default)]
struct NopSink;

impl retaia_agent::NotificationSink for NopSink {
    fn send(
        &self,
        _message: &retaia_agent::NotificationMessage,
        _source: &retaia_agent::SystemNotification,
    ) -> Result<(), retaia_agent::NotificationBridgeError> {
        Ok(())
    }
}

#[test]
fn e2e_polled_jobs_gateway_projection_triggers_new_job_and_all_jobs_done_notifications_once() {
    let mut runtime = AgentUiRuntime::new();
    let sink = NopSink;

    let first_gateway = MemoryGateway {
        jobs: vec![CoreJobView {
            job_id: "job-10".to_string(),
            asset_uuid: "asset-10".to_string(),
            state: CoreJobState::Claimed,
            required_capabilities: vec!["media.facts@1".to_string()],
        }],
    };
    let first_snapshot = poll_runtime_snapshot(&first_gateway).expect("poll should succeed");
    let first_notifications = runtime.update_snapshot(first_snapshot);
    let first_report = dispatch_notifications(&sink, &first_notifications);
    assert_eq!(first_report.delivered, 1);

    let second_gateway = MemoryGateway {
        jobs: vec![CoreJobView {
            job_id: "job-10".to_string(),
            asset_uuid: "asset-10".to_string(),
            state: CoreJobState::Claimed,
            required_capabilities: vec!["media.facts@1".to_string()],
        }],
    };
    let second_snapshot = poll_runtime_snapshot(&second_gateway).expect("poll should succeed");
    let second_notifications = runtime.update_snapshot(second_snapshot);
    let second_report = dispatch_notifications(&sink, &second_notifications);
    assert_eq!(second_report.delivered, 0);

    let done_gateway = MemoryGateway { jobs: Vec::new() };
    let done_snapshot = poll_runtime_snapshot(&done_gateway).expect("poll should succeed");
    let done_notifications = runtime.update_snapshot(done_snapshot);
    let done_report = dispatch_notifications(&sink, &done_notifications);
    assert_eq!(done_report.delivered, 1);
}

#[test]
fn e2e_polled_jobs_gateway_projection_ignores_claimed_job_without_declared_capability() {
    let gateway = MemoryGateway {
        jobs: vec![CoreJobView {
            job_id: "job-unsupported".to_string(),
            asset_uuid: "asset-unsupported".to_string(),
            state: CoreJobState::Claimed,
            required_capabilities: vec!["media.unknown@1".to_string()],
        }],
    };

    let snapshot = poll_runtime_snapshot(&gateway).expect("poll should succeed");
    assert!(!snapshot.running_job_ids.contains("job-unsupported"));
    assert!(snapshot.current_job.is_none());
}
