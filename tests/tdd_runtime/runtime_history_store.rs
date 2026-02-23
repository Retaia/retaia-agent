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

#[test]
fn tdd_runtime_history_store_completed_jobs_compaction_keeps_recent_rows() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("history.sqlite3");
    let mut store = RuntimeHistoryStore::open_at_path(&db_path).expect("open store");

    for index in 0..12 {
        store
            .insert_completed_job(&CompletedJobEntry {
                completed_at_unix_ms: 10_000 + index,
                job_id: format!("job-{index}"),
                duration_ms: 1_000 + index,
            })
            .expect("insert completed job");
    }

    let deleted = store
        .compact_old_completed_jobs(4)
        .expect("compact completed");
    assert!(deleted > 0);

    let jobs = store.recent_completed_jobs(20).expect("recent jobs");
    assert_eq!(jobs.len(), 4);
    assert_eq!(jobs[0].job_id, "job-11");
    assert_eq!(jobs[3].job_id, "job-8");
}

#[test]
fn tdd_runtime_history_store_high_volume_compaction_preserves_order_and_latest_rows() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("history.sqlite3");
    let mut store = RuntimeHistoryStore::open_at_path(&db_path).expect("open store");

    for index in 0..1_000u64 {
        store
            .insert_cycle(&DaemonCycleEntry {
                ts_unix_ms: 1_000_000 + index,
                tick: index,
                outcome: if index % 13 == 0 {
                    "degraded".to_string()
                } else {
                    "success".to_string()
                },
                run_state: "running".to_string(),
                job_id: Some(format!("job-{index}")),
                asset_uuid: Some(format!("asset-{index}")),
                progress_percent: Some((index % 100) as u8),
                stage: Some("processing".to_string()),
                short_status: Some("ok".to_string()),
            })
            .expect("insert cycle");
        store
            .insert_completed_job(&CompletedJobEntry {
                completed_at_unix_ms: 2_000_000 + index,
                job_id: format!("job-{index}"),
                duration_ms: 100 + index,
            })
            .expect("insert completed job");
    }

    store.compact_old_cycles(200).expect("compact cycles");
    store
        .compact_old_completed_jobs(120)
        .expect("compact completed jobs");

    let cycles = store.recent_cycles(5).expect("recent cycles");
    assert_eq!(cycles.len(), 5);
    assert_eq!(cycles[0].tick, 999);
    assert_eq!(cycles[4].tick, 995);

    let last_window = store.recent_cycles(300).expect("recent cycles window");
    assert_eq!(last_window.len(), 200);
    assert_eq!(last_window[199].tick, 800);

    let jobs = store.recent_completed_jobs(130).expect("recent completed");
    assert_eq!(jobs.len(), 120);
    assert_eq!(jobs[0].job_id, "job-999");
    assert_eq!(jobs[119].job_id, "job-880");
}
