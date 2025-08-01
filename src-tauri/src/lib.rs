use tauri::{AppHandle, Emitter, Manager, WindowEvent, Window};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, TrayIcon};
use tauri::menu::{MenuBuilder, MenuItem, MenuEvent};
use parking_lot::Mutex;
use std::sync::Arc;

pub mod config;
pub mod error;
pub mod file_tracker;
pub mod logger;
pub mod sync;

use config::Config;
use error::FileTrackerError;
use file_tracker::FileTracker;
use sync::{start_sync_loop};

// Estrutura para gerenciar o system tray de forma robusta
struct TrayManager {
    tray: Arc<Mutex<Option<TrayIcon<tauri::Wry>>>>,
    app_handle: AppHandle,
}

impl TrayManager {
    fn new(app_handle: AppHandle) -> Self {
        Self {
            tray: Arc::new(Mutex::new(None)),
            app_handle,
        }
    }

    fn create_tray(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Criando/recriando system tray...");
        
        // Remove o tray existente se houver
        {
            let mut tray_lock = self.tray.lock();
            if let Some(existing_tray) = tray_lock.take() {
                log::info!("Removendo tray existente");
                drop(existing_tray);
            }
        }

        // Cria novo menu
        let menu = create_tray_menu(&self.app_handle)?;
        
        // Cria novo tray
        let new_tray = TrayIconBuilder::new()
            .menu(&menu)
            .on_menu_event({
                let _app_handle = self.app_handle.clone();
                move |app, event| handle_menu_event(app, event)
            })
            .on_tray_icon_event({
                let _app_handle = self.app_handle.clone();
                move |tray, event| handle_tray_event(tray.app_handle(), event)
            })
            .build(&self.app_handle)?;

        // Armazena o novo tray
        {
            let mut tray_lock = self.tray.lock();
            *tray_lock = Some(new_tray);
        }

        log::info!("System tray criado com sucesso");
        Ok(())
    }

    fn recreate_tray(&self) {
        log::warn!("Tentando recriar system tray após erro...");
        
        // Tenta recriar até 3 vezes com delay
        for attempt in 1..=3 {
            match self.create_tray() {
                Ok(()) => {
                    log::info!("System tray recriado com sucesso na tentativa {}", attempt);
                    return;
                }
                Err(e) => {
                    log::error!("Falha ao recriar tray (tentativa {}): {}", attempt, e);
                    if attempt < 3 {
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                }
            }
        }
        
        log::error!("Falha ao recriar system tray após 3 tentativas");
    }
}

// Instância global do gerenciador de tray
static TRAY_MANAGER: std::sync::OnceLock<TrayManager> = std::sync::OnceLock::new();

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
async fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_autostart(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    
    let manager = app.autolaunch();
    let is_enabled = manager.is_enabled().map_err(|e| e.to_string())?;
    
    if is_enabled {
        manager.disable().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        manager.enable().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
async fn get_autostart_status(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    
    let manager = app.autolaunch();
    manager.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
async fn recreate_tray() -> Result<(), String> {
    if let Some(tray_manager) = TRAY_MANAGER.get() {
        tray_manager.recreate_tray();
        Ok(())
    } else {
        Err("Tray manager não inicializado".to_string())
    }
}

#[tauri::command]
async fn check_tray_status() -> Result<bool, String> {
    if let Some(tray_manager) = TRAY_MANAGER.get() {
        let tray_lock = tray_manager.tray.lock();
        Ok(tray_lock.is_some())
    } else {
        Ok(false)
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

fn create_tray_menu(app: &AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Mostrar", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Ocultar", true, None::<&str>)?;
    let separator1 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let recreate_item = MenuItem::with_id(app, "recreate_tray", "Recriar Tray", true, None::<&str>)?;
    let separator2 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    
    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &hide_item, &separator1, &recreate_item, &separator2, &quit_item])
        .build()?;
    
    Ok(menu)
}

fn handle_tray_event(app: &AppHandle, event: TrayIconEvent) {
    log::debug!("Evento do tray recebido: {:?}", event);
    
    match event {
        TrayIconEvent::Click { button, .. } => {
            if button == tauri::tray::MouseButton::Left {
                if let Some(window) = app.get_webview_window("main") {
                    match window.is_visible() {
                        Ok(true) => {
                            if let Err(e) = window.hide() {
                                log::error!("Erro ao ocultar janela: {}", e);
                            }
                        }
                        Ok(false) => {
                            if let Err(e) = window.show() {
                                log::error!("Erro ao mostrar janela: {}", e);
                            } else if let Err(e) = window.set_focus() {
                                log::error!("Erro ao focar janela: {}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("Erro ao verificar visibilidade da janela: {}", e);
                            // Tenta recriar o tray em caso de erro
                            if let Some(tray_manager) = TRAY_MANAGER.get() {
                                tray_manager.recreate_tray();
                            }
                        }
                    }
                } else {
                    log::error!("Janela principal não encontrada");
                }
            }
        }
        _ => {}
    }
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    log::debug!("Evento do menu recebido: {:?}", event.id());
    
    match event.id().as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.show() {
                    log::error!("Erro ao mostrar janela via menu: {}", e);
                    // Tenta recriar o tray em caso de erro
                    if let Some(tray_manager) = TRAY_MANAGER.get() {
                        tray_manager.recreate_tray();
                    }
                } else if let Err(e) = window.set_focus() {
                    log::error!("Erro ao focar janela via menu: {}", e);
                }
            } else {
                log::error!("Janela principal não encontrada no menu show");
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.hide() {
                    log::error!("Erro ao ocultar janela via menu: {}", e);
                }
            } else {
                log::error!("Janela principal não encontrada no menu hide");
            }
        }
        "quit" => {
            log::info!("Saindo da aplicação via menu");
            app.exit(0);
        }
        "recreate_tray" => {
            log::info!("Recriando tray via menu");
            if let Some(tray_manager) = TRAY_MANAGER.get() {
                tray_manager.recreate_tray();
            }
        }
        _ => {
            log::warn!("ID de menu desconhecido: {}", event.id().as_ref());
        }
    }
}

fn handle_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            window.hide().unwrap();
            api.prevent_close();
        }
        _ => {}
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
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // Inicializa o gerenciador de tray
            let tray_manager = TrayManager::new(app.handle().clone());
            
            // Cria o tray inicial
            if let Err(e) = tray_manager.create_tray() {
                log::error!("Erro ao criar system tray inicial: {}", e);
                return Err(e);
            }
            
            // Armazena o gerenciador globalmente
            if let Err(_) = TRAY_MANAGER.set(tray_manager) {
                log::error!("Falha ao definir gerenciador de tray global");
                return Err("Falha ao definir gerenciador de tray global".into());
            }

            let config = Config::default();
            if FileTracker::is_monitoring_active(&config) {
                start_sync_loop(app.handle().clone());
            }
            
            Ok(())
        })
        .on_window_event(handle_window_event)
        .invoke_handler(tauri::generate_handler![
            setup,
            get_save_state,
            get_monitoring_status,
            stop_monitoring,
            select_folder,
            show_window,
            hide_window,
            toggle_autostart,
            get_autostart_status,
            recreate_tray,
            check_tray_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
