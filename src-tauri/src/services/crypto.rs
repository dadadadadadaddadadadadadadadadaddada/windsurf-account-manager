use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use sha1::Sha1;
use std::process::Command;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

const SALT: &[u8] = b"saltysalt";
const IV: [u8; 16] = [0x20u8; 16]; // 16 bytes of space (0x20)
const ITERATIONS: u32 = 1003;
const KEY_LEN: usize = 16;

fn get_windsurf_safe_storage_password() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("security")
            .args(["find-generic-password", "-s", "Windsurf Safe Storage", "-w"])
            .output()
            .map_err(|e| format!("Failed to run security command: {}", e))?;

        if !output.status.success() {
            return Err("Failed to get Windsurf Safe Storage password from Keychain".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Keychain access not implemented for this platform".to_string())
    }
}

fn derive_key(password: &str) -> Result<[u8; KEY_LEN], String> {
    let mut key = [0u8; KEY_LEN];
    pbkdf2::pbkdf2_hmac::<Sha1>(
        password.as_bytes(),
        SALT,
        ITERATIONS,
        &mut key,
    );
    Ok(key)
}

pub fn encrypt_sessions(sessions_json: &str) -> Result<Vec<u8>, String> {
    let password = get_windsurf_safe_storage_password()?;
    let key = derive_key(&password)?;

    let data = sessions_json.as_bytes();
    let block_size = 16;
    let padded_len = ((data.len() / block_size) + 1) * block_size;
    let mut buf = vec![0u8; padded_len];
    buf[..data.len()].copy_from_slice(data);

    let encrypted = Aes128CbcEnc::new(&key.into(), &IV.into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, data.len())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // v10 prefix for macOS Chromium
    let mut result = Vec::with_capacity(3 + encrypted.len());
    result.extend_from_slice(b"v10");
    result.extend_from_slice(encrypted);

    Ok(result)
}

pub fn build_encrypted_sessions_value(encrypted: &[u8]) -> serde_json::Value {
    let data: Vec<serde_json::Value> = encrypted.iter().map(|&b| serde_json::json!(b)).collect();
    serde_json::json!({
        "type": "Buffer",
        "data": data
    })
}
