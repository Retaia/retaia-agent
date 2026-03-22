use retaia_agent::{
    AudioProxyRequest, AudioWaveformRequest, ClaimedDerivedJob, DerivedExecutionPlanner,
    DerivedJobType, DerivedKind, FactsPatchPayload, PhotoProxyRequest, ProxyGenerationError,
    ProxyGenerator, RuntimeDerivedPlanner, VideoProxyRequest, VideoThumbnailRequest,
};
use std::sync::Arc;
use std::sync::Mutex;

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
            ..FactsPatchPayload::default()
        })
    }
}

#[derive(Debug)]
struct ThumbnailFactsGenerator {
    duration_ms: i32,
    thumbnail_requests: Mutex<Vec<VideoThumbnailRequest>>,
}

#[derive(Debug, Default)]
struct UnknownDurationThumbnailGenerator;

impl ProxyGenerator for UnknownDurationThumbnailGenerator {
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
            duration_ms: None,
            media_format: Some("mp4".to_string()),
            video_codec: Some("h264".to_string()),
            audio_codec: Some("aac".to_string()),
            width: Some(1920),
            height: Some(1080),
            fps: Some(25.0),
            ..FactsPatchPayload::default()
        })
    }
}

impl ThumbnailFactsGenerator {
    fn new(duration_ms: i32) -> Self {
        Self {
            duration_ms,
            thumbnail_requests: Mutex::new(Vec::new()),
        }
    }

    fn thumbnail_requests(&self) -> Vec<VideoThumbnailRequest> {
        self.thumbnail_requests
            .lock()
            .expect("thumbnail requests lock")
            .clone()
    }
}

impl ProxyGenerator for ThumbnailFactsGenerator {
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
        self.thumbnail_requests
            .lock()
            .expect("thumbnail requests lock")
            .push(request.clone());
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
            duration_ms: Some(self.duration_ms),
            media_format: Some("mp4".to_string()),
            video_codec: Some("h264".to_string()),
            audio_codec: Some("aac".to_string()),
            width: Some(1920),
            height: Some(1080),
            fps: Some(25.0),
            ..FactsPatchPayload::default()
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
    let generator = Arc::new(ThumbnailFactsGenerator::new(180_000));
    let planner = RuntimeDerivedPlanner::new(generator, Arc::new(WritingPreviewGenerator));
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

    assert_eq!(plan.uploads.len(), 9);
    assert_eq!(plan.uploads[0].init.kind, DerivedKind::Thumb);
    assert_eq!(plan.uploads[0].init.content_type, "image/webp");
    assert!(
        plan.uploads[0].parts[0]
            .chunk_path
            .ends_with("clip.thumb.1.webp")
    );
    assert_eq!(
        plan.submit.manifest[0].reference,
        "/api/v1/assets/asset-thumb-1/derived/thumbs/1"
    );
    let metrics = plan.submit.metrics.expect("thumbnail metrics");
    assert_eq!(
        metrics.get("thumbnail_profile"),
        Some(&serde_json::json!("video_storyboard_v1"))
    );
    assert_eq!(metrics.get("thumbnail_count"), Some(&serde_json::json!(9)));
}

#[test]
fn tdd_runtime_derived_planner_uses_short_video_representative_seek() {
    let generator = Arc::new(ThumbnailFactsGenerator::new(90_000));
    let planner = RuntimeDerivedPlanner::new(generator.clone(), Arc::new(WritingPreviewGenerator));
    let claimed = ClaimedDerivedJob {
        job_id: "job-thumb-short".to_string(),
        asset_uuid: "asset-thumb-short".to_string(),
        lock_token: "lock-thumb-short".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::GenerateThumbnails,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/short.mov".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("short.mov");
    std::fs::write(&staged, b"generated-video").expect("write");

    planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[])
        .expect("plan");

    let requests = generator.thumbnail_requests();
    assert_eq!(requests.len(), 9);
    assert_eq!(requests.first().expect("first").seek_ms, 9_000);
    assert_eq!(requests.last().expect("last").seek_ms, 81_000);
}

#[test]
fn tdd_runtime_derived_planner_uses_long_video_storyboard_distribution() {
    let generator = Arc::new(ThumbnailFactsGenerator::new(600_000));
    let planner = RuntimeDerivedPlanner::new(generator.clone(), Arc::new(WritingPreviewGenerator));
    let claimed = ClaimedDerivedJob {
        job_id: "job-thumb-long".to_string(),
        asset_uuid: "asset-thumb-long".to_string(),
        lock_token: "lock-thumb-long".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::GenerateThumbnails,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/long.mov".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("long.mov");
    std::fs::write(&staged, b"generated-video").expect("write");

    planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[])
        .expect("plan");

    let requests = generator.thumbnail_requests();
    assert_eq!(requests.len(), 9);
    assert_eq!(requests.first().expect("first").seek_ms, 60_000);
    assert_eq!(requests.last().expect("last").seek_ms, 540_000);
}

#[test]
fn tdd_runtime_derived_planner_falls_back_to_single_representative_thumb_when_duration_is_unknown()
{
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(UnknownDurationThumbnailGenerator),
        Arc::new(WritingPreviewGenerator),
    );
    let claimed = ClaimedDerivedJob {
        job_id: "job-thumb-unknown".to_string(),
        asset_uuid: "asset-thumb-unknown".to_string(),
        lock_token: "lock-thumb-unknown".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::GenerateThumbnails,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/unknown.mov".to_string(),
        source_sidecars_relative: Vec::new(),
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("unknown.mov");
    std::fs::write(&staged, b"generated-video").expect("write");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[])
        .expect("plan");

    assert_eq!(plan.uploads.len(), 1);
    assert_eq!(plan.submit.manifest.len(), 1);
    assert_eq!(
        plan.submit.manifest[0].reference,
        "/api/v1/assets/asset-thumb-unknown/derived/thumb"
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

#[test]
fn tdd_runtime_derived_planner_extract_facts_merges_dji_srt_sidecar_fields() {
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );
    let claimed = ClaimedDerivedJob {
        job_id: "job-facts-srt-1".to_string(),
        asset_uuid: "asset-facts-srt-1".to_string(),
        lock_token: "lock-facts-srt-1".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::ExtractFacts,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/drone.mp4".to_string(),
        source_sidecars_relative: vec!["INBOX/drone.srt".to_string()],
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("INBOX/drone.mp4");
    let staged_srt = dir.path().join("INBOX/drone.srt");
    std::fs::create_dir_all(staged.parent().expect("parent")).expect("mkdir");
    std::fs::write(&staged, b"facts-source").expect("write source");
    std::fs::write(
        &staged_srt,
        br#"1
00:00:00,000 --> 00:00:00,040
[iso: 100] [shutter: 1/2500.0] [fnum: 1.7] [ev: 0] [focal_len: 24.00] [ct: 5200] [color_md: dlog_m]
[latitude: 50.1234] [longitude: 4.5678] [rel_alt: 12.3 m] [abs_alt: 123.4 m]
"#,
    )
    .expect("write srt");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[staged_srt])
        .expect("plan");
    let facts = plan.submit.facts_patch.expect("facts patch");

    assert_eq!(facts.iso, Some(100));
    assert_eq!(facts.exposure_time_s, Some(1.0 / 2500.0));
    assert_eq!(facts.aperture_f_number, Some(1.7));
    assert_eq!(facts.focal_length_mm, Some(24.0));
    assert_eq!(facts.exposure_compensation_ev, Some(0.0));
    assert_eq!(facts.color_mode.as_deref(), Some("dlog_m"));
    assert_eq!(facts.color_temperature_k, Some(5200));
    assert_eq!(facts.gps_latitude, Some(50.1234));
    assert_eq!(facts.gps_longitude, Some(4.5678));
    assert_eq!(facts.gps_altitude_relative_m, Some(12.3));
    assert_eq!(facts.gps_altitude_absolute_m, Some(123.4));
}

#[test]
fn tdd_runtime_derived_planner_extract_facts_uses_first_dji_srt_values() {
    let planner = RuntimeDerivedPlanner::new(
        Arc::new(WritingPreviewGenerator),
        Arc::new(WritingPreviewGenerator),
    );
    let claimed = ClaimedDerivedJob {
        job_id: "job-facts-srt-2".to_string(),
        asset_uuid: "asset-facts-srt-2".to_string(),
        lock_token: "lock-facts-srt-2".to_string(),
        fencing_token: 1,
        job_type: DerivedJobType::ExtractFacts,
        source_storage_id: "nas-main".to_string(),
        source_original_relative: "INBOX/drone.mp4".to_string(),
        source_sidecars_relative: vec!["INBOX/drone.srt".to_string()],
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let staged = dir.path().join("INBOX/drone.mp4");
    let staged_srt = dir.path().join("INBOX/drone.srt");
    std::fs::create_dir_all(staged.parent().expect("parent")).expect("mkdir");
    std::fs::write(&staged, b"facts-source").expect("write source");
    std::fs::write(
        &staged_srt,
        br#"1
00:00:00,000 --> 00:00:00,040
[iso: 100] [shutter: 1/2500.0] [latitude: 50.1000] [longitude: 4.1000]

2
00:00:00,040 --> 00:00:00,080
[iso: 200] [shutter: 1/1000.0] [latitude: 51.2000] [longitude: 5.2000]
"#,
    )
    .expect("write srt");

    let plan = planner
        .plan_for_claimed_job_with_source(&claimed, Some(staged.as_path()), &[staged_srt])
        .expect("plan");
    let facts = plan.submit.facts_patch.expect("facts patch");

    assert_eq!(facts.iso, Some(100));
    assert_eq!(facts.exposure_time_s, Some(1.0 / 2500.0));
    assert_eq!(facts.gps_latitude, Some(50.1));
    assert_eq!(facts.gps_longitude, Some(4.1));
}
