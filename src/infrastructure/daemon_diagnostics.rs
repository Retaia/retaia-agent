use std::fmt::Write as _;
use std::io::Write as _;
use std::process::{Command, Stdio};

use thiserror::Error;

use crate::application::daemon_manager::{
    DaemonLabelRequest, DaemonLevel, DaemonManager, DaemonStatus,
};
use crate::infrastructure::runtime_history_store::{
    CompletedJobEntry, DaemonCycleEntry, RuntimeHistoryStore,
};
use crate::infrastructure::runtime_stats_store::{DaemonRuntimeStats, load_runtime_stats};

pub const DEFAULT_DAEMON_LABEL: &str = "io.retaia.agent";

#[derive(Debug, Clone)]
pub struct DaemonDiagnosticsSnapshot {
    pub daemon_status: Option<DaemonStatus>,
    pub stats: Option<DaemonRuntimeStats>,
    pub completed_jobs: Vec<CompletedJobEntry>,
    pub cycles: Vec<DaemonCycleEntry>,
}

#[derive(Debug, Clone)]
pub struct BugReportMarkdown {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Copy)]
pub struct DiagnosticsLimits {
    pub history_limit: usize,
    pub cycles_limit: usize,
}

impl Default for DiagnosticsLimits {
    fn default() -> Self {
        Self {
            history_limit: 50,
            cycles_limit: 120,
        }
    }
}

pub fn daemon_status_as_label(status: Option<&DaemonStatus>) -> &'static str {
    match status {
        Some(DaemonStatus::Running) => "running",
        Some(DaemonStatus::NotInstalled) => "not_installed",
        Some(DaemonStatus::Stopped(_)) => "stopped",
        None => "unknown",
    }
}

pub fn collect_daemon_diagnostics<M: DaemonManager>(
    manager: &M,
    limits: DiagnosticsLimits,
) -> DaemonDiagnosticsSnapshot {
    let daemon_status = manager
        .status(DaemonLabelRequest {
            label: DEFAULT_DAEMON_LABEL.to_string(),
            level: DaemonLevel::User,
        })
        .ok();

    let stats = load_runtime_stats().ok();

    let history_store = RuntimeHistoryStore::open_default().ok();
    let completed_jobs = match history_store.as_ref() {
        Some(store) => store
            .recent_completed_jobs(limits.history_limit.max(1))
            .unwrap_or_default(),
        None => Vec::new(),
    };
    let cycles = match history_store.as_ref() {
        Some(store) => store
            .recent_cycles(limits.cycles_limit.max(1))
            .unwrap_or_default(),
        None => Vec::new(),
    };

    DaemonDiagnosticsSnapshot {
        daemon_status,
        stats,
        completed_jobs,
        cycles,
    }
}

pub fn build_bug_report_markdown(
    snapshot: &DaemonDiagnosticsSnapshot,
    title: Option<&str>,
    stats_file_name: &str,
    history_db_path: Option<&str>,
) -> BugReportMarkdown {
    let title = title
        .map(ToString::to_string)
        .unwrap_or_else(|| "Retaia agent daemon bug report".to_string());

    let mut body = String::new();
    let _ = writeln!(body, "## Context");
    let _ = writeln!(
        body,
        "- daemon_status: `{}`",
        daemon_status_as_label(snapshot.daemon_status.as_ref())
    );
    let _ = writeln!(body, "- stats_file: `{stats_file_name}`");
    let _ = writeln!(
        body,
        "- history_db: `{}`",
        history_db_path.unwrap_or("unavailable")
    );

    if let Some(stats) = snapshot.stats.as_ref() {
        let _ = writeln!(body, "- run_state: `{}`", stats.run_state);
        let _ = writeln!(body, "- daemon_tick: `{}`", stats.tick);
        if let Some(job) = stats.current_job.as_ref() {
            let _ = writeln!(body, "- current_job_id: `{}`", job.job_id);
            let _ = writeln!(body, "- current_asset_uuid: `{}`", job.asset_uuid);
            let _ = writeln!(body, "- current_progress: `{}`", job.progress_percent);
            let _ = writeln!(body, "- current_stage: `{}`", job.stage);
        }
        if let Some(last) = stats.last_job.as_ref() {
            let _ = writeln!(body, "- last_job_id: `{}`", last.job_id);
            let _ = writeln!(body, "- last_job_duration_ms: `{}`", last.duration_ms);
        }
    }

    let _ = writeln!(body, "\n## Recent Completed Jobs");
    if snapshot.completed_jobs.is_empty() {
        let _ = writeln!(body, "- none");
    } else {
        for row in snapshot.completed_jobs.as_slice() {
            let _ = writeln!(
                body,
                "- completed_at={} job_id=`{}` duration_ms={}",
                row.completed_at_unix_ms, row.job_id, row.duration_ms
            );
        }
    }

    let _ = writeln!(body, "\n## Recent Runtime Cycles");
    if snapshot.cycles.is_empty() {
        let _ = writeln!(body, "- none");
    } else {
        for row in snapshot.cycles.as_slice() {
            let _ = writeln!(
                body,
                "- ts={} tick={} outcome=`{}` run_state=`{}` job_id=`{}` progress=`{}` stage=`{}`",
                row.ts_unix_ms,
                row.tick,
                row.outcome,
                row.run_state,
                row.job_id.as_deref().unwrap_or("-"),
                row.progress_percent
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                row.stage.as_deref().unwrap_or("-"),
            );
        }
    }

    let _ = writeln!(body, "\n## Notes");
    let _ = writeln!(body, "- Attach steps to reproduce and expected behavior.");

    BugReportMarkdown { title, body }
}

pub fn render_daemon_inspect(
    snapshot: &DaemonDiagnosticsSnapshot,
    history_db_path: Option<&str>,
) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "daemon_status={}",
        daemon_status_as_label(snapshot.daemon_status.as_ref())
    );
    let _ = writeln!(out, "stats_file={}", crate::DAEMON_STATS_FILE_NAME);
    let _ = writeln!(
        out,
        "history_db_path={}",
        history_db_path.unwrap_or("unavailable")
    );

    if let Some(stats) = snapshot.stats.as_ref() {
        let _ = writeln!(out, "updated_at_unix_ms={}", stats.updated_at_unix_ms);
        let _ = writeln!(out, "run_state={}", stats.run_state);
        let _ = writeln!(out, "tick={}", stats.tick);
        if let Some(job) = stats.current_job.as_ref() {
            let _ = writeln!(out, "current_job_id={}", job.job_id);
            let _ = writeln!(out, "current_asset_uuid={}", job.asset_uuid);
            let _ = writeln!(out, "current_progress_percent={}", job.progress_percent);
            let _ = writeln!(out, "current_stage={}", job.stage);
            let _ = writeln!(out, "current_status={}", job.status);
            let _ = writeln!(out, "current_started_at_unix_ms={}", job.started_at_unix_ms);
        } else {
            let _ = writeln!(out, "current_job_id=-");
            let _ = writeln!(out, "current_asset_uuid=-");
            let _ = writeln!(out, "current_progress_percent=-");
            let _ = writeln!(out, "current_stage=-");
            let _ = writeln!(out, "current_status=idle");
            let _ = writeln!(out, "current_started_at_unix_ms=-");
        }

        if let Some(last) = stats.last_job.as_ref() {
            let _ = writeln!(out, "last_job_id={}", last.job_id);
            let _ = writeln!(out, "last_job_duration_ms={}", last.duration_ms);
            let _ = writeln!(
                out,
                "last_job_completed_at_unix_ms={}",
                last.completed_at_unix_ms
            );
        } else {
            let _ = writeln!(out, "last_job_id=-");
            let _ = writeln!(out, "last_job_duration_ms=-");
            let _ = writeln!(out, "last_job_completed_at_unix_ms=-");
        }
    } else {
        let _ = writeln!(out, "stats=unavailable");
    }

    let _ = writeln!(
        out,
        "completed_jobs_count={}",
        snapshot.completed_jobs.len()
    );
    for row in snapshot.completed_jobs.as_slice() {
        let _ = writeln!(
            out,
            "completed_at_unix_ms={} job_id={} duration_ms={}",
            row.completed_at_unix_ms, row.job_id, row.duration_ms
        );
    }

    let _ = writeln!(out, "cycles_count={}", snapshot.cycles.len());
    for row in snapshot.cycles.as_slice() {
        let _ = writeln!(
            out,
            "ts_unix_ms={} tick={} outcome={} run_state={} job_id={} asset_uuid={} progress_percent={} stage={} status={}",
            row.ts_unix_ms,
            row.tick,
            row.outcome,
            row.run_state,
            row.job_id.as_deref().unwrap_or("-"),
            row.asset_uuid.as_deref().unwrap_or("-"),
            row.progress_percent
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            row.stage.as_deref().unwrap_or("-"),
            row.short_status.as_deref().unwrap_or("-"),
        );
    }

    out
}

#[derive(Debug, Error)]
pub enum ClipboardCopyError {
    #[error("clipboard command failed: {0}")]
    Command(String),
    #[error("clipboard copy is not supported on this platform")]
    Unsupported,
}

pub fn copy_to_clipboard(content: &str) -> Result<(), ClipboardCopyError> {
    #[cfg(target_os = "macos")]
    {
        run_clipboard_command("pbcopy", &[], content)?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        run_clipboard_command("clip", &[], content)?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if run_clipboard_command("wl-copy", &[], content).is_ok() {
            return Ok(());
        }
        if run_clipboard_command("xclip", &["-selection", "clipboard"], content).is_ok() {
            return Ok(());
        }
        return Err(ClipboardCopyError::Command(
            "no clipboard command available (tried wl-copy, xclip)".to_string(),
        ));
    }

    #[allow(unreachable_code)]
    Err(ClipboardCopyError::Unsupported)
}

fn run_clipboard_command(
    program: &str,
    args: &[&str],
    content: &str,
) -> Result<(), ClipboardCopyError> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| ClipboardCopyError::Command(format!("{program}: {error}")))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|error| ClipboardCopyError::Command(format!("{program}: {error}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| ClipboardCopyError::Command(format!("{program}: {error}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(ClipboardCopyError::Command(format!(
            "{program} exited with status {}",
            output.status
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CompletedJobEntry, DaemonCycleEntry, DaemonRuntimeStats,
        infrastructure::runtime_stats_store::{DaemonCurrentJobStats, DaemonLastJobStats},
    };

    use super::{DaemonDiagnosticsSnapshot, build_bug_report_markdown, render_daemon_inspect};

    #[test]
    fn tdd_render_daemon_inspect_includes_counts() {
        let snapshot = DaemonDiagnosticsSnapshot {
            daemon_status: None,
            stats: Some(DaemonRuntimeStats {
                updated_at_unix_ms: 1,
                run_state: "running".to_string(),
                tick: 2,
                current_job: Some(DaemonCurrentJobStats {
                    job_id: "job-1".to_string(),
                    asset_uuid: "asset-1".to_string(),
                    progress_percent: 10,
                    stage: "derive".to_string(),
                    status: "running".to_string(),
                    started_at_unix_ms: 100,
                }),
                last_job: Some(DaemonLastJobStats {
                    job_id: "job-0".to_string(),
                    duration_ms: 42,
                    completed_at_unix_ms: 120,
                }),
            }),
            completed_jobs: vec![CompletedJobEntry {
                completed_at_unix_ms: 120,
                job_id: "job-0".to_string(),
                duration_ms: 42,
            }],
            cycles: vec![DaemonCycleEntry {
                ts_unix_ms: 121,
                tick: 2,
                outcome: "ok".to_string(),
                run_state: "running".to_string(),
                job_id: Some("job-1".to_string()),
                asset_uuid: Some("asset-1".to_string()),
                progress_percent: Some(10),
                stage: Some("derive".to_string()),
                short_status: Some("running".to_string()),
            }],
        };

        let rendered = render_daemon_inspect(&snapshot, Some("/tmp/history.sqlite3"));
        assert!(rendered.contains("completed_jobs_count=1"));
        assert!(rendered.contains("cycles_count=1"));
        assert!(rendered.contains("history_db_path=/tmp/history.sqlite3"));
    }

    #[test]
    fn tdd_bug_report_uses_default_title() {
        let snapshot = DaemonDiagnosticsSnapshot {
            daemon_status: None,
            stats: None,
            completed_jobs: Vec::new(),
            cycles: Vec::new(),
        };

        let markdown = build_bug_report_markdown(&snapshot, None, "daemon-stats.json", None);
        assert_eq!(markdown.title, "Retaia agent daemon bug report");
        assert!(markdown.body.contains("## Context"));
    }
}
