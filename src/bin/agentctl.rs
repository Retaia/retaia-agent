use std::env;
use std::path::PathBuf;
use std::process::exit;

use retaia_agent::{
    AgentRuntimeConfig, AuthMode, ConfigInterface, ConfigRepository, FileConfigRepository,
    LogLevel, RuntimeConfigUpdate, SystemConfigRepository, TechnicalAuthConfig,
    apply_config_update, compact_validation_reason, validate_config,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandKind {
    Path,
    Show,
    Validate,
    Init,
    Set,
}

#[derive(Debug, Clone)]
struct ParsedArgs {
    command: CommandKind,
    config_path: Option<PathBuf>,
    core_api_url: Option<String>,
    ollama_url: Option<String>,
    auth_mode: Option<AuthMode>,
    client_id: Option<String>,
    secret_key: Option<String>,
    clear_technical_auth: bool,
    max_parallel_jobs: Option<u16>,
    log_level: Option<LogLevel>,
}

impl Default for ParsedArgs {
    fn default() -> Self {
        Self {
            command: CommandKind::Show,
            config_path: None,
            core_api_url: None,
            ollama_url: None,
            auth_mode: None,
            client_id: None,
            secret_key: None,
            clear_technical_auth: false,
            max_parallel_jobs: None,
            log_level: None,
        }
    }
}

fn parse_auth_mode(value: &str) -> Result<AuthMode, String> {
    match value {
        "interactive" => Ok(AuthMode::Interactive),
        "technical" => Ok(AuthMode::Technical),
        _ => Err(format!("Invalid --auth-mode value: {value}")),
    }
}

fn parse_log_level(value: &str) -> Result<LogLevel, String> {
    match value {
        "error" => Ok(LogLevel::Error),
        "warn" => Ok(LogLevel::Warn),
        "info" => Ok(LogLevel::Info),
        "debug" => Ok(LogLevel::Debug),
        "trace" => Ok(LogLevel::Trace),
        _ => Err(format!("Invalid --log-level value: {value}")),
    }
}

fn parse_u16(value: &str, flag: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .map_err(|_| format!("Invalid {flag} value: {value}"))
}

fn expect_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    let Some(value) = args.get(*index) else {
        return Err(format!("Missing value for {flag}"));
    };
    Ok(value.clone())
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, String> {
    if args.len() < 2 || args[0] != "config" {
        return Err(usage());
    }

    let mut parsed = ParsedArgs::default();
    parsed.command = match args[1].as_str() {
        "path" => CommandKind::Path,
        "show" => CommandKind::Show,
        "validate" => CommandKind::Validate,
        "init" => CommandKind::Init,
        "set" => CommandKind::Set,
        _ => return Err(usage()),
    };

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                let value = expect_value(args, &mut i, "--config")?;
                parsed.config_path = Some(PathBuf::from(value));
            }
            "--core-api-url" => {
                parsed.core_api_url = Some(expect_value(args, &mut i, "--core-api-url")?);
            }
            "--ollama-url" => {
                parsed.ollama_url = Some(expect_value(args, &mut i, "--ollama-url")?);
            }
            "--auth-mode" => {
                let value = expect_value(args, &mut i, "--auth-mode")?;
                parsed.auth_mode = Some(parse_auth_mode(&value)?);
            }
            "--client-id" => {
                parsed.client_id = Some(expect_value(args, &mut i, "--client-id")?);
            }
            "--secret-key" => {
                parsed.secret_key = Some(expect_value(args, &mut i, "--secret-key")?);
            }
            "--clear-technical-auth" => {
                parsed.clear_technical_auth = true;
            }
            "--max-parallel-jobs" => {
                let value = expect_value(args, &mut i, "--max-parallel-jobs")?;
                parsed.max_parallel_jobs = Some(parse_u16(&value, "--max-parallel-jobs")?);
            }
            "--log-level" => {
                let value = expect_value(args, &mut i, "--log-level")?;
                parsed.log_level = Some(parse_log_level(&value)?);
            }
            unknown => return Err(format!("Unknown option: {unknown}\n\n{}", usage())),
        }
        i += 1;
    }

    Ok(parsed)
}

fn usage() -> String {
    [
        "Usage:",
        "  agentctl config path [--config PATH]",
        "  agentctl config show [--config PATH]",
        "  agentctl config validate [--config PATH]",
        "  agentctl config init --core-api-url URL --ollama-url URL [--auth-mode interactive|technical] [--client-id ID --secret-key KEY] [--max-parallel-jobs N] [--log-level error|warn|info|debug|trace] [--config PATH]",
        "  agentctl config set [--core-api-url URL] [--ollama-url URL] [--auth-mode interactive|technical] [--client-id ID] [--secret-key KEY] [--clear-technical-auth] [--max-parallel-jobs N] [--log-level error|warn|info|debug|trace] [--config PATH]",
    ]
    .join("\n")
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

fn initial_config_from_args(parsed: &ParsedArgs) -> Result<AgentRuntimeConfig, String> {
    let core_api_url = parsed
        .core_api_url
        .clone()
        .ok_or("--core-api-url is required for config init".to_string())?;
    let ollama_url = parsed
        .ollama_url
        .clone()
        .ok_or("--ollama-url is required for config init".to_string())?;

    let auth_mode = parsed.auth_mode.unwrap_or(AuthMode::Interactive);
    let technical_auth = match auth_mode {
        AuthMode::Interactive => None,
        AuthMode::Technical => Some(TechnicalAuthConfig {
            client_id: parsed
                .client_id
                .clone()
                .ok_or("--client-id is required in technical auth mode".to_string())?,
            secret_key: parsed
                .secret_key
                .clone()
                .ok_or("--secret-key is required in technical auth mode".to_string())?,
        }),
    };

    Ok(AgentRuntimeConfig {
        core_api_url,
        ollama_url,
        auth_mode,
        technical_auth,
        max_parallel_jobs: parsed.max_parallel_jobs.unwrap_or(1),
        log_level: parsed.log_level.unwrap_or(LogLevel::Info),
    })
}

fn update_from_args(parsed: &ParsedArgs) -> RuntimeConfigUpdate {
    RuntimeConfigUpdate {
        core_api_url: parsed.core_api_url.clone(),
        ollama_url: parsed.ollama_url.clone(),
        auth_mode: parsed.auth_mode,
        technical_client_id: parsed.client_id.clone(),
        technical_secret_key: parsed.secret_key.clone(),
        clear_technical_auth: parsed.clear_technical_auth,
        max_parallel_jobs: parsed.max_parallel_jobs,
        log_level: parsed.log_level,
    }
}

fn run_with_repository<R: ConfigRepository>(
    repository: &R,
    parsed: ParsedArgs,
) -> Result<(), String> {
    match parsed.command {
        CommandKind::Path => {
            let path = repository
                .config_path()
                .map_err(|e| format!("Unable to resolve config path: {e:?}"))?;
            println!("{}", path.display());
            Ok(())
        }
        CommandKind::Show => {
            let config = repository
                .load()
                .map_err(|e| format!("Unable to load config: {e:?}"))?;
            print_config(&config);
            Ok(())
        }
        CommandKind::Validate => {
            let config = repository
                .load()
                .map_err(|e| format!("Unable to load config: {e:?}"))?;
            validate_config(&config).map_err(|errors| {
                format!("Invalid config: {}", compact_validation_reason(&errors))
            })?;
            println!("Config is valid.");
            Ok(())
        }
        CommandKind::Init => {
            let config = initial_config_from_args(&parsed)?;
            validate_config(&config).map_err(|errors| {
                format!("Invalid config: {}", compact_validation_reason(&errors))
            })?;
            repository
                .save(&config)
                .map_err(|e| format!("Unable to save config: {e:?}"))?;
            println!("Config initialized.");
            Ok(())
        }
        CommandKind::Set => {
            let current = repository
                .load()
                .map_err(|e| format!("Unable to load current config for set: {e:?}"))?;
            let next =
                apply_config_update(&current, &update_from_args(&parsed), ConfigInterface::Cli)
                    .map_err(|errors| {
                        format!(
                            "Invalid config update: {}",
                            compact_validation_reason(&errors)
                        )
                    })?;
            repository
                .save(&next)
                .map_err(|e| format!("Unable to save config: {e:?}"))?;
            println!("Config updated.");
            Ok(())
        }
    }
}

fn run(parsed: ParsedArgs) -> Result<(), String> {
    match parsed.config_path.clone() {
        Some(path) => run_with_repository(&FileConfigRepository::new(path), parsed),
        None => run_with_repository(&SystemConfigRepository, parsed),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let parsed = match parse_args(&args) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

    if let Err(err) = run(parsed) {
        eprintln!("{err}");
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandKind, parse_args};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn tdd_parse_set_with_partial_update() {
        let parsed = parse_args(&args(&[
            "config",
            "set",
            "--max-parallel-jobs",
            "8",
            "--log-level",
            "debug",
        ]))
        .expect("set args should parse");

        assert_eq!(parsed.command, CommandKind::Set);
        assert_eq!(parsed.max_parallel_jobs, Some(8));
        assert_eq!(parsed.log_level, Some(retaia_agent::LogLevel::Debug));
    }

    #[test]
    fn tdd_parse_requires_known_subcommand() {
        let err = parse_args(&args(&["config", "unknown"])).expect_err("should fail");
        assert!(err.contains("Usage"));
    }
}
