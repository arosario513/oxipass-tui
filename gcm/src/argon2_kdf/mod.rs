use crate::prelude::*;
use argon2::Argon2;
use rand::TryRngCore;
use rand::rngs::OsRng;

pub fn derive_key(password: &str) -> Result<([u8; 32], [u8; 16]), Error> {
    derive_key_from_bytes(password.as_bytes())
}

pub fn derive_key_with_salt(password: &str, salt: &[u8; 16]) -> Result<[u8; 32], Error> {
    derive_key_from_bytes_with_salt(password.as_bytes(), salt)
}

pub fn derive_key_from_bytes(material: &[u8]) -> Result<([u8; 32], [u8; 16]), Error> {
    let mut out = [0u8; 32];
    let mut salt = [0u8; 16];
    OsRng
        .try_fill_bytes(&mut salt)
        .map_err(|e| RngFailed(e.to_string()))?;
    Argon2::default()
        .hash_password_into(material, &salt, &mut out)
        .map_err(|e| Argon2KeyDerivation(e.to_string()))?;
    Ok((out, salt))
}

pub fn derive_key_from_bytes_with_salt(
    material: &[u8],
    salt: &[u8; 16],
) -> Result<[u8; 32], Error> {
    let mut out = [0u8; 32];
    Argon2::default()
        .hash_password_into(material, salt, &mut out)
        .map_err(|e| Argon2KeyDerivation(e.to_string()))?;
    Ok(out)
}
