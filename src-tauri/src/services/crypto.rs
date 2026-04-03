use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use sha1::Sha1;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

const SALT: &[u8] = b"saltysalt";
const IV: [u8; 16] = [0x20u8; 16];
const ITERATIONS: u32 = 1003;
const KEY_LEN: usize = 16;

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

// ---- Windows: DPAPI encryption, "v10" prefix ----
#[cfg(target_os = "windows")]
pub fn encrypt_sessions(sessions_json: &str) -> Result<Vec<u8>, String> {
    use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN};

    let data = sessions_json.as_bytes();
    let mut input_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    unsafe {
        CryptProtectData(
            &mut input_blob,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output_blob,
        ).map_err(|e| format!("DPAPI CryptProtectData failed: {}", e))?;

        let encrypted = std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec();
        windows::Win32::System::Memory::LocalFree(Some(windows::Win32::Foundation::HLOCAL(output_blob.pbData as _)));

        let mut result = Vec::with_capacity(3 + encrypted.len());
        result.extend_from_slice(b"v10");
        result.extend_from_slice(&encrypted);
        Ok(result)
    }
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
