mod commands;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod state;

use state::ConnectionRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ConnectionRegistry::default())
        .invoke_handler(tauri::generate_handler![
            commands::connection::db_connect,
            commands::connection::db_test_connection,
            commands::connection::db_disconnect,
            commands::connection::db_list_active_connections,
            commands::schema::db_list_tables,
            commands::schema::db_get_table_columns,
            commands::table::db_fetch_table_rows,
            commands::table::db_update_json_cell,
            commands::table::db_update_cell_text,
            commands::table::db_delete_rows,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
