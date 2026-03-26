use crate::application::runtime_session::RuntimeSession;
use crate::domain::configuration::AgentRuntimeConfig;
use crate::domain::runtime_ui::{AgentRunState, JobStage, MenuAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellCommand {
    Menu,
    Status,
    Settings,
    Play,
    Pause,
    Stop,
    Help,
    Quit,
    Empty,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommandResult {
    pub output: String,
    pub should_exit: bool,
}

pub fn parse_shell_command(input: &str) -> ShellCommand {
    let normalized = input.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" => ShellCommand::Empty,
        "menu" | "m" => ShellCommand::Menu,
        "status" | "s" => ShellCommand::Status,
        "settings" | "cfg" => ShellCommand::Settings,
        "play" | "resume" | "p" => ShellCommand::Play,
        "pause" => ShellCommand::Pause,
        "stop" => ShellCommand::Stop,
        "help" | "h" | "?" => ShellCommand::Help,
        "quit" | "q" | "exit" => ShellCommand::Quit,
        _ => ShellCommand::Unknown(input.trim().to_string()),
    }
}

pub fn execute_shell_command(
    session: &mut RuntimeSession,
    command: ShellCommand,
) -> ShellCommandResult {
    match command {
        ShellCommand::Menu => ShellCommandResult {
            output: format_menu(session),
            should_exit: false,
        },
        ShellCommand::Status => ShellCommandResult {
            output: format_status(session),
            should_exit: false,
        },
        ShellCommand::Settings => ShellCommandResult {
            output: format_settings(session.settings()),
            should_exit: false,
        },
        ShellCommand::Play => ShellCommandResult {
            output: format!(
                "run_state={}\n",
                run_state_label(session.on_menu_action(MenuAction::PlayResume))
            ),
            should_exit: false,
        },
        ShellCommand::Pause => ShellCommandResult {
            output: format!(
                "run_state={}\n",
                run_state_label(session.on_menu_action(MenuAction::Pause))
            ),
            should_exit: false,
        },
        ShellCommand::Stop => ShellCommandResult {
            output: format!(
                "run_state={}\n",
                run_state_label(session.on_menu_action(MenuAction::Stop))
            ),
            should_exit: false,
        },
        ShellCommand::Help => ShellCommandResult {
            output: help_text(),
            should_exit: false,
        },
        ShellCommand::Quit => ShellCommandResult {
            output: "quitting runtime shell\n".to_string(),
            should_exit: true,
        },
        ShellCommand::Empty => ShellCommandResult {
            output: String::new(),
            should_exit: false,
        },
        ShellCommand::Unknown(raw) => ShellCommandResult {
            output: format!(
                "unknown command: {raw}\n{}\n",
                "type `help` to list supported commands"
            ),
            should_exit: false,
        },
    }
}

pub fn format_menu(session: &RuntimeSession) -> String {
    let model = session.tray_menu_model();
    let mut lines = Vec::new();
    lines.push(format!(
        "run_state={}",
        run_state_label(session.run_state())
    ));
    lines.push(format!(
        "show_play_resume={}",
        model.visibility.show_play_resume
    ));
    lines.push(format!("show_pause={}", model.visibility.show_pause));
    lines.push(format!(
        "play_resume_enabled={}",
        model.availability.can_play_resume
    ));
    lines.push(format!("pause_enabled={}", model.availability.can_pause));
    lines.push(format!("stop_enabled={}", model.availability.can_stop));
    lines.push("actions=open_status_window,open_settings,stop,quit".to_string());
    format!("{}\n", lines.join("\n"))
}

pub fn format_status(session: &RuntimeSession) -> String {
    let view = session.status_view();
    let mut lines = vec![format!("run_state={}", run_state_label(view.run_state))];

    if let Some(job) = view.current_job {
        lines.push(format!("job_id={}", job.job_id));
        lines.push(format!("asset_uuid={}", job.asset_uuid));
        lines.push(format!("progress_percent={}", job.progress_percent));
        lines.push(format!("stage={}", job_stage_label(job.stage)));
        lines.push(format!("status={}", job.short_status));
    } else {
        lines.push("job_id=-".to_string());
        lines.push("asset_uuid=-".to_string());
        lines.push("progress_percent=-".to_string());
        lines.push("stage=-".to_string());
        lines.push("status=idle".to_string());
    }

    format!("{}\n", lines.join("\n"))
}

pub fn format_settings(config: &AgentRuntimeConfig) -> String {
    let mut lines = vec![
        format!("core_api_url={}", config.core_api_url),
        format!("ollama_url={}", config.ollama_url),
        format!("auth_mode={}", auth_mode_label(config)),
        format!("technical_client_id={}", technical_client_id(config)),
        format!(
            "technical_secret_key_set={}",
            config.technical_auth.is_some()
        ),
        format!("max_parallel_jobs={}", config.max_parallel_jobs),
        format!("log_level={}", log_level_label(config)),
    ];
    lines.push(String::new());
    lines.join("\n")
}

pub fn help_text() -> String {
    [
        "commands:",
        "  menu      show runtime control visibility/availability",
        "  status    show current run state and current job",
        "  settings  show active runtime settings",
        "  play      apply play/resume",
        "  pause     apply pause",
        "  stop      apply stop",
        "  help      show this help",
        "  quit      exit shell",
        "",
    ]
    .join("\n")
}

fn run_state_label(state: AgentRunState) -> &'static str {
    match state {
        AgentRunState::Running => "running",
        AgentRunState::Paused => "paused",
        AgentRunState::Stopped => "stopped",
    }
}

fn job_stage_label(stage: JobStage) -> &'static str {
    match stage {
        JobStage::Claim => "claim",
        JobStage::Processing => "processing",
        JobStage::Upload => "upload",
        JobStage::Submit => "submit",
    }
}

fn auth_mode_label(config: &AgentRuntimeConfig) -> &'static str {
    use crate::domain::configuration::AuthMode;
    match config.auth_mode {
        AuthMode::Interactive => "interactive",
        AuthMode::Technical => "technical",
    }
}

fn technical_client_id(config: &AgentRuntimeConfig) -> &str {
    config
        .technical_auth
        .as_ref()
        .map(|auth| auth.client_id.as_str())
        .unwrap_or("-")
}

fn log_level_label(config: &AgentRuntimeConfig) -> &'static str {
    use crate::domain::configuration::LogLevel;
    match config.log_level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    }
}
