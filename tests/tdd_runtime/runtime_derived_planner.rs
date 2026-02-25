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

#[test]
fn tdd_runtime_derived_planner_with_staged_source_builds_upload_plan() {
    let planner = RuntimeDerivedPlanner;
    let claimed = ClaimedDerivedJob {
        job_id: "job-video-1".to_string(),
        asset_uuid: "asset-video-1".to_string(),
        lock_token: "lock-video-1".to_string(),
        job_type: DerivedJobType::GenerateProxy,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/clip.mov".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("clip.mov");
    std::fs::write(&staged, b"staged-bytes").expect("write");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()))
        .expect("plan");
    assert_eq!(plan.uploads.len(), 1);
    assert_eq!(plan.uploads[0].init.kind, DerivedKind::ProxyVideo);
    assert_eq!(plan.uploads[0].init.content_type, "video/mp4");
    assert_eq!(plan.uploads[0].parts.len(), 1);
    assert_eq!(plan.uploads[0].parts[0].part_number, 1);
    assert_eq!(plan.submit.manifest[0].size_bytes, Some(12));
}
