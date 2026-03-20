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
        fencing_token: 1,
        job_type: DerivedJobType::GeneratePreview,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/clip.mp3".to_string(),
        source_sidecars_relative: Vec::new(),
    };

    let plan = planner.plan_for_claimed_job(&claimed).expect("plan");
    assert_eq!(plan.submit.job_type, DerivedJobType::GeneratePreview);
    assert_eq!(plan.submit.manifest.len(), 1);
    assert_eq!(plan.submit.manifest[0].kind, DerivedKind::PreviewAudio);
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
        fencing_token: 1,
        job_type: DerivedJobType::GeneratePreview,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/clip.mov".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("clip.mov");
    std::fs::write(&staged, b"staged-bytes").expect("write");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[])
        .expect("plan");
    assert_eq!(plan.uploads.len(), 1);
    assert_eq!(plan.uploads[0].init.kind, DerivedKind::PreviewVideo);
    assert_eq!(plan.uploads[0].init.content_type, "video/mp4");
    assert_eq!(plan.uploads[0].parts.len(), 1);
    assert_eq!(plan.uploads[0].parts[0].part_number, 1);
    assert_eq!(plan.uploads[0].parts[0].chunk_path, staged);
    assert_eq!(plan.submit.manifest[0].size_bytes, Some(12));
}

#[test]
fn tdd_runtime_derived_planner_extract_facts_stays_uploadless_with_staged_source() {
    let planner = RuntimeDerivedPlanner;
    let claimed = ClaimedDerivedJob {
        job_id: "job-facts-1".to_string(),
        asset_uuid: "asset-facts-1".to_string(),
        lock_token: "lock-facts-1".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::ExtractFacts,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/clip.mov".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("clip.mov");
    std::fs::write(&staged, b"facts-source").expect("write");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[])
        .expect("plan");
    assert_eq!(plan.submit.job_type, DerivedJobType::ExtractFacts);
    assert!(plan.submit.manifest.is_empty());
    assert!(plan.uploads.is_empty());
}

#[test]
fn tdd_runtime_derived_planner_includes_sidecar_metrics_when_sidecars_are_staged() {
    let planner = RuntimeDerivedPlanner;
    let claimed = ClaimedDerivedJob {
        job_id: "job-facts-2".to_string(),
        asset_uuid: "asset-facts-2".to_string(),
        lock_token: "lock-facts-2".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::ExtractFacts,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/clip.mov".to_string(),
        source_sidecars_relative: vec![
            "INBOX/clip.xmp".to_string(),
            "INBOX/clip.srt".to_string(),
            "INBOX/clip.XMP".to_string(),
        ],
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("INBOX/clip.mov");
    let staged_xmp = dir.path().join("INBOX/clip.xmp");
    let staged_srt = dir.path().join("INBOX/clip.srt");
    let staged_xmp_upper = dir.path().join("INBOX/clip.XMP");
    std::fs::create_dir_all(staged.parent().expect("parent")).expect("mkdir");
    std::fs::write(&staged, b"source").expect("write source");
    std::fs::write(&staged_xmp, b"xmp-a").expect("write xmp");
    std::fs::write(&staged_srt, b"srt-a").expect("write srt");
    std::fs::write(&staged_xmp_upper, b"xmp-b").expect("write xmp upper");

    let sidecars = vec![staged_xmp, staged_srt, staged_xmp_upper];
    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &sidecars)
        .expect("plan");
    let metrics = plan.submit.metrics.expect("metrics must be set");
    assert_eq!(
        metrics.get("staged_sidecars_count"),
        Some(&serde_json::json!(3))
    );
    assert_eq!(
        metrics.get("staged_source_size_bytes"),
        Some(&serde_json::json!(6))
    );
    assert_eq!(
        metrics.get("staged_sidecars_total_bytes"),
        Some(&serde_json::json!(15))
    );
    assert_eq!(
        metrics
            .get("staged_sidecars_by_extension")
            .and_then(|value| value.get("xmp")),
        Some(&serde_json::json!(2))
    );
}
