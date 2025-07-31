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
    let config = Config::default();
    FileTracker::stop_monitoring_and_delete_state(&config)
}

#[tauri::command]
fn get_save_state() -> Result<FileTracker, FileTrackerError> {
    FileTracker::get(&Config::default())
}

#[tauri::command]
fn get_monitoring_status() -> bool {
    let config = Config::default();
    FileTracker::is_monitoring_active(&config)
}

#[tauri::command]
async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use tokio::sync::oneshot;
    
    let (tx, rx) = oneshot::channel();
    
    app.dialog().file().pick_folder(move |folder_path| {
        let _ = tx.send(folder_path.map(|p| p.to_string()));
    });
    
    match rx.await {
        Ok(result) => Ok(result),
        Err(_) => Ok(None),
    }
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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config = Config::default();
            if FileTracker::is_monitoring_active(&config) {
                start_sync_loop(app.handle().clone());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            setup,
            get_save_state,
            get_monitoring_status,
            stop_monitoring,
            select_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
