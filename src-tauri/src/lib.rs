use tauri::{Emitter, Manager};
use crate::file_tracker::{FileTracker, FileTrackerError};

#[derive(serde::Serialize, Clone)]
struct FileDiffPayload {
    folder: String,
    changes: Vec<String>,
}

mod file_tracker;

#[tauri::command]
fn stop_monitoring() -> Result<(), FileTrackerError> {
    match FileTracker::stop_monitoring_and_delete_state() {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

#[tauri::command]
fn get_save_state() -> Result<FileTracker, FileTrackerError> {
    match FileTracker::get() {
        Ok(data) => Ok(data),
        Err(err) => Err(err),
    }
}

#[tauri::command]
fn get_monitoring_status() -> bool {
    // Check if state.json exists to infer monitoring status
    std::path::Path::new("./state.json").exists()
}

#[tauri::command]
fn setup(app: tauri::AppHandle, target_folder: &str) {
    let target_folder = String::from(target_folder);
    tauri::async_runtime::spawn(async move {
        match file_tracker::FileTracker::new(target_folder) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Erro ao iniciar um novo FileTracker: {}", e);
                let _ = app.emit("sync_error", format!("Erro ao iniciar: {}", e));
                return;
            }
        };

        // Emit sync_started event
        let _ = app.emit("sync_started", "Monitoramento iniciado");
        // Call Sync in background
        start_sync_loop(app.app_handle().clone());
    });
}

fn start_sync_loop(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut file_tracker = match file_tracker::FileTracker::get() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Estado não encontrado em storage, terminando execução do sync: {}", e);
                let _ = app_handle.emit("sync_error", format!("Estado não encontrado: {}", e));
                return;
            }
        };

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        println!("File tracker configurado, iniciando sync em background");

        loop {
            interval.tick().await;

            match file_tracker.diff().await {
                Ok(changes) => {
                    if !changes.is_empty() {
                        println!("Mudanças detectadas:");
                        for change in &changes {
                            println!("{}", change);
                        }

                        // Convert changes to strings for the frontend
                        let change_strings: Vec<String> = changes.iter().map(|c| c.to_string()).collect();
                        let payload = FileDiffPayload {
                            folder: file_tracker.root_target.display().to_string(),
                            changes: change_strings,
                        };
                        let _ = app_handle.emit("file_diffs", payload);
                        let _ = file_tracker.save().map_err(|err| {
                            println!("Erro ao salvar estado em storage: {}", err);
                            let _ = app_handle.emit("sync_error", format!("Erro ao salvar estado: {}", err));
                        });
                    }
                }
                Err(e) => {
                    eprintln!("Erro ao calcular diff: {}", e);
                    let _ = app_handle.emit("sync_error", format!("Erro ao calcular diff: {}", e));
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
            // Start sync in background if state.json exists
            if std::path::Path::new("./state.json").exists() {
                start_sync_loop(app.handle().clone());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![setup, get_save_state, get_monitoring_status, stop_monitoring])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
