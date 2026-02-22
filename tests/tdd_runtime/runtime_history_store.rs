use retaia_agent::{CompletedJobEntry, DaemonCycleEntry, RuntimeHistoryStore};
use tempfile::tempdir;

#[test]
fn tdd_runtime_history_store_persists_and_reads_recent_entries() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("history.sqlite3");
    let mut store = RuntimeHistoryStore::open_at_path(&db_path).expect("open store");

    store
        .insert_cycle(&DaemonCycleEntry {
            ts_unix_ms: 1000,
            tick: 1,
            outcome: "success".to_string(),
            run_state: "running".to_string(),
            job_id: Some("job-1".to_string()),
            asset_uuid: Some("asset-1".to_string()),
            progress_percent: Some(10),
            stage: Some("processing".to_string()),
            short_status: Some("warming up".to_string()),
        })
        .expect("insert cycle");
    store
        .insert_completed_job(&CompletedJobEntry {
            completed_at_unix_ms: 2000,
            job_id: "job-1".to_string(),
            duration_ms: 1234,
        })
        .expect("insert completed job");

    let cycles = store.recent_cycles(5).expect("load cycles");
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].job_id.as_deref(), Some("job-1"));

    let jobs = store.recent_completed_jobs(5).expect("load jobs");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].job_id, "job-1");
    assert_eq!(jobs[0].duration_ms, 1234);
}

#[test]
fn tdd_runtime_history_store_compaction_keeps_recent_rows() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("history.sqlite3");
    let mut store = RuntimeHistoryStore::open_at_path(&db_path).expect("open store");

    for index in 0..10 {
        store
            .insert_cycle(&DaemonCycleEntry {
                ts_unix_ms: 1000 + index,
                tick: index,
                outcome: "success".to_string(),
                run_state: "running".to_string(),
                job_id: Some(format!("job-{index}")),
                asset_uuid: None,
                progress_percent: Some(index as u8),
                stage: Some("processing".to_string()),
                short_status: None,
            })
            .expect("insert cycle");
    }

    let deleted = store.compact_old_cycles(3).expect("compact");
    assert!(deleted > 0);

    let cycles = store.recent_cycles(10).expect("recent");
    assert_eq!(cycles.len(), 3);
    assert_eq!(cycles[0].tick, 9);
}
