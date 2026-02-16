use std::env;
use std::fs;
use std::process::exit;

use serde_json::Value;

fn parse_args() -> Result<(String, f64), String> {
    let mut file = String::from("coverage/llvm-cov-summary.json");
    let mut min = 80.0_f64;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => {
                let Some(v) = args.next() else {
                    return Err("Missing value for --file".to_string());
                };
                file = v;
            }
            "--min" => {
                let Some(v) = args.next() else {
                    return Err("Missing value for --min".to_string());
                };
                min = v
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid --min value: {v}"))?;
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }

    Ok((file, min))
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

fn main() {
    let (file, min) = match parse_args() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

    let content = match fs::read_to_string(&file) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Unable to read coverage summary at {file}: {err}");
            exit(1);
        }
    };

    let summary: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Invalid JSON in coverage summary {file}: {err}");
            exit(1);
        }
    };

    let Some(coverage) = read_coverage_percent(&summary) else {
        eprintln!(
            "Invalid coverage summary format. Expected one of: total.lines.pct, totals.lines.percent, data[0].totals.lines.percent"
        );
        exit(1);
    };

    if coverage < min {
        eprintln!("Coverage check failed: {coverage}% < {min}% (minimum required).");
        exit(1);
    }

    println!("Coverage check passed: {coverage}% >= {min}%.");
}
