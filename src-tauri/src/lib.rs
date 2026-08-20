mod commands;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod state;

use state::{ConnectionRegistry, ExportJobRegistry};
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::connection::db_connect,
        commands::connection::db_test_connection,
        commands::connection::db_disconnect,
        commands::connection::db_list_active_connections,
        commands::connection::db_save_connection,
        commands::connection::db_list_saved_connections,
        commands::connection::db_connect_saved,
        commands::connection::db_delete_saved_connection,
        commands::connection::db_rename_saved_connection,
        commands::schema::db_list_tables,
        commands::schema::db_get_table_columns,
        commands::table::db_fetch_table_rows,
        commands::table::db_count_table_rows,
        commands::table::db_update_json_cell,
        commands::table::db_update_cell_text,
        commands::table::db_delete_rows,
        commands::table::db_insert_row,
        commands::query::db_execute_query,
        commands::query::db_save_query,
        commands::query::db_list_saved_queries,
        commands::query::db_delete_saved_query,
        commands::query::db_list_query_history,
        commands::query::db_clear_query_history,
        commands::export::export_default_directory,
        commands::export::export_pick_directory,
        commands::export::export_run_table,
        commands::export::export_run_rows,
        commands::export::export_cancel,
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/tauri/bindings.ts",
        )
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(ConnectionRegistry::default())
        .manage(ExportJobRegistry::default())
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
