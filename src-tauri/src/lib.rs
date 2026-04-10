mod models;
mod services;
mod commands;

use services::db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::new().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            commands::account::list_accounts,
            commands::account::add_account,
            commands::account::delete_accounts,
            commands::account::import_accounts,
            commands::account::export_accounts,
            commands::account::get_account_stats,
            commands::switcher::switch_account,
            commands::switcher::get_account_token,
            commands::switcher::refresh_plan_status,
            commands::switcher::check_windsurf_status,
            commands::switcher::get_windsurf_path,
            commands::switcher::set_windsurf_path,
            commands::switcher::check_update,
            commands::switcher::get_active_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
