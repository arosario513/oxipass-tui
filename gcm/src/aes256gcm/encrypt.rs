use crate::prelude::*;

use aes_gcm::aead::{Aead, Nonce};
use aes_gcm::{Aes256Gcm, Key, KeyInit};

pub fn encrypt(
    plaintext: &[u8],
    key: Key<Aes256Gcm>,
    nonce: Nonce<Aes256Gcm>,
) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new(&key);

    let enc: Vec<u8> = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| GcmEncryptFailed)?;

    let mut blob = Vec::<u8>::new();

    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&enc);

    Ok(blob)
}
