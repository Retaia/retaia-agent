use std::path::{Path, PathBuf};
use std::{fs, io};

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::infrastructure::config_store::{ConfigStoreError, system_config_file_path};

pub const DAEMON_HISTORY_DB_FILE_NAME: &str = "daemon-history.sqlite3";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonCycleEntry {
    pub ts_unix_ms: u64,
    pub tick: u64,
    pub outcome: String,
    pub run_state: String,
    pub job_id: Option<String>,
    pub asset_uuid: Option<String>,
    pub progress_percent: Option<u8>,
    pub stage: Option<String>,
    pub short_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedJobEntry {
    pub completed_at_unix_ms: u64,
    pub job_id: String,
    pub duration_ms: u64,
}

#[derive(Debug, Error)]
pub enum RuntimeHistoryStoreError {
    #[error("history db path unavailable: {0}")]
    Path(ConfigStoreError),
    #[error("io error: {0}")]
    Io(io::Error),
    #[error("sqlite error: {0}")]
    Sql(rusqlite::Error),
}

pub struct RuntimeHistoryStore {
    conn: Connection,
}

impl RuntimeHistoryStore {
    pub fn open_default() -> Result<Self, RuntimeHistoryStoreError> {
        let path = runtime_history_db_path()?;
        Self::open_at_path(&path)
    }

    pub fn open_at_path(path: &Path) -> Result<Self, RuntimeHistoryStoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(RuntimeHistoryStoreError::Io)?;
        }
        let conn = Connection::open(path).map_err(RuntimeHistoryStoreError::Sql)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(RuntimeHistoryStoreError::Sql)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(RuntimeHistoryStoreError::Sql)?;
        conn.pragma_update(None, "temp_store", "MEMORY")
            .map_err(RuntimeHistoryStoreError::Sql)?;
        init_schema(&conn)?;
        Ok(Self { conn })
    }

    pub fn insert_cycle(
        &mut self,
        entry: &DaemonCycleEntry,
    ) -> Result<(), RuntimeHistoryStoreError> {
        self.conn
            .execute(
                "INSERT INTO daemon_cycles (
                    ts_unix_ms, tick, outcome, run_state, job_id, asset_uuid, progress_percent, stage, short_status
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    entry.ts_unix_ms as i64,
                    entry.tick as i64,
                    entry.outcome,
                    entry.run_state,
                    entry.job_id,
                    entry.asset_uuid,
                    entry.progress_percent.map(i64::from),
                    entry.stage,
                    entry.short_status
                ],
            )
            .map_err(RuntimeHistoryStoreError::Sql)?;
        Ok(())
    }

    pub fn insert_completed_job(
        &mut self,
        entry: &CompletedJobEntry,
    ) -> Result<(), RuntimeHistoryStoreError> {
        self.conn
            .execute(
                "INSERT INTO completed_jobs (completed_at_unix_ms, job_id, duration_ms)
                 VALUES (?1, ?2, ?3)",
                params![
                    entry.completed_at_unix_ms as i64,
                    entry.job_id,
                    entry.duration_ms as i64
                ],
            )
            .map_err(RuntimeHistoryStoreError::Sql)?;
        Ok(())
    }

    pub fn recent_cycles(
        &self,
        limit: usize,
    ) -> Result<Vec<DaemonCycleEntry>, RuntimeHistoryStoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT ts_unix_ms, tick, outcome, run_state, job_id, asset_uuid, progress_percent, stage, short_status
                 FROM daemon_cycles
                 ORDER BY id DESC
                 LIMIT ?1",
            )
            .map_err(RuntimeHistoryStoreError::Sql)?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(DaemonCycleEntry {
                    ts_unix_ms: row.get::<_, i64>(0)? as u64,
                    tick: row.get::<_, i64>(1)? as u64,
                    outcome: row.get(2)?,
                    run_state: row.get(3)?,
                    job_id: row.get(4)?,
                    asset_uuid: row.get(5)?,
                    progress_percent: row.get::<_, Option<i64>>(6)?.map(|v| v as u8),
                    stage: row.get(7)?,
                    short_status: row.get(8)?,
                })
            })
            .map_err(RuntimeHistoryStoreError::Sql)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(RuntimeHistoryStoreError::Sql)?);
        }
        Ok(entries)
    }

    pub fn recent_completed_jobs(
        &self,
        limit: usize,
    ) -> Result<Vec<CompletedJobEntry>, RuntimeHistoryStoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT completed_at_unix_ms, job_id, duration_ms
                 FROM completed_jobs
                 ORDER BY id DESC
                 LIMIT ?1",
            )
            .map_err(RuntimeHistoryStoreError::Sql)?;

        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(CompletedJobEntry {
                    completed_at_unix_ms: row.get::<_, i64>(0)? as u64,
                    job_id: row.get(1)?,
                    duration_ms: row.get::<_, i64>(2)? as u64,
                })
            })
            .map_err(RuntimeHistoryStoreError::Sql)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(RuntimeHistoryStoreError::Sql)?);
        }
        Ok(entries)
    }

    pub fn compact_old_cycles(
        &mut self,
        keep_last: usize,
    ) -> Result<usize, RuntimeHistoryStoreError> {
        if keep_last == 0 {
            let deleted = self
                .conn
                .execute("DELETE FROM daemon_cycles", [])
                .map_err(RuntimeHistoryStoreError::Sql)?;
            return Ok(deleted);
        }
        let cutoff_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM daemon_cycles ORDER BY id DESC LIMIT 1 OFFSET ?1",
                [(keep_last.saturating_sub(1)) as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(RuntimeHistoryStoreError::Sql)?;

        let Some(cutoff_id) = cutoff_id else {
            return Ok(0);
        };

        let deleted = self
            .conn
            .execute("DELETE FROM daemon_cycles WHERE id < ?1", [cutoff_id])
            .map_err(RuntimeHistoryStoreError::Sql)?;
        Ok(deleted)
    }
}

pub fn runtime_history_db_path() -> Result<PathBuf, RuntimeHistoryStoreError> {
    let config_path = system_config_file_path().map_err(RuntimeHistoryStoreError::Path)?;
    let parent = config_path.parent().ok_or(RuntimeHistoryStoreError::Path(
        ConfigStoreError::SystemConfigDirectoryUnavailable,
    ))?;
    Ok(parent.join(DAEMON_HISTORY_DB_FILE_NAME))
}

fn init_schema(conn: &Connection) -> Result<(), RuntimeHistoryStoreError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS daemon_cycles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts_unix_ms INTEGER NOT NULL,
            tick INTEGER NOT NULL,
            outcome TEXT NOT NULL,
            run_state TEXT NOT NULL,
            job_id TEXT NULL,
            asset_uuid TEXT NULL,
            progress_percent INTEGER NULL,
            stage TEXT NULL,
            short_status TEXT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_daemon_cycles_ts ON daemon_cycles(ts_unix_ms DESC);
        CREATE INDEX IF NOT EXISTS idx_daemon_cycles_job ON daemon_cycles(job_id);

        CREATE TABLE IF NOT EXISTS completed_jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            completed_at_unix_ms INTEGER NOT NULL,
            job_id TEXT NOT NULL,
            duration_ms INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_completed_jobs_completed_at ON completed_jobs(completed_at_unix_ms DESC);
        CREATE INDEX IF NOT EXISTS idx_completed_jobs_job ON completed_jobs(job_id);",
    )
    .map_err(RuntimeHistoryStoreError::Sql)?;
    Ok(())
}
