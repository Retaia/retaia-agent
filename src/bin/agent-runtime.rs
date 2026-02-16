use std::io::{self, Write};
use std::path::PathBuf;
use std::process::exit;

use clap::{Parser, ValueEnum};
use retaia_agent::{
    AgentRuntimeConfig, ClientRuntimeTarget, ConfigRepository, FileConfigRepository,
    RuntimeSession, SystemConfigRepository, compact_validation_reason, execute_shell_command,
    format_menu, help_text, parse_shell_command,
};

#[derive(Debug, Parser)]
#[command(name = "agent-runtime", about = "Retaia runtime interactive shell")]
struct Cli {
    #[arg(long = "config")]
    config: Option<PathBuf>,
    #[arg(long = "target", value_enum, default_value_t = TargetArg::Agent)]
    target: TargetArg,
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
    let mut session = RuntimeSession::new(cli.target.into(), settings).map_err(|errors| {
        format!(
            "invalid runtime config: {}",
            compact_validation_reason(&errors)
        )
    })?;

    println!("{}", help_text());
    print!("{}", format_menu(&session));

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

        let result = execute_shell_command(&mut session, parse_shell_command(&line));
        if !result.output.is_empty() {
            print!("{}", result.output);
        }
        if result.should_exit {
            break;
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        exit(1);
    }
}
