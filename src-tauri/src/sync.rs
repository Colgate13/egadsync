use crate::config::Config;
use crate::file_tracker::{FileChange, FileTracker};
use tauri::{AppHandle, Emitter};
use tokio::time::{self, Duration};

/// Payload for file difference events sent to the frontend.
#[derive(serde::Serialize, Clone)]
pub struct FileDiffPayload {
    folder: String,
    changes: Vec<String>,
}

/// Starts the background sync loop to monitor file changes.
pub fn start_sync_loop(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let config = Config::default();
        let mut file_tracker = match FileTracker::get(&config) {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to load state: {}", e);
                let _ = app_handle.emit("sync_error", format!("Estado não encontrado: {}", e));
                return;
            }
        };

        let mut interval = time::interval(Duration::from_secs(config.sync_interval_secs));
        log::info!("Starting background sync loop with interval {}s", config.sync_interval_secs);

        loop {
            interval.tick().await;
            match file_tracker.diff().await {
                Ok(changes) => {
                    if !changes.is_empty() {
                        log_changes(&changes);
                        let changes = FileTracker::get_only_file_changes(changes);

                        let payload = create_payload(&file_tracker, &changes);
                        let _ = app_handle.emit("file_diffs", payload);
                        if let Err(e) = file_tracker.save(&config) {
                            log::error!("Failed to save state: {}", e);
                            let _ = app_handle.emit("sync_error", format!("Erro ao salvar estado: {}", e));
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to compute diff: {}", e);
                    let _ = app_handle.emit("sync_error", format!("Erro ao calcular diff: {}", e));
                }
            }
        }
    });
}

/// Logs detected file changes.
fn log_changes(changes: &[FileChange]) {
    log::info!("Detected changes:");
    for change in changes {
        log::info!("{}", change);
    }
}

/// Creates a payload for the frontend from file changes.
fn create_payload(file_tracker: &FileTracker, changes: &[FileChange]) -> FileDiffPayload {
    FileDiffPayload {
        folder: file_tracker.root_target.display().to_string(),
        changes: changes.iter().map(|c| c.to_string()).collect(),
    }
}
