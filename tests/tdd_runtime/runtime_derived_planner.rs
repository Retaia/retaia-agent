use retaia_agent::{
    ClaimedDerivedJob, DerivedExecutionPlanner, DerivedJobType, DerivedKind, RuntimeDerivedPlanner,
};

#[test]
fn tdd_runtime_derived_planner_infers_audio_proxy_manifest_from_extension() {
    let planner = RuntimeDerivedPlanner;
    let claimed = ClaimedDerivedJob {
        job_id: "job-audio-1".to_string(),
        asset_uuid: "asset-audio-1".to_string(),
        lock_token: "lock-audio-1".to_string(),
        job_type: DerivedJobType::GenerateProxy,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/clip.mp3".to_string(),
        source_sidecars_relative: Vec::new(),
    };

    let plan = planner.plan_for_claimed_job(&claimed).expect("plan");
    assert_eq!(plan.submit.job_type, DerivedJobType::GenerateProxy);
    assert_eq!(plan.submit.manifest.len(), 1);
    assert_eq!(plan.submit.manifest[0].kind, DerivedKind::ProxyAudio);
    assert!(plan.uploads.is_empty());
    assert_eq!(plan.submit_idempotency_key, "agent-submit-job-audio-1");
}
