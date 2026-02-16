use std::fs;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(name = "check_coverage", about = "Validate line coverage threshold")]
struct Cli {
    #[arg(long = "file", default_value = "coverage/llvm-cov-summary.json")]
    file: PathBuf,
    #[arg(long = "min", default_value_t = 80.0)]
    min: f64,
}

#[derive(Debug, Error)]
enum CoverageError {
    #[error("unable to read coverage summary at {path}: {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },
    #[error("invalid JSON in coverage summary {path}: {source}")]
    InvalidJson {
        path: String,
        source: serde_json::Error,
    },
    #[error(
        "invalid coverage summary format. Expected one of: total.lines.pct, totals.lines.percent, data[0].totals.lines.percent"
    )]
    InvalidFormat,
    #[error("coverage check failed: {coverage}% < {min}% (minimum required)")]
    BelowThreshold { coverage: f64, min: f64 },
}

fn read_coverage_percent(summary: &Value) -> Option<f64> {
    summary
        .get("total")
        .and_then(|v| v.get("lines"))
        .and_then(|v| v.get("pct"))
        .and_then(Value::as_f64)
        .or_else(|| {
            summary
                .get("totals")
                .and_then(|v| v.get("lines"))
                .and_then(|v| v.get("percent"))
                .and_then(Value::as_f64)
        })
        .or_else(|| {
            summary
                .get("data")
                .and_then(Value::as_array)
                .and_then(|arr| arr.first())
                .and_then(|v| v.get("totals"))
                .and_then(|v| v.get("lines"))
                .and_then(|v| v.get("percent"))
                .and_then(Value::as_f64)
        })
}

fn run(cli: &Cli) -> Result<f64, CoverageError> {
    let path = cli.file.display().to_string();
    let content = fs::read_to_string(&cli.file).map_err(|source| CoverageError::ReadFile {
        path: path.clone(),
        source,
    })?;

    let summary: Value =
        serde_json::from_str(&content).map_err(|source| CoverageError::InvalidJson {
            path: path.clone(),
            source,
        })?;

    let coverage = read_coverage_percent(&summary).ok_or(CoverageError::InvalidFormat)?;
    if coverage < cli.min {
        return Err(CoverageError::BelowThreshold {
            coverage,
            min: cli.min,
        });
    }

    Ok(coverage)
}

fn main() {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(coverage) => println!("Coverage check passed: {coverage}% >= {}%.", cli.min),
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    }
}
