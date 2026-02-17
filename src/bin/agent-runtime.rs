use std::io::{self, Write};
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use retaia_agent::{
    AgentRuntimeConfig, ClientRuntimeTarget, ConfigRepository, CoreApiGateway, CoreApiGatewayError,
    FileConfigRepository, LogLevel, RuntimePollCycleStatus, RuntimeSession, SystemConfigRepository,
    compact_validation_reason, execute_shell_command, format_menu, help_text,
    notification_sink_profile_for_target, parse_shell_command, run_runtime_poll_cycle,
    select_notification_sink,
};
use tracing::{info, warn};

#[derive(Debug, Parser)]
#[command(name = "agent-runtime", about = "Retaia runtime interactive shell")]
struct Cli {
    #[arg(long = "config")]
    config: Option<PathBuf>,
    #[arg(long = "target", value_enum, default_value_t = TargetArg::Agent)]
    target: TargetArg,
    #[command(subcommand)]
    mode: Option<ModeCommand>,
}

#[derive(Debug, Subcommand)]
enum ModeCommand {
    Daemon(DaemonArgs),
}

#[derive(Debug, Clone, Args)]
struct DaemonArgs {
    #[arg(long = "tick-ms", default_value_t = 5000)]
    tick_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum TargetArg {
    Agent,
    Mcp,
    UiWeb,
    UiMobile,
}

impl From<TargetArg> for ClientRuntimeTarget {
    fn from(value: TargetArg) -> Self {
        match value {
            TargetArg::Agent => ClientRuntimeTarget::Agent,
            TargetArg::Mcp => ClientRuntimeTarget::Mcp,
            TargetArg::UiWeb => ClientRuntimeTarget::UiWeb,
            TargetArg::UiMobile => ClientRuntimeTarget::UiMobile,
        }
    }
}

fn load_settings(config_path: Option<PathBuf>) -> Result<AgentRuntimeConfig, String> {
    match config_path {
        Some(path) => FileConfigRepository::new(path)
            .load()
            .map_err(|error| format!("unable to load config: {error}")),
        None => SystemConfigRepository
            .load()
            .map_err(|error| format!("unable to load config: {error}")),
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let settings = load_settings(cli.config)?;
    init_logging(settings.log_level);
    let mut session = RuntimeSession::new(cli.target.into(), settings).map_err(|errors| {
        format!(
            "invalid runtime config: {}",
            compact_validation_reason(&errors)
        )
    })?;

    match cli.mode {
        Some(ModeCommand::Daemon(args)) => run_daemon_loop(&mut session, args.tick_ms),
        None => run_interactive_shell(&mut session),
    }
}

fn run_interactive_shell(session: &mut RuntimeSession) -> Result<(), String> {
    println!("{}", help_text());
    print!("{}", format_menu(session));

    let stdin = io::stdin();
    loop {
        print!("agent-runtime> ");
        io::stdout()
            .flush()
            .map_err(|error| format!("unable to flush stdout: {error}"))?;

        let mut line = String::new();
        let read = stdin
            .read_line(&mut line)
            .map_err(|error| format!("unable to read input: {error}"))?;
        if read == 0 {
            break;
        }

        let result = execute_shell_command(session, parse_shell_command(&line));
        if !result.output.is_empty() {
            print!("{}", result.output);
        }
        if result.should_exit {
            break;
        }
    }

    Ok(())
}

fn run_daemon_loop(session: &mut RuntimeSession, tick_ms: u64) -> Result<(), String> {
    let gateway = build_gateway(session.settings());
    let sink = select_notification_sink(notification_sink_profile_for_target(session.target()));
    let sleep_duration = Duration::from_millis(tick_ms.max(100));
    let mut tick = 0_u64;
    info!(
        target = ?session.target(),
        run_state = ?session.run_state(),
        "runtime daemon started"
    );
    loop {
        tick += 1;
        let outcome = run_runtime_poll_cycle(
            session,
            gateway.as_ref(),
            &sink,
            retaia_agent::PollEndpoint::Jobs,
            tick_ms.max(100),
            tick,
        );
        let status = session.status_view();
        if let Some(job) = status.current_job {
            info!(
                tick,
                outcome = ?outcome.status,
                run_state = ?status.run_state,
                job_id = %job.job_id,
                asset_uuid = %job.asset_uuid,
                progress_percent = job.progress_percent,
                stage = ?job.stage,
                short_status = %job.short_status,
                "runtime cycle"
            );
        } else {
            info!(
                tick,
                outcome = ?outcome.status,
                run_state = ?status.run_state,
                "runtime cycle"
            );
        }
        if outcome.status == RuntimePollCycleStatus::Throttled {
            warn!(tick, "core API throttled; backoff plan applied");
        }
        std::thread::sleep(sleep_duration);
    }
}

fn init_logging(level: LogLevel) {
    let level = match level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(level)
        .with_target(false)
        .with_line_number(false)
        .compact()
        .try_init();
}

#[cfg(feature = "core-api-client")]
fn build_gateway(settings: &AgentRuntimeConfig) -> Box<dyn CoreApiGateway> {
    use retaia_agent::{OpenApiJobsGateway, build_core_api_client, with_bearer_token};

    let mut client = build_core_api_client(settings);
    if let Ok(token) = std::env::var("RETAIA_AGENT_BEARER_TOKEN") {
        client = with_bearer_token(client, token);
    }
    Box::new(OpenApiJobsGateway::new(client))
}

#[cfg(not(feature = "core-api-client"))]
fn build_gateway(_settings: &AgentRuntimeConfig) -> Box<dyn CoreApiGateway> {
    Box::new(FeatureDisabledCoreGateway)
}

#[cfg(not(feature = "core-api-client"))]
#[derive(Debug, Clone, Copy)]
struct FeatureDisabledCoreGateway;

#[cfg(not(feature = "core-api-client"))]
impl CoreApiGateway for FeatureDisabledCoreGateway {
    fn poll_jobs(&self) -> Result<Vec<retaia_agent::CoreJobView>, CoreApiGatewayError> {
        Err(CoreApiGatewayError::Transport(
            "core-api-client feature is disabled for this build".to_string(),
        ))
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        exit(1);
    }
}
