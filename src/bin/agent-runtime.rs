use std::path::PathBuf;
use std::process::exit;
use std::time::{Duration, Instant};

use clap::{Args, Parser, Subcommand, ValueEnum};
use retaia_agent::{
    AgentRuntimeConfig, ClientRuntimeTarget, ConfigRepository, CoreApiGateway, CoreApiGatewayError,
    DaemonCurrentJobStats, DaemonLastJobStats, DaemonRuntimeStats, FileConfigRepository, LogLevel,
    RuntimePollCycleStatus, RuntimeSession, SystemConfigRepository, compact_validation_reason,
    notification_sink_profile_for_target, now_unix_ms, run_runtime_poll_cycle, run_state_label,
    save_runtime_stats, select_notification_sink,
};
use tracing::{info, warn};

#[derive(Debug, Parser)]
#[command(name = "agent-runtime", about = "Retaia runtime daemon process")]
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
        None => Err(
            "interactive mode is disabled; run `agent-runtime daemon` and control it with `agentctl daemon ...`"
                .to_string(),
        ),
    }
}

fn run_daemon_loop(session: &mut RuntimeSession, tick_ms: u64) -> Result<(), String> {
    let gateway = build_gateway(session.settings());
    let sink = select_notification_sink(notification_sink_profile_for_target(session.target()));
    let sleep_duration = Duration::from_millis(tick_ms.max(100));
    let mut tick = 0_u64;
    let mut current_job_id: Option<String> = None;
    let mut current_job_started_at: Option<Instant> = None;
    let mut current_job_started_at_unix_ms: Option<u64> = None;
    let mut last_job: Option<DaemonLastJobStats> = None;
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
        let status = session.status_view();
        let current_job_snapshot = status.current_job.clone();
        match current_job_snapshot.as_ref() {
            Some(job) => match current_job_id.as_deref() {
                Some(existing) if existing == job.job_id.as_str() => {}
                Some(existing) => {
                    if let Some(started) = current_job_started_at.take() {
                        last_job = Some(DaemonLastJobStats {
                            job_id: existing.to_string(),
                            duration_ms: started.elapsed().as_millis() as u64,
                            completed_at_unix_ms: now_unix_ms(),
                        });
                    }
                    current_job_id = Some(job.job_id.clone());
                    current_job_started_at = Some(Instant::now());
                    current_job_started_at_unix_ms = Some(now_unix_ms());
                }
                None => {
                    current_job_id = Some(job.job_id.clone());
                    current_job_started_at = Some(Instant::now());
                    current_job_started_at_unix_ms = Some(now_unix_ms());
                }
            },
            None => {
                if let Some(existing) = current_job_id.take()
                    && let Some(started) = current_job_started_at.take()
                {
                    last_job = Some(DaemonLastJobStats {
                        job_id: existing,
                        duration_ms: started.elapsed().as_millis() as u64,
                        completed_at_unix_ms: now_unix_ms(),
                    });
                }
                current_job_started_at_unix_ms = None;
            }
        }

        let current_job = current_job_snapshot.map(|job| DaemonCurrentJobStats {
            job_id: job.job_id,
            asset_uuid: job.asset_uuid,
            progress_percent: job.progress_percent,
            stage: format!("{:?}", job.stage).to_lowercase(),
            status: job.short_status,
            started_at_unix_ms: current_job_started_at_unix_ms.unwrap_or_else(now_unix_ms),
        });
        let stats = DaemonRuntimeStats {
            updated_at_unix_ms: now_unix_ms(),
            run_state: run_state_label(status.run_state).to_string(),
            tick,
            current_job,
            last_job: last_job.clone(),
        };
        if let Err(error) = save_runtime_stats(&stats) {
            warn!(tick, error = %error, "unable to persist daemon stats");
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
