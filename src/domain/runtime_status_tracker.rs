use std::collections::BTreeMap;

use crate::domain::runtime_ui::{
    ConnectivityState, JobFailure, JobStage, JobStatus, RuntimeSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeStatusEvent {
    JobDiscovered {
        job_id: String,
    },
    JobClaimed {
        job_id: String,
        asset_uuid: String,
    },
    JobProgress {
        job_id: String,
        asset_uuid: String,
        progress_percent: u8,
        stage: JobStage,
        short_status: String,
    },
    JobCompleted {
        job_id: String,
    },
    JobFailed {
        job_id: String,
        error_code: String,
    },
    ConnectivityChanged {
        connectivity: ConnectivityState,
    },
    AuthReauthRequired {
        required: bool,
    },
    UpdateAvailable {
        version: Option<String>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeStatusTracker {
    snapshot: RuntimeSnapshot,
    running_order: Vec<String>,
    latest_job_status: BTreeMap<String, JobStatus>,
}

impl RuntimeStatusTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> &RuntimeSnapshot {
        &self.snapshot
    }

    pub fn apply(&mut self, event: RuntimeStatusEvent) -> &RuntimeSnapshot {
        match event {
            RuntimeStatusEvent::JobDiscovered { job_id } => {
                self.snapshot.known_job_ids.insert(job_id);
            }
            RuntimeStatusEvent::JobClaimed { job_id, asset_uuid } => {
                self.snapshot.known_job_ids.insert(job_id.clone());
                self.snapshot.running_job_ids.insert(job_id.clone());
                self.ensure_running_order(&job_id);

                self.latest_job_status.insert(
                    job_id.clone(),
                    JobStatus {
                        job_id,
                        asset_uuid,
                        progress_percent: 0,
                        stage: JobStage::Claim,
                        short_status: "claimed".to_string(),
                    },
                );
            }
            RuntimeStatusEvent::JobProgress {
                job_id,
                asset_uuid,
                progress_percent,
                stage,
                short_status,
            } => {
                self.snapshot.known_job_ids.insert(job_id.clone());
                self.snapshot.running_job_ids.insert(job_id.clone());
                self.ensure_running_order(&job_id);

                self.latest_job_status.insert(
                    job_id.clone(),
                    JobStatus {
                        job_id,
                        asset_uuid,
                        progress_percent,
                        stage,
                        short_status,
                    },
                );
            }
            RuntimeStatusEvent::JobCompleted { job_id } => {
                self.snapshot.running_job_ids.remove(&job_id);
                self.running_order.retain(|id| id != &job_id);
                self.latest_job_status.remove(&job_id);
            }
            RuntimeStatusEvent::JobFailed { job_id, error_code } => {
                self.snapshot.running_job_ids.remove(&job_id);
                self.running_order.retain(|id| id != &job_id);
                self.latest_job_status.remove(&job_id);

                if let Some(existing) = self
                    .snapshot
                    .failed_jobs
                    .iter_mut()
                    .find(|failure| failure.job_id == job_id)
                {
                    existing.error_code = error_code;
                } else {
                    self.snapshot
                        .failed_jobs
                        .push(JobFailure { job_id, error_code });
                }
            }
            RuntimeStatusEvent::ConnectivityChanged { connectivity } => {
                self.snapshot.connectivity = connectivity;
            }
            RuntimeStatusEvent::AuthReauthRequired { required } => {
                self.snapshot.auth_reauth_required = required;
            }
            RuntimeStatusEvent::UpdateAvailable { version } => {
                self.snapshot.available_update = version;
            }
        }

        self.snapshot.current_job = self.pick_current_job();
        &self.snapshot
    }

    fn ensure_running_order(&mut self, job_id: &str) {
        if !self.running_order.iter().any(|id| id == job_id) {
            self.running_order.push(job_id.to_string());
        }
    }

    fn pick_current_job(&self) -> Option<JobStatus> {
        self.running_order
            .iter()
            .find_map(|job_id| self.latest_job_status.get(job_id).cloned())
    }
}
