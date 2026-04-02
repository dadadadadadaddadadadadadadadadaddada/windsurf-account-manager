use tauri::State;
use crate::models::{Account, AddAccountInput};
use crate::services::db::Database;

#[tauri::command]
pub fn list_accounts(db: State<'_, Database>) -> Result<Vec<Account>, String> {
    db.list_accounts()
}

#[tauri::command]
pub fn add_account(db: State<'_, Database>, input: AddAccountInput) -> Result<Account, String> {
    db.add_account(&input)
}

#[tauri::command]
pub fn delete_accounts(db: State<'_, Database>, ids: Vec<i64>) -> Result<usize, String> {
    db.delete_accounts(&ids)
}

#[tauri::command]
pub fn import_accounts(db: State<'_, Database>, accounts: Vec<AddAccountInput>) -> Result<usize, String> {
    db.import_accounts(&accounts)
}

#[tauri::command]
pub fn export_accounts(db: State<'_, Database>) -> Result<Vec<Account>, String> {
    db.list_accounts()
}

#[tauri::command]
pub fn get_account_stats(db: State<'_, Database>) -> Result<serde_json::Value, String> {
    let total = db.get_account_count()?;
    let free = db.get_account_count_by_type("Free")?;
    let trial = db.get_account_count_by_type("Trial")?;
    let pro = db.get_account_count_by_type("Pro")?;
    let unknown = db.get_account_count_by_type("Unknown")?;
    Ok(serde_json::json!({
        "total": total,
        "free": free,
        "trial": trial,
        "pro": pro,
        "unknown": unknown
    }))
}
