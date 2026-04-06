use rusqlite::{Connection, params};
use std::path::PathBuf;
use uuid::Uuid;
use crate::services::crypto;

fn get_vscdb_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        Ok(home.join("Library/Application Support/Windsurf/User/globalStorage/state.vscdb"))
    }
    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::new();
        if let Some(roaming) = dirs::config_dir() {
            candidates.push(roaming.join("Windsurf/User/globalStorage/state.vscdb"));
        }
        if let Some(local) = dirs::data_local_dir() {
            candidates.push(local.join("Windsurf/User/globalStorage/state.vscdb"));
        }
        candidates.into_iter().find(|p| p.exists())
            .ok_or_else(|| "Windsurf state.vscdb not found in Roaming or Local AppData".to_string())
    }
    #[cfg(target_os = "linux")]
    {
        let config = dirs::config_dir().ok_or("Cannot find config directory")?;
        Ok(config.join("Windsurf/User/globalStorage/state.vscdb"))
    }
}

pub fn write_auth_data(
    api_key: &str,
    name: &str,
    email: &str,
    api_server_url: &str,
) -> Result<(), String> {
    let db_path = get_vscdb_path()?;
    if !db_path.exists() {
        return Err(format!("Windsurf database not found: {}", db_path.display()));
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open state.vscdb: {}", e))?;

    // Step 1: Clear old auth data
    conn.execute("DELETE FROM ItemTable WHERE key LIKE 'windsurf_auth-%'", [])
        .map_err(|e| format!("Failed to clear windsurf_auth: {}", e))?;
    conn.execute("DELETE FROM ItemTable WHERE key LIKE 'secret://%'", [])
        .map_err(|e| format!("Failed to clear secrets: {}", e))?;
    conn.execute("DELETE FROM ItemTable WHERE key = 'windsurfAuthStatus'", [])
        .map_err(|e| format!("Failed to clear auth status: {}", e))?;

    // Step 2: Build and encrypt sessions
    let session_id = Uuid::new_v4().to_string();
    let sessions_data = serde_json::json!([{
        "id": session_id,
        "accessToken": api_key,
        "account": { "label": name, "id": name },
        "scopes": []
    }]);
    let sessions_json = serde_json::to_string(&sessions_data)
        .map_err(|e| format!("Failed to serialize sessions: {}", e))?;

    let encrypted = crypto::encrypt_sessions(&sessions_json)?;
    let encrypted_value = crypto::build_encrypted_sessions_value(&encrypted);
    let encrypted_str = serde_json::to_string(&encrypted_value)
        .map_err(|e| format!("Failed to serialize encrypted sessions: {}", e))?;

    let sessions_key = r#"secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.sessions"}"#;
    upsert_item(&conn, sessions_key, &encrypted_str)?;

    // Step 2.5: Encrypt and write apiServerUrl
    let url = if api_server_url.is_empty() { "https://server.self-serve.windsurf.com" } else { api_server_url };
    let encrypted_url = crypto::encrypt_sessions(url)?;
    let encrypted_url_value = crypto::build_encrypted_sessions_value(&encrypted_url);
    let encrypted_url_str = serde_json::to_string(&encrypted_url_value)
        .map_err(|e| format!("Failed to serialize encrypted apiServerUrl: {}", e))?;
    let url_key = r#"secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.apiServerUrl"}"#;
    upsert_item(&conn, url_key, &encrypted_url_str)?;

    // Step 3: Write auth status
    let auth_status = serde_json::json!({
        "name": name,
        "apiKey": api_key,
        "email": email,
        "teamId": Uuid::new_v4().to_string(),
        "planName": "Pro"
    });
    let auth_status_str = serde_json::to_string(&auth_status)
        .map_err(|e| format!("Failed to serialize auth status: {}", e))?;
    upsert_item(&conn, "windsurfAuthStatus", &auth_status_str)?;

    // Step 4: Write Codeium config
    let codeium_config = serde_json::json!({
        "codeium.installationId": Uuid::new_v4().to_string(),
        "codeium.apiKey": api_key,
        "apiServerUrl": if api_server_url.is_empty() { "https://server.self-serve.windsurf.com" } else { api_server_url },
        "codeium.hasOneTimeUpdatedUnspecifiedMode": true
    });
    let codeium_config_str = serde_json::to_string(&codeium_config)
        .map_err(|e| format!("Failed to serialize codeium config: {}", e))?;
    upsert_item(&conn, "codeium.windsurf", &codeium_config_str)?;

    // Step 5: Write auth name and user ID
    upsert_item(&conn, "codeium.windsurf-windsurf_auth", name)?;
    upsert_item(&conn, "codeium.windsurf-windsurf_auth-", &Uuid::new_v4().to_string())?;

    // Step 5.5: Write auth session entry
    let auth_session_key = format!("windsurf_auth-{}", name);
    upsert_item(&conn, &auth_session_key, "[]")?;

    // Step 6: Force WAL checkpoint and switch to rollback journal mode
    // The old Electron project uses sql.js which exports a fresh DB in rollback mode,
    // so we must replicate that behavior: flush WAL → switch to DELETE mode → clean up files
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| format!("WAL checkpoint failed: {}", e))?;
    conn.execute_batch("PRAGMA journal_mode=DELETE;")
        .map_err(|e| format!("Failed to switch journal mode: {}", e))?;

    // Explicitly close the connection before cleaning up WAL/SHM files
    drop(conn);

    // Remove residual WAL and SHM files to match the old project's behavior
    let db_str = db_path.to_string_lossy().to_string();
    let _ = std::fs::remove_file(format!("{}-wal", db_str));
    let _ = std::fs::remove_file(format!("{}-shm", db_str));

    eprintln!("[write_auth_data] 写入完成(journal_mode=DELETE): email={}, name={}", email, name);
    Ok(())
}

fn upsert_item(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
        params![key, value],
    ).map_err(|e| format!("Failed to upsert key '{}': {}", key, e))?;
    Ok(())
}
