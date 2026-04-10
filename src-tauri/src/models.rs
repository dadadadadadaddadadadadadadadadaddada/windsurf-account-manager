use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub refresh_token: String,
    #[serde(default)]
    pub id_token: String,
    #[serde(default)]
    pub id_token_expires_at: i64,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub api_server_url: String,
    #[serde(default)]
    pub account_type: String,
    #[serde(default)]
    pub daily_remaining: i32,
    #[serde(default)]
    pub weekly_remaining: i32,
    #[serde(default)]
    pub daily_reset_at: i64,
    #[serde(default)]
    pub weekly_reset_at: i64,
    #[serde(default)]
    pub expires_at: i64,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub group_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AddAccountInput {
    pub email: String,
    pub password: String,
    pub refresh_token: String,
    #[serde(default)]
    pub id_token: String,
    #[serde(default)]
    pub id_token_expires_at: i64,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub api_server_url: String,
    #[serde(default)]
    pub account_type: String,
    #[serde(default)]
    pub group_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirebaseTokenResponse {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUserResponse {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub name: String,
    #[serde(rename = "apiServerUrl")]
    pub api_server_url: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SwitchProgress {
    pub step: u32,
    pub total: u32,
    pub message: String,
}
