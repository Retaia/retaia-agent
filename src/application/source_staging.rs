use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::application::derived_processing_gateway::ClaimedDerivedJob;
use crate::{AgentRuntimeConfig, resolve_source_path};

#[derive(Debug)]
pub struct StagedSourceFile {
    _temp_dir: tempfile::TempDir,
    path: PathBuf,
    pub size_bytes: u64,
}

impl StagedSourceFile {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SourceStagingError {
    #[error("unable to resolve source path: {0}")]
    ResolvePath(String),
    #[error("source file is missing or unreadable: {0}")]
    SourceIo(String),
    #[error("source path is not a regular file: {0}")]
    SourceNotFile(String),
    #[error(
        "insufficient local disk space for source staging (required={required_bytes} available={available_bytes})"
    )]
    InsufficientDiskSpace {
        required_bytes: u64,
        available_bytes: u64,
    },
    #[error("unable to stage source file copy: {0}")]
    Copy(String),
}

pub trait DiskSpaceProbe {
    fn available_space(&self, path: &Path) -> Result<u64, SourceStagingError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Fs2DiskSpaceProbe;

impl DiskSpaceProbe for Fs2DiskSpaceProbe {
    fn available_space(&self, path: &Path) -> Result<u64, SourceStagingError> {
        fs2::available_space(path).map_err(|error| SourceStagingError::Copy(error.to_string()))
    }
}

pub fn stage_claimed_job_source(
    settings: &AgentRuntimeConfig,
    claimed: &ClaimedDerivedJob,
) -> Result<StagedSourceFile, SourceStagingError> {
    stage_claimed_job_source_with_probe(settings, claimed, &Fs2DiskSpaceProbe)
}

pub fn stage_claimed_job_source_with_probe<P: DiskSpaceProbe>(
    settings: &AgentRuntimeConfig,
    claimed: &ClaimedDerivedJob,
    probe: &P,
) -> Result<StagedSourceFile, SourceStagingError> {
    let source = resolve_source_path(
        settings,
        &claimed.source_storage_id,
        &claimed.source_original_relative,
    )
    .map_err(|error| SourceStagingError::ResolvePath(error.to_string()))?;

    let metadata = std::fs::metadata(&source)
        .map_err(|error| SourceStagingError::SourceIo(error.to_string()))?;
    if !metadata.is_file() {
        return Err(SourceStagingError::SourceNotFile(
            source.display().to_string(),
        ));
    }

    let temp_dir = tempfile::Builder::new()
        .prefix("retaia-agent-source-")
        .tempdir()
        .map_err(|error| SourceStagingError::Copy(error.to_string()))?;
    let available = probe.available_space(temp_dir.path())?;
    let required = metadata.len();
    if available < required {
        return Err(SourceStagingError::InsufficientDiskSpace {
            required_bytes: required,
            available_bytes: available,
        });
    }

    let staged_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToString::to_string)
        .unwrap_or_else(|| "source.bin".to_string());
    let staged_path = temp_dir.path().join(staged_name);
    std::fs::copy(&source, &staged_path)
        .map_err(|error| SourceStagingError::Copy(error.to_string()))?;

    Ok(StagedSourceFile {
        _temp_dir: temp_dir,
        path: staged_path,
        size_bytes: required,
    })
}
