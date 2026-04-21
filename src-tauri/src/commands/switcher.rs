use tauri::{AppHandle, Emitter, State};
use crate::models::SwitchProgress;
use crate::services::{db::Database, firebase, machine_id, windsurf_db, windsurf_process};

fn emit_progress(app: &AppHandle, step: u32, total: u32, message: &str) {
    let _ = app.emit("switch-progress", SwitchProgress {
        step,
        total,
        message: message.to_string(),
    });
}

#[tauri::command]
pub async fn switch_account(app: AppHandle, db: State<'_, Database>, account_id: i64) -> Result<String, String> {
    let accounts = db.list_accounts()?;
    let account = accounts.iter().find(|a| a.id == account_id)
        .ok_or_else(|| format!("Account with id {} not found", account_id))?
        .clone();

    let total = 5;

    // Step 1: Close Windsurf (blocking I/O + thread::sleep, must use spawn_blocking)
    emit_progress(&app, 1, total, "正在检测并关闭 Windsurf...");
    tokio::task::spawn_blocking(|| {
        if windsurf_process::is_running() {
            windsurf_process::close_windsurf()?;
            // 额外等待3秒确保文件句柄释放（与老项目一致）
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
        Ok::<(), String>(())
    }).await.map_err(|e| format!("spawn_blocking failed: {}", e))?.map_err(|e| e)?;

    // Step 2: Reset machine ID (blocking file I/O)
    emit_progress(&app, 2, total, "正在重置机器 ID...");
    tokio::task::spawn_blocking(|| {
        machine_id::reset_machine_id()
    }).await.map_err(|e| format!("spawn_blocking failed: {}", e))?.map_err(|e| e)?;

    // Step 3: Get credentials
    emit_progress(&app, 3, total, "正在获取账号凭证...");
    let (api_key, name, api_server_url, new_id_token, new_refresh_token) = get_credentials(&account).await?;

    // 获取 session token: 优先 Devin 登录，降级 apiKey
    let session_token = {
        let mut st: Option<String> = None;
        if !account.email.is_empty() && !account.password.is_empty() {
            if let Ok(login_resp) = firebase::devin_auth_login(&account.email, &account.password).await {
                let srv = if api_server_url.is_empty() { "https://server.self-serve.windsurf.com" } else { &api_server_url };
                if let Ok(post_auth) = firebase::windsurf_post_auth(&login_resp.token, srv).await {
                    st = Some(post_auth.session_token);
                }
            }
        }
        if st.is_none() && !api_key.is_empty() && !api_server_url.is_empty() {
            st = firebase::get_self_devin_session_token(&api_key, &api_server_url).await.ok();
        }
        st
    };

    let account_type = match &session_token {
        Some(st) => match firebase::get_current_user(st).await {
            Ok(user_resp) => {
                let plan = user_resp.plan_name.unwrap_or_else(|| "Free".to_string());
                eprintln!("[切换] GetCurrentUser planName={}", plan);
                plan
            }
            Err(e) => {
                eprintln!("[切换] GetCurrentUser 失败(不影响主流程): {}", e);
                account.account_type.clone()
            }
        }
        None => {
            eprintln!("[切换] 无SessionToken, 跳过GetCurrentUser");
            account.account_type.clone()
        }
    };

    // Update local DB with new credentials
    let _ = db.update_account_credentials(
        &account.email,
        &new_id_token,
        chrono::Utc::now().timestamp() + 3600,
        &api_key,
        &name,
        &api_server_url,
        &account_type,
        &new_refresh_token,
    );

    // Step 4: Write to Windsurf database (blocking SQLite I/O)
    emit_progress(&app, 4, total, "正在写入认证数据...");
    let ak = api_key.clone();
    let nm = name.clone();
    let em = account.email.clone();
    let asu = api_server_url.clone();
    tokio::task::spawn_blocking(move || {
        windsurf_db::write_auth_data(&ak, &nm, &em, &asu)
    }).await.map_err(|e| format!("spawn_blocking failed: {}", e))?.map_err(|e| e)?;

    // Step 5: Launch Windsurf (blocking I/O + thread::sleep)
    emit_progress(&app, 5, total, "正在启动 Windsurf...");
    let windsurf_path = db.get_setting("windsurf_path").unwrap_or(None);
    tokio::task::spawn_blocking(move || {
        windsurf_process::launch_windsurf(windsurf_path)
    }).await.map_err(|e| format!("spawn_blocking failed: {}", e))?.map_err(|e| e)?;

    let _ = db.set_setting("active_account_email", &account.email);
    Ok(format!("切换成功: {}", account.email))
}

#[tauri::command]
pub fn get_active_account(db: State<'_, Database>) -> Result<Option<String>, String> {
    db.get_setting("active_account_email")
}

/// 获取 Firebase idToken（优先 refresh_token，降级到邮箱密码登录）
/// 返回 (id_token, refresh_token)
async fn get_firebase_token(account: &crate::models::Account) -> Result<(String, String), String> {
    // 1. 已有有效 idToken 且未过期
    let now = chrono::Utc::now().timestamp();
    if !account.id_token.is_empty() && account.id_token_expires_at > now {
        return Ok((account.id_token.clone(), account.refresh_token.clone()));
    }

    // 2. 有 refresh_token → 用 refresh_token 刷新
    if !account.refresh_token.is_empty() {
        match firebase::refresh_firebase_token(&account.refresh_token).await {
            Ok(resp) => return Ok((resp.id_token, resp.refresh_token)),
            Err(e) => {
                eprintln!("[refresh_token 刷新失败] {}: {}, 降级到邮箱密码登录", account.email, e);
            }
        }
    }

    // 3. 降级：用邮箱密码登录
    if !account.email.is_empty() && !account.password.is_empty() {
        let login_resp = firebase::login_with_email_password(&account.email, &account.password).await?;
        return Ok((login_resp.id_token, login_resp.refresh_token));
    }

    Err("账号缺少 refresh_token 和邮箱密码，无法获取凭证".to_string())
}

async fn get_credentials(account: &crate::models::Account) -> Result<(String, String, String, String, String), String> {
    // 已有完整凭证 → 直接复用，不走 Firebase
    let has_full_credentials = !account.api_key.is_empty()
        && !account.name.is_empty()
        && !account.api_server_url.is_empty();

    if has_full_credentials {
        return Ok((
            account.api_key.clone(),
            account.name.clone(),
            account.api_server_url.clone(),
            account.id_token.clone(),
            account.refresh_token.clone(),
        ));
    }

    // 无 apiKey → 必须通过 Firebase + RegisterUser 获取
    let (id_token, refresh_token) = get_firebase_token(account).await?;
    let register_resp = firebase::register_user(&id_token).await?;

    Ok((
        register_resp.api_key.unwrap_or_default(),
        register_resp.name.unwrap_or_default(),
        register_resp.api_server_url.unwrap_or_else(|| "https://server.self-serve.windsurf.com".to_string()),
        id_token,
        refresh_token,
    ))
}

#[tauri::command]
pub async fn get_account_token(db: State<'_, Database>, account_id: i64) -> Result<String, String> {
    eprintln!("[获取Token] 开始, account_id={}", account_id);
    let accounts = db.list_accounts()?;
    let account = accounts.iter().find(|a| a.id == account_id)
        .ok_or_else(|| format!("Account with id {} not found", account_id))?
        .clone();

    eprintln!("[获取Token] 账号: {}, password长度: {}, refresh_token长度: {}", account.email, account.password.len(), account.refresh_token.len());

    // ====== 新流程：优先 Devin 原生登录（无需 Firebase / Google API） ======
    let mut session_token: Option<String> = None;
    let mut api_key = account.api_key.clone();
    let mut name = account.name.clone();
    let mut api_server_url = account.api_server_url.clone();
    let mut id_token = account.id_token.clone();
    let mut new_refresh_token = account.refresh_token.clone();

    // 路径1: Devin 原生登录 → WindsurfPostAuth → session_token
    if !account.email.is_empty() && !account.password.is_empty() {
        match firebase::devin_auth_login(&account.email, &account.password).await {
            Ok(login_resp) => {
                eprintln!("[获取Token] Devin登录成功: auth1 len={}", login_resp.token.len());
                let srv = if api_server_url.is_empty() { "https://server.self-serve.windsurf.com" } else { &api_server_url };
                match firebase::windsurf_post_auth(&login_resp.token, srv).await {
                    Ok(post_auth) => {
                        eprintln!("[获取Token] WindsurfPostAuth成功, session_token len={}", post_auth.session_token.len());
                        session_token = Some(post_auth.session_token);
                    }
                    Err(e) => eprintln!("[获取Token] WindsurfPostAuth失败: {}", e),
                }
            }
            Err(e) => eprintln!("[获取Token] Devin登录失败: {}, 降级Firebase", e),
        }
    }

    // 路径2: 如果还没有 apiKey，通过 Firebase → RegisterUser 获取
    if api_key.is_empty() {
        eprintln!("[获取Token] 无apiKey, 通过Firebase获取...");
        let (firebase_id_token, firebase_refresh_token) = get_firebase_token(&account).await?;
        id_token = firebase_id_token.clone();
        new_refresh_token = firebase_refresh_token;

        let register_resp = firebase::register_user(&firebase_id_token).await.map_err(|e| {
            eprintln!("[获取Token] register_user 失败: {}", e);
            e
        })?;
        api_key = register_resp.api_key.unwrap_or_default();
        name = register_resp.name.unwrap_or_default();
        api_server_url = register_resp.api_server_url
            .unwrap_or_else(|| "https://server.self-serve.windsurf.com".to_string());
        eprintln!("[获取Token] RegisterUser成功: apiKey len={}", api_key.len());
    }

    // 路径3: 如果 Devin 登录未获取 session_token，降级用 apiKey → GetSelfDevinSessionToken
    if session_token.is_none() && !api_key.is_empty() && !api_server_url.is_empty() {
        eprintln!("[获取Token] 降级: apiKey → GetSelfDevinSessionToken");
        session_token = firebase::get_self_devin_session_token(&api_key, &api_server_url).await.ok();
    }

    // 调用 GetCurrentUser 获取真实订阅类型
    let account_type = match &session_token {
        Some(st) => match firebase::get_current_user(st).await {
            Ok(user_resp) => {
                let plan = user_resp.plan_name.unwrap_or_else(|| "Free".to_string());
                eprintln!("[获取Token] GetCurrentUser planName={}", plan);
                plan
            }
            Err(e) => {
                eprintln!("[获取Token] GetCurrentUser 失败(不影响主流程): {}", e);
                account.account_type.clone()
            }
        }
        None => account.account_type.clone(),
    };

    db.update_account_credentials(
        &account.email,
        &id_token,
        chrono::Utc::now().timestamp() + 3600,
        &api_key,
        &name,
        &api_server_url,
        &account_type,
        &new_refresh_token,
    )?;
    eprintln!("[获取Token] DB更新成功: {}", account.email);

    // 调用 GetPlanStatus 获取额度信息
    if let Some(st) = &session_token {
        match firebase::get_plan_status(st).await {
            Ok(plan) => {
                let plan_type = if plan.plan_name.is_empty() { &account_type } else { &plan.plan_name };
                db.update_plan_status(
                    &account.email,
                    plan_type,
                    plan.daily_remaining,
                    plan.weekly_remaining,
                    plan.daily_reset_at,
                    plan.weekly_reset_at,
                    plan.expires_at,
                )?;
                eprintln!("[获取Token] 额度信息已更新: daily={}%, weekly={}%", plan.daily_remaining, plan.weekly_remaining);
            }
            Err(e) => eprintln!("[获取Token] GetPlanStatus 失败(不影响主流程): {}", e),
        }
    }

    Ok(format!("获取Token成功: {}", account.email))
}

#[tauri::command]
pub async fn refresh_plan_status(db: State<'_, Database>, account_id: i64) -> Result<String, String> {
    let accounts = db.list_accounts()?;
    let account = accounts.iter().find(|a| a.id == account_id)
        .ok_or_else(|| format!("Account with id {} not found", account_id))?
        .clone();

    // 优先 Devin 原生登录获取 session token，降级 apiKey → GetSelfDevinSessionToken
    let session_token = if !account.email.is_empty() && !account.password.is_empty() {
        match firebase::devin_auth_login(&account.email, &account.password).await {
            Ok(login_resp) => {
                let srv = if account.api_server_url.is_empty() { "https://server.self-serve.windsurf.com" } else { &account.api_server_url };
                match firebase::windsurf_post_auth(&login_resp.token, srv).await {
                    Ok(post_auth) => Ok(post_auth.session_token),
                    Err(e) => Err(format!("WindsurfPostAuth失败: {}", e)),
                }
            }
            Err(e) => Err(format!("Devin登录失败: {}", e)),
        }
    } else if !account.api_key.is_empty() && !account.api_server_url.is_empty() {
        firebase::get_self_devin_session_token(&account.api_key, &account.api_server_url).await
    } else {
        Err("缺少凭证，请先获取Token".to_string())
    }?;
    let plan = firebase::get_plan_status(&session_token).await?;
    let plan_type = if plan.plan_name.is_empty() { &account.account_type } else { &plan.plan_name };
    db.update_plan_status(
        &account.email, plan_type,
        plan.daily_remaining, plan.weekly_remaining,
        plan.daily_reset_at, plan.weekly_reset_at,
        plan.expires_at,
    )?;
    Ok(format!("{}%/{}%", plan.daily_remaining, plan.weekly_remaining))
}

#[tauri::command]
pub fn check_windsurf_status() -> serde_json::Value {
    serde_json::json!({
        "running": windsurf_process::is_running()
    })
}

#[tauri::command]
pub fn get_windsurf_path(db: State<'_, Database>) -> Result<serde_json::Value, String> {
    let custom = db.get_setting("windsurf_path").unwrap_or(None);
    let detected = windsurf_process::detect_windsurf_path();
    Ok(serde_json::json!({
        "custom": custom,
        "detected": detected,
        "effective": custom.as_deref().filter(|p| !p.is_empty() && std::path::Path::new(p).exists()).map(String::from).or(detected),
    }))
}

#[tauri::command]
pub fn set_windsurf_path(db: State<'_, Database>, path: String) -> Result<(), String> {
    db.set_setting("windsurf_path", &path)
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<serde_json::Value, String> {
    let current_version = app.config().version.clone().unwrap_or_default();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let resp = client
        .get("https://adminabc.xiaobiao.ltd/api/app-versions/check/windsurf")
        .send()
        .await
        .map_err(|e| format!("检查更新失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("检查更新失败 ({})", resp.status()));
    }

    let body: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析更新响应失败: {}", e))?;

    let remote_version = body["data"]["version"].as_str().unwrap_or("").to_string();
    let normalize_ver = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v').split('.').filter_map(|s| s.parse().ok()).collect()
    };
    let remote_parts = normalize_ver(&remote_version);
    let current_parts = normalize_ver(&current_version);
    let has_update = !remote_parts.is_empty() && remote_parts > current_parts;

    Ok(serde_json::json!({
        "has_update": has_update,
        "current_version": current_version,
        "remote_version": remote_version,
        "update_content": body["data"]["update_content"].as_str().unwrap_or(""),
        "download_url": body["data"]["download_url"].as_str().unwrap_or(""),
    }))
}
