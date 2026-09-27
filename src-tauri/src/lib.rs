mod api;
mod clipboard;
mod commands;
mod settings;
mod vault;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::AppState::default())
        .manage(clipboard::Clipboard::default())
        .manage(settings::SettingsState::default())
        .setup(|app| {
            use tauri::Manager;
            let path = settings::settings_path(app.handle())?;
            *app.state::<settings::SettingsState>().0.lock().unwrap() = settings::Settings::load(&path);
            Ok(())
        })
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
            settings::get_settings,
            settings::set_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
