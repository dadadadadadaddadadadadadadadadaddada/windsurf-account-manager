use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const FIREBASE_API_KEY: &str = "AIzaSyDsOl-1XpT5err0Tcnx8FFod1H8gVGIycY";
const WORKER_PROXY_URL: &str = "https://windsurf.hfhddfj.cn";
const FIREBASE_TOKEN_URL: &str = "https://securetoken.googleapis.com/v1/token";
const FIREBASE_LOGIN_URL: &str = "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword";
const REGISTER_USER_URL: &str = "https://register.windsurf.com/exa.seat_management_pb.SeatManagementService/RegisterUser";
const GET_CURRENT_USER_URL: &str = "https://windsurf.com/_backend/exa.seat_management_pb.SeatManagementService/GetCurrentUser";
const GET_PLAN_STATUS_URL: &str = "https://windsurf.com/_backend/exa.seat_management_pb.SeatManagementService/GetPlanStatus";
const DEVIN_AUTH_LOGIN_URL: &str = "https://windsurf.com/_devin-auth/password/login";
const DEFAULT_API_SERVER_URL: &str = "https://server.self-serve.windsurf.com";
const REQUEST_TIMEOUT: u64 = 30;
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
const CONNECT_USER_AGENT: &str = "connect-es/1.6.1";

fn build_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

fn network_error(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        "连接超时，请检查网络连接或开启代理".to_string()
    } else if e.is_connect() {
        "无法连接到服务器，请检查网络连接或开启代理".to_string()
    } else {
        format!("网络请求失败: {}", e)
    }
}

#[derive(Serialize)]
struct RefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenResponse {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: String,
}

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
    #[serde(rename = "returnSecureToken")]
    return_secure_token: bool,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "idToken")]
    pub id_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    pub email: Option<String>,
    #[serde(rename = "expiresIn", default)]
    pub expires_in: Option<String>,
}

#[derive(Serialize)]
struct RegisterUserRequest {
    firebase_id_token: String,
}

#[derive(Deserialize)]
pub struct RegisterUserResponseRaw {
    pub api_key: Option<String>,
    pub name: Option<String>,
    pub api_server_url: Option<String>,
}

pub struct GetCurrentUserResponse {
    pub plan_name: Option<String>,
}

#[derive(Deserialize)]
pub struct DevinLoginResponse {
    pub token: String,
    pub user_id: String,
    pub email: String,
}

pub struct WindsurfPostAuthResponse {
    pub session_token: String,
    pub account_id: String,
    pub org_id: String,
}

#[derive(Serialize)]
struct WorkerProxyRequest {
    grant_type: String,
    refresh_token: String,
    api_key: String,
}

/// 用 refresh_token 刷新 Firebase token
/// 优先通过 Cloudflare Workers 代理（与老项目一致），失败后降级直连 Google API
pub async fn refresh_firebase_token(refresh_token: &str) -> Result<RefreshTokenResponse, String> {
    // 1. 优先尝试 Workers 代理（中国网络环境下 Google API 不可达）
    match refresh_via_worker_proxy(refresh_token).await {
        Ok(resp) => return Ok(resp),
        Err(e) => eprintln!("[refresh_token] Workers代理失败: {}, 降级直连Google", e),
    }

    // 2. 降级：直连 Google API
    let client = build_client()?;
    let url = format!("{}?key={}", FIREBASE_TOKEN_URL, FIREBASE_API_KEY);
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .header("Origin", "https://windsurf.com")
        .header("Referer", "https://windsurf.com/")
        .json(&RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
        })
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token刷新失败 ({}): {}", status, body));
    }

    resp.json::<RefreshTokenResponse>()
        .await
        .map_err(|e| format!("解析Token刷新响应失败: {}", e))
}

async fn refresh_via_worker_proxy(refresh_token: &str) -> Result<RefreshTokenResponse, String> {
    let client = build_client()?;
    let resp = client
        .post(WORKER_PROXY_URL)
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .json(&WorkerProxyRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            api_key: FIREBASE_API_KEY.to_string(),
        })
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Workers代理Token刷新失败 ({}): {}", status, body));
    }

    resp.json::<RefreshTokenResponse>()
        .await
        .map_err(|e| format!("解析Workers代理响应失败: {}", e))
}

/// 用邮箱密码登录获取 Firebase token（直连 identitytoolkit.googleapis.com）
pub async fn login_with_email_password(email: &str, password: &str) -> Result<LoginResponse, String> {
    let client = build_client()?;
    let url = format!("{}?key={}", FIREBASE_LOGIN_URL, FIREBASE_API_KEY);
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .header("Origin", "https://windsurf.com")
        .header("Referer", "https://windsurf.com/")
        .header("x-client-version", "Chrome/JsCore/11.0.0/FirebaseCore-web")
        .json(&LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
            return_secure_token: true,
        })
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let friendly = if body.contains("EMAIL_NOT_FOUND") {
            "邮箱不存在".to_string()
        } else if body.contains("INVALID_PASSWORD") || body.contains("INVALID_LOGIN_CREDENTIALS") {
            "邮箱或密码错误".to_string()
        } else if body.contains("USER_DISABLED") {
            "账号已被禁用".to_string()
        } else if body.contains("TOO_MANY_ATTEMPTS") {
            "尝试次数过多，请稍后再试".to_string()
        } else {
            format!("登录失败 ({}): {}", status, body)
        };
        return Err(friendly);
    }

    resp.json::<LoginResponse>()
        .await
        .map_err(|e| format!("解析登录响应失败: {}", e))
}

/// 查询用户当前订阅计划（对应老版本 getUsageInfo 中的 GetCurrentUser）
/// 2026-04 新版: request/response 均为 protobuf 格式
pub async fn get_current_user(id_token: &str) -> Result<GetCurrentUserResponse, String> {
    let client = build_client()?;
    // field 1 = token, field 2 = 1, field 4 = 1
    let body = build_proto_auth_request(id_token, &[(2, 1), (4, 1)]);

    let resp = client
        .post(GET_CURRENT_USER_URL)
        .header("Content-Type", "application/proto")
        .header("Connect-Protocol-Version", "1")
        .header("x-auth-token", id_token)
        .header("User-Agent", USER_AGENT)
        .header("Origin", "https://windsurf.com")
        .header("Referer", "https://windsurf.com/profile")
        .body(body)
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("查询用户信息失败 ({}): {}", status, body));
    }

    let raw = resp.bytes().await.map_err(|e| format!("读取响应失败: {}", e))?;
    eprintln!("[get_current_user] 响应 {} bytes", raw.len());

    // 解析 protobuf 响应，寻找 plan name
    let top_fields = parse_protobuf_fields(&raw);
    for (fnum, wt, data) in &top_fields {
        match wt {
            0 => {
                let mut bytes = [0u8; 8];
                let len = data.len().min(8);
                bytes[..len].copy_from_slice(&data[..len]);
                eprintln!("[get_current_user] field {} varint={}", fnum, u64::from_le_bytes(bytes));
            }
            2 => {
                if let Ok(s) = String::from_utf8(data.clone()) {
                    eprintln!("[get_current_user] field {} str=\"{}\"", fnum, s);
                } else {
                    eprintln!("[get_current_user] field {} bytes len={}", fnum, data.len());
                    let sub = parse_protobuf_fields(data);
                    for (sfnum, swt, sdata) in &sub {
                        match swt {
                            0 => {
                                let mut sb = [0u8; 8];
                                let sl = sdata.len().min(8);
                                sb[..sl].copy_from_slice(&sdata[..sl]);
                                eprintln!("[get_current_user]   sub field {} varint={}", sfnum, u64::from_le_bytes(sb));
                            }
                            2 => {
                                if let Ok(ss) = String::from_utf8(sdata.clone()) {
                                    eprintln!("[get_current_user]   sub field {} str=\"{}\"", sfnum, ss);
                                } else {
                                    eprintln!("[get_current_user]   sub field {} bytes len={}", sfnum, sdata.len());
                                }
                            }
                            _ => eprintln!("[get_current_user]   sub field {} wire={}", sfnum, swt),
                        }
                    }
                }
            }
            _ => eprintln!("[get_current_user] field {} wire={}", fnum, wt),
        }
    }

    // 尝试多种路径提取 plan name：
    // 路径1: field 1 (inner) -> field 1 (config) -> field 2 (plan_name) — 与 GetPlanStatus 同构
    // 路径2: 直接在顶层找 string 字段
    let mut plan_name = None;

    // 路径1: 类似 PlanStatus 结构
    if let Some(inner_data) = get_submessage_field(&top_fields, 1) {
        let inner_fields = parse_protobuf_fields(inner_data);
        // 尝试 inner.field1 (config submessage) -> field2 (plan_name)
        if let Some(config_data) = get_submessage_field(&inner_fields, 1) {
            let config_fields = parse_protobuf_fields(config_data);
            if let Some(name) = get_string_field(&config_fields, 2) {
                if !name.is_empty() {
                    plan_name = Some(name);
                }
            }
        }
        // 也尝试 inner 中直接找 plan_name string 字段
        if plan_name.is_none() {
            if let Some(name) = get_string_field(&inner_fields, 2) {
                if !name.is_empty() {
                    plan_name = Some(name);
                }
            }
        }
    }

    // 路径2: 遍历所有 submessage 找含 "Trial"/"Free"/"Pro" 的 string
    if plan_name.is_none() {
        for (_, wt, data) in &top_fields {
            if *wt == 2 {
                if let Ok(s) = String::from_utf8(data.clone()) {
                    let lower = s.to_lowercase();
                    if lower == "trial" || lower == "free" || lower == "pro" {
                        plan_name = Some(s);
                        break;
                    }
                }
            }
        }
    }

    eprintln!("[get_current_user] extracted plan_name={:?}", plan_name);

    Ok(GetCurrentUserResponse { plan_name })
}

#[derive(Debug, Default)]
pub struct PlanStatus {
    pub plan_name: String,
    pub daily_remaining: i32,
    pub weekly_remaining: i32,
    pub daily_reset_at: i64,
    pub weekly_reset_at: i64,
    pub expires_at: i64,
}

fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            buf.push(byte);
            break;
        }
        buf.push(byte | 0x80);
    }
    buf
}

fn decode_varint(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut result: u64 = 0;
    let mut shift = 0;
    loop {
        if *pos >= data.len() { return None; }
        let byte = data[*pos];
        *pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 { break; }
        shift += 7;
        if shift >= 64 { return None; }
    }
    Some(result)
}

fn build_proto_auth_request(jwt_token: &str, extra_fields: &[(u32, u64)]) -> Vec<u8> {
    let token_bytes = jwt_token.as_bytes();
    let mut body = Vec::new();
    // field 1, wire type 2 (length-delimited) = tag 0x0A
    body.push(0x0A);
    body.extend_from_slice(&encode_varint(token_bytes.len() as u64));
    body.extend_from_slice(token_bytes);
    // additional varint fields
    for &(field_number, value) in extra_fields {
        let tag = (field_number << 3) | 0; // wire type 0 = varint
        body.extend_from_slice(&encode_varint(tag as u64));
        body.extend_from_slice(&encode_varint(value));
    }
    body
}

fn parse_protobuf_fields(data: &[u8]) -> Vec<(u32, u8, Vec<u8>)> {
    let mut fields = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let tag = match decode_varint(data, &mut pos) {
            Some(t) => t,
            None => break,
        };
        let field_number = (tag >> 3) as u32;
        let wire_type = (tag & 0x07) as u8;
        match wire_type {
            0 => { // varint
                let start = pos;
                if let Some(val) = decode_varint(data, &mut pos) {
                    fields.push((field_number, wire_type, val.to_le_bytes().to_vec()));
                    let _ = start;
                } else { break; }
            }
            2 => { // length-delimited
                if let Some(len) = decode_varint(data, &mut pos) {
                    let len = len as usize;
                    if pos + len > data.len() { break; }
                    fields.push((field_number, wire_type, data[pos..pos + len].to_vec()));
                    pos += len;
                } else { break; }
            }
            1 => { pos += 8; } // 64-bit
            5 => { pos += 4; } // 32-bit
            _ => break,
        }
    }
    fields
}

fn get_varint_field(fields: &[(u32, u8, Vec<u8>)], field_number: u32) -> Option<u64> {
    fields.iter()
        .find(|(num, wt, _)| *num == field_number && *wt == 0)
        .and_then(|(_, _, data)| {
            let mut bytes = [0u8; 8];
            let len = data.len().min(8);
            bytes[..len].copy_from_slice(&data[..len]);
            Some(u64::from_le_bytes(bytes))
        })
}

fn get_string_field(fields: &[(u32, u8, Vec<u8>)], field_number: u32) -> Option<String> {
    fields.iter()
        .find(|(num, wt, _)| *num == field_number && *wt == 2)
        .and_then(|(_, _, data)| String::from_utf8(data.clone()).ok())
}

fn get_submessage_field<'a>(fields: &'a [(u32, u8, Vec<u8>)], field_number: u32) -> Option<&'a [u8]> {
    fields.iter()
        .find(|(num, wt, _)| *num == field_number && *wt == 2)
        .map(|(_, _, data)| data.as_slice())
}

pub async fn get_plan_status(id_token: &str) -> Result<PlanStatus, String> {
    let client = build_client()?;
    // field 1 = token, field 2 = 1
    let body = build_proto_auth_request(id_token, &[(2, 1)]);

    let resp = client
        .post(GET_PLAN_STATUS_URL)
        .header("Content-Type", "application/proto")
        .header("Connect-Protocol-Version", "1")
        .header("x-auth-token", id_token)
        .header("Origin", "https://windsurf.com")
        .header("Referer", "https://windsurf.com/profile")
        .header("User-Agent", USER_AGENT)
        .body(body)
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GetPlanStatus 失败 ({}): {}", status, body));
    }

    let raw = resp.bytes().await.map_err(|e| format!("读取响应失败: {}", e))?;
    eprintln!("[get_plan_status] 响应 {} bytes", raw.len());

    // 解析顶层 message，取 field 1（PlanStatus 子消息）
    let top_fields = parse_protobuf_fields(&raw);
    let inner_data = get_submessage_field(&top_fields, 1)
        .ok_or_else(|| "响应中未找到 PlanStatus 字段".to_string())?;
    let inner_fields = parse_protobuf_fields(inner_data);

    // DEBUG: 打印所有内层 protobuf 字段（用于定位到期时间字段）
    for (fnum, wt, data) in &inner_fields {
        match wt {
            0 => {
                let mut bytes = [0u8; 8];
                let len = data.len().min(8);
                bytes[..len].copy_from_slice(&data[..len]);
                let val = u64::from_le_bytes(bytes);
                eprintln!("[protobuf] inner field {}  wire=varint  value={}", fnum, val);
            }
            2 => {
                if let Ok(s) = String::from_utf8(data.clone()) {
                    eprintln!("[protobuf] inner field {}  wire=bytes   str=\"{}\"", fnum, s);
                } else {
                    eprintln!("[protobuf] inner field {}  wire=bytes   len={}", fnum, data.len());
                    // 递归解析子消息字段
                    let sub = parse_protobuf_fields(data);
                    for (sfnum, swt, sdata) in &sub {
                        match swt {
                            0 => {
                                let mut sb = [0u8; 8];
                                let sl = sdata.len().min(8);
                                sb[..sl].copy_from_slice(&sdata[..sl]);
                                eprintln!("[protobuf]   sub field {}  wire=varint  value={}", sfnum, u64::from_le_bytes(sb));
                            }
                            2 => {
                                if let Ok(ss) = String::from_utf8(sdata.clone()) {
                                    eprintln!("[protobuf]   sub field {}  wire=bytes   str=\"{}\"", sfnum, ss);
                                } else {
                                    eprintln!("[protobuf]   sub field {}  wire=bytes   len={}", sfnum, sdata.len());
                                }
                            }
                            _ => eprintln!("[protobuf]   sub field {}  wire={}", sfnum, swt),
                        }
                    }
                }
            }
            _ => eprintln!("[protobuf] inner field {}  wire={}", fnum, wt),
        }
    }

    // 提取 plan_name: field 1 (PlanConfig) -> field 2 (plan_name)
    let plan_name = get_submessage_field(&inner_fields, 1)
        .map(|config_data| {
            let config_fields = parse_protobuf_fields(config_data);
            get_string_field(&config_fields, 2).unwrap_or_default()
        })
        .unwrap_or_default();

    let daily_remaining = get_varint_field(&inner_fields, 14).unwrap_or(0) as i32;
    let weekly_remaining = get_varint_field(&inner_fields, 15).unwrap_or(0) as i32;
    let daily_reset_at = get_varint_field(&inner_fields, 17).unwrap_or(0) as i64;
    let weekly_reset_at = get_varint_field(&inner_fields, 18).unwrap_or(0) as i64;

    // 到期时间: inner field 3 (submessage) -> sub field 1 (varint, unix timestamp)
    let expires_at = get_submessage_field(&inner_fields, 3)
        .map(|data| {
            let sub = parse_protobuf_fields(data);
            get_varint_field(&sub, 1).unwrap_or(0) as i64
        })
        .unwrap_or(0);

    eprintln!("[get_plan_status] plan={}, daily={}%, weekly={}%, daily_reset={}, weekly_reset={}, expires_at={}",
        plan_name, daily_remaining, weekly_remaining, daily_reset_at, weekly_reset_at, expires_at);

    Ok(PlanStatus {
        plan_name,
        daily_remaining,
        weekly_remaining,
        daily_reset_at,
        weekly_reset_at,
        expires_at,
    })
}

/// Devin 原生登录（绕过 Firebase）
pub async fn devin_auth_login(email: &str, password: &str) -> Result<DevinLoginResponse, String> {
    let client = build_client()?;
    let resp = client
        .post(DEVIN_AUTH_LOGIN_URL)
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .header("Origin", "https://windsurf.com")
        .header("Referer", "https://windsurf.com/account/login")
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let friendly = if body.contains("Invalid email or password") {
            "邮箱或密码错误".to_string()
        } else {
            format!("Devin登录失败 ({}): {}", status, body)
        };
        return Err(friendly);
    }

    resp.json::<DevinLoginResponse>()
        .await
        .map_err(|e| format!("解析Devin登录响应失败: {}", e))
}

/// WindsurfPostAuth: auth1_token → session_token（一步到位，无需 apiKey）
pub async fn windsurf_post_auth(auth1_token: &str, api_server_url: &str) -> Result<WindsurfPostAuthResponse, String> {
    let client = build_client()?;
    let url = format!("{}/exa.seat_management_pb.SeatManagementService/WindsurfPostAuth", api_server_url);

    let mut body = Vec::new();
    encode_string_field(&mut body, 1, auth1_token);

    let resp = client
        .post(&url)
        .header("Content-Type", "application/proto")
        .header("Connect-Protocol-Version", "1")
        .header("X-Api-Key", auth1_token)
        .header("User-Agent", CONNECT_USER_AGENT)
        .body(body)
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("WindsurfPostAuth失败 ({}): {}", status, body));
    }

    let raw = resp.bytes().await.map_err(|e| format!("读取PostAuth响应失败: {}", e))?;
    let fields = parse_protobuf_fields(&raw);

    let session_token = get_string_field(&fields, 1)
        .ok_or_else(|| "PostAuth响应中未找到session_token".to_string())?;
    let account_id = get_string_field(&fields, 4).unwrap_or_default();
    let org_id = get_string_field(&fields, 5).unwrap_or_default();

    eprintln!("[windsurf_post_auth] session_token len={}, account={}, org={}", session_token.len(), account_id, org_id);

    Ok(WindsurfPostAuthResponse {
        session_token,
        account_id,
        org_id,
    })
}

/// 用 apiKey 获取 devin-session-token（新版 auth 流程）
/// api_server_url 形如 "https://server.self-serve.windsurf.com"
pub async fn get_self_devin_session_token(api_key: &str, api_server_url: &str) -> Result<String, String> {
    let client = build_client()?;
    let url = format!("{}/exa.seat_management_pb.SeatManagementService/GetSelfDevinSessionToken", api_server_url);

    // 构造 Metadata 子消息: ide_name(1), extension_version(2), api_key(3), locale(4), os(5), ide_version(7), session_id(10), extension_name(12)
    let session_id = uuid::Uuid::new_v4().to_string();
    let mut metadata = Vec::new();
    encode_string_field(&mut metadata, 1, "windsurf");
    encode_string_field(&mut metadata, 7, "1.99.0");
    encode_string_field(&mut metadata, 12, "windsurf");
    encode_string_field(&mut metadata, 2, "2.25.0");
    encode_string_field(&mut metadata, 3, api_key);
    encode_string_field(&mut metadata, 4, "en");
    encode_string_field(&mut metadata, 5, "darwin");
    encode_string_field(&mut metadata, 10, &session_id);

    // 构造 GetSelfDevinSessionTokenRequest: metadata(1)
    let mut body = Vec::new();
    let tag = (1u64 << 3) | 2;
    body.extend_from_slice(&encode_varint(tag));
    body.extend_from_slice(&encode_varint(metadata.len() as u64));
    body.extend_from_slice(&metadata);

    let resp = client
        .post(&url)
        .header("Content-Type", "application/proto")
        .header("Connect-Protocol-Version", "1")
        .header("X-Api-Key", api_key)
        .header("User-Agent", CONNECT_USER_AGENT)
        .body(body)
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("获取SessionToken失败 ({}): {}", status, body));
    }

    let raw = resp.bytes().await.map_err(|e| format!("读取SessionToken响应失败: {}", e))?;
    let fields = parse_protobuf_fields(&raw);
    let token = get_string_field(&fields, 1)
        .ok_or_else(|| "SessionToken响应中未找到token字段".to_string())?;

    if !token.starts_with("devin-session-token$") {
        return Err(format!("SessionToken格式异常: {}", &token[..token.len().min(50)]));
    }

    eprintln!("[get_self_devin_session_token] 获取成功, len={}", token.len());
    Ok(token)
}

fn encode_string_field(buf: &mut Vec<u8>, field_number: u32, value: &str) {
    let tag = ((field_number as u64) << 3) | 2;
    let data = value.as_bytes();
    buf.extend_from_slice(&encode_varint(tag));
    buf.extend_from_slice(&encode_varint(data.len() as u64));
    buf.extend_from_slice(data);
}

/// 用 token（session_token 或 idToken）注册，获取 apiKey / name / apiServerUrl
/// 使用 protobuf 格式，兼容 session_token 和 Firebase idToken
pub async fn register_user(token: &str) -> Result<RegisterUserResponseRaw, String> {
    let client = build_client()?;

    // 构造 protobuf 请求: field 1 = token
    let mut body = Vec::new();
    encode_string_field(&mut body, 1, token);

    let resp = client
        .post(REGISTER_USER_URL)
        .header("Content-Type", "application/proto")
        .header("Connect-Protocol-Version", "1")
        .header("User-Agent", CONNECT_USER_AGENT)
        .header("Origin", "https://windsurf.com")
        .header("Referer", "https://windsurf.com/")
        .body(body)
        .send()
        .await
        .map_err(|e| network_error(&e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("注册用户失败 ({}): {}", status, body));
    }

    let raw = resp.bytes().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let fields = parse_protobuf_fields(&raw);

    let api_key = get_string_field(&fields, 1);
    let name = get_string_field(&fields, 2);
    let api_server_url = get_string_field(&fields, 3);

    eprintln!("[register_user] api_key len={}, name={:?}, url={:?}",
        api_key.as_ref().map(|s| s.len()).unwrap_or(0), name, api_server_url);

    Ok(RegisterUserResponseRaw { api_key, name, api_server_url })
}
