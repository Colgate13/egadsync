use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt;

pub mod config;
pub mod error;
pub mod file_tracker;
pub mod logger;
pub mod sync;

use config::Config;
use error::FileTrackerError;
use file_tracker::FileTracker;
use sync::start_sync_loop;

#[derive(Debug, Clone, PartialEq)]
enum TrayMenuId {
    Open,
    Quit,
}

impl TrayMenuId {
    fn as_str(&self) -> &'static str {
        match self {
            TrayMenuId::Open => "open",
            TrayMenuId::Quit => "quit",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "open" => Some(TrayMenuId::Open),
            "quit" => Some(TrayMenuId::Quit),
            _ => None,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            TrayMenuId::Open => "Abrir",
            TrayMenuId::Quit => "Sair",
        }
    }
}

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

fn create_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let open_item = MenuItem::with_id(
        app,
        TrayMenuId::Open.as_str(),
        TrayMenuId::Open.label(),
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(
        app,
        TrayMenuId::Quit.as_str(),
        TrayMenuId::Quit.label(),
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;
    Ok(menu)
}

fn handle_tray_event(app: &AppHandle, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click { .. } => {
            handle_menu_action(app, TrayMenuId::Open);
        }
        TrayIconEvent::DoubleClick { .. } => {
            handle_menu_action(app, TrayMenuId::Open);
        }
        _ => {}
    }
}

fn handle_menu_action(app: &AppHandle, menu_id: TrayMenuId) {
    match menu_id {
        TrayMenuId::Open => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        TrayMenuId::Quit => {
            app.exit(0);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(|app| {
            // Configure system tray
            let menu = create_tray_menu(app.handle())?;

            let _tray = TrayIconBuilder::with_id("main_tray")
                .menu(&menu)
                .tooltip("EgadSync")
                .on_tray_icon_event(|tray, event| {
                    handle_tray_event(tray.app_handle(), event);
                })
                .on_menu_event(|app, event| {
                    if let Some(menu_id) = TrayMenuId::from_str(event.id().as_ref()) {
                        handle_menu_action(app, menu_id);
                    }
                })
                .build(app)?;

            // Configure window behavior to hide instead of closing completely
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            // Ensure auto-start is always enabled
            let autostart_manager = app.autolaunch();
            let _ = autostart_manager.enable();

            // Start sync if it was previously active
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
