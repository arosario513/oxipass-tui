use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Crypto error: {0}")]
    Crypto(#[from] gcm::prelude::Error),
    #[error("Invalid vault file")]
    InvalidFile,
    #[error("Vault already exists at {0}")]
    VaultExists(std::path::PathBuf),
    #[error("Passwords do not match")]
    PasswordMismatch,
}
