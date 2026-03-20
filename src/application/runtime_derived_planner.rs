use crate::application::derived_job_executor::{
    DerivedExecutionPlan, DerivedExecutionPlanner, DerivedJobExecutorError, DerivedUploadPlan,
};
use crate::application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedJobType, DerivedKind, DerivedManifestItem, DerivedUploadComplete,
    DerivedUploadInit, DerivedUploadPart, SubmitDerivedPayload,
};
use crate::application::proxy_generator::{
    AudioProxyFormat, AudioProxyRequest, PhotoProxyFormat, PhotoProxyRequest, ProxyGenerationError,
    ProxyGenerator, ThumbnailFormat, VideoProxyRequest, VideoThumbnailRequest,
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
        if claimed.job_type == DerivedJobType::ExtractFacts {
            return Ok(plan);
        }
        let Some(source_path) = staged_source_path else {
            return Ok(plan);
        };

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
            DerivedJobType::GenerateThumbnails => self.generate_thumbnail_artifact(source_path)?,
            _ => source_path.to_path_buf(),
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
                kind: upload_kind,
                content_type: content_type_for_kind(upload_kind).to_string(),
                size_bytes,
                sha256: None,
                idempotency_key: format!("init-{}-{}", claimed.job_id, upload_kind.as_str()),
            },
            parts: vec![DerivedUploadPart {
                asset_uuid: claimed.asset_uuid.clone(),
                upload_id: upload_id.clone(),
                part_number: 1,
                chunk_path: generated_path.clone(),
            }],
            complete: DerivedUploadComplete {
                asset_uuid: claimed.asset_uuid.clone(),
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
    }
}

fn manifest_item_for_kind(claimed: &ClaimedDerivedJob, kind: DerivedKind) -> DerivedManifestItem {
    DerivedManifestItem {
        kind,
        reference: format!(
            "agent://derived/{}/{}/v1",
            claimed.asset_uuid,
            kind.as_str()
        ),
        size_bytes: None,
        sha256: None,
    }
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

    fn generate_thumbnail_artifact(
        &self,
        source_path: &Path,
    ) -> Result<PathBuf, DerivedJobExecutorError> {
        let output_path = generated_preview_output_path(source_path, DerivedKind::Thumb);
        self.av_generator
            .generate_video_thumbnail(&canonical_thumbnail_request(
                source_path.to_string_lossy().to_string(),
                output_path.to_string_lossy().to_string(),
            ))
            .map_err(map_preview_generation_error)?;
        Ok(output_path)
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

fn canonical_thumbnail_request(input_path: String, output_path: String) -> VideoThumbnailRequest {
    VideoThumbnailRequest {
        input_path,
        output_path,
        format: ThumbnailFormat::Webp,
        max_width: 480,
        seek_ms: 1_000,
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
        metrics.insert(
            "thumbnail_profile".to_string(),
            Value::from("video_representative_v1"),
        );
        metrics.insert("thumbnail_count".to_string(), Value::from(1_u64));
    }

    if metrics.is_empty() {
        None
    } else {
        Some(metrics)
    }
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
