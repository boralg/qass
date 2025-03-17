use aes_gcm_siv::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256GcmSiv,
};
use anyhow::anyhow;
use argon2::Argon2;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as b64, Engine as _};
use rand::RngCore;

pub fn generate_salt() -> String {
    let mut salt = [0u8; 16];
    rand::rng().fill_bytes(&mut salt);
    b64.encode(salt)
}

pub fn derive_key(master_pwd: &str, base64_salt: &str) -> anyhow::Result<[u8; 32]> {
    let salt_bytes = b64.decode(base64_salt)?;
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(master_pwd.as_bytes(), &salt_bytes, &mut key)
        .map_err(|_| anyhow!("Failed to derive key from master password"))?;
    Ok(key)
}

pub fn encrypt(cleartext: &str, key: &[u8; 32]) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let cipher = Aes256GcmSiv::new_from_slice(key)
        .map_err(|_| anyhow!("Failed to initialize cipher from derived key"))?;
    let nonce = Aes256GcmSiv::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, cleartext.as_bytes())
        .map_err(|_| anyhow!("Failed to encrypt cleartext"))?;

    Ok((nonce.to_vec(), ciphertext))
}

// TODO: zeroizing?
pub fn decrypt(ciphertext: &[u8], key: &[u8; 32], nonce: &[u8]) -> anyhow::Result<String> {
    let cipher = Aes256GcmSiv::new_from_slice(key)
        .map_err(|_| anyhow!("Failed to initialize cipher from derived key"))?;

    let nonce = aes_gcm_siv::Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow!("Failed to decrypt ciphertext"))?;

    String::from_utf8(plaintext).map_err(|_| anyhow!("Result is not valid UTF-8"))
}
