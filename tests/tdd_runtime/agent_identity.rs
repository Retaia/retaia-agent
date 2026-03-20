use std::fs;
use std::sync::{Mutex, OnceLock};

use retaia_agent::{AgentIdentity, AgentIdentityError};
use tempfile::tempdir;

fn env_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

fn set_test_identity_env(identity_path: &std::path::Path) {
    unsafe {
        std::env::set_var("RETAIA_AGENT_IDENTITY_PATH", identity_path);
        std::env::set_var("RETAIA_AGENT_SECRET_STORE_BACKEND", "memory");
    }
}

fn clear_test_identity_env() {
    unsafe {
        std::env::remove_var("RETAIA_AGENT_IDENTITY_PATH");
        std::env::remove_var("RETAIA_AGENT_SECRET_STORE_BACKEND");
    }
}

#[test]
fn tdd_agent_identity_load_or_create_persists_only_public_metadata_and_reloads_secret() {
    let _guard = env_guard().lock().expect("env guard");
    let tempdir = tempdir().expect("tempdir");
    let identity_path = tempdir.path().join("identity.json");
    set_test_identity_env(&identity_path);

    let created = AgentIdentity::load_or_create(None).expect("create identity");
    let persisted = fs::read_to_string(&identity_path).expect("identity file");
    let reloaded = AgentIdentity::load_or_create(Some(&created.agent_id)).expect("reload identity");

    assert!(persisted.contains(&created.agent_id));
    assert!(persisted.contains("BEGIN PGP PUBLIC KEY BLOCK"));
    assert!(!persisted.contains("BEGIN PGP PRIVATE KEY BLOCK"));
    assert_eq!(reloaded.agent_id, created.agent_id);
    assert_eq!(reloaded.openpgp_private_key, created.openpgp_private_key);
    assert_eq!(reloaded.openpgp_fingerprint, created.openpgp_fingerprint);

    clear_test_identity_env();
}

#[test]
fn tdd_agent_identity_load_or_create_migrates_legacy_plaintext_identity_file() {
    let _guard = env_guard().lock().expect("env guard");
    let tempdir = tempdir().expect("tempdir");
    let identity_path = tempdir.path().join("identity.json");
    set_test_identity_env(&identity_path);

    let legacy = AgentIdentity::generate_ephemeral(Some("550e8400-e29b-41d4-a716-446655440099"))
        .expect("legacy identity");
    let legacy_json = serde_json::json!({
        "agent_id": legacy.agent_id,
        "openpgp_public_key": legacy.openpgp_public_key,
        "openpgp_private_key": legacy.openpgp_private_key,
        "openpgp_fingerprint": legacy.openpgp_fingerprint,
    });
    fs::write(
        &identity_path,
        serde_json::to_vec_pretty(&legacy_json).expect("legacy json"),
    )
    .expect("write legacy identity");

    let reloaded = AgentIdentity::load_or_create(Some("550e8400-e29b-41d4-a716-446655440099"))
        .expect("reload migrated identity");
    let migrated = fs::read_to_string(&identity_path).expect("identity file");

    assert_eq!(reloaded.agent_id, "550e8400-e29b-41d4-a716-446655440099");
    assert!(!reloaded.openpgp_private_key.is_empty());
    assert!(!migrated.contains("BEGIN PGP PRIVATE KEY BLOCK"));

    clear_test_identity_env();
}

#[test]
fn tdd_agent_identity_load_or_create_rejects_requested_agent_id_mismatch() {
    let _guard = env_guard().lock().expect("env guard");
    let tempdir = tempdir().expect("tempdir");
    let identity_path = tempdir.path().join("identity.json");
    set_test_identity_env(&identity_path);

    let existing = AgentIdentity::load_or_create(Some("550e8400-e29b-41d4-a716-446655440099"))
        .expect("existing identity");
    let error = AgentIdentity::load_or_create(Some("550e8400-e29b-41d4-a716-446655440000"))
        .expect_err("mismatch should fail");

    match error {
        AgentIdentityError::AgentIdMismatch { expected, actual } => {
            assert_eq!(expected, "550e8400-e29b-41d4-a716-446655440000");
            assert_eq!(actual, existing.agent_id);
        }
        other => panic!("unexpected error: {other:?}"),
    }

    clear_test_identity_env();
}
