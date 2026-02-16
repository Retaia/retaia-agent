use retaia_agent::{
    CoreApiGateway, CoreApiGatewayError, CoreJobState, CoreJobView, poll_runtime_snapshot,
};

struct StubGateway {
    jobs: Vec<CoreJobView>,
}

impl CoreApiGateway for StubGateway {
    fn poll_jobs(&self) -> Result<Vec<CoreJobView>, CoreApiGatewayError> {
        Ok(self.jobs.clone())
    }
}

#[test]
fn bdd_given_polled_jobs_when_building_runtime_snapshot_then_running_and_current_job_follow_claimed_jobs()
 {
    let gateway = StubGateway {
        jobs: vec![
            CoreJobView {
                job_id: "job-pending".to_string(),
                asset_uuid: "asset-1".to_string(),
                state: CoreJobState::Pending,
            },
            CoreJobView {
                job_id: "job-claimed".to_string(),
                asset_uuid: "asset-2".to_string(),
                state: CoreJobState::Claimed,
            },
        ],
    };

    let snapshot = poll_runtime_snapshot(&gateway).expect("poll should succeed");
    assert_eq!(snapshot.running_job_ids.len(), 1);
    assert!(snapshot.running_job_ids.contains("job-claimed"));
    assert_eq!(
        snapshot.current_job.as_ref().map(|job| job.job_id.as_str()),
        Some("job-claimed")
    );
}
