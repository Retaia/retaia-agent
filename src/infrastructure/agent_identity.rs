use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::OnceLock;

use directories::ProjectDirs;
#[cfg(not(test))]
use keyring::use_named_store;
#[cfg(not(test))]
use keyring_core::Entry;
use sequoia_openpgp as openpgp;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use openpgp::cert::prelude::*;
use openpgp::parse::Parse;
use openpgp::policy::StandardPolicy;
use openpgp::serialize::Serialize as OpenPgpSerialize;
use openpgp::serialize::stream::{Armorer, Message, Signer};

const IDENTITY_PATH_ENV: &str = "RETAIA_AGENT_IDENTITY_PATH";
#[allow(dead_code)]
const SECRET_STORE_BACKEND_ENV: &str = "RETAIA_AGENT_SECRET_STORE_BACKEND";
const IDENTITY_FILE_NAME: &str = "identity.json";
#[cfg(not(test))]
const KEYRING_SERVICE: &str = "io.retaia.retaia-agent";
const KEYRING_ACCOUNT_PREFIX: &str = "openpgp-private-key:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub openpgp_public_key: String,
    pub openpgp_private_key: String,
    pub openpgp_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedAgentIdentity {
    agent_id: String,
    openpgp_public_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    openpgp_private_key: Option<String>,
    openpgp_fingerprint: String,
}

#[derive(Debug, Error)]
pub enum AgentIdentityError {
    #[error("agent identity directory unavailable")]
    DirectoryUnavailable,
    #[error("agent identity mismatch (expected={expected} actual={actual})")]
    AgentIdMismatch { expected: String, actual: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("secret store error: {0}")]
    SecretStore(String),
    #[error("openpgp error: {0}")]
    OpenPgp(#[from] openpgp::Error),
    #[error("openpgp anyhow error: {0}")]
    OpenPgpAnyhow(#[from] anyhow::Error),
}

impl AgentIdentity {
    pub fn load_or_create(requested_agent_id: Option<&str>) -> Result<Self, AgentIdentityError> {
        let path = identity_file_path()?;
        if path.exists() {
            let persisted: PersistedAgentIdentity =
                serde_json::from_str(&fs::read_to_string(&path)?)?;
            if let Some(expected) = requested_agent_id {
                if persisted.agent_id != expected {
                    return Err(AgentIdentityError::AgentIdMismatch {
                        expected: expected.to_string(),
                        actual: persisted.agent_id,
                    });
                }
            }
            return load_persisted_identity(path.as_path(), persisted);
        }

        let identity = Self::generate_ephemeral(requested_agent_id)?;
        persist_identity(&path, &identity)?;
        Ok(identity)
    }

    pub fn generate_ephemeral(
        requested_agent_id: Option<&str>,
    ) -> Result<Self, AgentIdentityError> {
        let agent_id = requested_agent_id
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let userid = format!("retaia-agent-{agent_id}");
        let (cert, _) = CertBuilder::general_purpose(None, Some(userid)).generate()?;

        let mut public_key = Vec::new();
        cert.armored().serialize(&mut public_key)?;

        let mut private_key = Vec::new();
        cert.as_tsk().armored().serialize(&mut private_key)?;

        Ok(Self {
            agent_id,
            openpgp_public_key: String::from_utf8(public_key)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
            openpgp_private_key: String::from_utf8(private_key)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
            openpgp_fingerprint: cert.fingerprint().to_string(),
        })
    }

    pub fn detached_signature_ascii_armored(
        &self,
        payload: &[u8],
    ) -> Result<String, AgentIdentityError> {
        let cert = openpgp::Cert::from_bytes(self.openpgp_private_key.as_bytes())?;
        let policy = StandardPolicy::new();
        let signer = cert
            .keys()
            .with_policy(&policy, None)
            .alive()
            .revoked(false)
            .for_signing()
            .secret()
            .next()
            .ok_or_else(|| std::io::Error::other("no signing key available"))?;
        let keypair = signer.key().clone().into_keypair()?;

        let mut sink = Vec::new();
        let message = Message::new(&mut sink);
        let message = Armorer::new(message)
            .kind(openpgp::armor::Kind::Signature)
            .build()?;
        let mut signer = Signer::new(message, keypair).detached().build()?;
        signer.write_all(payload)?;
        signer.finalize()?;

        String::from_utf8(sink).map_err(|error| {
            AgentIdentityError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        })
    }

    pub fn detached_signature_http_header_value(
        &self,
        payload: &[u8],
    ) -> Result<String, AgentIdentityError> {
        let signature = self.detached_signature_ascii_armored(payload)?;
        Ok(signature.replace('\r', "").lines().collect::<String>())
    }
}

fn identity_file_path() -> Result<PathBuf, AgentIdentityError> {
    if let Ok(path) = std::env::var(IDENTITY_PATH_ENV) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    let dirs = ProjectDirs::from("io", "Retaia", "retaia-agent")
        .ok_or(AgentIdentityError::DirectoryUnavailable)?;
    Ok(dirs.config_dir().join(IDENTITY_FILE_NAME))
}

fn persist_identity(path: &Path, identity: &AgentIdentity) -> Result<(), AgentIdentityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    persist_private_key(&identity.agent_id, &identity.openpgp_private_key)?;
    let persisted = PersistedAgentIdentity {
        agent_id: identity.agent_id.clone(),
        openpgp_public_key: identity.openpgp_public_key.clone(),
        openpgp_private_key: None,
        openpgp_fingerprint: identity.openpgp_fingerprint.clone(),
    };
    let payload = serde_json::to_vec_pretty(&persisted)?;
    let mut file = fs::File::create(path)?;
    file.write_all(&payload)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn load_persisted_identity(
    path: &Path,
    persisted: PersistedAgentIdentity,
) -> Result<AgentIdentity, AgentIdentityError> {
    let private_key = match persisted.openpgp_private_key.clone() {
        Some(private_key) => {
            persist_private_key(&persisted.agent_id, &private_key)?;
            let identity = AgentIdentity {
                agent_id: persisted.agent_id.clone(),
                openpgp_public_key: persisted.openpgp_public_key.clone(),
                openpgp_private_key: private_key.clone(),
                openpgp_fingerprint: persisted.openpgp_fingerprint.clone(),
            };
            persist_identity(path, &identity)?;
            private_key
        }
        None => load_private_key(&persisted.agent_id)?,
    };

    Ok(AgentIdentity {
        agent_id: persisted.agent_id,
        openpgp_public_key: persisted.openpgp_public_key,
        openpgp_private_key: private_key,
        openpgp_fingerprint: persisted.openpgp_fingerprint,
    })
}

fn secret_store_account(agent_id: &str) -> String {
    format!("{KEYRING_ACCOUNT_PREFIX}{agent_id}")
}

#[cfg(not(test))]
fn persist_private_key(agent_id: &str, private_key: &str) -> Result<(), AgentIdentityError> {
    if use_memory_secret_store() {
        let mut store = test_secret_store().lock().map_err(|_| {
            AgentIdentityError::SecretStore("memory secret store poisoned".to_string())
        })?;
        store.insert(secret_store_account(agent_id), private_key.to_string());
        return Ok(());
    }
    initialize_secret_store()?;
    secret_store_entry(agent_id)?
        .set_password(private_key)
        .map_err(secret_store_error)
}

#[cfg(test)]
fn persist_private_key(agent_id: &str, private_key: &str) -> Result<(), AgentIdentityError> {
    let mut store = test_secret_store()
        .lock()
        .map_err(|_| AgentIdentityError::SecretStore("test secret store poisoned".to_string()))?;
    store.insert(secret_store_account(agent_id), private_key.to_string());
    Ok(())
}

#[cfg(not(test))]
fn load_private_key(agent_id: &str) -> Result<String, AgentIdentityError> {
    if use_memory_secret_store() {
        let store = test_secret_store().lock().map_err(|_| {
            AgentIdentityError::SecretStore("memory secret store poisoned".to_string())
        })?;
        return store
            .get(&secret_store_account(agent_id))
            .cloned()
            .ok_or_else(|| AgentIdentityError::SecretStore("missing memory secret".to_string()));
    }
    initialize_secret_store()?;
    secret_store_entry(agent_id)?
        .get_password()
        .map_err(secret_store_error)
}

#[cfg(test)]
fn load_private_key(agent_id: &str) -> Result<String, AgentIdentityError> {
    let store = test_secret_store()
        .lock()
        .map_err(|_| AgentIdentityError::SecretStore("test secret store poisoned".to_string()))?;
    store
        .get(&secret_store_account(agent_id))
        .cloned()
        .ok_or_else(|| AgentIdentityError::SecretStore("missing test secret".to_string()))
}

#[cfg(not(test))]
fn initialize_secret_store() -> Result<(), AgentIdentityError> {
    static SECRET_STORE_INIT: OnceLock<Result<(), String>> = OnceLock::new();
    SECRET_STORE_INIT
        .get_or_init(|| {
            #[cfg(target_os = "macos")]
            {
                use_named_store("keychain").map_err(|error| error.to_string())
            }
            #[cfg(target_os = "windows")]
            {
                use_named_store("windows").map_err(|error| error.to_string())
            }
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            {
                use_named_store("secret-service").map_err(|error| error.to_string())
            }
            #[cfg(not(any(
                target_os = "macos",
                target_os = "windows",
                target_os = "linux",
                target_os = "freebsd"
            )))]
            {
                Err("no supported secret store backend for this platform".to_string())
            }
        })
        .clone()
        .map_err(AgentIdentityError::SecretStore)
}

#[cfg(not(test))]
fn secret_store_entry(agent_id: &str) -> Result<Entry, AgentIdentityError> {
    Entry::new(KEYRING_SERVICE, &secret_store_account(agent_id))
        .map_err(|error| AgentIdentityError::SecretStore(error.to_string()))
}

#[cfg(not(test))]
fn secret_store_error(error: keyring_core::Error) -> AgentIdentityError {
    AgentIdentityError::SecretStore(error.to_string())
}

#[allow(dead_code)]
fn use_memory_secret_store() -> bool {
    matches!(
        std::env::var(SECRET_STORE_BACKEND_ENV).ok().as_deref(),
        Some("memory")
    )
}

fn test_secret_store() -> &'static Mutex<HashMap<String, String>> {
    static TEST_SECRET_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    TEST_SECRET_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
fn test_env_guard() -> &'static Mutex<()> {
    static TEST_ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_ENV_GUARD.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{AgentIdentity, PersistedAgentIdentity};

    #[test]
    fn tdd_agent_identity_generates_stable_openpgp_material() {
        let identity =
            AgentIdentity::generate_ephemeral(Some("550e8400-e29b-41d4-a716-446655440099"))
                .expect("identity");
        assert_eq!(identity.agent_id, "550e8400-e29b-41d4-a716-446655440099");
        assert!(
            identity
                .openpgp_public_key
                .contains("BEGIN PGP PUBLIC KEY BLOCK")
        );
        assert!(
            identity
                .openpgp_private_key
                .contains("BEGIN PGP PRIVATE KEY BLOCK")
        );
        assert!(!identity.openpgp_fingerprint.is_empty());
    }

    #[test]
    fn tdd_agent_identity_can_sign_detached_payload() {
        let identity = AgentIdentity::generate_ephemeral(None).expect("identity");
        let signature = identity
            .detached_signature_ascii_armored(b"payload")
            .expect("signature");
        assert!(signature.contains("BEGIN PGP SIGNATURE"));
        assert!(signature.contains('\n'));
    }

    #[test]
    fn tdd_agent_identity_header_signature_does_not_escape_newlines() {
        let identity = AgentIdentity::generate_ephemeral(None).expect("identity");
        let signature = identity
            .detached_signature_http_header_value(b"payload")
            .expect("signature");
        assert!(signature.contains("BEGIN PGP SIGNATURE"));
        assert!(!signature.contains("\\n"));
        assert!(!signature.contains('\n'));
    }

    #[test]
    fn tdd_agent_identity_persists_metadata_without_private_key() {
        let _guard = super::test_env_guard().lock().expect("env guard");
        super::test_secret_store()
            .lock()
            .expect("test store")
            .clear();
        let tempdir = tempdir().expect("tempdir");
        let identity_path = tempdir.path().join("identity.json");
        unsafe {
            std::env::set_var("RETAIA_AGENT_IDENTITY_PATH", &identity_path);
        }

        let identity = AgentIdentity::load_or_create(None).expect("identity");
        let persisted = fs::read_to_string(&identity_path).expect("identity file");
        assert!(persisted.contains(&identity.agent_id));
        assert!(!persisted.contains("BEGIN PGP PRIVATE KEY BLOCK"));

        let reloaded = AgentIdentity::load_or_create(Some(&identity.agent_id)).expect("reload");
        assert_eq!(reloaded.openpgp_private_key, identity.openpgp_private_key);

        unsafe {
            std::env::remove_var("RETAIA_AGENT_IDENTITY_PATH");
        }
    }

    #[test]
    fn tdd_agent_identity_migrates_legacy_private_key_out_of_json() {
        let _guard = super::test_env_guard().lock().expect("env guard");
        super::test_secret_store()
            .lock()
            .expect("test store")
            .clear();
        let tempdir = tempdir().expect("tempdir");
        let identity_path = tempdir.path().join("identity.json");
        unsafe {
            std::env::set_var("RETAIA_AGENT_IDENTITY_PATH", &identity_path);
        }

        let identity =
            AgentIdentity::generate_ephemeral(Some("550e8400-e29b-41d4-a716-446655440099"))
                .expect("identity");
        let legacy = PersistedAgentIdentity {
            agent_id: identity.agent_id.clone(),
            openpgp_public_key: identity.openpgp_public_key.clone(),
            openpgp_private_key: Some(identity.openpgp_private_key.clone()),
            openpgp_fingerprint: identity.openpgp_fingerprint.clone(),
        };
        fs::write(
            &identity_path,
            serde_json::to_vec_pretty(&legacy).expect("legacy json"),
        )
        .expect("write legacy identity");

        let reloaded = AgentIdentity::load_or_create(Some(&identity.agent_id)).expect("reload");
        let migrated = fs::read_to_string(&identity_path).expect("identity file");
        assert_eq!(reloaded.openpgp_private_key, identity.openpgp_private_key);
        assert!(!migrated.contains("BEGIN PGP PRIVATE KEY BLOCK"));

        unsafe {
            std::env::remove_var("RETAIA_AGENT_IDENTITY_PATH");
        }
    }
}
