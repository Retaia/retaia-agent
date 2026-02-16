use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest};
use genai::resolver::{Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget, WebConfig};
use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigInterface, ConfigRepository, ConfigRepositoryError,
    ConfigValidationError, FileConfigRepository, LogLevel, RuntimeConfigUpdate,
    SystemConfigRepository, TechnicalAuthConfig, apply_config_update, compact_validation_reason,
    normalize_core_api_url, validate_config,
};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(name = "agentctl", about = "Retaia agent CLI utilities")]
struct Cli {
    #[command(subcommand)]
    command: RootCommand,
}

#[derive(Debug, Subcommand)]
enum RootCommand {
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Path(CommonConfigArgs),
    Show(CommonConfigArgs),
    Validate(ConfigValidateArgs),
    Init(ConfigInitArgs),
    Set(ConfigSetArgs),
}

impl ConfigCommand {
    fn config_path(&self) -> Option<&PathBuf> {
        match self {
            Self::Path(args) | Self::Show(args) => args.config.as_ref(),
            Self::Validate(args) => args.common.config.as_ref(),
            Self::Init(args) => args.common.config.as_ref(),
            Self::Set(args) => args.common.config.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Args)]
struct CommonConfigArgs {
    #[arg(long = "config")]
    config: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
struct ConfigValidateArgs {
    #[command(flatten)]
    common: CommonConfigArgs,
    #[arg(long = "check-respond", default_value_t = false)]
    check_respond: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum AuthModeArg {
    Interactive,
    Technical,
}

impl From<AuthModeArg> for AuthMode {
    fn from(value: AuthModeArg) -> Self {
        match value {
            AuthModeArg::Interactive => AuthMode::Interactive,
            AuthModeArg::Technical => AuthMode::Technical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum LogLevelArg {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevelArg> for LogLevel {
    fn from(value: LogLevelArg) -> Self {
        match value {
            LogLevelArg::Error => LogLevel::Error,
            LogLevelArg::Warn => LogLevel::Warn,
            LogLevelArg::Info => LogLevel::Info,
            LogLevelArg::Debug => LogLevel::Debug,
            LogLevelArg::Trace => LogLevel::Trace,
        }
    }
}

#[derive(Debug, Clone, Args)]
struct ConfigInitArgs {
    #[command(flatten)]
    common: CommonConfigArgs,
    #[arg(long = "core-api-url")]
    core_api_url: String,
    #[arg(long = "ollama-url")]
    ollama_url: String,
    #[arg(long = "auth-mode", value_enum)]
    auth_mode: Option<AuthModeArg>,
    #[arg(long = "client-id")]
    client_id: Option<String>,
    #[arg(long = "secret-key")]
    secret_key: Option<String>,
    #[arg(long = "max-parallel-jobs")]
    max_parallel_jobs: Option<u16>,
    #[arg(long = "log-level", value_enum)]
    log_level: Option<LogLevelArg>,
}

#[derive(Debug, Clone, Args)]
struct ConfigSetArgs {
    #[command(flatten)]
    common: CommonConfigArgs,
    #[arg(long = "core-api-url")]
    core_api_url: Option<String>,
    #[arg(long = "ollama-url")]
    ollama_url: Option<String>,
    #[arg(long = "auth-mode", value_enum)]
    auth_mode: Option<AuthModeArg>,
    #[arg(long = "client-id")]
    client_id: Option<String>,
    #[arg(long = "secret-key")]
    secret_key: Option<String>,
    #[arg(long = "clear-technical-auth")]
    clear_technical_auth: bool,
    #[arg(long = "max-parallel-jobs")]
    max_parallel_jobs: Option<u16>,
    #[arg(long = "log-level", value_enum)]
    log_level: Option<LogLevelArg>,
}

#[derive(Debug, Error)]
enum AgentCtlError {
    #[error("unable to resolve config path: {0}")]
    ResolvePath(ConfigRepositoryError),
    #[error("unable to load config: {0}")]
    Load(ConfigRepositoryError),
    #[error("unable to load current config for set: {0}")]
    LoadCurrentForSet(ConfigRepositoryError),
    #[error("unable to save config: {0}")]
    Save(ConfigRepositoryError),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("invalid config update: {0}")]
    InvalidConfigUpdate(String),
    #[error("config endpoint unreachable ({url}): {reason}")]
    EndpointUnreachable { url: String, reason: String },
    #[error("config endpoint incompatible ({url}): {reason}")]
    EndpointIncompatible { url: String, reason: String },
}

fn print_config(config: &AgentRuntimeConfig) {
    let auth_mode = match config.auth_mode {
        AuthMode::Interactive => "interactive",
        AuthMode::Technical => "technical",
    };
    let log_level = match config.log_level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    };
    println!("core_api_url={}", config.core_api_url);
    println!("ollama_url={}", config.ollama_url);
    println!("auth_mode={auth_mode}");
    println!(
        "technical_client_id={}",
        config
            .technical_auth
            .as_ref()
            .map(|v| v.client_id.as_str())
            .unwrap_or("-")
    );
    println!(
        "technical_secret_key_set={}",
        config.technical_auth.is_some()
    );
    println!("max_parallel_jobs={}", config.max_parallel_jobs);
    println!("log_level={log_level}");
}

fn validation_error(errors: Vec<ConfigValidationError>) -> String {
    compact_validation_reason(&errors)
}

fn http_get(url: &str) -> Result<reqwest::blocking::Response, AgentCtlError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|error| AgentCtlError::EndpointUnreachable {
            url: url.to_string(),
            reason: error.to_string(),
        })?;

    client
        .get(url)
        .send()
        .map_err(|error| AgentCtlError::EndpointUnreachable {
            url: url.to_string(),
            reason: error.to_string(),
        })
}

fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn check_core_api_compatible(core_api_url: &str) -> Result<(), AgentCtlError> {
    let url = join_url(core_api_url, "/jobs");
    let response = http_get(&url)?;
    let status = response.status().as_u16();
    let body = response.text().unwrap_or_default();

    if !matches!(status, 200 | 401 | 403 | 429) {
        return Err(AgentCtlError::EndpointIncompatible {
            url,
            reason: format!("unexpected status {status} on /jobs"),
        });
    }

    let parsed = serde_json::from_str::<serde_json::Value>(&body).map_err(|error| {
        AgentCtlError::EndpointIncompatible {
            url: url.clone(),
            reason: format!("non-JSON response on /jobs: {error}"),
        }
    })?;

    match status {
        200 if parsed.is_array() => Ok(()),
        401 | 403 | 429 if parsed.is_object() => Ok(()),
        200 => Err(AgentCtlError::EndpointIncompatible {
            url,
            reason: "expected JSON array on /jobs (status 200)".to_string(),
        }),
        _ => Err(AgentCtlError::EndpointIncompatible {
            url,
            reason: "expected JSON object error payload on /jobs".to_string(),
        }),
    }
}

fn check_ollama_api_compatible(ollama_url: &str) -> Result<(), AgentCtlError> {
    let endpoint = normalize_ollama_openai_base_url(ollama_url);
    let endpoint_for_resolver = endpoint.clone();
    let endpoint_for_error = endpoint.clone();
    let target_resolver = ServiceTargetResolver::from_resolver_fn(move |target: ServiceTarget| {
        let ServiceTarget { auth, model, .. } = target;
        Ok(ServiceTarget {
            endpoint: Endpoint::from_owned(endpoint_for_resolver.clone()),
            auth,
            model: ModelIden::new(AdapterKind::Ollama, model.model_name),
        })
    });

    let client = Client::builder()
        .with_web_config(WebConfig::default().with_timeout(Duration::from_secs(3)))
        .with_service_target_resolver(target_resolver)
        .build();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| AgentCtlError::EndpointUnreachable {
            url: endpoint.clone(),
            reason: error.to_string(),
        })?;

    let request = ChatRequest::new(vec![ChatMessage::user("compatibility check")]);
    match runtime.block_on(client.exec_chat("retaia-compat-check", request, None)) {
        Ok(_) => Ok(()),
        Err(genai::Error::WebModelCall { webc_error, .. }) => {
            map_ollama_genai_error(&endpoint_for_error, webc_error)
        }
        Err(error) => Err(AgentCtlError::EndpointIncompatible {
            url: endpoint_for_error,
            reason: error.to_string(),
        }),
    }
}

fn normalize_ollama_openai_base_url(ollama_url: &str) -> String {
    let trimmed = ollama_url.trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        format!("{trimmed}/")
    } else if trimmed.ends_with("/api") {
        let root = trimmed.trim_end_matches("/api");
        format!("{root}/v1/")
    } else {
        format!("{trimmed}/v1/")
    }
}

fn map_ollama_genai_error(url: &str, error: genai::webc::Error) -> Result<(), AgentCtlError> {
    match error {
        genai::webc::Error::Reqwest(source) => Err(AgentCtlError::EndpointUnreachable {
            url: url.to_string(),
            reason: source.to_string(),
        }),
        genai::webc::Error::ResponseFailedStatus { status, body, .. } => {
            if body.trim().is_empty() {
                return Err(AgentCtlError::EndpointIncompatible {
                    url: url.to_string(),
                    reason: format!("unexpected HTTP {status} with empty body"),
                });
            }

            let parsed =
                serde_json::from_str::<serde_json::Value>(&body).map_err(|parse_error| {
                    AgentCtlError::EndpointIncompatible {
                        url: url.to_string(),
                        reason: format!(
                            "HTTP {status} returned non-JSON error body: {parse_error}"
                        ),
                    }
                })?;

            if parsed.is_object() {
                Ok(())
            } else {
                Err(AgentCtlError::EndpointIncompatible {
                    url: url.to_string(),
                    reason: format!("HTTP {status} returned non-object JSON body"),
                })
            }
        }
        other => Err(AgentCtlError::EndpointIncompatible {
            url: url.to_string(),
            reason: other.to_string(),
        }),
    }
}

fn check_config_respond(config: &AgentRuntimeConfig) -> Result<(), AgentCtlError> {
    check_core_api_compatible(&config.core_api_url)?;
    check_ollama_api_compatible(&config.ollama_url)
}

fn init_config(args: &ConfigInitArgs) -> Result<AgentRuntimeConfig, AgentCtlError> {
    let auth_mode = args.auth_mode.unwrap_or(AuthModeArg::Interactive).into();
    let technical_auth = match auth_mode {
        AuthMode::Interactive => None,
        AuthMode::Technical => Some(TechnicalAuthConfig {
            client_id: args.client_id.clone().unwrap_or_default(),
            secret_key: args.secret_key.clone().unwrap_or_default(),
        }),
    };

    let config = AgentRuntimeConfig {
        core_api_url: normalize_core_api_url(&args.core_api_url),
        ollama_url: args.ollama_url.clone(),
        auth_mode,
        technical_auth,
        max_parallel_jobs: args.max_parallel_jobs.unwrap_or(1),
        log_level: args.log_level.unwrap_or(LogLevelArg::Info).into(),
    };

    validate_config(&config)
        .map_err(validation_error)
        .map_err(AgentCtlError::InvalidConfig)?;
    Ok(config)
}

fn update_from_args(args: &ConfigSetArgs) -> RuntimeConfigUpdate {
    RuntimeConfigUpdate {
        core_api_url: args.core_api_url.clone(),
        ollama_url: args.ollama_url.clone(),
        auth_mode: args.auth_mode.map(Into::into),
        technical_client_id: args.client_id.clone(),
        technical_secret_key: args.secret_key.clone(),
        clear_technical_auth: args.clear_technical_auth,
        max_parallel_jobs: args.max_parallel_jobs,
        log_level: args.log_level.map(Into::into),
    }
}

fn run_with_repository<R: ConfigRepository>(
    repository: &R,
    command: ConfigCommand,
) -> Result<(), AgentCtlError> {
    match command {
        ConfigCommand::Path(_) => {
            let path = repository
                .config_path()
                .map_err(AgentCtlError::ResolvePath)?;
            println!("{}", path.display());
            Ok(())
        }
        ConfigCommand::Show(_) => {
            let config = repository.load().map_err(AgentCtlError::Load)?;
            print_config(&config);
            Ok(())
        }
        ConfigCommand::Validate(args) => {
            let config = repository.load().map_err(AgentCtlError::Load)?;
            validate_config(&config)
                .map_err(validation_error)
                .map_err(AgentCtlError::InvalidConfig)?;
            if args.check_respond {
                check_config_respond(&config)?;
            }
            println!("Config is valid.");
            Ok(())
        }
        ConfigCommand::Init(args) => {
            let config = init_config(&args)?;
            repository.save(&config).map_err(AgentCtlError::Save)?;
            println!("Config initialized.");
            Ok(())
        }
        ConfigCommand::Set(args) => {
            let current = repository
                .load()
                .map_err(AgentCtlError::LoadCurrentForSet)?;
            let next =
                apply_config_update(&current, &update_from_args(&args), ConfigInterface::Cli)
                    .map_err(validation_error)
                    .map_err(AgentCtlError::InvalidConfigUpdate)?;
            repository.save(&next).map_err(AgentCtlError::Save)?;
            println!("Config updated.");
            Ok(())
        }
    }
}

fn run(cli: Cli) -> Result<(), AgentCtlError> {
    match cli.command {
        RootCommand::Config { command } => match command.config_path().cloned() {
            Some(path) => run_with_repository(&FileConfigRepository::new(path), command),
            None => run_with_repository(&SystemConfigRepository, command),
        },
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("{err}");
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, ConfigCommand, LogLevelArg, RootCommand};

    #[test]
    fn tdd_clap_parses_set_with_partial_update() {
        let cli = Cli::try_parse_from([
            "agentctl",
            "config",
            "set",
            "--max-parallel-jobs",
            "8",
            "--log-level",
            "debug",
        ])
        .expect("set args should parse");

        match cli.command {
            RootCommand::Config {
                command: ConfigCommand::Set(args),
            } => {
                assert_eq!(args.max_parallel_jobs, Some(8));
                assert_eq!(args.log_level, Some(LogLevelArg::Debug));
            }
            _ => panic!("unexpected parse result"),
        }
    }

    #[test]
    fn tdd_clap_rejects_unknown_subcommand() {
        let err = Cli::try_parse_from(["agentctl", "config", "unknown"])
            .expect_err("unknown command must fail");
        let message = err.to_string();
        assert!(message.contains("unrecognized subcommand"));
    }
}
