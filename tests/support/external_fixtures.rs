use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalFixtureEntry {
    pub relative_path: String,
    pub sha256: String,
    pub kind: String,
    pub expected: String,
    pub notes: String,
}

impl ExternalFixtureEntry {
    pub fn absolute_path(&self) -> PathBuf {
        fixtures_root().join(&self.relative_path)
    }
}

pub fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/external")
}

pub fn manifest_path() -> PathBuf {
    fixtures_root().join("manifest.tsv")
}

pub fn load_manifest_entries() -> Vec<ExternalFixtureEntry> {
    let content = fs::read_to_string(manifest_path()).expect("read external fixture manifest");
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(parse_manifest_entry)
        .collect()
}

fn parse_manifest_entry(line: &str) -> ExternalFixtureEntry {
    let mut columns = line.splitn(5, '\t');
    let relative_path = columns.next().unwrap_or_default().trim();
    let sha256 = columns.next().unwrap_or_default().trim();
    let kind = columns.next().unwrap_or_default().trim();
    let expected = columns.next().unwrap_or_default().trim();
    let notes = columns.next().unwrap_or_default().trim();

    assert!(
        !relative_path.is_empty() && !sha256.is_empty() && !kind.is_empty() && !expected.is_empty(),
        "invalid manifest row: {line}"
    );

    ExternalFixtureEntry {
        relative_path: relative_path.to_string(),
        sha256: sha256.to_string(),
        kind: kind.to_string(),
        expected: expected.to_string(),
        notes: notes.to_string(),
    }
}
