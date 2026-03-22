use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};

pub trait Clock {
    fn now_utc(&self) -> DateTime<Utc>;

    fn now_unix_ms(&self) -> u64 {
        self.now_utc().timestamp_millis().max(0) as u64
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StdClock;

impl Clock for StdClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub trait FileTimestampProvider {
    fn created_at_utc(&self, path: &Path) -> Option<DateTime<Utc>>;
    fn modified_at_utc(&self, path: &Path) -> Option<DateTime<Utc>>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StdFileTimestampProvider;

impl FileTimestampProvider for StdFileTimestampProvider {
    fn created_at_utc(&self, path: &Path) -> Option<DateTime<Utc>> {
        fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.created().ok())
            .map(DateTime::<Utc>::from)
    }

    fn modified_at_utc(&self, path: &Path) -> Option<DateTime<Utc>> {
        fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(DateTime::<Utc>::from)
    }
}
