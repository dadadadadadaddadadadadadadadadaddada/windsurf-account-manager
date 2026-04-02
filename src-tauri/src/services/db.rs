use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;
use crate::models::{Account, AddAccountInput};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, String> {
        let db_path = Self::get_db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create db dir: {}", e))?;
        }
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        let db = Database { conn: Mutex::new(conn) };
        db.init_tables()?;
        Ok(db)
    }

    fn get_db_path() -> Result<PathBuf, String> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| "Cannot find data directory".to_string())?;
        Ok(data_dir.join("windsurf-account-manager").join("accounts.db"))
    }

    fn init_tables(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL DEFAULT '',
                refresh_token TEXT NOT NULL DEFAULT '',
                id_token TEXT NOT NULL DEFAULT '',
                id_token_expires_at INTEGER NOT NULL DEFAULT 0,
                api_key TEXT NOT NULL DEFAULT '',
                name TEXT NOT NULL DEFAULT '',
                api_server_url TEXT NOT NULL DEFAULT '',
                account_type TEXT NOT NULL DEFAULT 'Unknown',
                daily_remaining INTEGER NOT NULL DEFAULT -1,
                weekly_remaining INTEGER NOT NULL DEFAULT -1,
                daily_reset_at INTEGER NOT NULL DEFAULT 0,
                weekly_reset_at INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).map_err(|e| format!("Failed to create tables: {}", e))?;
        // Migration: check if old schema (daily_quota TEXT) exists, rebuild if needed
        let has_old_col: bool = conn.prepare("SELECT daily_quota FROM accounts LIMIT 1").is_ok();
        if has_old_col {
            eprintln!("[db] 检测到旧 schema，迁移中...");
            conn.execute_batch(
                "ALTER TABLE accounts RENAME TO accounts_old;
                 CREATE TABLE accounts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    email TEXT NOT NULL UNIQUE,
                    password TEXT NOT NULL DEFAULT '',
                    refresh_token TEXT NOT NULL DEFAULT '',
                    id_token TEXT NOT NULL DEFAULT '',
                    id_token_expires_at INTEGER NOT NULL DEFAULT 0,
                    api_key TEXT NOT NULL DEFAULT '',
                    name TEXT NOT NULL DEFAULT '',
                    api_server_url TEXT NOT NULL DEFAULT '',
                    account_type TEXT NOT NULL DEFAULT 'Unknown',
                    daily_remaining INTEGER NOT NULL DEFAULT -1,
                    weekly_remaining INTEGER NOT NULL DEFAULT -1,
                    daily_reset_at INTEGER NOT NULL DEFAULT 0,
                    weekly_reset_at INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                 );
                 INSERT INTO accounts (id, email, password, refresh_token, id_token, id_token_expires_at,
                    api_key, name, api_server_url, account_type, created_at, updated_at)
                 SELECT id, email, password, refresh_token, id_token, id_token_expires_at,
                    api_key, name, api_server_url, account_type, created_at, updated_at
                 FROM accounts_old;
                 DROP TABLE accounts_old;"
            ).map_err(|e| format!("Migration failed: {}", e))?;
            eprintln!("[db] 迁移完成");
        }
        Ok(())
    }

    pub fn add_account(&self, input: &AddAccountInput) -> Result<Account, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let account_type = if input.account_type.is_empty() { "Unknown" } else { &input.account_type };
        conn.execute(
            "INSERT INTO accounts (email, password, refresh_token, id_token, id_token_expires_at, api_key, name, api_server_url, account_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(email) DO UPDATE SET
                password = excluded.password,
                refresh_token = excluded.refresh_token,
                id_token = excluded.id_token,
                id_token_expires_at = excluded.id_token_expires_at,
                api_key = excluded.api_key,
                name = excluded.name,
                api_server_url = excluded.api_server_url,
                account_type = excluded.account_type,
                updated_at = datetime('now')",
            params![
                input.email,
                input.password,
                input.refresh_token,
                input.id_token,
                input.id_token_expires_at,
                input.api_key,
                input.name,
                input.api_server_url,
                account_type,
            ],
        ).map_err(|e| format!("Failed to add account: {}", e))?;

        conn.query_row(
            "SELECT id, email, password, refresh_token, id_token, id_token_expires_at,
                    api_key, name, api_server_url, account_type,
                    daily_remaining, weekly_remaining, daily_reset_at, weekly_reset_at,
                    created_at, updated_at
             FROM accounts WHERE email = ?1",
            params![input.email],
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password: row.get(2)?,
                    refresh_token: row.get(3)?,
                    id_token: row.get(4)?,
                    id_token_expires_at: row.get(5)?,
                    api_key: row.get(6)?,
                    name: row.get(7)?,
                    api_server_url: row.get(8)?,
                    account_type: row.get(9)?,
                    daily_remaining: row.get(10)?,
                    weekly_remaining: row.get(11)?,
                    daily_reset_at: row.get(12)?,
                    weekly_reset_at: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                })
            },
        ).map_err(|e| format!("Failed to get account: {}", e))
    }

    pub fn get_account_by_email(&self, email: &str) -> Result<Account, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT id, email, password, refresh_token, id_token, id_token_expires_at,
                    api_key, name, api_server_url, account_type,
                    daily_remaining, weekly_remaining, daily_reset_at, weekly_reset_at,
                    created_at, updated_at
             FROM accounts WHERE email = ?1",
            params![email],
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password: row.get(2)?,
                    refresh_token: row.get(3)?,
                    id_token: row.get(4)?,
                    id_token_expires_at: row.get(5)?,
                    api_key: row.get(6)?,
                    name: row.get(7)?,
                    api_server_url: row.get(8)?,
                    account_type: row.get(9)?,
                    daily_remaining: row.get(10)?,
                    weekly_remaining: row.get(11)?,
                    daily_reset_at: row.get(12)?,
                    weekly_reset_at: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                })
            },
        ).map_err(|e| format!("Failed to get account: {}", e))
    }

    pub fn list_accounts(&self) -> Result<Vec<Account>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, email, password, refresh_token, id_token, id_token_expires_at,
                    api_key, name, api_server_url, account_type,
                    daily_remaining, weekly_remaining, daily_reset_at, weekly_reset_at,
                    created_at, updated_at
             FROM accounts ORDER BY id ASC"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let accounts = stmt.query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                email: row.get(1)?,
                password: row.get(2)?,
                refresh_token: row.get(3)?,
                id_token: row.get(4)?,
                id_token_expires_at: row.get(5)?,
                api_key: row.get(6)?,
                name: row.get(7)?,
                api_server_url: row.get(8)?,
                account_type: row.get(9)?,
                daily_remaining: row.get(10)?,
                weekly_remaining: row.get(11)?,
                daily_reset_at: row.get(12)?,
                weekly_reset_at: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        }).map_err(|e| format!("Failed to query accounts: {}", e))?;

        let mut result = Vec::new();
        for account in accounts {
            result.push(account.map_err(|e| format!("Failed to read account: {}", e))?);
        }
        Ok(result)
    }

    pub fn delete_accounts(&self, ids: &[i64]) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!("DELETE FROM accounts WHERE id IN ({})", placeholders.join(","));
        let params: Vec<Box<dyn rusqlite::types::ToSql>> = ids.iter()
            .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let deleted = conn.execute(&sql, refs.as_slice())
            .map_err(|e| format!("Failed to delete accounts: {}", e))?;
        Ok(deleted)
    }

    pub fn update_account_credentials(
        &self,
        email: &str,
        id_token: &str,
        id_token_expires_at: i64,
        api_key: &str,
        name: &str,
        api_server_url: &str,
        account_type: &str,
        refresh_token: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE accounts SET
                id_token = ?1, id_token_expires_at = ?2, api_key = ?3,
                name = ?4, api_server_url = ?5, account_type = ?6,
                refresh_token = ?7, updated_at = datetime('now')
             WHERE email = ?8",
            params![id_token, id_token_expires_at, api_key, name, api_server_url, account_type, refresh_token, email],
        ).map_err(|e| format!("Failed to update account: {}", e))?;
        Ok(())
    }

    pub fn import_accounts(&self, accounts: &[AddAccountInput]) -> Result<usize, String> {
        let mut count = 0;
        for account in accounts {
            self.add_account(account)?;
            count += 1;
        }
        Ok(count)
    }

    pub fn update_plan_status(
        &self,
        email: &str,
        account_type: &str,
        daily_remaining: i32,
        weekly_remaining: i32,
        daily_reset_at: i64,
        weekly_reset_at: i64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE accounts SET
                account_type = ?1, daily_remaining = ?2, weekly_remaining = ?3,
                daily_reset_at = ?4, weekly_reset_at = ?5, updated_at = datetime('now')
             WHERE email = ?6",
            params![account_type, daily_remaining, weekly_remaining, daily_reset_at, weekly_reset_at, email],
        ).map_err(|e| format!("Failed to update plan status: {}", e))?;
        Ok(())
    }

    pub fn get_account_count(&self) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count accounts: {}", e))
    }

    pub fn get_account_count_by_type(&self, account_type: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT COUNT(*) FROM accounts WHERE account_type = ?1",
            params![account_type],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count accounts by type: {}", e))
    }
}
