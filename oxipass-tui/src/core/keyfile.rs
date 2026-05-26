use base64::{Engine, engine::general_purpose::STANDARD};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::core::VaultError;

#[derive(Serialize, Deserialize)]
pub struct KeyFile {
    version: String,
    hash: String,
    data: String,
}

impl KeyFile {
    pub fn generate() -> Result<Self, VaultError> {
        let mut raw = [0u8; 32];
        rand::rng().fill(&mut raw);
        Ok(Self {
            version: "1.0.0".to_string(),
            hash: compute_hash(&raw),
            data: STANDARD.encode(raw),
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), VaultError> {
        let json = serde_json::to_string(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, VaultError> {
        let content = std::fs::read_to_string(path)?;
        let kf: KeyFile = serde_json::from_str(&content)?;
        let raw = kf.key_bytes()?;
        if kf.hash != compute_hash(&raw) {
            return Err(VaultError::InvalidKeyFile);
        }
        Ok(kf)
    }

    pub fn key_bytes(&self) -> Result<[u8; 32], VaultError> {
        let bytes = STANDARD
            .decode(&self.data)
            .map_err(|_| VaultError::InvalidKeyFile)?;
        bytes.try_into().map_err(|_| VaultError::InvalidKeyFile)
    }
}

fn compute_hash(raw: &[u8]) -> String {
    let digest = Sha256::digest(raw);
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    )
}
