use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use clap::{Args, Parser, Subcommand, ValueEnum};
use retaia_agent::{
    AgentRuntimeConfig, ClientRuntimeTarget, CompletedJobEntry, ConfigRepository, CoreApiGateway,
    DaemonCurrentJobStats, DaemonCycleEntry, DaemonLastJobStats, DaemonRuntimeStats,
    DerivedProcessingGateway, FileConfigRepository, LogLevel, RuntimeDerivedPlanner,
    RuntimeHistoryStore, RuntimePollCycleStatus, RuntimeSession, SystemConfigRepository,
    compact_validation_reason, detect_language, notification_sink_profile_for_target, now_unix_ms,
    process_next_pending_job, run_runtime_poll_cycle, run_state_label, save_runtime_stats,
    select_notification_sink, t,
};
use tracing::{info, warn};

#[cfg(not(feature = "core-api-client"))]
use retaia_agent::CoreApiGatewayError;
#[cfg(not(feature = "core-api-client"))]
use retaia_agent::{DerivedProcessingError, UploadedDerivedPart};

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
    UiWeb,
    UiMobile,
}

impl From<TargetArg> for ClientRuntimeTarget {
    fn from(value: TargetArg) -> Self {
        match value {
            TargetArg::Agent => ClientRuntimeTarget::Agent,
            TargetArg::UiWeb => ClientRuntimeTarget::UiWeb,
            TargetArg::UiMobile => ClientRuntimeTarget::UiMobile,
        }
    }
}

fn load_settings(config_path: Option<PathBuf>) -> Result<AgentRuntimeConfig, String> {
    let lang = detect_language();
    match config_path {
        Some(path) => FileConfigRepository::new(path)
            .load()
            .map_err(|error| format!("{}: {error}", t(lang, "runtime.load_config_failed"))),
        None => SystemConfigRepository
            .load()
            .map_err(|error| format!("{}: {error}", t(lang, "runtime.load_config_failed"))),
    }
}

fn run() -> Result<(), String> {
    let lang = detect_language();
    let cli = Cli::parse();
    let settings = load_settings(cli.config)?;
    init_logging(settings.log_level);
    let mut session = RuntimeSession::new(cli.target.into(), settings).map_err(|errors| {
        format!(
            "{}: {}",
            t(lang, "runtime.invalid_config"),
            compact_validation_reason(&errors)
        )
    })?;

    match cli.mode {
        Some(ModeCommand::Daemon(args)) => run_daemon_loop(&mut session, args.tick_ms),
        None => Err(t(lang, "runtime.interactive_disabled").to_string()),
    }
}

fn run_daemon_loop(session: &mut RuntimeSession, tick_ms: u64) -> Result<(), String> {
    const COMPACTION_INTERVAL_TICKS: u64 = 600;
    const KEEP_LAST_CYCLES: usize = 250_000;
    const KEEP_LAST_COMPLETED_JOBS: usize = 150_000;

    let lang = detect_language();
    #[cfg(feature = "core-api-client")]
    register_daemon_agent(session.settings())?;
    let gateway = build_gateway(session.settings());
    let derived_gateway = build_derived_gateway(session.settings());
    let planner = RuntimeDerivedPlanner::default();
    let sink = select_notification_sink(notification_sink_profile_for_target(session.target()));
    let sleep_duration = Duration::from_millis(tick_ms.max(100));
    let mut history_store = match RuntimeHistoryStore::open_default() {
        Ok(store) => Some(store),
        Err(error) => {
            warn!(error = %error, "{}", t(lang, "runtime.history_store_unavailable"));
            None
        }
    };
    let mut tick = 0_u64;
    let mut current_job_id: Option<String> = None;
    let mut current_job_started_at: Option<Instant> = None;
    let mut current_job_started_at_unix_ms: Option<u64> = None;
    let mut last_job: Option<DaemonLastJobStats> = None;
    let mut last_cycle_fingerprint: Option<String> = None;
    let mut last_persisted_cycle_tick: u64 = 0;
    let shutdown_requested = install_shutdown_signal()?;
    let mut shutdown_log_emitted = false;
    info!(
        target = ?session.target(),
        run_state = ?session.run_state(),
        "{}",
        t(lang, "runtime.daemon_started")
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
                "{}",
                t(lang, "runtime.cycle")
            );
        } else {
            info!(
                tick,
                outcome = ?outcome.status,
                run_state = ?status.run_state,
                "{}",
                t(lang, "runtime.cycle")
            );
        }
        if outcome.status == RuntimePollCycleStatus::Throttled {
            warn!(tick, "{}", t(lang, "runtime.throttled"));
        }
        match process_next_pending_job(
            session,
            gateway.as_ref(),
            derived_gateway.as_ref(),
            &planner,
        ) {
            Ok(Some(report)) => {
                info!(
                    tick,
                    job_id = %report.job_id,
                    asset_uuid = %report.asset_uuid,
                    uploads = report.upload_count,
                    "runtime processed one pending job"
                );
            }
            Ok(None) => {}
            Err(error) => {
                warn!(tick, error = %error, "runtime processing pass failed");
            }
        }
        let status = session.status_view();
        let current_job_snapshot = status.current_job.clone();
        match current_job_snapshot.as_ref() {
            Some(job) => match current_job_id.as_deref() {
                Some(existing) if existing == job.job_id.as_str() => {}
                Some(existing) => {
                    if let Some(started) = current_job_started_at.take() {
                        let completed = DaemonLastJobStats {
                            job_id: existing.to_string(),
                            duration_ms: started.elapsed().as_millis() as u64,
                            completed_at_unix_ms: now_unix_ms(),
                        };
                        if let Some(store) = history_store.as_mut() {
                            let entry = CompletedJobEntry {
                                completed_at_unix_ms: completed.completed_at_unix_ms,
                                job_id: completed.job_id.clone(),
                                duration_ms: completed.duration_ms,
                            };
                            if let Err(error) = store.insert_completed_job(&entry) {
                                warn!(tick, error = %error, "{}", t(lang, "runtime.persist_completed_failed"));
                            }
                        }
                        last_job = Some(completed);
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
                    let completed = DaemonLastJobStats {
                        job_id: existing,
                        duration_ms: started.elapsed().as_millis() as u64,
                        completed_at_unix_ms: now_unix_ms(),
                    };
                    if let Some(store) = history_store.as_mut() {
                        let entry = CompletedJobEntry {
                            completed_at_unix_ms: completed.completed_at_unix_ms,
                            job_id: completed.job_id.clone(),
                            duration_ms: completed.duration_ms,
                        };
                        if let Err(error) = store.insert_completed_job(&entry) {
                            warn!(tick, error = %error, "{}", t(lang, "runtime.persist_completed_failed"));
                        }
                    }
                    last_job = Some(completed);
                }
                current_job_started_at_unix_ms = None;
            }
        }
        if shutdown_requested.load(Ordering::Relaxed) {
            if !shutdown_log_emitted {
                info!(tick, "shutdown requested: draining active job before stop");
                shutdown_log_emitted = true;
            }
            if current_job_id.is_none() {
                info!(tick, "graceful shutdown complete");
                return Ok(());
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
            warn!(tick, error = %error, "{}", t(lang, "runtime.persist_stats_failed"));
        }
        if let Some(store) = history_store.as_mut() {
            let fingerprint = cycle_fingerprint(
                outcome.status,
                status.run_state,
                stats.current_job.as_ref().map(|job| {
                    (
                        job.job_id.as_str(),
                        job.progress_percent,
                        job.stage.as_str(),
                    )
                }),
            );
            let changed = last_cycle_fingerprint
                .as_ref()
                .map(|previous| previous != &fingerprint)
                .unwrap_or(true);
            let should_persist = changed
                || !matches!(outcome.status, RuntimePollCycleStatus::Success)
                || tick.saturating_sub(last_persisted_cycle_tick) >= 60;
            if should_persist {
                let entry = DaemonCycleEntry {
                    ts_unix_ms: stats.updated_at_unix_ms,
                    tick,
                    outcome: outcome_label(outcome.status).to_string(),
                    run_state: run_state_label(status.run_state).to_string(),
                    job_id: stats.current_job.as_ref().map(|job| job.job_id.clone()),
                    asset_uuid: stats.current_job.as_ref().map(|job| job.asset_uuid.clone()),
                    progress_percent: stats.current_job.as_ref().map(|job| job.progress_percent),
                    stage: stats.current_job.as_ref().map(|job| job.stage.clone()),
                    short_status: stats.current_job.as_ref().map(|job| job.status.clone()),
                };
                if let Err(error) = store.insert_cycle(&entry) {
                    warn!(tick, error = %error, "{}", t(lang, "runtime.persist_cycle_failed"));
                } else {
                    last_cycle_fingerprint = Some(fingerprint);
                    last_persisted_cycle_tick = tick;
                }
            }
            if tick % COMPACTION_INTERVAL_TICKS == 0 {
                if let Err(error) = store.compact_old_cycles(KEEP_LAST_CYCLES) {
                    warn!(tick, error = %error, "{}", t(lang, "runtime.compact_failed"));
                }
                if let Err(error) = store.compact_old_completed_jobs(KEEP_LAST_COMPLETED_JOBS) {
                    warn!(tick, error = %error, "{}", t(lang, "runtime.compact_failed"));
                }
            }
        }
        std::thread::sleep(sleep_duration);
    }
}

#[cfg(feature = "core-api-client")]
fn register_daemon_agent(settings: &AgentRuntimeConfig) -> Result<(), String> {
    use retaia_agent::{
        AgentIdentity, AgentRegistrationIntent, OpenApiAgentRegistrationGateway,
        build_core_api_client, mint_technical_bearer, register_agent, with_bearer_token,
    };

    let identity = AgentIdentity::load_or_create(None).map_err(|error| error.to_string())?;
    let technical_auth = settings
        .technical_auth
        .as_ref()
        .ok_or_else(|| "technical auth is required for daemon registration".to_string())?;
    let client = build_core_api_client(settings);
    let token =
        mint_technical_bearer(&client, technical_auth).map_err(|error| error.to_string())?;
    let client = with_bearer_token(client, token);
    let gateway = OpenApiAgentRegistrationGateway::new_with_identity(client, identity.clone());

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        "arm" => "armv7",
        _ => "other",
    };

    register_agent(
        &gateway,
        AgentRegistrationIntent {
            agent_id: identity.agent_id,
            agent_name: "retaia-agent".to_string(),
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
            os_name: std::env::consts::OS.to_string(),
            os_version: std::env::consts::OS.to_string(),
            arch: arch.to_string(),
            client_feature_flags_contract_version: None,
            max_parallel_jobs: Some(settings.max_parallel_jobs),
        },
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn install_shutdown_signal() -> Result<Arc<AtomicBool>, String> {
    let flag = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&flag);
    ctrlc::set_handler(move || {
        handler_flag.store(true, Ordering::Relaxed);
    })
    .map_err(|error| format!("unable to install shutdown signal handler: {error}"))?;
    Ok(flag)
}

fn cycle_fingerprint(
    outcome: RuntimePollCycleStatus,
    run_state: retaia_agent::AgentRunState,
    job: Option<(&str, u8, &str)>,
) -> String {
    let (job_id, progress, stage) = match job {
        Some((id, p, s)) => (id, p.to_string(), s.to_string()),
        None => ("-", "-".to_string(), "-".to_string()),
    };
    format!(
        "{}|{}|{}|{}|{}",
        outcome_label(outcome),
        run_state_label(run_state),
        job_id,
        progress,
        stage
    )
}

fn outcome_label(status: RuntimePollCycleStatus) -> &'static str {
    match status {
        RuntimePollCycleStatus::Success => "success",
        RuntimePollCycleStatus::Throttled => "throttled",
        RuntimePollCycleStatus::Degraded => "degraded",
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
    use retaia_agent::{
        OpenApiJobsGateway, build_core_api_client, mint_technical_bearer, with_bearer_token,
    };

    let mut client = build_core_api_client(settings);
    if let Some(technical_auth) = settings.technical_auth.as_ref() {
        if let Ok(token) = mint_technical_bearer(&client, technical_auth) {
            client = with_bearer_token(client, token);
        }
    } else if let Ok(token) = std::env::var("RETAIA_AGENT_BEARER_TOKEN") {
        client = with_bearer_token(client, token);
    }
    Box::new(OpenApiJobsGateway::new(client))
}

#[cfg(feature = "core-api-client")]
fn build_derived_gateway(settings: &AgentRuntimeConfig) -> Box<dyn DerivedProcessingGateway> {
    use retaia_agent::{
        AgentIdentity, OpenApiDerivedProcessingGateway, build_core_api_client,
        mint_technical_bearer, with_bearer_token,
    };

    let mut client = build_core_api_client(settings);
    if let Some(technical_auth) = settings.technical_auth.as_ref() {
        if let Ok(token) = mint_technical_bearer(&client, technical_auth) {
            client = with_bearer_token(client, token);
        }
    } else if let Ok(token) = std::env::var("RETAIA_AGENT_BEARER_TOKEN") {
        client = with_bearer_token(client, token);
    }
    let identity = AgentIdentity::load_or_create(None).expect("agent identity must load");
    Box::new(OpenApiDerivedProcessingGateway::new_with_identity(
        client, identity,
    ))
}

#[cfg(not(feature = "core-api-client"))]
fn build_gateway(_settings: &AgentRuntimeConfig) -> Box<dyn CoreApiGateway> {
    Box::new(FeatureDisabledCoreGateway)
}

#[cfg(not(feature = "core-api-client"))]
fn build_derived_gateway(_settings: &AgentRuntimeConfig) -> Box<dyn DerivedProcessingGateway> {
    Box::new(FeatureDisabledDerivedGateway)
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

#[cfg(not(feature = "core-api-client"))]
#[derive(Debug, Clone, Copy)]
struct FeatureDisabledDerivedGateway;

#[cfg(not(feature = "core-api-client"))]
impl DerivedProcessingGateway for FeatureDisabledDerivedGateway {
    fn claim_job(
        &self,
        _job_id: &str,
    ) -> Result<retaia_agent::ClaimedDerivedJob, DerivedProcessingError> {
        Err(DerivedProcessingError::Transport(
            "core-api-client feature is disabled for this build".to_string(),
        ))
    }

    fn heartbeat(
        &self,
        _job_id: &str,
        _lock_token: &str,
        _fencing_token: i32,
    ) -> Result<retaia_agent::HeartbeatReceipt, DerivedProcessingError> {
        Err(DerivedProcessingError::Transport(
            "core-api-client feature is disabled for this build".to_string(),
        ))
    }

    fn submit_derived(
        &self,
        _job_id: &str,
        _lock_token: &str,
        _fencing_token: i32,
        _idempotency_key: &str,
        _payload: &retaia_agent::SubmitDerivedPayload,
    ) -> Result<(), DerivedProcessingError> {
        Err(DerivedProcessingError::Transport(
            "core-api-client feature is disabled for this build".to_string(),
        ))
    }

    fn upload_init(
        &self,
        _request: &retaia_agent::DerivedUploadInit,
    ) -> Result<(), DerivedProcessingError> {
        Err(DerivedProcessingError::Transport(
            "core-api-client feature is disabled for this build".to_string(),
        ))
    }

    fn upload_part(
        &self,
        _request: &retaia_agent::DerivedUploadPart,
    ) -> Result<UploadedDerivedPart, DerivedProcessingError> {
        Err(DerivedProcessingError::Transport(
            "core-api-client feature is disabled for this build".to_string(),
        ))
    }

    fn upload_complete(
        &self,
        _request: &retaia_agent::DerivedUploadComplete,
    ) -> Result<(), DerivedProcessingError> {
        Err(DerivedProcessingError::Transport(
            "core-api-client feature is disabled for this build".to_string(),
        ))
    }
}

fn main() {
    let lang = detect_language();
    if let Err(error) = run() {
        eprintln!("{error}");
        if error.contains("daemon mode") {
            eprintln!("{}", t(lang, "runtime.feature_required"));
        }
        exit(1);
    }
}
