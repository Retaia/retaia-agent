use crate::application::derived_job_executor::{
    DerivedExecutionPlan, DerivedExecutionPlanner, DerivedJobExecutorError,
};
use crate::application::derived_processing_gateway::{
    ClaimedDerivedJob, DerivedJobType, DerivedKind, DerivedManifestItem, SubmitDerivedPayload,
};
use crate::domain::capabilities::photo_source_extension_supported;

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
