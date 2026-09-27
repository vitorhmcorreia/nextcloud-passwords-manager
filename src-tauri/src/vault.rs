//! The encrypted file on disk that holds credentials and the offline copy.
//!
//! File layout (all binary):
//!
//! ```text
//! | "NCPV1" (5 bytes) | salt (16) | nonce (24) | ciphertext + auth tag |
//! ```
//!
//! - The key is derived from the master password with Argon2id (slow on
//!   purpose, so guessing master passwords is expensive).
//! - The contents are encrypted with XChaCha20-Poly1305, which also detects
//!   any tampering: a wrong password or a modified file both fail to decrypt.

use crate::api::{Credentials, Snapshot};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use zeroize::Zeroizing;

const MAGIC: &[u8] = b"NCPV1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;

/// What gets encrypted.
#[derive(Clone, Serialize, Deserialize)]
pub struct VaultData {
    pub credentials: Credentials,
    pub snapshot: Option<Snapshot>,
}

/// A 32-byte key that is wiped from memory when dropped.
pub type Key = Zeroizing<[u8; 32]>;

pub fn derive_key(master: &str, salt: &[u8]) -> Result<Key, String> {
    // 64 MiB of memory, 3 passes, 1 thread.
    let params = Params::new(64 * 1024, 3, 1, Some(32)).map_err(|e| e.to_string())?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon
        .hash_password_into(master.as_bytes(), salt, key.as_mut())
        .map_err(|e| e.to_string())?;
    Ok(key)
}

pub fn new_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Read the salt from an existing vault file (needed to re-derive the key).
pub fn read_salt(path: &Path) -> Result<[u8; SALT_LEN], String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    check_header(&bytes)?;
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&bytes[MAGIC.len()..MAGIC.len() + SALT_LEN]);
    Ok(salt)
}

fn check_header(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < MAGIC.len() + SALT_LEN + NONCE_LEN || !bytes.starts_with(MAGIC) {
        return Err("Vault file is not valid".into());
    }
    Ok(())
}

pub fn load(path: &Path, key: &Key) -> Result<VaultData, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    check_header(&bytes)?;
    let nonce_start = MAGIC.len() + SALT_LEN;
    let nonce = XNonce::from_slice(&bytes[nonce_start..nonce_start + NONCE_LEN]);
    let cipher = XChaCha20Poly1305::new(key.as_ref().into());
    let plain = Zeroizing::new(
        cipher
            .decrypt(nonce, &bytes[nonce_start + NONCE_LEN..])
            .map_err(|_| "Wrong master password")?,
    );
    serde_json::from_slice(&plain).map_err(|e| e.to_string())
}

pub fn save(path: &Path, key: &Key, salt: &[u8; SALT_LEN], data: &VaultData) -> Result<(), String> {
    let plain = Zeroizing::new(serde_json::to_vec(data).map_err(|e| e.to_string())?);
    let cipher = XChaCha20Poly1305::new(key.as_ref().into());
    // A fresh random nonce every time we save. Never reuse one with the same key.
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plain.as_slice())
        .map_err(|e| e.to_string())?;

    let mut out = Vec::with_capacity(MAGIC.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    // Write to a temp file then rename, so a crash mid-write can't corrupt the vault.
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &out).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> VaultData {
        VaultData {
            credentials: Credentials {
                server: "https://example.org".into(),
                user: "u".into(),
                password: "p".into(),
            },
            snapshot: None,
        }
    }

    #[test]
    fn roundtrip_and_wrong_password() {
        let dir = std::env::temp_dir().join(format!("ncpass-test-{}", std::process::id()));
        let path = dir.join("vault.bin");
        let salt = new_salt();
        let key = derive_key("correct horse", &salt).unwrap();
        save(&path, &key, &salt, &sample()).unwrap();

        assert_eq!(read_salt(&path).unwrap(), salt);
        let back = load(&path, &key).unwrap();
        assert_eq!(back.credentials.user, "u");

        let wrong = derive_key("wrong", &salt).unwrap();
        assert!(load(&path, &wrong).is_err());
        fs::remove_dir_all(dir).ok();
    }
}

/// End-to-end check against a real server: download, encrypt to disk, decrypt.
/// Opt-in (needs network + credentials):
///   set -a; . ../.env; set +a; cargo test live -- --ignored --nocapture
#[cfg(test)]
mod live {
    use super::*;
    use crate::api::Client;

    #[tokio::test]
    #[ignore]
    async fn live_sync_roundtrip() {
        let env = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k} not set"));
        let credentials = Credentials {
            server: env("NEXTCLOUD_INSTANCE"),
            user: env("NEXTCLOUD_USER"),
            password: env("NEXTCLOUD_PASSWORD"),
        };
        let snapshot = Client::new(credentials.clone()).unwrap().fetch_all().await.unwrap();

        let path = std::env::temp_dir().join(format!("ncpass-live-{}.bin", std::process::id()));
        let salt = new_salt();
        let key = derive_key("test master", &salt).unwrap();
        save(&path, &key, &salt, &VaultData { credentials, snapshot: Some(snapshot) }).unwrap();
        let size = fs::metadata(&path).unwrap().len();

        let back = load(&path, &key).unwrap().snapshot.unwrap();
        fs::remove_file(&path).ok();
        let with_pw = back.passwords.iter().filter(|p| !p.password.is_empty()).count();
        println!(
            "vault: {size} bytes | passwords: {} ({with_pw} with a secret) | folders: {} | tags: {} | tagged: {}",
            back.passwords.len(),
            back.folders.len(),
            back.tags.len(),
            back.passwords.iter().filter(|p| !p.tags.is_empty()).count(),
        );
        assert!(!back.passwords.is_empty());
    }
}
