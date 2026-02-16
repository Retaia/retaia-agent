use std::env;
use std::process::{Command, exit};

use thiserror::Error;

#[derive(Debug, Error)]
enum BranchCheckError {
    #[error("missing base branch reference")]
    MissingBaseBranch,
    #[error("failed to execute {command}: {source}")]
    CommandExec {
        command: String,
        source: std::io::Error,
    },
    #[error("command failed: {command} {args}{stderr}")]
    CommandFailed {
        command: String,
        args: String,
        stderr: String,
    },
    #[error("branch is behind origin/{base_ref}; expected merge-base {expected}, got {actual}")]
    BehindBase {
        base_ref: String,
        expected: String,
        actual: String,
    },
    #[error("linear history required: merge commits found in branch")]
    MergeCommitsFound,
}

fn run_capture(command: &str, args: &[&str]) -> Result<String, BranchCheckError> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|source| BranchCheckError::CommandExec {
            command: command.to_string(),
            source,
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(BranchCheckError::CommandFailed {
            command: command.to_string(),
            args: args.join(" "),
            stderr: if stderr.is_empty() {
                String::new()
            } else {
                format!(" ({stderr})")
            },
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_status(command: &str, args: &[&str]) -> Result<(), BranchCheckError> {
    let status = Command::new(command)
        .args(args)
        .status()
        .map_err(|source| BranchCheckError::CommandExec {
            command: command.to_string(),
            source,
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(BranchCheckError::CommandFailed {
            command: command.to_string(),
            args: args.join(" "),
            stderr: String::new(),
        })
    }
}

fn run() -> Result<(), BranchCheckError> {
    let event_name = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    let base_ref = if event_name == "pull_request" {
        env::var("GITHUB_BASE_REF").ok()
    } else {
        env::var("BASE_BRANCH")
            .ok()
            .or_else(|| Some("master".to_string()))
    }
    .ok_or(BranchCheckError::MissingBaseBranch)?;

    let head_ref = env::var("GITHUB_HEAD_REF").ok();

    run_status("git", &["fetch", "--no-tags", "origin", &base_ref])?;
    if event_name == "pull_request" {
        if let Some(head_ref) = &head_ref {
            run_status("git", &["fetch", "--no-tags", "origin", head_ref])?;
        }
    }

    let base_head = run_capture("git", &["rev-parse", &format!("origin/{base_ref}")])?;
    let head = if event_name == "pull_request" {
        if let Some(head_ref) = &head_ref {
            run_capture("git", &["rev-parse", &format!("origin/{head_ref}")])?
        } else {
            run_capture("git", &["rev-parse", "HEAD"])?
        }
    } else {
        run_capture("git", &["rev-parse", "HEAD"])?
    };

    let merge_base = run_capture("git", &["merge-base", &head, &base_head])?;
    if merge_base != base_head {
        return Err(BranchCheckError::BehindBase {
            base_ref,
            expected: base_head,
            actual: merge_base,
        });
    }

    let merge_commits_raw = run_capture(
        "git",
        &[
            "rev-list",
            "--merges",
            &format!("origin/{base_ref}..{head}"),
        ],
    )?;
    let merge_commits: Vec<&str> = merge_commits_raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if !merge_commits.is_empty() {
        eprintln!("Linear history required: merge commits found in branch.");
        for sha in &merge_commits {
            let subject =
                run_capture("git", &["show", "-s", "--format=%s", sha]).unwrap_or_default();
            eprintln!("- {sha} {subject}");
        }
        return Err(BranchCheckError::MergeCommitsFound);
    }

    println!("Branch is up to date with origin/{base_ref} and has linear history.");
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        if let BranchCheckError::BehindBase {
            base_ref,
            expected,
            actual,
        } = &err
        {
            eprintln!("Expected merge-base {expected}, got {actual}.");
            eprintln!("Please rebase on the latest origin/{base_ref}.");
        }
        exit(1);
    }
}
