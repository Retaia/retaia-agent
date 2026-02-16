use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, ConnectivityState, JobStage, JobStatus,
    LogLevel, RuntimeSession, RuntimeSnapshot, ShellCommand, execute_shell_command, format_menu,
    format_settings, format_status,
};

fn settings() -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        core_api_url: "https://core.retaia.local".to_string(),
        ollama_url: "http://127.0.0.1:11434".to_string(),
        auth_mode: AuthMode::Interactive,
        technical_auth: None,
        max_parallel_jobs: 2,
        log_level: LogLevel::Info,
    }
}

#[test]
fn e2e_runtime_shell_flow_renders_menu_status_and_settings_from_shared_runtime_session() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    session.update_snapshot(RuntimeSnapshot {
        known_job_ids: ["job-42".to_string()].into_iter().collect(),
        running_job_ids: ["job-42".to_string()].into_iter().collect(),
        failed_jobs: Vec::new(),
        connectivity: ConnectivityState::Connected,
        auth_reauth_required: false,
        available_update: None,
        current_job: Some(JobStatus {
            job_id: "job-42".to_string(),
            asset_uuid: "asset-uuid-42".to_string(),
            progress_percent: 55,
            stage: JobStage::Processing,
            short_status: "processing".to_string(),
        }),
    });

    let menu = format_menu(&session);
    let status = format_status(&session);
    let cfg = format_settings(session.settings());

    assert!(menu.contains("actions=open_status_window,open_settings,stop,quit"));
    assert!(status.contains("job_id=job-42"));
    assert!(status.contains("stage=processing"));
    assert!(cfg.contains("core_api_url=https://core.retaia.local"));
}

#[test]
fn e2e_runtime_shell_flow_executes_play_pause_stop_help_and_quit_commands() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    let pause = execute_shell_command(&mut session, ShellCommand::Pause);
    assert!(pause.output.contains("run_state=paused"));

    let play = execute_shell_command(&mut session, ShellCommand::Play);
    assert!(play.output.contains("run_state=running"));

    let stop = execute_shell_command(&mut session, ShellCommand::Stop);
    assert!(stop.output.contains("run_state=stopped"));

    let help = execute_shell_command(&mut session, ShellCommand::Help);
    assert!(help.output.contains("commands:"));

    let quit = execute_shell_command(&mut session, ShellCommand::Quit);
    assert!(quit.should_exit);
}
