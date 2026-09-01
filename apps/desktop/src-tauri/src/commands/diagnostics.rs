#[tauri::command]
#[specta::specta]
pub fn log_frontend_error(message: String, stack: Option<String>) {
    match stack {
        Some(stack) => log::error!("frontend crash: {message}\n{stack}"),
        None => log::error!("frontend crash: {message}"),
    }
}
