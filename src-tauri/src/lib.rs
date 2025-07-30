use tauri::{AppHandle, Emitter};

pub mod config;
pub mod error;
pub mod file_tracker;
pub mod logger;
pub mod sync;

use config::Config;
use error::FileTrackerError;
use file_tracker::FileTracker;
use sync::{start_sync_loop};

#[tauri::command]
fn stop_monitoring() -> Result<(), FileTrackerError> {
    FileTracker::stop_monitoring_and_delete_state()
}

#[tauri::command]
fn get_save_state() -> Result<FileTracker, FileTrackerError> {
    FileTracker::get(&Config::default())
}

#[tauri::command]
fn get_monitoring_status() -> bool {
    FileTracker::is_monitoring_active()
}

#[tauri::command]
fn setup(app: AppHandle, target_folder: &str) {
    let target_folder = target_folder.to_string();
    let config = Config::default();
    tauri::async_runtime::spawn(async move {
        match FileTracker::new(&target_folder, &config) {
            Ok(_) => {
                let _ = app.emit("sync_started", "Monitoramento iniciado");
                start_sync_loop(app);
            }
            Err(e) => {
                log::error!("Failed to initialize FileTracker: {}", e);
                let _ = app.emit("sync_error", format!("Erro ao iniciar: {}", e));
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if FileTracker::is_monitoring_active() {
                start_sync_loop(app.handle().clone());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            setup,
            get_save_state,
            get_monitoring_status,
            stop_monitoring
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
