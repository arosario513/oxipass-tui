use aes_gcm::{AeadCore, Aes256Gcm, aead::Nonce};
use aes_gcm::aead::OsRng;

pub fn generate_nonce() -> Nonce<Aes256Gcm> {
    Aes256Gcm::generate_nonce(OsRng)
}
