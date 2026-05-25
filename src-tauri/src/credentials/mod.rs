//! Credential management module.

mod vault;

pub use vault::{delete_key, load_key, save_key, zeroize_string, VaultError};
