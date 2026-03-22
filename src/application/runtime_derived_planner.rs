use crate::application::derived_job_executor::{
    DerivedExecutionPlan, DerivedExecutionPlanner, DerivedJobExecutorError, DerivedUploadPlan,
};
use crate::application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedJobType, DerivedKind, DerivedManifestItem, DerivedUploadComplete,
    DerivedUploadInit, DerivedUploadPart, FactsPatchPayload, SubmitDerivedPayload,
};
use crate::application::proxy_generator::{
    AudioProxyFormat, AudioProxyRequest, AudioWaveformRequest, PhotoProxyFormat, PhotoProxyRequest,
    ProxyGenerationError, ProxyGenerator, ThumbnailFormat, VideoProxyRequest,
    VideoThumbnailRequest,
};
use crate::domain::capabilities::photo_source_extension_supported;
use crate::infrastructure::ffmpeg_proxy_generator::FfmpegProxyGenerator;
use crate::infrastructure::rust_photo_proxy_generator::RustPhotoProxyGenerator;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct RuntimeDerivedPlanner {
    av_generator: Arc<dyn ProxyGenerator>,
    photo_generator: Arc<dyn ProxyGenerator>,
}

impl std::fmt::Debug for RuntimeDerivedPlanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeDerivedPlanner").finish()
    }
}

impl Default for RuntimeDerivedPlanner {
    fn default() -> Self {
        Self {
            av_generator: Arc::new(FfmpegProxyGenerator::default()),
            photo_generator: Arc::new(RustPhotoProxyGenerator::default()),
        }
    }
}

impl RuntimeDerivedPlanner {
    pub fn new(
        av_generator: Arc<dyn ProxyGenerator>,
        photo_generator: Arc<dyn ProxyGenerator>,
    ) -> Self {
        Self {
            av_generator,
            photo_generator,
        }
    }
}

impl DerivedExecutionPlanner for RuntimeDerivedPlanner {
    fn plan_for_claimed_job(
        &self,
        claimed: &ClaimedDerivedJob,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        let manifest = default_manifest_for_job(claimed);
        Ok(DerivedExecutionPlan {
            uploads: Vec::new(),
            submit: SubmitDerivedPayload {
                job_type: claimed.job_type,
                manifest,
                facts_patch: None,
                transcript_patch: None,
                warnings: None,
                metrics: base_metrics_for_job(claimed),
            },
            submit_idempotency_key: format!("agent-submit-{}", claimed.job_id),
        })
    }

    fn plan_for_claimed_job_with_source(
        &self,
        claimed: &ClaimedDerivedJob,
        staged_source_path: Option<&Path>,
        staged_sidecar_paths: &[PathBuf],
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        let mut plan = self.plan_for_claimed_job(claimed)?;
        merge_metrics(
            &mut plan.submit.metrics,
            sidecar_metrics(staged_source_path, staged_sidecar_paths)?,
        );
        let Some(source_path) = staged_source_path else {
            return Ok(plan);
        };
        if claimed.job_type == DerivedJobType::ExtractFacts {
            plan.submit.facts_patch = Some(self.extract_facts(source_path, claimed)?);
            return Ok(plan);
        }
        if claimed.job_type == DerivedJobType::GenerateThumbnails {
            let thumbnail_artifacts = self.generate_thumbnail_artifacts(source_path)?;
            plan.uploads = thumbnail_uploads_for_claimed_job(claimed, &thumbnail_artifacts)?;
            plan.submit.manifest =
                thumbnail_manifest_for_claimed_job(claimed, &thumbnail_artifacts);
            merge_metrics(
                &mut plan.submit.metrics,
                Some(thumbnail_metrics(
                    thumbnail_artifacts.profile,
                    thumbnail_artifacts.files.len(),
                )),
            );
            return Ok(plan);
        }

        let upload_kind = plan
            .submit
            .manifest
            .first()
            .map(|item| item.kind)
            .unwrap_or_else(|| infer_preview_kind(claimed));
        let generated_path = match claimed.job_type {
            DerivedJobType::GeneratePreview => {
                self.generate_preview_artifact(source_path, upload_kind)?
            }
            DerivedJobType::GenerateThumbnails => unreachable!("handled above"),
            DerivedJobType::GenerateAudioWaveform => {
                self.generate_waveform_artifact(source_path)?
            }
            DerivedJobType::ExtractFacts => source_path.to_path_buf(),
            DerivedJobType::TranscribeAudio => {
                return Err(DerivedJobExecutorError::Planner(
                    "transcribe_audio is not implemented yet".to_string(),
                ));
            }
        };

        let metadata = std::fs::metadata(&generated_path)
            .map_err(|error| DerivedJobExecutorError::Planner(error.to_string()))?;
        let size_bytes = metadata.len();
        let upload_id = format!(
            "upload-{}-{}",
            claimed.asset_uuid,
            upload_kind.as_str().replace('_', "-")
        );

        plan.uploads = vec![DerivedUploadPlan {
            init: DerivedUploadInit {
                asset_uuid: claimed.asset_uuid.clone(),
                revision_etag: String::new(),
                kind: upload_kind,
                content_type: content_type_for_kind(upload_kind).to_string(),
                size_bytes,
                sha256: None,
                idempotency_key: format!("init-{}-{}", claimed.job_id, upload_kind.as_str()),
            },
            parts: vec![DerivedUploadPart {
                asset_uuid: claimed.asset_uuid.clone(),
                revision_etag: String::new(),
                upload_id: upload_id.clone(),
                part_number: 1,
                chunk_path: generated_path.clone(),
            }],
            complete: DerivedUploadComplete {
                asset_uuid: claimed.asset_uuid.clone(),
                revision_etag: String::new(),
                upload_id,
                idempotency_key: format!("complete-{}-{}", claimed.job_id, upload_kind.as_str()),
                parts: None,
            },
        }];

        if let Some(first) = plan.submit.manifest.first_mut() {
            first.size_bytes = Some(size_bytes);
        }
        Ok(plan)
    }
}

fn default_manifest_for_job(claimed: &ClaimedDerivedJob) -> Vec<DerivedManifestItem> {
    match claimed.job_type {
        DerivedJobType::ExtractFacts => Vec::new(),
        DerivedJobType::GeneratePreview => {
            vec![manifest_item_for_kind(claimed, infer_preview_kind(claimed))]
        }
        DerivedJobType::GenerateThumbnails => {
            vec![manifest_item_for_kind(claimed, DerivedKind::Thumb)]
        }
        DerivedJobType::GenerateAudioWaveform => {
            vec![manifest_item_for_kind(claimed, DerivedKind::Waveform)]
        }
        DerivedJobType::TranscribeAudio => Vec::new(),
    }
}

fn manifest_item_for_kind(claimed: &ClaimedDerivedJob, kind: DerivedKind) -> DerivedManifestItem {
    DerivedManifestItem {
        kind,
        reference: stable_core_derived_reference(&claimed.asset_uuid, kind),
        size_bytes: None,
        sha256: None,
    }
}

fn stable_core_derived_reference(asset_uuid: &str, kind: DerivedKind) -> String {
    format!("/api/v1/assets/{asset_uuid}/derived/{}", kind.as_str())
}

fn infer_preview_kind(claimed: &ClaimedDerivedJob) -> DerivedKind {
    let extension = claimed
        .source_original_relative
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if photo_source_extension_supported(&extension) {
        return DerivedKind::PreviewPhoto;
    }
    if is_audio_extension(&extension) {
        return DerivedKind::PreviewAudio;
    }
    DerivedKind::PreviewVideo
}

impl RuntimeDerivedPlanner {
    fn generate_preview_artifact(
        &self,
        source_path: &Path,
        kind: DerivedKind,
    ) -> Result<PathBuf, DerivedJobExecutorError> {
        let output_path = generated_preview_output_path(source_path, kind);
        let input_path = source_path.to_string_lossy().to_string();

        let result = match kind {
            DerivedKind::PreviewVideo => {
                self.av_generator
                    .generate_video_proxy(&canonical_video_preview_request(
                        input_path,
                        output_path.to_string_lossy().to_string(),
                    ))
            }
            DerivedKind::PreviewAudio => {
                self.av_generator
                    .generate_audio_proxy(&canonical_audio_preview_request(
                        input_path,
                        output_path.to_string_lossy().to_string(),
                    ))
            }
            DerivedKind::PreviewPhoto => {
                self.photo_generator
                    .generate_photo_proxy(&canonical_photo_preview_request(
                        input_path,
                        output_path.to_string_lossy().to_string(),
                    ))
            }
            DerivedKind::Thumb | DerivedKind::Waveform => Ok(()),
        };

        result.map_err(map_preview_generation_error)?;
        Ok(output_path)
    }

    fn generate_thumbnail_artifacts(
        &self,
        source_path: &Path,
    ) -> Result<GeneratedThumbnailArtifacts, DerivedJobExecutorError> {
        let duration_ms = self
            .av_generator
            .extract_media_facts(&source_path.to_string_lossy())
            .ok()
            .and_then(|facts| facts.duration_ms)
            .and_then(|value| u64::try_from(value).ok());

        let (profile, seek_points) = storyboard_plan_for_duration(duration_ms);
        let mut files = Vec::with_capacity(seek_points.len());
        for (index, seek_ms) in seek_points.iter().enumerate() {
            let output_path = generated_thumb_output_path(source_path, index);
            self.av_generator
                .generate_video_thumbnail(&canonical_thumbnail_request(
                    source_path.to_string_lossy().to_string(),
                    output_path.to_string_lossy().to_string(),
                    *seek_ms,
                ))
                .map_err(map_preview_generation_error)?;
            files.push(output_path);
        }

        Ok(GeneratedThumbnailArtifacts { profile, files })
    }

    fn generate_waveform_artifact(
        &self,
        source_path: &Path,
    ) -> Result<PathBuf, DerivedJobExecutorError> {
        let output_path = generated_preview_output_path(source_path, DerivedKind::Waveform);
        self.av_generator
            .generate_audio_waveform(&canonical_waveform_request(
                source_path.to_string_lossy().to_string(),
                output_path.to_string_lossy().to_string(),
            ))
            .map_err(map_preview_generation_error)?;
        Ok(output_path)
    }

    fn extract_facts(
        &self,
        source_path: &Path,
        claimed: &ClaimedDerivedJob,
    ) -> Result<FactsPatchPayload, DerivedJobExecutorError> {
        let generator: &Arc<dyn ProxyGenerator> =
            if infer_preview_kind(claimed) == DerivedKind::PreviewPhoto {
                &self.photo_generator
            } else {
                &self.av_generator
            };
        generator
            .extract_media_facts(&source_path.to_string_lossy())
            .map_err(map_preview_generation_error)
    }
}

fn generated_preview_output_path(source_path: &Path, kind: DerivedKind) -> PathBuf {
    let parent = source_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = source_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("derived");
    let extension = match kind {
        DerivedKind::PreviewVideo => "mp4",
        DerivedKind::PreviewAudio => "m4a",
        DerivedKind::PreviewPhoto => "webp",
        DerivedKind::Thumb => "webp",
        DerivedKind::Waveform => "json",
    };
    parent.join(format!("{stem}.{}.{}", kind.as_str(), extension))
}

fn generated_thumb_output_path(source_path: &Path, index: usize) -> PathBuf {
    let parent = source_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = source_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("derived");
    parent.join(format!("{stem}.thumb.{}.webp", index + 1))
}

fn canonical_video_preview_request(input_path: String, output_path: String) -> VideoProxyRequest {
    VideoProxyRequest {
        input_path,
        output_path,
        max_width: 1280,
        max_height: 720,
        video_bitrate_kbps: 2_500,
        audio_bitrate_kbps: 128,
    }
}

fn canonical_audio_preview_request(input_path: String, output_path: String) -> AudioProxyRequest {
    AudioProxyRequest {
        input_path,
        output_path,
        format: AudioProxyFormat::Mp4Aac,
        audio_bitrate_kbps: 128,
        sample_rate_hz: 48_000,
    }
}

fn canonical_photo_preview_request(input_path: String, output_path: String) -> PhotoProxyRequest {
    PhotoProxyRequest {
        input_path,
        output_path,
        format: PhotoProxyFormat::Webp,
        max_width: 1920,
        max_height: 1920,
    }
}

fn canonical_thumbnail_request(
    input_path: String,
    output_path: String,
    seek_ms: u64,
) -> VideoThumbnailRequest {
    VideoThumbnailRequest {
        input_path,
        output_path,
        format: ThumbnailFormat::Webp,
        max_width: 480,
        seek_ms,
    }
}

fn representative_thumbnail_seek_ms(duration_ms: Option<u64>) -> u64 {
    const SHORT_VIDEO_THRESHOLD_MS: u64 = 120_000;
    const MIN_REPRESENTATIVE_SEEK_MS: u64 = 1_000;
    const LONG_VIDEO_MAX_SEEK_MS: u64 = 20_000;

    let Some(duration_ms) = duration_ms else {
        return MIN_REPRESENTATIVE_SEEK_MS;
    };

    if duration_ms < SHORT_VIDEO_THRESHOLD_MS {
        return (duration_ms / 10).max(MIN_REPRESENTATIVE_SEEK_MS);
    }

    ((duration_ms * 5) / 100)
        .min(LONG_VIDEO_MAX_SEEK_MS)
        .max(MIN_REPRESENTATIVE_SEEK_MS)
}

fn storyboard_seek_points_ms(duration_ms: u64, frame_count: usize) -> Vec<u64> {
    (0..frame_count)
        .map(|index| (((index + 1) as u64) * duration_ms) / ((frame_count + 1) as u64))
        .map(|seek_ms| seek_ms.max(1_000))
        .collect()
}

fn storyboard_plan_for_duration(duration_ms: Option<u64>) -> (&'static str, Vec<u64>) {
    const STORYBOARD_FRAME_COUNT: usize = 9;

    match duration_ms {
        Some(duration_ms) if duration_ms > 0 => (
            "video_storyboard_v1",
            storyboard_seek_points_ms(duration_ms, STORYBOARD_FRAME_COUNT),
        ),
        _ => (
            "video_representative_v1",
            vec![representative_thumbnail_seek_ms(duration_ms)],
        ),
    }
}

fn canonical_waveform_request(input_path: String, output_path: String) -> AudioWaveformRequest {
    AudioWaveformRequest {
        input_path,
        output_path,
        bucket_count: 1_000,
    }
}

fn map_preview_generation_error(error: ProxyGenerationError) -> DerivedJobExecutorError {
    DerivedJobExecutorError::Planner(format!("preview generation failed: {error}"))
}

fn is_audio_extension(extension: &str) -> bool {
    matches!(
        extension,
        "aac" | "aif" | "aiff" | "alac" | "flac" | "m4a" | "mp3" | "ogg" | "opus" | "wav"
    )
}

fn content_type_for_kind(kind: DerivedKind) -> &'static str {
    match kind {
        DerivedKind::PreviewVideo => "video/mp4",
        DerivedKind::PreviewAudio => "audio/mp4",
        DerivedKind::PreviewPhoto => "image/webp",
        DerivedKind::Thumb => "image/webp",
        DerivedKind::Waveform => "application/json",
    }
}

fn base_metrics_for_job(claimed: &ClaimedDerivedJob) -> Option<HashMap<String, Value>> {
    let mut metrics = HashMap::new();
    if claimed.job_type == DerivedJobType::GeneratePreview {
        let kind = infer_preview_kind(claimed);
        metrics.insert(
            "preview_kind".to_string(),
            Value::from(kind.as_str().to_string()),
        );
        metrics.insert(
            "preview_profile".to_string(),
            Value::from(canonical_preview_profile_for_kind(kind)),
        );
    } else if claimed.job_type == DerivedJobType::GenerateThumbnails {
        metrics.extend(thumbnail_metrics("video_representative_v1", 1));
    } else if claimed.job_type == DerivedJobType::GenerateAudioWaveform {
        metrics.insert("waveform_bucket_count".to_string(), Value::from(1_000_u64));
        metrics.insert("waveform_format".to_string(), Value::from("json"));
    }

    if metrics.is_empty() {
        None
    } else {
        Some(metrics)
    }
}

fn thumbnail_metrics(profile: &str, count: usize) -> HashMap<String, Value> {
    let mut metrics = HashMap::new();
    metrics.insert("thumbnail_profile".to_string(), Value::from(profile));
    metrics.insert(
        "thumbnail_count".to_string(),
        Value::from(u64::try_from(count).unwrap_or(u64::MAX)),
    );
    metrics
}

fn thumbnail_reference(asset_uuid: &str, index: usize, count: usize) -> String {
    if count <= 1 {
        format!("/api/v1/assets/{asset_uuid}/derived/thumb")
    } else {
        format!("/api/v1/assets/{asset_uuid}/derived/thumbs/{}", index + 1)
    }
}

fn thumbnail_manifest_for_claimed_job(
    claimed: &ClaimedDerivedJob,
    artifacts: &GeneratedThumbnailArtifacts,
) -> Vec<DerivedManifestItem> {
    artifacts
        .files
        .iter()
        .enumerate()
        .map(|(index, path)| DerivedManifestItem {
            kind: DerivedKind::Thumb,
            reference: thumbnail_reference(&claimed.asset_uuid, index, artifacts.files.len()),
            size_bytes: std::fs::metadata(path).ok().map(|meta| meta.len()),
            sha256: None,
        })
        .collect()
}

fn thumbnail_uploads_for_claimed_job(
    claimed: &ClaimedDerivedJob,
    artifacts: &GeneratedThumbnailArtifacts,
) -> Result<Vec<DerivedUploadPlan>, DerivedJobExecutorError> {
    let mut uploads = Vec::with_capacity(artifacts.files.len());

    for (index, path) in artifacts.files.iter().enumerate() {
        let size_bytes = std::fs::metadata(path)
            .map_err(|error| DerivedJobExecutorError::Planner(error.to_string()))?
            .len();
        let upload_id = format!("upload-{}-thumb-{}", claimed.asset_uuid, index + 1);
        uploads.push(DerivedUploadPlan {
            init: DerivedUploadInit {
                asset_uuid: claimed.asset_uuid.clone(),
                revision_etag: String::new(),
                kind: DerivedKind::Thumb,
                content_type: content_type_for_kind(DerivedKind::Thumb).to_string(),
                size_bytes,
                sha256: None,
                idempotency_key: format!("init-{}-thumb-{}", claimed.job_id, index + 1),
            },
            parts: vec![DerivedUploadPart {
                asset_uuid: claimed.asset_uuid.clone(),
                revision_etag: String::new(),
                upload_id: upload_id.clone(),
                part_number: 1,
                chunk_path: path.clone(),
            }],
            complete: DerivedUploadComplete {
                asset_uuid: claimed.asset_uuid.clone(),
                revision_etag: String::new(),
                upload_id,
                idempotency_key: format!("complete-{}-thumb-{}", claimed.job_id, index + 1),
                parts: None,
            },
        });
    }

    Ok(uploads)
}

struct GeneratedThumbnailArtifacts {
    profile: &'static str,
    files: Vec<PathBuf>,
}

fn canonical_preview_profile_for_kind(kind: DerivedKind) -> &'static str {
    match kind {
        DerivedKind::PreviewVideo => "video_review_default_v1",
        DerivedKind::PreviewAudio => "audio_review_default_v1",
        DerivedKind::PreviewPhoto => "photo_review_default_v1",
        DerivedKind::Thumb | DerivedKind::Waveform => "unsupported",
    }
}

fn merge_metrics(
    target: &mut Option<HashMap<String, Value>>,
    extra: Option<HashMap<String, Value>>,
) {
    let Some(extra) = extra else {
        return;
    };
    let merged = target.get_or_insert_with(HashMap::new);
    for (key, value) in extra {
        merged.insert(key, value);
    }
}

fn sidecar_metrics(
    staged_source_path: Option<&Path>,
    staged_sidecar_paths: &[PathBuf],
) -> Result<Option<HashMap<String, Value>>, DerivedJobExecutorError> {
    let mut metrics = HashMap::new();

    if let Some(source_path) = staged_source_path {
        let source_size = std::fs::metadata(source_path)
            .map_err(|error| DerivedJobExecutorError::Planner(error.to_string()))?
            .len();
        metrics.insert(
            "staged_source_size_bytes".to_string(),
            Value::from(source_size),
        );
    }

    if !staged_sidecar_paths.is_empty() {
        let mut extension_counts = Map::new();
        let mut total_sidecars_bytes = 0_u64;
        for path in staged_sidecar_paths {
            let size = std::fs::metadata(path)
                .map_err(|error| DerivedJobExecutorError::Planner(error.to_string()))?
                .len();
            total_sidecars_bytes = total_sidecars_bytes.saturating_add(size);
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase())
                .unwrap_or_else(|| "(none)".to_string());
            let next_count = extension_counts
                .get(&extension)
                .and_then(|value| value.as_u64())
                .unwrap_or(0)
                + 1;
            extension_counts.insert(extension, Value::from(next_count));
        }
        metrics.insert(
            "staged_sidecars_count".to_string(),
            Value::from(staged_sidecar_paths.len() as u64),
        );
        metrics.insert(
            "staged_sidecars_total_bytes".to_string(),
            Value::from(total_sidecars_bytes),
        );
        metrics.insert(
            "staged_sidecars_by_extension".to_string(),
            Value::Object(extension_counts),
        );
    }

    if metrics.is_empty() {
        Ok(None)
    } else {
        Ok(Some(metrics))
    }
}
