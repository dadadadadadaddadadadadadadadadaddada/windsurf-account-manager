use sha2::{Sha256, Sha512, Digest};
use rand::RngCore;
use uuid::Uuid;
use std::path::PathBuf;

fn get_windsurf_storage_json_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        Ok(home.join("Library/Application Support/Windsurf/User/globalStorage/storage.json"))
    }
    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::new();
        if let Some(roaming) = dirs::config_dir() {
            candidates.push(roaming.join("Windsurf/User/globalStorage/storage.json"));
        }
        if let Some(local) = dirs::data_local_dir() {
            candidates.push(local.join("Windsurf/User/globalStorage/storage.json"));
        }
        candidates.into_iter().find(|p| p.exists())
            .ok_or_else(|| "Windsurf storage.json not found in Roaming or Local AppData".to_string())
    }
    #[cfg(target_os = "linux")]
    {
        let config = dirs::config_dir().ok_or("Cannot find config directory")?;
        Ok(config.join("Windsurf/User/globalStorage/storage.json"))
    }
}

pub fn reset_machine_id() -> Result<(), String> {
    let path = get_windsurf_storage_json_path()?;

    let mut storage: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read storage.json: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse storage.json: {}", e))?
    } else {
        serde_json::json!({})
    };

    let mut rng = rand::thread_rng();

    let mut bytes32 = [0u8; 32];
    rng.fill_bytes(&mut bytes32);
    let machine_id = format!("{:x}", Sha256::digest(&bytes32));

    let mut bytes64 = [0u8; 64];
    rng.fill_bytes(&mut bytes64);
    let mac_machine_id = format!("{:x}", Sha512::digest(&bytes64));

    let sqm_id = format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase());
    let dev_device_id = Uuid::new_v4().to_string();

    if let Some(obj) = storage.as_object_mut() {
        obj.insert("telemetry.machineId".to_string(), serde_json::json!(machine_id));
        obj.insert("telemetry.macMachineId".to_string(), serde_json::json!(mac_machine_id));
        obj.insert("telemetry.sqmId".to_string(), serde_json::json!(sqm_id));
        obj.insert("telemetry.devDeviceId".to_string(), serde_json::json!(dev_device_id));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(&storage)
        .map_err(|e| format!("Failed to serialize storage.json: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write storage.json: {}", e))?;

    Ok(())
}
