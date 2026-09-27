mod api;
mod clipboard;
mod commands;
mod vault;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::AppState::default())
        .manage(clipboard::Clipboard::default())
        .invoke_handler(tauri::generate_handler![
            commands::status,
            commands::setup,
            commands::unlock,
            commands::lock,
            commands::reset,
            commands::get_vault,
            commands::reveal,
            commands::copy,
            commands::sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
