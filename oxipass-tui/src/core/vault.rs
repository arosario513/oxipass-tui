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
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

fn key_material(password: &str, keyfile: Option<&[u8]>) -> Zeroizing<Vec<u8>> {
    let kf = keyfile.unwrap_or_default();
    let mut material = Zeroizing::new(Vec::with_capacity(password.len() + kf.len()));
    material.extend_from_slice(password.as_bytes());
    material.extend_from_slice(kf);
    material
}

/// Argon2-derived vault key plus the salt it was derived under. Derived once
/// at login so the master password and keyfile can be wiped right after;
/// every save reuses it with a fresh nonce.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    key: [u8; 32],
    salt: [u8; 16],
}

impl MasterKey {
    /// Derives a key under a new random salt (for new vaults).
    pub fn new(password: &str, keyfile: Option<&[u8]>) -> Result<Self, VaultError> {
        let material = key_material(password, keyfile);
        let (key, salt) = derive_key_from_bytes(&material)?;
        Ok(Self { key, salt })
    }

    fn with_salt(
        password: &str,
        keyfile: Option<&[u8]>,
        salt: [u8; 16],
    ) -> Result<Self, VaultError> {
        let material = key_material(password, keyfile);
        let key = derive_key_from_bytes_with_salt(&material, &salt)?;
        Ok(Self { key, salt })
    }

    fn cipher_key(&self) -> &Key<Aes256Gcm> {
        Key::<Aes256Gcm>::from_slice(&self.key)
    }
}

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

    pub fn save(&self, path: &Path, key: &MasterKey) -> Result<(), VaultError> {
        let path = path.with_extension("opdb");
        let json = Zeroizing::new(serde_json::to_vec(self)?);
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json)?;
        let plaintext = Zeroizing::new(encoder.finish()?);
        let nonce = generate_nonce();
        let blob = encrypt(&plaintext, key.cipher_key(), nonce)?;

        let mut file_data = Vec::with_capacity(key.salt.len() + blob.len());
        file_data.extend_from_slice(&key.salt);
        file_data.extend_from_slice(&blob);

        // Backup
        let bak = path.with_extension("opdb.bak");
        if path.exists() {
            std::fs::copy(&path, bak)?;
        }

        std::fs::write(path, file_data)?;
        Ok(())
    }

    pub fn load(
        path: &Path,
        password: &str,
        keyfile: Option<&[u8]>,
    ) -> Result<(Self, MasterKey), VaultError> {
        let data = std::fs::read(path)?;

        if data.len() < 44 {
            return Err(VaultError::InvalidFile);
        }

        let (salt_bytes, blob) = data.split_at(16);
        let salt: [u8; 16] = salt_bytes.try_into().map_err(|_| VaultError::InvalidFile)?;
        let key = MasterKey::with_salt(password, keyfile, salt)?;
        let compressed = Zeroizing::new(decrypt(blob, key.cipher_key())?);
        let mut decoder = DeflateDecoder::new(&compressed[..]);
        let mut plaintext = Zeroizing::new(Vec::new());
        decoder.read_to_end(&mut plaintext)?;

        Ok((serde_json::from_slice(&plaintext)?, key))
    }
}
