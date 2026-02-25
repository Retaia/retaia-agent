use crate::application::derived_job_executor::{
    DerivedExecutionPlan, DerivedExecutionPlanner, DerivedJobExecutorError, DerivedUploadPlan,
};
use crate::application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedJobType, DerivedKind, DerivedManifestItem, DerivedUploadComplete,
    DerivedUploadInit, DerivedUploadPart, SubmitDerivedPayload,
};
use crate::domain::capabilities::photo_source_extension_supported;
use std::path::Path;

#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeDerivedPlanner;

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
                metrics: None,
            },
            submit_idempotency_key: format!("agent-submit-{}", claimed.job_id),
        })
    }

    fn plan_for_claimed_job_with_source(
        &self,
        claimed: &ClaimedDerivedJob,
        staged_source_path: Option<&Path>,
    ) -> Result<DerivedExecutionPlan, DerivedJobExecutorError> {
        let mut plan = self.plan_for_claimed_job(claimed)?;
        let Some(source_path) = staged_source_path else {
            return Ok(plan);
        };

        let upload_kind = plan
            .submit
            .manifest
            .first()
            .map(|item| item.kind)
            .unwrap_or_else(|| infer_proxy_kind(claimed));
        let metadata = std::fs::metadata(source_path)
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
        DerivedJobType::GenerateProxy => {
            vec![manifest_item_for_kind(claimed, infer_proxy_kind(claimed))]
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

fn infer_proxy_kind(claimed: &ClaimedDerivedJob) -> DerivedKind {
    let extension = claimed
        .source_original_relative
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if photo_source_extension_supported(&extension) {
        return DerivedKind::ProxyPhoto;
    }
    if is_audio_extension(&extension) {
        return DerivedKind::ProxyAudio;
    }
    DerivedKind::ProxyVideo
}

fn is_audio_extension(extension: &str) -> bool {
    matches!(
        extension,
        "aac" | "aif" | "aiff" | "alac" | "flac" | "m4a" | "mp3" | "ogg" | "opus" | "wav"
    )
}

fn content_type_for_kind(kind: DerivedKind) -> &'static str {
    match kind {
        DerivedKind::ProxyVideo => "video/mp4",
        DerivedKind::ProxyAudio => "audio/mp4",
        DerivedKind::ProxyPhoto | DerivedKind::Thumb => "image/jpeg",
        DerivedKind::Waveform => "application/json",
    }
}
