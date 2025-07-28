use tauri::{Emitter, Manager};

use crate::file_tracker::{FileTracker, FileTrackerError};
mod file_tracker;


#[tauri::command]
fn get_save_state() -> Result<FileTracker, FileTrackerError> {
    match FileTracker::get() {
        Ok(data) => Ok(data),
        Err(err) => Err(err)
    }
}

// Before go make a genric response and generic responses error enums, and mapped FileTrackerError to convert in generic response erro
#[tauri::command]
fn setup(app: tauri::AppHandle, target_folder: &str) {
    let target_folder = String::from(target_folder);
    tauri::async_runtime::spawn(async move {
        match file_tracker::FileTracker::new(target_folder) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Erro ao iniciar um novo FileTracker: {}", e);
                return;
            }
        };

        // Call Sync in background
        start_sync_loop(app.app_handle().clone());
    });
}

fn start_sync_loop(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut file_tracker = match file_tracker::FileTracker::get() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Estato nao encontrado em storage termiando execucao do sync: {}", e);
                return;
            }
        };

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        println!("File tracker configurado iniciando sync em background");

        loop {
            interval.tick().await;

            match file_tracker.diff().await {
                Ok(changes) => {
                    println!("Mudanças detectadas:");
                    for change in &changes {
                        println!("{}", change);
                    }

                    let _ = app_handle.emit("file_diffs", changes);
                    let _ = file_tracker
                        .save()
                        .map_err(|err| {
                            println!("Error to save state in storage {err}");
                        });
                }
                Err(e) => {
                    eprintln!("Erro ao calcular diff: {}", e);
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Start sync in background
            start_sync_loop(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![setup, get_save_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
