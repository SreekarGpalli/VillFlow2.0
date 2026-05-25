//! Credential storage using the OS keychain (Windows Credential Manager).

use thiserror::Error;

/// Errors that can occur during credential operations.
#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}

/// The service namespace used for all VillFlow entries.
const SERVICE_NAME: &str = "villflow";

/// Build a [`keyring::Entry`] for the given logical service name.
fn entry(service: &str) -> Result<keyring::Entry, VaultError> {
    Ok(keyring::Entry::new(SERVICE_NAME, service)?)
}

/// Store an API key (or other secret) in the OS credential manager.
pub fn save_key(service: &str, key: &str) -> Result<(), VaultError> {
    let e = entry(service)?;
    e.set_password(key)?;
    tracing::info!("Saved credential for service `{service}`");
    Ok(())
}

/// Load an API key from the credential manager.
///
/// Returns `Ok(None)` when no credential is stored (rather than an error).
pub fn load_key(service: &str) -> Result<Option<String>, VaultError> {
    let e = entry(service)?;
    match e.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => {
            tracing::debug!("No credential stored for service `{service}`");
            Ok(None)
        }
        Err(err) => Err(VaultError::Keyring(err)),
    }
}

/// Delete a stored credential. Returns `Ok(())` even if nothing was stored.
pub fn delete_key(service: &str) -> Result<(), VaultError> {
    let e = entry(service)?;
    match e.delete_credential() {
        Ok(()) => {
            tracing::info!("Deleted credential for service `{service}`");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            tracing::debug!("No credential to delete for service `{service}`");
            Ok(())
        }
        Err(err) => Err(VaultError::Keyring(err)),
    }
}

/// Zero out the contents of a mutable string to clear secrets from memory.
pub fn zeroize_string(s: &mut String) {
    unsafe {
        let bytes = s.as_mut_vec();
        for b in bytes.iter_mut() {
            std::ptr::write_volatile(b as *mut u8, 0);
        }
        bytes.clear();
        bytes.reserve(0);
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_vault_save_load() {
        let service = "test_service_name";
        let test_key = "my-secret-test-key-12345";
        
        // Clean up first
        let _ = delete_key(service);
        
        // Save
        save_key(service, test_key).unwrap();
        
        // Load
        let loaded = load_key(service).unwrap();
        assert_eq!(loaded, Some(test_key.to_string()));
        
        // Delete
        delete_key(service).unwrap();
        
        // Load again (should be None)
        let loaded_after_delete = load_key(service).unwrap();
        assert_eq!(loaded_after_delete, None);
    }

    #[test]
    fn test_zeroize_string() {
        let mut key = "secret_value_123".to_string();
        let ptr = key.as_ptr();
        let cap = key.capacity();
        zeroize_string(&mut key);
        assert_eq!(key.len(), 0);
        unsafe {
            let bytes = std::slice::from_raw_parts(ptr, cap);
            for &b in bytes {
                assert_eq!(b, 0);
            }
        }
    }
}

