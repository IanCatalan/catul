// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:catul.db", utils::migrations::migrations()) // Add migrations for the SQLite database using 'migrations' function from utils/migrations.rs
                .build(),
        )
        .invoke_handler(tauri::generate_handler![commands::saludo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
