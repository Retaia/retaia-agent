use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, ConnectivityState, JobStage, JobStatus,
    LogLevel, RuntimeSession, RuntimeSnapshot, ShellCommand, execute_shell_command, format_menu,
    format_settings, format_status, help_text, parse_shell_command,
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
fn tdd_runtime_cli_shell_parses_supported_commands() {
    assert_eq!(parse_shell_command("menu"), ShellCommand::Menu);
    assert_eq!(parse_shell_command("status"), ShellCommand::Status);
    assert_eq!(parse_shell_command("settings"), ShellCommand::Settings);
    assert_eq!(parse_shell_command("play"), ShellCommand::Play);
    assert_eq!(parse_shell_command("pause"), ShellCommand::Pause);
    assert_eq!(parse_shell_command("stop"), ShellCommand::Stop);
    assert_eq!(parse_shell_command("help"), ShellCommand::Help);
    assert_eq!(parse_shell_command("quit"), ShellCommand::Quit);
}

#[test]
fn tdd_runtime_cli_shell_applies_play_pause_stop_transitions() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    let paused = execute_shell_command(&mut session, ShellCommand::Pause);
    assert!(paused.output.contains("run_state=paused"));

    let resumed = execute_shell_command(&mut session, ShellCommand::Play);
    assert!(resumed.output.contains("run_state=running"));

    let stopped = execute_shell_command(&mut session, ShellCommand::Stop);
    assert!(stopped.output.contains("run_state=stopped"));
}

#[test]
fn tdd_runtime_cli_shell_formats_current_job_status() {
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
            asset_uuid: "asset-9".to_string(),
            progress_percent: 73,
            stage: JobStage::Upload,
            short_status: "uploading".to_string(),
        }),
    });

    let rendered = format_status(&session);
    assert!(rendered.contains("job_id=job-42"));
    assert!(rendered.contains("asset_uuid=asset-9"));
    assert!(rendered.contains("progress_percent=73"));
    assert!(rendered.contains("stage=upload"));
    assert!(rendered.contains("status=uploading"));
}

#[test]
fn tdd_runtime_cli_shell_unknown_command_does_not_exit() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    let result = execute_shell_command(&mut session, ShellCommand::Unknown("noop".to_string()));

    assert!(!result.should_exit);
    assert!(result.output.contains("unknown command"));
}

#[test]
fn tdd_runtime_cli_shell_menu_settings_help_and_quit_outputs_are_stable() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    let menu = execute_shell_command(&mut session, ShellCommand::Menu);
    assert!(menu.output.contains("run_state=running"));
    assert!(menu.output.contains("show_pause=true"));

    let settings_output = execute_shell_command(&mut session, ShellCommand::Settings);
    assert!(
        settings_output
            .output
            .contains("core_api_url=https://core.retaia.local")
    );
    assert!(settings_output.output.contains("auth_mode=interactive"));

    let help = execute_shell_command(&mut session, ShellCommand::Help);
    assert!(help.output.contains("commands:"));
    assert!(help.output.contains("quit"));

    let quit = execute_shell_command(&mut session, ShellCommand::Quit);
    assert!(quit.should_exit);
}

#[test]
fn tdd_runtime_cli_shell_empty_and_idle_status_render_expected_defaults() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    let empty = execute_shell_command(&mut session, ShellCommand::Empty);
    assert!(!empty.should_exit);
    assert_eq!(empty.output, "");

    let idle_status = format_status(&session);
    assert!(idle_status.contains("job_id=-"));
    assert!(idle_status.contains("status=idle"));

    let menu = format_menu(&session);
    assert!(menu.contains("actions=open_status_window,open_settings,stop,quit"));

    let cfg = format_settings(session.settings());
    assert!(cfg.contains("technical_client_id=-"));

    let help = help_text();
    assert!(help.contains("play"));
    assert!(help.contains("pause"));
}
