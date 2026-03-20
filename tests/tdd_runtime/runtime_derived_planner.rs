use retaia_agent::{
    AudioProxyRequest, AudioWaveformRequest, ClaimedDerivedJob, DerivedExecutionPlanner,
    DerivedJobType, DerivedKind, FactsPatchPayload, PhotoProxyRequest, ProxyGenerationError,
    ProxyGenerator, RuntimeDerivedPlanner, VideoProxyRequest, VideoThumbnailRequest,
};
use std::sync::Arc;

#[derive(Debug, Default)]
struct WritingPreviewGenerator;

impl ProxyGenerator for WritingPreviewGenerator {
    fn generate_video_proxy(
        &self,
        request: &VideoProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        std::fs::write(&request.output_path, b"generated-video")
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))
    }

    fn generate_audio_proxy(
        &self,
        request: &AudioProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        std::fs::write(&request.output_path, b"generated-audio")
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))
    }

    fn generate_photo_proxy(
        &self,
        request: &PhotoProxyRequest,
    ) -> Result<(), ProxyGenerationError> {
        std::fs::write(&request.output_path, b"generated-photo")
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))
    }

    fn generate_video_thumbnail(
        &self,
        request: &VideoThumbnailRequest,
    ) -> Result<(), ProxyGenerationError> {
        std::fs::write(&request.output_path, b"generated-thumbnail")
            .map_err(|error| ProxyGenerationError::Process(error.to_string()))
    }

    fn generate_audio_waveform(
        &self,
        request: &AudioWaveformRequest,
    ) -> Result<(), ProxyGenerationError> {
        std::fs::write(
            &request.output_path,
            br#"{"duration_ms":1000,"bucket_count":1000,"samples":[0.1,0.5]}"#,
        )
        .map_err(|error| ProxyGenerationError::Process(error.to_string()))
    }

    fn extract_media_facts(
        &self,
        _input_path: &str,
    ) -> Result<FactsPatchPayload, ProxyGenerationError> {
        Ok(FactsPatchPayload {
            duration_ms: Some(2_000),
            media_format: Some("mp4".to_string()),
            video_codec: Some("h264".to_string()),
            audio_codec: Some("aac".to_string()),
            width: Some(1920),
            height: Some(1080),
            fps: Some(25.0),
        })
    }
}

#[test]
fn tdd_runtime_derived_planner_infers_audio_proxy_manifest_from_extension() {
    let planner = RuntimeDerivedPlanner::default();
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
    assert_eq!(
        plan.submit.manifest[0].reference,
        "/api/v1/assets/asset-audio-1/derived/preview_audio"
    );
    assert!(plan.uploads.is_empty());
    assert_eq!(plan.submit_idempotency_key, "agent-submit-job-audio-1");
    let metrics = plan.submit.metrics.expect("preview metrics");
    assert_eq!(
        metrics.get("preview_kind"),
        Some(&serde_json::json!("preview_audio"))
    );
    assert_eq!(
        metrics.get("preview_profile"),
        Some(&serde_json::json!("audio_review_default_v1"))
    );
}

#[test]
fn tdd_runtime_derived_planner_with_staged_source_builds_upload_plan() {
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );
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
    assert_ne!(plan.uploads[0].parts[0].chunk_path, staged);
    assert!(
        plan.uploads[0].parts[0]
            .chunk_path
            .ends_with("clip.preview_video.mp4")
    );
    assert_eq!(
        plan.submit.manifest[0].reference,
        "/api/v1/assets/asset-video-1/derived/preview_video"
    );
    assert_eq!(plan.submit.manifest[0].size_bytes, Some(15));
    let metrics = plan.submit.metrics.expect("preview metrics");
    assert_eq!(
        metrics.get("preview_kind"),
        Some(&serde_json::json!("preview_video"))
    );
    assert_eq!(
        metrics.get("preview_profile"),
        Some(&serde_json::json!("video_review_default_v1"))
    );
}

#[test]
fn tdd_runtime_derived_planner_builds_photo_preview_upload_as_webp() {
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );
    let claimed = ClaimedDerivedJob {
        job_id: "job-photo-1".to_string(),
        asset_uuid: "asset-photo-1".to_string(),
        lock_token: "lock-photo-1".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::GeneratePreview,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/frame.jpg".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("frame.jpg");
    std::fs::write(&staged, b"generated-photo").expect("write");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[])
        .expect("plan");

    assert_eq!(plan.uploads.len(), 1);
    assert_eq!(plan.uploads[0].init.kind, DerivedKind::PreviewPhoto);
    assert_eq!(plan.uploads[0].init.content_type, "image/webp");
    assert!(
        plan.uploads[0].parts[0]
            .chunk_path
            .ends_with("frame.preview_photo.webp")
    );
    assert_eq!(
        plan.submit.manifest[0].reference,
        "/api/v1/assets/asset-photo-1/derived/preview_photo"
    );
    let metrics = plan.submit.metrics.expect("preview metrics");
    assert_eq!(
        metrics.get("preview_profile"),
        Some(&serde_json::json!("photo_review_default_v1"))
    );
}

#[test]
fn tdd_runtime_derived_planner_builds_thumbnail_upload_as_webp() {
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );
    let claimed = ClaimedDerivedJob {
        job_id: "job-thumb-1".to_string(),
        asset_uuid: "asset-thumb-1".to_string(),
        lock_token: "lock-thumb-1".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::GenerateThumbnails,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/clip.mov".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("clip.mov");
    std::fs::write(&staged, b"generated-video").expect("write");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[])
        .expect("plan");

    assert_eq!(plan.uploads.len(), 1);
    assert_eq!(plan.uploads[0].init.kind, DerivedKind::Thumb);
    assert_eq!(plan.uploads[0].init.content_type, "image/webp");
    assert!(
        plan.uploads[0].parts[0]
            .chunk_path
            .ends_with("clip.thumb.webp")
    );
    assert_eq!(
        plan.submit.manifest[0].reference,
        "/api/v1/assets/asset-thumb-1/derived/thumb"
    );
    let metrics = plan.submit.metrics.expect("thumbnail metrics");
    assert_eq!(
        metrics.get("thumbnail_profile"),
        Some(&serde_json::json!("video_representative_v1"))
    );
    assert_eq!(metrics.get("thumbnail_count"), Some(&serde_json::json!(1)));
}

#[test]
fn tdd_runtime_derived_planner_builds_waveform_upload_as_json() {
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );
    let claimed = ClaimedDerivedJob {
        job_id: "job-wave-1".to_string(),
        asset_uuid: "asset-wave-1".to_string(),
        lock_token: "lock-wave-1".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::GenerateAudioWaveform,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/audio.wav".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("audio.wav");
    std::fs::write(&staged, b"audio-source").expect("write");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[])
        .expect("plan");

    assert_eq!(plan.uploads.len(), 1);
    assert_eq!(plan.uploads[0].init.kind, DerivedKind::Waveform);
    assert_eq!(plan.uploads[0].init.content_type, "application/json");
    assert!(
        plan.uploads[0].parts[0]
            .chunk_path
            .ends_with("audio.waveform.json")
    );
    assert_eq!(
        plan.submit.manifest[0].reference,
        "/api/v1/assets/asset-wave-1/derived/waveform"
    );
    let metrics = plan.submit.metrics.expect("waveform metrics");
    assert_eq!(
        metrics.get("waveform_bucket_count"),
        Some(&serde_json::json!(1000))
    );
    assert_eq!(
        metrics.get("waveform_format"),
        Some(&serde_json::json!("json"))
    );
}

#[test]
fn tdd_runtime_derived_planner_extract_facts_populates_facts_patch_without_uploads() {
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );
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
    let facts = plan.submit.facts_patch.expect("facts patch");
    assert_eq!(facts.duration_ms, Some(2_000));
    assert_eq!(facts.media_format.as_deref(), Some("mp4"));
    assert_eq!(facts.video_codec.as_deref(), Some("h264"));
    assert_eq!(facts.audio_codec.as_deref(), Some("aac"));
    assert_eq!(facts.width, Some(1920));
    assert_eq!(facts.height, Some(1080));
    assert_eq!(facts.fps, Some(25.0));
}

#[test]
fn tdd_runtime_derived_planner_includes_sidecar_metrics_when_sidecars_are_staged() {
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );
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
