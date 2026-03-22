use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use clap::{Args, Parser, Subcommand};
use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ClientRuntimeTarget, CompletedJobEntry, ConfigRepository,
    CoreApiGateway, DaemonCurrentJobStats, DaemonCycleEntry, DaemonLastJobStats,
    DaemonRuntimeStats, DerivedProcessingGateway, FileConfigRepository, LogLevel,
    RuntimeDerivedPlanner, RuntimeHistoryStore, RuntimePollCycleStatus, RuntimeSession,
    SystemConfigRepository, compact_validation_reason, detect_language,
    notification_sink_profile_for_target, now_unix_ms, process_next_pending_job,
    run_runtime_poll_cycle, run_state_label, save_runtime_stats, select_notification_sink, t,
};
use tracing::{info, warn};

#[cfg(feature = "core-api-client")]
use retaia_agent::{ConfigInterface, RuntimeConfigUpdate, apply_config_update};

#[cfg(not(feature = "core-api-client"))]
use retaia_agent::CoreApiGatewayError;
#[cfg(not(feature = "core-api-client"))]
use retaia_agent::{DerivedProcessingError, UploadedDerivedPart};

#[derive(Debug, Parser)]
#[command(name = "agent-runtime", about = "Retaia runtime daemon process")]
struct Cli {
    #[arg(long = "config")]
    config: Option<PathBuf>,
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

fn run() -> Result<(), String> {
    let lang = detect_language();
    let cli = Cli::parse();
    let config_path = cli.config.clone();
    match config_path {
        Some(path) => run_with_repository(&FileConfigRepository::new(path), cli, lang),
        None => run_with_repository(&SystemConfigRepository, cli, lang),
    }
}

fn run_with_repository<R: ConfigRepository>(
    repository: &R,
    cli: Cli,
    lang: retaia_agent::Language,
) -> Result<(), String> {
    let settings = repository
        .load()
        .map_err(|error| format!("{}: {error}", t(lang, "runtime.load_config_failed")))?;
    init_logging(settings.log_level);
    let mut session =
        RuntimeSession::new(ClientRuntimeTarget::Agent, settings).map_err(|errors| {
            format!(
                "{}: {}",
                t(lang, "runtime.invalid_config"),
                compact_validation_reason(&errors)
            )
        })?;

    match cli.mode {
        Some(ModeCommand::Daemon(args)) => run_daemon_loop(&mut session, repository, args.tick_ms),
        None => Err(t(lang, "runtime.interactive_disabled").to_string()),
    }
}

fn run_daemon_loop<R: ConfigRepository>(
    session: &mut RuntimeSession,
    repository: &R,
    tick_ms: u64,
) -> Result<(), String> {
    const COMPACTION_INTERVAL_TICKS: u64 = 600;
    const KEEP_LAST_CYCLES: usize = 250_000;
    const KEEP_LAST_COMPLETED_JOBS: usize = 150_000;

    let lang = detect_language();
    #[cfg(not(feature = "core-api-client"))]
    let _ = repository;
    #[cfg(feature = "core-api-client")]
    let mut pending_device_flow = start_daemon_device_flow_if_needed(session)?;
    #[cfg(feature = "core-api-client")]
    let mut next_device_flow_poll_at = Instant::now();
    #[cfg(feature = "core-api-client")]
    if pending_device_flow.is_none() && daemon_has_technical_auth(session.settings()) {
        register_daemon_agent(session.settings())?;
    }
    #[cfg_attr(not(feature = "core-api-client"), allow(unused_mut))]
    let mut gateway = build_gateway(session.settings());
    #[cfg_attr(not(feature = "core-api-client"), allow(unused_mut))]
    let mut derived_gateway = build_derived_gateway(session.settings());
    let planner = RuntimeDerivedPlanner::default();
    let sink = select_notification_sink(notification_sink_profile_for_target(session.target()));
    let sleep_duration = Duration::from_millis(tick_ms.max(100));
    let mut next_policy_poll_at = Instant::now();
    let mut next_jobs_poll_at = Instant::now();
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
    let mut last_outcome_status = RuntimePollCycleStatus::Success;
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
        let now = Instant::now();
        #[cfg(feature = "core-api-client")]
        if now >= next_device_flow_poll_at {
            let outcome =
                poll_daemon_device_flow_once(session, repository, &mut pending_device_flow, tick)?;
            if let Some(wait_ms) = outcome.wait_ms {
                next_device_flow_poll_at = now + Duration::from_millis(wait_ms);
            }
            if outcome.auth_changed {
                register_daemon_agent(session.settings())?;
                gateway = build_gateway(session.settings());
                derived_gateway = build_derived_gateway(session.settings());
                next_policy_poll_at = Instant::now();
                next_jobs_poll_at = Instant::now();
            }
        }

        if daemon_has_technical_auth(session.settings()) && now >= next_policy_poll_at {
            let wait_ms = poll_server_policy_once(session, gateway.as_ref(), tick);
            next_policy_poll_at = now + Duration::from_millis(wait_ms);
        }

        let jobs_interval_ms = session.jobs_poll_interval_ms();
        let outcome = if daemon_has_technical_auth(session.settings())
            && now >= next_jobs_poll_at
            && session.can_process_jobs()
        {
            let outcome = run_runtime_poll_cycle(
                session,
                gateway.as_ref(),
                &sink,
                retaia_agent::PollEndpoint::Jobs,
                jobs_interval_ms,
                tick,
            );
            last_outcome_status = outcome.status;
            next_jobs_poll_at =
                now + Duration::from_millis(scheduled_wait_ms_from_plan(&outcome.plan));
            Some(outcome)
        } else {
            if now >= next_jobs_poll_at {
                next_jobs_poll_at = now + Duration::from_millis(jobs_interval_ms.max(100));
            }
            None
        };
        let status = session.status_view();
        if let Some(job) = status.current_job {
            info!(
                tick,
                outcome = ?last_outcome_status,
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
                outcome = ?last_outcome_status,
                run_state = ?status.run_state,
                "{}",
                t(lang, "runtime.cycle")
            );
        }
        if outcome.as_ref().map(|cycle| cycle.status) == Some(RuntimePollCycleStatus::Throttled) {
            warn!(tick, "{}", t(lang, "runtime.throttled"));
        }
        if outcome.is_some() {
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
                last_outcome_status,
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
                || !matches!(last_outcome_status, RuntimePollCycleStatus::Success)
                || tick.saturating_sub(last_persisted_cycle_tick) >= 60;
            if should_persist {
                let entry = DaemonCycleEntry {
                    ts_unix_ms: stats.updated_at_unix_ms,
                    tick,
                    outcome: outcome_label(last_outcome_status).to_string(),
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

fn daemon_has_technical_auth(settings: &AgentRuntimeConfig) -> bool {
    matches!(settings.auth_mode, AuthMode::Technical) && settings.technical_auth.is_some()
}

#[cfg(feature = "core-api-client")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingDeviceFlow {
    device_code: String,
    verification_uri_complete: String,
    interval_seconds: u64,
}

#[cfg(feature = "core-api-client")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeviceFlowPollOutcome {
    wait_ms: Option<u64>,
    auth_changed: bool,
}

#[cfg(feature = "core-api-client")]
fn start_daemon_device_flow_if_needed(
    session: &RuntimeSession,
) -> Result<Option<PendingDeviceFlow>, String> {
    if daemon_has_technical_auth(session.settings()) {
        return Ok(None);
    }
    let client = retaia_agent::build_core_api_client(session.settings());
    let flow = start_daemon_device_flow_with(
        || retaia_agent::start_device_bootstrap(&client, Some("retaia-agent daemon".to_string())),
        open_url_in_browser,
    )?;
    Ok(Some(flow))
}

#[cfg(feature = "core-api-client")]
fn start_daemon_device_flow_with<S, O>(
    start: S,
    open_browser: O,
) -> Result<PendingDeviceFlow, String>
where
    S: FnOnce() -> Result<retaia_agent::DeviceBootstrapStart, retaia_agent::DeviceBootstrapError>,
    O: FnOnce(&str) -> Result<(), String>,
{
    let flow = start().map_err(|error| error.to_string())?;
    open_browser(&flow.verification_uri_complete)?;
    Ok(PendingDeviceFlow {
        device_code: flow.device_code,
        verification_uri_complete: flow.verification_uri_complete,
        interval_seconds: flow.interval_seconds.max(1),
    })
}

#[cfg(feature = "core-api-client")]
fn open_url_in_browser(url: &str) -> Result<(), String> {
    let status = if let Some(command) = std::env::var_os("RETAIA_AGENT_BROWSER_OPEN_COMMAND") {
        std::process::Command::new(command).arg(url).status()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
    } else {
        std::process::Command::new("xdg-open").arg(url).status()
    }
    .map_err(|error| format!("browser open failed: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("browser command exited with status {status}"))
    }
}

#[cfg(feature = "core-api-client")]
fn poll_daemon_device_flow_once<R: ConfigRepository>(
    session: &mut RuntimeSession,
    repository: &R,
    pending_device_flow: &mut Option<PendingDeviceFlow>,
    jitter_seed: u64,
) -> Result<DeviceFlowPollOutcome, String> {
    let Some(flow) = pending_device_flow.clone() else {
        if daemon_has_technical_auth(session.settings()) {
            return Ok(DeviceFlowPollOutcome {
                wait_ms: None,
                auth_changed: false,
            });
        }
        *pending_device_flow = start_daemon_device_flow_if_needed(session)?;
        return Ok(DeviceFlowPollOutcome {
            wait_ms: pending_device_flow.as_ref().map(|pending| {
                schedule_device_flow_wait_ms(session, pending.interval_seconds, false)
            }),
            auth_changed: false,
        });
    };

    let client = retaia_agent::build_core_api_client(session.settings());
    advance_daemon_device_flow_with(
        session,
        repository,
        &flow,
        |device_code| retaia_agent::poll_device_bootstrap(&client, device_code),
        pending_device_flow,
        jitter_seed,
    )
}

#[cfg(feature = "core-api-client")]
fn advance_daemon_device_flow_with<R, P>(
    session: &mut RuntimeSession,
    repository: &R,
    flow: &PendingDeviceFlow,
    poll: P,
    pending_device_flow: &mut Option<PendingDeviceFlow>,
    _jitter_seed: u64,
) -> Result<DeviceFlowPollOutcome, String>
where
    R: ConfigRepository,
    P: FnOnce(
        &str,
    ) -> Result<
        retaia_agent::DeviceBootstrapPollStatus,
        retaia_agent::DeviceBootstrapError,
    >,
{
    match poll(&flow.device_code).map_err(|error| error.to_string())? {
        retaia_agent::DeviceBootstrapPollStatus::Pending { interval_seconds } => {
            let next_interval = interval_seconds.unwrap_or(flow.interval_seconds).max(1);
            pending_device_flow.replace(PendingDeviceFlow {
                device_code: flow.device_code.clone(),
                verification_uri_complete: flow.verification_uri_complete.clone(),
                interval_seconds: next_interval,
            });
            Ok(DeviceFlowPollOutcome {
                wait_ms: Some(schedule_device_flow_wait_ms(session, next_interval, false)),
                auth_changed: false,
            })
        }
        retaia_agent::DeviceBootstrapPollStatus::Approved {
            client_id,
            secret_key,
        } => {
            let next = apply_config_update(
                session.settings(),
                &RuntimeConfigUpdate {
                    core_api_url: None,
                    ollama_url: None,
                    auth_mode: Some(AuthMode::Technical),
                    technical_client_id: Some(client_id),
                    technical_secret_key: Some(secret_key),
                    clear_technical_auth: false,
                    storage_mounts: None,
                    clear_storage_mounts: false,
                    max_parallel_jobs: None,
                    log_level: None,
                },
                ConfigInterface::Cli,
            )
            .map_err(|errors| compact_validation_reason(&errors))?;
            repository.save(&next).map_err(|error| error.to_string())?;
            session
                .replace_settings(next)
                .map_err(|errors| compact_validation_reason(&errors))?;
            pending_device_flow.take();
            Ok(DeviceFlowPollOutcome {
                wait_ms: None,
                auth_changed: true,
            })
        }
        retaia_agent::DeviceBootstrapPollStatus::Denied
        | retaia_agent::DeviceBootstrapPollStatus::Expired => {
            pending_device_flow.take();
            let plan =
                session.on_poll_success(retaia_agent::PollEndpoint::DeviceFlow, 15_000, false);
            Ok(DeviceFlowPollOutcome {
                wait_ms: Some(scheduled_wait_ms_from_plan(&plan)),
                auth_changed: false,
            })
        }
    }
}

#[cfg(feature = "core-api-client")]
fn schedule_device_flow_wait_ms(
    session: &mut RuntimeSession,
    interval_seconds: u64,
    throttled: bool,
) -> u64 {
    let plan = if throttled {
        session.on_poll_throttled_tracked(
            retaia_agent::PollEndpoint::DeviceFlow,
            retaia_agent::PollSignal::SlowDown429,
            interval_seconds,
        )
    } else {
        session.on_poll_success(
            retaia_agent::PollEndpoint::DeviceFlow,
            interval_seconds.max(1).saturating_mul(1_000),
            false,
        )
    };
    scheduled_wait_ms_from_plan(&plan)
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

fn scheduled_wait_ms_from_plan(plan: &retaia_agent::RuntimeSyncPlan) -> u64 {
    match plan {
        retaia_agent::RuntimeSyncPlan::SchedulePoll(decision) => decision.wait_ms.max(100),
        retaia_agent::RuntimeSyncPlan::None
        | retaia_agent::RuntimeSyncPlan::TriggerPollNow { .. } => 100,
    }
}

fn policy_refresh_interval_ms() -> u64 {
    30_000
}

fn policy_poll_wait_ms_from_plan(plan: &retaia_agent::RuntimeSyncPlan) -> u64 {
    const POLICY_EARLY_REFRESH_FLOOR_MS: u64 = 15_000;

    match plan {
        retaia_agent::RuntimeSyncPlan::SchedulePoll(decision)
            if matches!(
                decision.reason,
                retaia_agent::PollDecisionReason::BackoffFrom429
            ) =>
        {
            decision.wait_ms.max(POLICY_EARLY_REFRESH_FLOOR_MS)
        }
        _ => scheduled_wait_ms_from_plan(plan),
    }
}

fn poll_server_policy_once<G: CoreApiGateway + ?Sized>(
    session: &mut RuntimeSession,
    gateway: &G,
    jitter_seed: u64,
) -> u64 {
    match gateway.fetch_server_policy() {
        Ok(policy) => {
            session.apply_server_policy(policy);
            let plan = session.on_poll_success(
                retaia_agent::PollEndpoint::Policy,
                policy_refresh_interval_ms(),
                false,
            );
            policy_poll_wait_ms_from_plan(&plan)
        }
        Err(retaia_agent::CoreApiGatewayError::Throttled { retry_after_ms }) => {
            let signal = retry_after_ms
                .map(|wait_ms| retaia_agent::PollSignal::RetryAfter429 { wait_ms })
                .unwrap_or(retaia_agent::PollSignal::SlowDown429);
            let plan = session.on_poll_throttled_tracked(
                retaia_agent::PollEndpoint::Policy,
                signal,
                jitter_seed,
            );
            warn!(tick = jitter_seed, "runtime policy poll throttled");
            policy_poll_wait_ms_from_plan(&plan)
        }
        Err(error) => {
            let plan = session.on_poll_success(
                retaia_agent::PollEndpoint::Policy,
                policy_refresh_interval_ms(),
                false,
            );
            warn!(tick = jitter_seed, error = %error, "runtime policy poll failed");
            policy_poll_wait_ms_from_plan(&plan)
        }
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

    fn fetch_asset_revision_etag(
        &self,
        _asset_uuid: &str,
    ) -> Result<String, DerivedProcessingError> {
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

#[cfg(test)]
mod tests {
    use super::{
        policy_poll_wait_ms_from_plan, policy_refresh_interval_ms, poll_server_policy_once,
    };
    use retaia_agent::{
        AgentRuntimeConfig, AuthMode, CORE_JOBS_RUNTIME_FEATURE, ClientRuntimeTarget,
        CoreApiGateway, CoreApiGatewayError, CoreServerPolicy, LogLevel, PollEndpoint,
        RuntimeSession, RuntimeSyncPlan,
    };
    use std::collections::BTreeMap;

    fn settings() -> AgentRuntimeConfig {
        AgentRuntimeConfig {
            core_api_url: "https://core.retaia.local".to_string(),
            ollama_url: "http://127.0.0.1:11434".to_string(),
            auth_mode: AuthMode::Interactive,
            technical_auth: None,
            storage_mounts: std::collections::BTreeMap::new(),
            max_parallel_jobs: 2,
            log_level: LogLevel::Info,
        }
    }

    struct PolicyGateway {
        result: Result<CoreServerPolicy, CoreApiGatewayError>,
    }

    impl CoreApiGateway for PolicyGateway {
        fn poll_jobs(&self) -> Result<Vec<retaia_agent::CoreJobView>, CoreApiGatewayError> {
            Ok(Vec::new())
        }

        fn fetch_server_policy(&self) -> Result<CoreServerPolicy, CoreApiGatewayError> {
            self.result.clone()
        }
    }

    #[test]
    fn tdd_policy_refresh_interval_is_thirty_seconds() {
        assert_eq!(policy_refresh_interval_ms(), 30_000);
    }

    #[test]
    fn tdd_policy_success_plan_reuses_thirty_second_daemon_cadence() {
        let mut session =
            RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
        let plan =
            session.on_poll_success(PollEndpoint::Policy, policy_refresh_interval_ms(), false);

        match plan {
            RuntimeSyncPlan::SchedulePoll(decision) => {
                assert_eq!(decision.endpoint, PollEndpoint::Policy);
                assert_eq!(
                    policy_poll_wait_ms_from_plan(&RuntimeSyncPlan::SchedulePoll(decision)),
                    30_000
                );
            }
            other => panic!("unexpected plan: {other:?}"),
        }
    }

    #[test]
    fn tdd_policy_early_refresh_is_floored_to_fifteen_seconds() {
        let plan = RuntimeSyncPlan::SchedulePoll(retaia_agent::PollDecision {
            endpoint: PollEndpoint::Policy,
            wait_ms: 2_000,
            reason: retaia_agent::PollDecisionReason::BackoffFrom429,
        });

        assert_eq!(policy_poll_wait_ms_from_plan(&plan), 15_000);
    }

    #[test]
    fn tdd_policy_early_refresh_keeps_longer_waits() {
        let plan = RuntimeSyncPlan::SchedulePoll(retaia_agent::PollDecision {
            endpoint: PollEndpoint::Policy,
            wait_ms: 22_000,
            reason: retaia_agent::PollDecisionReason::BackoffFrom429,
        });

        assert_eq!(policy_poll_wait_ms_from_plan(&plan), 22_000);
    }

    #[test]
    fn tdd_daemon_policy_poll_applies_server_policy_and_enables_jobs_runtime() {
        let mut session =
            RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
        let gateway = PolicyGateway {
            result: Ok(CoreServerPolicy {
                min_poll_interval_seconds: Some(9),
                feature_flags: BTreeMap::from([(CORE_JOBS_RUNTIME_FEATURE.to_string(), true)]),
            }),
        };

        let wait_ms = poll_server_policy_once(&mut session, &gateway, 42);

        assert_eq!(wait_ms, 30_000);
        assert_eq!(session.server_policy().min_poll_interval_seconds, Some(9));
        assert!(session.effective_feature_enabled(CORE_JOBS_RUNTIME_FEATURE));
        assert!(session.can_process_jobs());
    }

    #[cfg(feature = "core-api-client")]
    #[test]
    fn tdd_daemon_device_flow_start_opens_browser_and_tracks_poll_state() {
        use super::{PendingDeviceFlow, start_daemon_device_flow_with};
        use retaia_agent::DeviceBootstrapStart;

        let mut opened_url = None;
        let pending = start_daemon_device_flow_with(
            || {
                Ok(DeviceBootstrapStart {
                    device_code: "dev-123".to_string(),
                    user_code: "ABCD-EFGH".to_string(),
                    verification_uri: "https://ui.retaia.local/device".to_string(),
                    verification_uri_complete: "https://ui.retaia.local/device?user_code=ABCD-EFGH"
                        .to_string(),
                    expires_in_seconds: 900,
                    interval_seconds: 7,
                })
            },
            |url: &str| {
                opened_url = Some(url.to_string());
                Ok(())
            },
        )
        .expect("device flow should start");

        assert_eq!(
            opened_url.as_deref(),
            Some("https://ui.retaia.local/device?user_code=ABCD-EFGH")
        );
        assert_eq!(
            pending,
            PendingDeviceFlow {
                device_code: "dev-123".to_string(),
                verification_uri_complete: "https://ui.retaia.local/device?user_code=ABCD-EFGH"
                    .to_string(),
                interval_seconds: 7,
            }
        );
    }

    #[cfg(feature = "core-api-client")]
    #[test]
    fn tdd_daemon_device_flow_approval_persists_technical_auth() {
        use super::{PendingDeviceFlow, advance_daemon_device_flow_with};
        use retaia_agent::{
            ConfigRepository, DeviceBootstrapPollStatus, FileConfigRepository,
            load_config_from_path,
        };
        use tempfile::tempdir;

        let dir = tempdir().expect("temp dir");
        unsafe {
            std::env::set_var("RETAIA_AGENT_SECRET_STORE_BACKEND", "memory");
            std::env::set_var(
                "RETAIA_AGENT_SECRET_STORE_FILE",
                dir.path().join("secrets.json"),
            );
        }
        let config_path = dir.path().join("config.toml");
        let repository = FileConfigRepository::new(config_path.clone());
        repository.save(&settings()).expect("save initial config");
        let mut session =
            RuntimeSession::new(ClientRuntimeTarget::Agent, settings()).expect("session");
        let flow = PendingDeviceFlow {
            device_code: "dev-123".to_string(),
            verification_uri_complete: "https://ui.retaia.local/device?user_code=ABCD-EFGH"
                .to_string(),
            interval_seconds: 5,
        };
        let mut pending = Some(flow.clone());

        let outcome = advance_daemon_device_flow_with(
            &mut session,
            &repository,
            &flow,
            |_device_code| {
                Ok(DeviceBootstrapPollStatus::Approved {
                    client_id: "agent-approved".to_string(),
                    secret_key: "approved-secret".to_string(),
                })
            },
            &mut pending,
            42,
        )
        .expect("device flow approval");

        assert!(outcome.auth_changed);
        assert!(pending.is_none());
        assert_eq!(session.settings().auth_mode, AuthMode::Technical);
        assert_eq!(
            session
                .settings()
                .technical_auth
                .as_ref()
                .expect("technical auth")
                .client_id,
            "agent-approved"
        );

        let saved = load_config_from_path(&config_path).expect("saved config");
        assert_eq!(saved.auth_mode, AuthMode::Technical);
        assert_eq!(
            saved.technical_auth.expect("persisted auth").client_id,
            "agent-approved"
        );
    }
}
