use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use sha1::Sha1;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

const SALT: &[u8] = b"saltysalt";
const IV: [u8; 16] = [0x20u8; 16];
const ITERATIONS: u32 = 1003;
const KEY_LEN: usize = 16;

#[allow(dead_code)]
fn derive_key(password: &str) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2::pbkdf2_hmac::<Sha1>(
        password.as_bytes(),
        SALT,
        ITERATIONS,
        &mut key,
    );
    key
}

#[allow(dead_code)]
fn encrypt_aes_cbc(password: &str, prefix: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let key = derive_key(password);
    let block_size = 16;
    let padded_len = ((data.len() / block_size) + 1) * block_size;
    let mut buf = vec![0u8; padded_len];
    buf[..data.len()].copy_from_slice(data);

    let encrypted = Aes128CbcEnc::new(&key.into(), &IV.into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, data.len())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut result = Vec::with_capacity(prefix.len() + encrypted.len());
    result.extend_from_slice(prefix);
    result.extend_from_slice(encrypted);
    Ok(result)
}

// ---- macOS: Keychain password + AES-128-CBC, "v10" prefix ----
#[cfg(target_os = "macos")]
pub fn encrypt_sessions(sessions_json: &str) -> Result<Vec<u8>, String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Windsurf Safe Storage", "-w"])
        .output()
        .map_err(|e| format!("Failed to run security command: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get Windsurf Safe Storage password from Keychain".to_string());
    }

    let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
    encrypt_aes_cbc(&password, b"v10", sessions_json.as_bytes())
}

// ---- Windows: AES-256-GCM with DPAPI-protected key from Local State (Chromium v80+) ----
#[cfg(target_os = "windows")]
pub fn encrypt_sessions(sessions_json: &str) -> Result<Vec<u8>, String> {
    let key = get_os_crypt_key()?;

    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    use rand::RngCore;

    let cipher = Aes256Gcm::new((&key).into());

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, sessions_json.as_bytes())
        .map_err(|e| format!("AES-256-GCM encryption failed: {}", e))?;

    // 格式: "v10" + nonce(12 bytes) + ciphertext(含 16-byte tag)
    let mut result = Vec::with_capacity(3 + 12 + ciphertext.len());
    result.extend_from_slice(b"v10");
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

#[cfg(target_os = "windows")]
fn dpapi_decrypt(encrypted: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

    let input_blob = CRYPT_INTEGER_BLOB {
        cbData: encrypted.len() as u32,
        pbData: encrypted.as_ptr() as *mut u8,
    };
    let mut output_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    unsafe {
        CryptUnprotectData(
            &input_blob,
            None,
            None,
            None,
            None,
            0,
            &mut output_blob,
        ).map_err(|e| format!("DPAPI CryptUnprotectData failed: {}", e))?;

        let decrypted = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec();
        extern "system" { fn LocalFree(hmem: *mut u8) -> *mut u8; }
        LocalFree(output_blob.pbData);
        Ok(decrypted)
    }
}

#[cfg(target_os = "windows")]
fn get_os_crypt_key() -> Result<[u8; 32], String> {
    use base64::Engine;

    // 读取 Local State 文件
    let appdata = dirs::config_dir().ok_or("Cannot find config directory")?;
    let local_state_path = appdata.join("Windsurf").join("Local State");

    let content = std::fs::read_to_string(&local_state_path)
        .map_err(|e| format!("Failed to read Local State ({}): {}", local_state_path.display(), e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse Local State: {}", e))?;

    let encrypted_key_b64 = json.get("os_crypt")
        .and_then(|v| v.get("encrypted_key"))
        .and_then(|v| v.as_str())
        .ok_or("os_crypt.encrypted_key not found in Local State")?;

    // Base64 解码
    let encrypted_key = base64::engine::general_purpose::STANDARD
        .decode(encrypted_key_b64)
        .map_err(|e| format!("Failed to base64 decode key: {}", e))?;

    // 去掉 "DPAPI" 前缀 (5 bytes)
    if encrypted_key.len() < 5 || &encrypted_key[..5] != b"DPAPI" {
        return Err("Invalid encrypted key: missing DPAPI prefix".to_string());
    }

    // DPAPI 解密得到原始 AES-256 密钥
    let raw_key = dpapi_decrypt(&encrypted_key[5..])?;
    if raw_key.len() != 32 {
        return Err(format!("Unexpected AES key length: {} (expected 32)", raw_key.len()));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&raw_key);
    Ok(key)
}

// ---- Linux: default password "peanuts" + AES-128-CBC, "v11" prefix ----
#[cfg(target_os = "linux")]
pub fn encrypt_sessions(sessions_json: &str) -> Result<Vec<u8>, String> {
    encrypt_aes_cbc("peanuts", b"v11", sessions_json.as_bytes())
}

pub fn build_encrypted_sessions_value(encrypted: &[u8]) -> serde_json::Value {
    let data: Vec<serde_json::Value> = encrypted.iter().map(|&b| serde_json::json!(b)).collect();
    serde_json::json!({
        "type": "Buffer",
        "data": data
    })
}
