use std::env;
use std::process::{Command, exit};

fn run(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("failed to execute {command}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "command failed: {} {}{}",
            command,
            args.join(" "),
            if stderr.is_empty() {
                String::new()
            } else {
                format!(" ({stderr})")
            }
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_inherit(command: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(command)
        .args(args)
        .status()
        .map_err(|e| format!("failed to execute {command}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {} {}", command, args.join(" ")))
    }
}

fn main() {
    let event_name = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    let base_ref = if event_name == "pull_request" {
        env::var("GITHUB_BASE_REF").ok()
    } else {
        env::var("BASE_BRANCH")
            .ok()
            .or_else(|| Some("master".to_string()))
    };
    let head_ref = env::var("GITHUB_HEAD_REF").ok();

    let Some(base_ref) = base_ref else {
        eprintln!("Missing base branch reference.");
        exit(1);
    };

    if let Err(err) = run_inherit("git", &["fetch", "--no-tags", "origin", &base_ref]) {
        eprintln!("Failed to fetch origin/{base_ref}: {err}");
        exit(1);
    }

    if event_name == "pull_request" {
        if let Some(head_ref) = &head_ref {
            if let Err(err) = run_inherit("git", &["fetch", "--no-tags", "origin", head_ref]) {
                eprintln!("Failed to fetch origin/{head_ref}: {err}");
                exit(1);
            }
        }
    }

    let base_head = match run("git", &["rev-parse", &format!("origin/{base_ref}")]) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

    let head = if event_name == "pull_request" {
        if let Some(head_ref) = &head_ref {
            match run("git", &["rev-parse", &format!("origin/{head_ref}")]) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{err}");
                    exit(1);
                }
            }
        } else {
            match run("git", &["rev-parse", "HEAD"]) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{err}");
                    exit(1);
                }
            }
        }
    } else {
        match run("git", &["rev-parse", "HEAD"]) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("{err}");
                exit(1);
            }
        }
    };

    let merge_base = match run("git", &["merge-base", &head, &base_head]) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

    if merge_base != base_head {
        eprintln!("Branch is behind origin/{base_ref}.");
        eprintln!("Expected merge-base {base_head}, got {merge_base}.");
        eprintln!("Please rebase on the latest base branch.");
        exit(1);
    }

    let merge_commits_raw = run(
        "git",
        &[
            "rev-list",
            "--merges",
            &format!("origin/{base_ref}..{head}"),
        ],
    )
    .unwrap_or_default();
    let merge_commits: Vec<&str> = merge_commits_raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if !merge_commits.is_empty() {
        eprintln!("Linear history required: merge commits found in branch.");
        for sha in &merge_commits {
            let subject = run("git", &["show", "-s", "--format=%s", sha]).unwrap_or_default();
            eprintln!("- {sha} {subject}");
        }
        eprintln!("Please rebase and remove merge commits before pushing.");
        exit(1);
    }

    println!("Branch is up to date with origin/{base_ref} and has linear history.");
}
