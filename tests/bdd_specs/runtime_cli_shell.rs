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
fn bdd_given_shell_aliases_when_parsing_then_supported_commands_are_resolved() {
    assert_eq!(parse_shell_command("m"), ShellCommand::Menu);
    assert_eq!(parse_shell_command("s"), ShellCommand::Status);
    assert_eq!(parse_shell_command("cfg"), ShellCommand::Settings);
    assert_eq!(parse_shell_command("resume"), ShellCommand::Play);
    assert_eq!(parse_shell_command("h"), ShellCommand::Help);
    assert_eq!(parse_shell_command("exit"), ShellCommand::Quit);
}

#[test]
fn bdd_given_runtime_shell_when_requesting_menu_status_and_settings_then_rendering_matches_contract()
 {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
    session.update_snapshot(RuntimeSnapshot {
        known_job_ids: ["job-9".to_string()].into_iter().collect(),
        running_job_ids: ["job-9".to_string()].into_iter().collect(),
        failed_jobs: Vec::new(),
        connectivity: ConnectivityState::Connected,
        auth_reauth_required: false,
        available_update: None,
        current_job: Some(JobStatus {
            job_id: "job-9".to_string(),
            asset_uuid: "asset-9".to_string(),
            progress_percent: 10,
            stage: JobStage::Claim,
            short_status: "claiming".to_string(),
        }),
    });

    let menu = format_menu(&session);
    assert!(menu.contains("run_state=running"));
    assert!(menu.contains("show_pause=true"));

    let status = format_status(&session);
    assert!(status.contains("stage=claim"));
    assert!(status.contains("progress_percent=10"));

    let settings = format_settings(session.settings());
    assert!(settings.contains("auth_mode=interactive"));
    assert!(settings.contains("technical_secret_key_set=false"));
}

#[test]
fn bdd_given_runtime_shell_controls_when_play_pause_stop_then_state_transitions_apply() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    let paused = execute_shell_command(&mut session, ShellCommand::Pause);
    assert!(paused.output.contains("run_state=paused"));

    let resumed = execute_shell_command(&mut session, ShellCommand::Play);
    assert!(resumed.output.contains("run_state=running"));

    let stopped = execute_shell_command(&mut session, ShellCommand::Stop);
    assert!(stopped.output.contains("run_state=stopped"));
}

#[test]
fn bdd_given_runtime_shell_help_and_unknown_when_requested_then_messages_are_explicit() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    let help = execute_shell_command(&mut session, ShellCommand::Help);
    assert!(help.output.contains("commands:"));
    assert_eq!(help.output, help_text());

    let unknown = execute_shell_command(&mut session, ShellCommand::Unknown("wtf".to_string()));
    assert!(unknown.output.contains("unknown command"));
    assert!(!unknown.should_exit);
}

#[test]
fn bdd_given_runtime_shell_quit_and_empty_when_requested_then_exit_behavior_is_correct() {
    let mut session = RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");

    let empty = execute_shell_command(&mut session, ShellCommand::Empty);
    assert_eq!(empty.output, "");
    assert!(!empty.should_exit);

    let quit = execute_shell_command(&mut session, ShellCommand::Quit);
    assert!(quit.should_exit);
}
