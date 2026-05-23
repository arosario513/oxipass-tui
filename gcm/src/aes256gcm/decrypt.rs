use crate::prelude::*;
use aes_gcm::{
    Aes256Gcm, Key, KeyInit,
    aead::{Aead, Nonce},
};

const NONCE_SIZE: usize = 12;

pub fn decrypt(encrypted: &[u8], key: Key<Aes256Gcm>) -> Result<Vec<u8>, Error> {
    if encrypted.len() < NONCE_SIZE {
        return Err(EncDataTooshort);
    }

    let (nonce_bytes, enc_data) = encrypted.split_at(NONCE_SIZE);
    let nonce = Nonce::<Aes256Gcm>::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(&key);
    let dec = cipher
        .decrypt(nonce, enc_data)
        .map_err(|_| GcmDecryptFailed)?;
    Ok(dec)
}
