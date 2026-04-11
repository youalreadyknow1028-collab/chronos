use zeroize::Zeroizing;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptFailed(String),
    #[error("Key not found in keychain")]
    KeyNotFound,
}

/// EncryptedBlob: zeroizes Vec<u8> on Drop — prevents key material from lingering in memory
pub struct EncryptedBlob(pub Zeroizing<Vec<u8>>);

impl Drop for EncryptedBlob {
    fn drop(&mut self) {
        // Zeroizing<T> auto-zeroizes the inner value on Drop (RAII pattern)
        // No manual call needed
    }
}

/// Encrypt data using AES-GCM-256
/// TODO: Use `keyring` crate for OS keychain (macOS Keychain / Windows CredMgr / Linux libsecret)
pub fn encrypt(data: &[u8], _key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // TODO: AES-GCM-256-GCM with random nonce
    Ok(data.to_vec())
}

pub fn decrypt(data: &[u8], _key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // TODO: AES-GCM-256-GCM decryption
    Ok(data.to_vec())
}
