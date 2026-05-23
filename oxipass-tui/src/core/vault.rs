use crate::core::{Entry, VaultError};
use aes_gcm::{Aes256Gcm, Key};
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use gcm::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    entries: Vec<Entry>,
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn push_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    pub fn replace_entry(&mut self, id: Uuid, new_entry: Entry) {
        for entry in &mut self.entries {
            let entry_id = match entry {
                Entry::Login { id, .. } => *id,
                Entry::Payment { id, .. } => *id,
                Entry::Note { id, .. } => *id,
            };
            if entry_id == id {
                *entry = new_entry;
                return;
            }
        }
    }

    pub fn remove_entry(&mut self, id: Uuid) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| {
            let entry_id = match entry {
                Entry::Login { id, .. } => id,
                Entry::Payment { id, .. } => id,
                Entry::Note { id, .. } => id,
            };
            *entry_id != id
        });
        self.entries.len() < before
    }

    pub fn save(&self, path: &Path, password: &str) -> Result<(), VaultError> {
        let path = path.with_extension("opdb");
        let json = serde_json::to_vec(self)?;
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json)?;
        let plaintext = encoder.finish()?;
        let (key_bytes, salt) = derive_key(password)?;
        let key = Key::<Aes256Gcm>::from(key_bytes);
        let nonce = generate_nonce();
        let blob = encrypt(&plaintext, key, nonce)?;

        let mut file_data = Vec::with_capacity(salt.len() + blob.len());
        file_data.extend_from_slice(&salt);
        file_data.extend_from_slice(&blob);

        std::fs::write(path, file_data)?;
        Ok(())
    }

    pub fn load(path: &Path, password: &str) -> Result<Self, VaultError> {
        let data = std::fs::read(path)?;

        if data.len() < 44 {
            return Err(VaultError::InvalidFile);
        }

        let (salt_bytes, blob) = data.split_at(16);
        let salt: [u8; 16] = salt_bytes.try_into().map_err(|_| VaultError::InvalidFile)?;
        let key_bytes = derive_key_with_salt(password, &salt)?;
        let key = Key::<Aes256Gcm>::from(key_bytes);
        let compressed = decrypt(blob, key)?;
        let mut decoder = DeflateDecoder::new(&compressed[..]);
        let mut plaintext = Vec::new();
        decoder.read_to_end(&mut plaintext)?;

        Ok(serde_json::from_slice(&plaintext)?)
    }
}
