use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use sequoia_openpgp as openpgp;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use openpgp::cert::prelude::*;
use openpgp::parse::Parse;
use openpgp::policy::StandardPolicy;
use openpgp::serialize::Serialize as OpenPgpSerialize;
use openpgp::serialize::stream::{Armorer, Message, Signer};

const IDENTITY_PATH_ENV: &str = "RETAIA_AGENT_IDENTITY_PATH";
const IDENTITY_FILE_NAME: &str = "identity.json";

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
    openpgp_private_key: String,
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
            return Ok(persisted.into());
        }

        let identity = Self::generate_ephemeral(requested_agent_id)?;
        persist_identity(&path, &identity)?;
        Ok(identity)
    }

    pub fn generate_ephemeral(requested_agent_id: Option<&str>) -> Result<Self, AgentIdentityError> {
        let agent_id = requested_agent_id
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let userid = format!("retaia-agent-{agent_id}");
        let (cert, _) = CertBuilder::general_purpose(None, Some(userid))
            .generate()?;

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

        let signature = String::from_utf8(sink).map_err(|error| {
            AgentIdentityError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        })?;
        Ok(signature.replace('\r', "").replace('\n', "\\n"))
    }
}

impl From<PersistedAgentIdentity> for AgentIdentity {
    fn from(value: PersistedAgentIdentity) -> Self {
        Self {
            agent_id: value.agent_id,
            openpgp_public_key: value.openpgp_public_key,
            openpgp_private_key: value.openpgp_private_key,
            openpgp_fingerprint: value.openpgp_fingerprint,
        }
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
    let persisted = PersistedAgentIdentity {
        agent_id: identity.agent_id.clone(),
        openpgp_public_key: identity.openpgp_public_key.clone(),
        openpgp_private_key: identity.openpgp_private_key.clone(),
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

#[cfg(test)]
mod tests {
    use super::AgentIdentity;

    #[test]
    fn tdd_agent_identity_generates_stable_openpgp_material() {
        let identity =
            AgentIdentity::generate_ephemeral(Some("550e8400-e29b-41d4-a716-446655440099"))
                .expect("identity");
        assert_eq!(identity.agent_id, "550e8400-e29b-41d4-a716-446655440099");
        assert!(identity.openpgp_public_key.contains("BEGIN PGP PUBLIC KEY BLOCK"));
        assert!(identity.openpgp_private_key.contains("BEGIN PGP PRIVATE KEY BLOCK"));
        assert!(!identity.openpgp_fingerprint.is_empty());
    }

    #[test]
    fn tdd_agent_identity_can_sign_detached_payload() {
        let identity = AgentIdentity::generate_ephemeral(None).expect("identity");
        let signature = identity
            .detached_signature_ascii_armored(b"payload")
            .expect("signature");
        assert!(signature.contains("BEGIN PGP SIGNATURE"));
    }
}
