use std::collections::HashMap;
#[cfg(not(test))]
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

#[cfg(not(test))]
use keyring::use_named_store;
#[cfg(not(test))]
use keyring_core::Entry;

#[cfg(not(test))]
const SECRET_STORE_BACKEND_ENV: &str = "RETAIA_AGENT_SECRET_STORE_BACKEND";
#[cfg(not(test))]
const SECRET_STORE_FILE_ENV: &str = "RETAIA_AGENT_SECRET_STORE_FILE";
#[cfg(not(test))]
const KEYRING_SERVICE: &str = "io.retaia.retaia-agent";
const KEYRING_ACCOUNT_PREFIX: &str = "technical-auth:";

pub fn persist_technical_secret(
    config_path: &Path,
    client_id: &str,
    secret_key: &str,
) -> Result<(), String> {
    #[cfg(test)]
    {
        let mut store = test_secret_store()
            .lock()
            .map_err(|_| "test technical secret store poisoned".to_string())?;
        store.insert(
            secret_store_account(config_path, client_id),
            secret_key.to_string(),
        );
        Ok(())
    }

    #[cfg(not(test))]
    {
        if let Some(store_file) = file_secret_store_path() {
            let mut store = load_file_secret_store(store_file)?;
            store.insert(
                secret_store_account(config_path, client_id),
                secret_key.to_string(),
            );
            return save_file_secret_store(store_file, &store);
        }
        if use_memory_secret_store() {
            let mut store = test_secret_store()
                .lock()
                .map_err(|_| "memory technical secret store poisoned".to_string())?;
            store.insert(
                secret_store_account(config_path, client_id),
                secret_key.to_string(),
            );
            return Ok(());
        }

        initialize_secret_store()?;
        secret_store_entry(config_path, client_id)?
            .set_password(secret_key)
            .map_err(|error| error.to_string())
    }
}

pub fn load_technical_secret(config_path: &Path, client_id: &str) -> Result<String, String> {
    #[cfg(test)]
    {
        let store = test_secret_store()
            .lock()
            .map_err(|_| "test technical secret store poisoned".to_string())?;
        return store
            .get(&secret_store_account(config_path, client_id))
            .cloned()
            .ok_or_else(|| "missing technical secret".to_string());
    }

    #[cfg(not(test))]
    {
        if let Some(store_file) = file_secret_store_path() {
            let store = load_file_secret_store(store_file)?;
            return store
                .get(&secret_store_account(config_path, client_id))
                .cloned()
                .ok_or_else(|| "missing technical secret".to_string());
        }
        if use_memory_secret_store() {
            let store = test_secret_store()
                .lock()
                .map_err(|_| "memory technical secret store poisoned".to_string())?;
            return store
                .get(&secret_store_account(config_path, client_id))
                .cloned()
                .ok_or_else(|| "missing technical secret".to_string());
        }

        initialize_secret_store()?;
        secret_store_entry(config_path, client_id)?
            .get_password()
            .map_err(|error| error.to_string())
    }
}

pub fn delete_technical_secret(config_path: &Path, client_id: &str) -> Result<(), String> {
    #[cfg(test)]
    {
        let mut store = test_secret_store()
            .lock()
            .map_err(|_| "test technical secret store poisoned".to_string())?;
        store.remove(&secret_store_account(config_path, client_id));
        Ok(())
    }

    #[cfg(not(test))]
    {
        if let Some(store_file) = file_secret_store_path() {
            let mut store = load_file_secret_store(store_file)?;
            store.remove(&secret_store_account(config_path, client_id));
            return save_file_secret_store(store_file, &store);
        }
        if use_memory_secret_store() {
            let mut store = test_secret_store()
                .lock()
                .map_err(|_| "memory technical secret store poisoned".to_string())?;
            store.remove(&secret_store_account(config_path, client_id));
            return Ok(());
        }

        initialize_secret_store()?;
        match secret_store_entry(config_path, client_id)?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

fn secret_store_account(config_path: &Path, client_id: &str) -> String {
    format!(
        "{KEYRING_ACCOUNT_PREFIX}{}:{client_id}",
        stable_path_fingerprint(config_path)
    )
}

fn stable_path_fingerprint(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    raw.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(not(test))]
fn initialize_secret_store() -> Result<(), String> {
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
                Err("no supported technical secret store backend for this platform".to_string())
            }
        })
        .clone()
}

#[cfg(not(test))]
fn secret_store_entry(config_path: &Path, client_id: &str) -> Result<Entry, String> {
    Entry::new(
        KEYRING_SERVICE,
        &secret_store_account(config_path, client_id),
    )
    .map_err(|error| error.to_string())
}

#[cfg(not(test))]
fn use_memory_secret_store() -> bool {
    matches!(
        std::env::var(SECRET_STORE_BACKEND_ENV).ok().as_deref(),
        Some("memory")
    )
}

#[cfg(not(test))]
fn file_secret_store_path() -> Option<&'static Path> {
    static STORE_PATH: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
    STORE_PATH
        .get_or_init(|| {
            std::env::var(SECRET_STORE_FILE_ENV)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .map(std::path::PathBuf::from)
        })
        .as_deref()
}

#[cfg(not(test))]
fn load_file_secret_store(path: &Path) -> Result<HashMap<String, String>, String> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| error.to_string())
}

#[cfg(not(test))]
fn save_file_secret_store(path: &Path, store: &HashMap<String, String>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let payload = serde_json::to_vec(store).map_err(|error| error.to_string())?;
    fs::write(path, payload).map_err(|error| error.to_string())
}

fn test_secret_store() -> &'static Mutex<HashMap<String, String>> {
    static TEST_SECRET_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    TEST_SECRET_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{delete_technical_secret, load_technical_secret, persist_technical_secret};

    #[test]
    fn tdd_technical_secret_store_roundtrip_uses_config_path_scoped_account() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("config.toml");

        persist_technical_secret(&path, "agent-a", "secret-a").expect("persist");
        let loaded = load_technical_secret(&path, "agent-a").expect("load");
        assert_eq!(loaded, "secret-a");
    }

    #[test]
    fn tdd_technical_secret_store_delete_is_idempotent() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("config.toml");

        persist_technical_secret(&path, "agent-a", "secret-a").expect("persist");
        delete_technical_secret(&path, "agent-a").expect("delete");
        delete_technical_secret(&path, "agent-a").expect("delete twice");
        assert!(load_technical_secret(&path, "agent-a").is_err());
    }
}
