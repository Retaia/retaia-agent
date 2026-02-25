use std::path::{Component, Path, PathBuf};

use thiserror::Error;

use crate::application::derived_processing_gateway::ClaimedDerivedJob;
use crate::{AgentRuntimeConfig, resolve_source_path};

#[derive(Debug)]
pub struct StagedSourceFile {
    _temp_dir: tempfile::TempDir,
    path: PathBuf,
    sidecar_paths: Vec<PathBuf>,
    pub size_bytes: u64,
}

impl StagedSourceFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn sidecar_paths(&self) -> &[PathBuf] {
        &self.sidecar_paths
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

    let source_size = validated_file_size(&source)?;
    let mut sidecars = Vec::with_capacity(claimed.source_sidecars_relative.len());
    let mut required = source_size;
    for relative in &claimed.source_sidecars_relative {
        let resolved = resolve_source_path(settings, &claimed.source_storage_id, relative)
            .map_err(|error| SourceStagingError::ResolvePath(error.to_string()))?;
        let sidecar_size = validated_file_size(&resolved)?;
        required = required.saturating_add(sidecar_size);
        sidecars.push((resolved, relative.as_str()));
    }

    let temp_dir = tempfile::Builder::new()
        .prefix("retaia-agent-source-")
        .tempdir()
        .map_err(|error| SourceStagingError::Copy(error.to_string()))?;
    let available = probe.available_space(temp_dir.path())?;
    if available < required {
        return Err(SourceStagingError::InsufficientDiskSpace {
            required_bytes: required,
            available_bytes: available,
        });
    }

    let staged_path =
        copy_into_staging_dir(&source, &claimed.source_original_relative, temp_dir.path())?;
    let mut staged_sidecars = Vec::with_capacity(sidecars.len());
    for (sidecar, relative) in sidecars {
        staged_sidecars.push(copy_into_staging_dir(&sidecar, relative, temp_dir.path())?);
    }

    Ok(StagedSourceFile {
        _temp_dir: temp_dir,
        path: staged_path,
        sidecar_paths: staged_sidecars,
        size_bytes: required,
    })
}

fn validated_file_size(path: &Path) -> Result<u64, SourceStagingError> {
    let metadata =
        std::fs::metadata(path).map_err(|error| SourceStagingError::SourceIo(error.to_string()))?;
    if !metadata.is_file() {
        return Err(SourceStagingError::SourceNotFile(
            path.display().to_string(),
        ));
    }
    Ok(metadata.len())
}

fn copy_into_staging_dir(
    source: &Path,
    relative_path: &str,
    staging_dir: &Path,
) -> Result<PathBuf, SourceStagingError> {
    let staged_relative = staged_relative_path(relative_path)?;
    let staged_path = staging_dir.join(staged_relative);
    if let Some(parent) = staged_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| SourceStagingError::Copy(error.to_string()))?;
    }
    std::fs::copy(source, &staged_path)
        .map_err(|error| SourceStagingError::Copy(error.to_string()))?;
    Ok(staged_path)
}

fn staged_relative_path(value: &str) -> Result<PathBuf, SourceStagingError> {
    let path = Path::new(value);
    let mut sanitized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => sanitized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(SourceStagingError::Copy(format!(
                    "unsafe relative staging path: {value}"
                )));
            }
        }
    }

    if sanitized.as_os_str().is_empty() {
        return Err(SourceStagingError::Copy(format!(
            "empty relative staging path: {value}"
        )));
    }
    Ok(sanitized)
}
