use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunState {
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    PlayResume,
    Pause,
    Stop,
    Quit,
    OpenStatusWindow,
    OpenSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuVisibility {
    pub show_play_resume: bool,
    pub show_pause: bool,
}

pub fn menu_visibility(run_state: AgentRunState) -> MenuVisibility {
    match run_state {
        AgentRunState::Running => MenuVisibility {
            show_play_resume: false,
            show_pause: true,
        },
        AgentRunState::Paused => MenuVisibility {
            show_play_resume: true,
            show_pause: false,
        },
        AgentRunState::Stopped => MenuVisibility {
            show_play_resume: true,
            show_pause: false,
        },
    }
}

pub fn base_menu_actions() -> Vec<MenuAction> {
    vec![
        MenuAction::OpenStatusWindow,
        MenuAction::OpenSettings,
        MenuAction::Stop,
        MenuAction::Quit,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStage {
    Claim,
    Processing,
    Upload,
    Submit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobStatus {
    pub job_id: String,
    pub asset_uuid: String,
    pub progress_percent: u8,
    pub stage: JobStage,
    pub short_status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityState {
    Connected,
    Disconnected,
    Reconnecting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobFailure {
    pub job_id: String,
    pub error_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    pub known_job_ids: BTreeSet<String>,
    pub running_job_ids: BTreeSet<String>,
    pub failed_jobs: Vec<JobFailure>,
    pub connectivity: ConnectivityState,
    pub auth_reauth_required: bool,
    pub available_update: Option<String>,
    pub current_job: Option<JobStatus>,
}

impl Default for RuntimeSnapshot {
    fn default() -> Self {
        Self {
            known_job_ids: BTreeSet::new(),
            running_job_ids: BTreeSet::new(),
            failed_jobs: Vec::new(),
            connectivity: ConnectivityState::Connected,
            auth_reauth_required: false,
            available_update: None,
            current_job: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemNotification {
    NewJobReceived { job_id: String },
    AllJobsDone,
    JobFailed { job_id: String, error_code: String },
    AgentDisconnectedOrReconnecting,
    AuthExpiredReauthRequired,
    SettingsSaved,
    SettingsInvalid { reason: String },
    UpdatesAvailable { version: String },
}

#[derive(Debug, Clone)]
pub struct AgentUiRuntime {
    last_snapshot: RuntimeSnapshot,
    seen_job_ids: BTreeSet<String>,
    notified_failed_jobs: BTreeSet<String>,
    last_invalid_settings_reason: Option<String>,
    notified_update_version: Option<String>,
}

impl Default for AgentUiRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentUiRuntime {
    pub fn new() -> Self {
        Self {
            last_snapshot: RuntimeSnapshot::default(),
            seen_job_ids: BTreeSet::new(),
            notified_failed_jobs: BTreeSet::new(),
            last_invalid_settings_reason: None,
            notified_update_version: None,
        }
    }

    pub fn update_snapshot(&mut self, snapshot: RuntimeSnapshot) -> Vec<SystemNotification> {
        let mut notifications = Vec::new();

        for job_id in &snapshot.known_job_ids {
            if !self.seen_job_ids.contains(job_id) {
                notifications.push(SystemNotification::NewJobReceived {
                    job_id: job_id.clone(),
                });
                self.seen_job_ids.insert(job_id.clone());
            }
        }

        let had_running_jobs = !self.last_snapshot.running_job_ids.is_empty();
        let has_running_jobs = !snapshot.running_job_ids.is_empty();
        if had_running_jobs && !has_running_jobs {
            notifications.push(SystemNotification::AllJobsDone);
        }

        for failure in &snapshot.failed_jobs {
            if !self.notified_failed_jobs.contains(&failure.job_id) {
                notifications.push(SystemNotification::JobFailed {
                    job_id: failure.job_id.clone(),
                    error_code: failure.error_code.clone(),
                });
                self.notified_failed_jobs.insert(failure.job_id.clone());
            }
        }

        let prev_connectivity = self.last_snapshot.connectivity;
        if snapshot.connectivity != ConnectivityState::Connected
            && snapshot.connectivity != prev_connectivity
        {
            notifications.push(SystemNotification::AgentDisconnectedOrReconnecting);
        }

        if !self.last_snapshot.auth_reauth_required && snapshot.auth_reauth_required {
            notifications.push(SystemNotification::AuthExpiredReauthRequired);
        }

        if let Some(version) = &snapshot.available_update {
            if self.notified_update_version.as_ref() != Some(version) {
                notifications.push(SystemNotification::UpdatesAvailable {
                    version: version.clone(),
                });
                self.notified_update_version = Some(version.clone());
            }
        }

        self.last_snapshot = snapshot;
        notifications
    }

    pub fn notify_settings_saved(&mut self) -> SystemNotification {
        self.last_invalid_settings_reason = None;
        SystemNotification::SettingsSaved
    }

    pub fn notify_settings_invalid(&mut self, reason: &str) -> Option<SystemNotification> {
        if self.last_invalid_settings_reason.as_deref() == Some(reason) {
            return None;
        }
        self.last_invalid_settings_reason = Some(reason.to_string());
        Some(SystemNotification::SettingsInvalid {
            reason: reason.to_string(),
        })
    }

    pub fn status_window_job(snapshot: &RuntimeSnapshot) -> Option<&JobStatus> {
        snapshot.current_job.as_ref()
    }
}
