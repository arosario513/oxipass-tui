use base64::{Engine, engine::general_purpose::STANDARD};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::core::VaultError;

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct KeyFile {
    version: String,
    hash: String,
    data: String,
}

impl KeyFile {
    pub fn generate() -> Result<Self, VaultError> {
        let mut raw = Zeroizing::new([0u8; 32]);
        rand::rng().fill(&mut *raw);
        Ok(Self {
            version: "1.0.0".to_string(),
            hash: compute_hash(&*raw),
            data: STANDARD.encode(*raw),
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), VaultError> {
        let json = serde_json::to_string(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, VaultError> {
        let content = Zeroizing::new(std::fs::read_to_string(path)?);
        let kf: KeyFile = serde_json::from_str(&content)?;
        let raw = kf.key_bytes()?;
        if kf.hash != compute_hash(&*raw) {
            return Err(VaultError::InvalidKeyFile);
        }
        Ok(kf)
    }

    pub fn key_bytes(&self) -> Result<Zeroizing<[u8; 32]>, VaultError> {
        let bytes = Zeroizing::new(
            STANDARD
                .decode(&self.data)
                .map_err(|_| VaultError::InvalidKeyFile)?,
        );
        if bytes.len() != 32 {
            return Err(VaultError::InvalidKeyFile);
        }
        let mut raw = Zeroizing::new([0u8; 32]);
        raw.copy_from_slice(&bytes);
        Ok(raw)
    }
}

fn compute_hash(raw: &[u8]) -> String {
    let digest = Sha256::digest(raw);
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    )
}
