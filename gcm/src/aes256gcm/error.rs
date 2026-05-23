use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Couldn't encrypt data. Make sure the parameters are set correctly")]
    GcmEncryptFailed,
    #[error("Couldn't decrypt data. Make sure the parameters are set correctly")]
    GcmDecryptFailed,
    #[error("Data is too short (Must have a length of >= 12)")]
    EncDataTooshort,
    #[error("color_eyre error: {0}")]
    ColorEyreReport(#[from] color_eyre::Report),
    #[error("Could not derive key: {0}")]
    Argon2KeyDerivation(String),
    #[error("Could not generate random bytes: {0}")]
    RngFailed(String),
}
