import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Folder, Play, Square, Trash2, RefreshCw, Settings, ChevronDown, ChevronUp, FolderOpen, EyeOff, Power, RotateCcw } from "lucide-react";
import "./App.css";

interface FileDiffEvent {
  folder: string;
  changes: string[];
}

interface Change {
  timestamp: string;
  folder: string;
  changes: string[];
}

function App() {
  const [monitoredFolder, setMonitoredFolder] = useState<string>("");
  const [inputFolder, setInputFolder] = useState<string>("");
  const [isMonitoring, setIsMonitoring] = useState<boolean>(false);
  const [syncStatus, setSyncStatus] = useState<string>("Parado");
  const [changes, setChanges] = useState<Change[]>([]);
  const [error, setError] = useState<string>("");
  const [showChangelog, setShowChangelog] = useState<boolean>(true);
  const [isConfiguring, setIsConfiguring] = useState<boolean>(false);
  const [autostartEnabled, setAutostartEnabled] = useState<boolean>(false);
  const [trayStatus, setTrayStatus] = useState<boolean>(true);

  useEffect(() => {
    checkMonitoringStatus();
    loadSavedState();
    checkAutostartStatus();
    checkTrayStatus();

    const unlistenSyncStarted = listen<string>("sync_started", (event) => {
      setSyncStatus("Monitorando");
      setIsMonitoring(true);
      setError("");
      console.log("Monitoramento iniciado:", event.payload);
    });

    const unlistenSyncStopped = listen<string>("sync_stopped", (event) => {
      setSyncStatus("Parado");
      setIsMonitoring(false);
      console.log("Monitoramento parado:", event.payload);
    });

    const unlistenFileDiffs = listen<FileDiffEvent>("file_diffs", (event) => {
      const data = event.payload;
      if (data.changes && Array.isArray(data.changes)) {
        const newChange = {
          timestamp: new Date().toLocaleString('pt-BR'),
          folder: data.folder,
          changes: data.changes,
        };
        setChanges((prev) => {
          const updated = [newChange, ...prev];
          return updated.slice(0, 100);
        });
      } else {
        console.error("Formato de dados inválido no evento file_diffs:", data);
        setError("Erro: Dados de alterações inválidos recebidos");
      }
    });

    const unlistenSyncError = listen<string>("sync_error", (event) => {
      setError(event.payload);
      setSyncStatus("Erro");
      setIsMonitoring(false);
    });

    return () => {
      unlistenSyncStarted.then(fn => fn());
      unlistenSyncStopped.then(fn => fn());
      unlistenFileDiffs.then(fn => fn());
      unlistenSyncError.then(fn => fn());
    };
  }, []);

  async function checkMonitoringStatus(): Promise<void> {
    try {
      const status = await invoke<boolean>("get_monitoring_status");
      setIsMonitoring(status);
      setSyncStatus(status ? "Monitorando" : "Parado");
    } catch (error) {
      console.error("Erro ao verificar status:", error);
      setError(`Erro ao verificar status: ${error}`);
    }
  }

  async function loadSavedState(): Promise<void> {
    try {
      const savedState: any = await invoke("get_save_state");
      if (savedState && savedState.root_target) {
        setMonitoredFolder(savedState.root_target);
        setInputFolder(savedState.root_target);
      }
    } catch (error) {
      console.log("Nenhum estado salvo encontrado");
    }
  }

  async function selectFolder(): Promise<void> {
    try {
      const result = await invoke<string | null>("select_folder");
      if (result) {
        setInputFolder(result);
      }
    } catch (error) {
      setError(`Erro ao selecionar pasta: ${error}`);
      console.error("Erro ao selecionar pasta:", error);
    }
  }

  async function startMonitoring(): Promise<void> {
    if (!inputFolder.trim()) {
      setError("Por favor, selecione ou digite um caminho válido");
      return;
    }

    try {
      setError("");
      await invoke("setup", { targetFolder: inputFolder });
      setMonitoredFolder(inputFolder);
      setIsConfiguring(false);
      setChanges([]);
    } catch (error) {
      setError(`Erro ao iniciar monitoramento: ${error}`);
      console.error("Erro ao configurar pasta:", error);
    }
  }

  async function stopMonitoring(): Promise<void> {
    try {
      await invoke("stop_monitoring");
      setMonitoredFolder("");
      setInputFolder("");
      setChanges([]);
      setError("");
      setIsConfiguring(false);
      setSyncStatus("Parado");
      setIsMonitoring(false);
    } catch (error) {
      setError(`Erro ao parar monitoramento: ${error}`);
      console.error("Erro ao parar monitoramento:", error);
    }
  }

  function clearChanges(): void {
    setChanges([]);
  }

  function toggleConfiguration(): void {
    setIsConfiguring(!isConfiguring);
    if (!isConfiguring) {
      setInputFolder(monitoredFolder);
    }
    setError("");
  }

  function getStatusColor(): string {
    switch (syncStatus) {
      case "Monitorando":
        return "status-monitoring";
      case "Erro":
        return "status-error";
      default:
        return "status-stopped";
    }
  }

  function getStatusDot() {
    const statusClass = `status-dot ${getStatusColor()}`;
    return <div className={statusClass}></div>;
  }

  async function checkAutostartStatus(): Promise<void> {
    try {
      const status = await invoke<boolean>("get_autostart_status");
      setAutostartEnabled(status);
    } catch (error) {
      console.error("Erro ao verificar autostart:", error);
    }
  }

  async function toggleAutostart(): Promise<void> {
    try {
      const newStatus = await invoke<boolean>("toggle_autostart");
      setAutostartEnabled(newStatus);
    } catch (error) {
      setError(`Erro ao alterar autostart: ${error}`);
      console.error("Erro ao alterar autostart:", error);
    }
  }

  async function hideToTray(): Promise<void> {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("Erro ao ocultar janela:", error);
    }
  }

  async function checkTrayStatus(): Promise<void> {
    try {
      const status = await invoke<boolean>("check_tray_status");
      setTrayStatus(status);
    } catch (error) {
      console.error("Erro ao verificar status do tray:", error);
      setTrayStatus(false);
    }
  }

  async function recreateTray(): Promise<void> {
    try {
      await invoke("recreate_tray");
      setTrayStatus(true);
      setError(""); // Limpa erro se houver
    } catch (error) {
      setError(`Erro ao recriar tray: ${error}`);
      console.error("Erro ao recriar tray:", error);
      setTrayStatus(false);
    }
  }

  const showConfiguration = isConfiguring || !monitoredFolder;

  return (
    <div className="app-container">
      <div className="app-content">
        {/* Header */}
        <header className="app-header">
          <div className="header-content">
            <div className="logo-section">
              <FolderOpen className="logo-icon" />
              <div>
                <h1 className="app-title">EgadSync</h1>
                <p className="app-subtitle">Monitor de arquivos em tempo real</p>
              </div>
            </div>
            
            <div className="header-controls">
              <button
                onClick={toggleAutostart}
                className={`control-btn ${autostartEnabled ? 'active' : ''}`}
                title={autostartEnabled ? "Desativar inicialização automática" : "Ativar inicialização automática"}
              >
                <Power className="icon" />
                {autostartEnabled ? 'Auto-start ON' : 'Auto-start OFF'}
              </button>
              
              <button
                onClick={recreateTray}
                className={`control-btn ${!trayStatus ? 'warning' : ''}`}
                title={trayStatus ? "Recriar system tray" : "System tray com problema - clique para recriar"}
              >
                <RotateCcw className="icon" />
                {trayStatus ? 'Tray OK' : 'Recriar Tray'}
              </button>
              
              <button
                onClick={hideToTray}
                className="control-btn"
                title="Minimizar para bandeja do sistema"
              >
                <EyeOff className="icon" />
                Ocultar
              </button>
            </div>
          </div>
        </header>

        {/* Main Card */}
        <div className="main-card">
          {/* Status Header */}
          <div className="status-header">
            <div className="status-info">
              <div className="status-indicator">
                {getStatusDot()}
                <span className={`status-text ${getStatusColor()}`}>
                  {syncStatus}
                </span>
              </div>
              
              {monitoredFolder && !showConfiguration && (
                <button
                  onClick={toggleConfiguration}
                  className="settings-btn"
                  title="Configurações"
                >
                  <Settings className="icon" />
                </button>
              )}
            </div>

            {/* Current Folder Display */}
            {monitoredFolder && !showConfiguration && (
              <div className="folder-display">
                <div className="folder-label">
                  <Folder className="folder-icon" />
                  <span>Pasta monitorada</span>
                </div>
                <div className="folder-path">
                  {monitoredFolder}
                </div>
              </div>
            )}
          </div>

          {/* Configuration Panel */}
          {showConfiguration && (
            <div className="config-panel">
              <div className="form-group">
                <label className="form-label">
                  Pasta para monitorar
                </label>
                <div className="input-group">
                  <input
                    type="text"
                    value={inputFolder}
                    onChange={(e) => setInputFolder(e.target.value)}
                    placeholder="Caminho da pasta..."
                    className="form-input"
                    disabled={isMonitoring}
                  />
                  <button
                    onClick={selectFolder}
                    disabled={isMonitoring}
                    className="browse-btn"
                    title="Procurar pasta"
                  >
                    <Folder className="icon" />
                  </button>
                </div>
              </div>

              <div className="button-group">
                {!isMonitoring ? (
                  <button
                    onClick={startMonitoring}
                    className="btn btn-primary"
                  >
                    <Play className="icon" />
                    Iniciar Monitoramento
                  </button>
                ) : (
                  <button
                    onClick={stopMonitoring}
                    className="btn btn-danger"
                  >
                    <Square className="icon" />
                    Parar Monitoramento
                  </button>
                )}
                
                {isConfiguring && monitoredFolder && (
                  <button
                    onClick={toggleConfiguration}
                    className="btn btn-secondary"
                  >
                    Cancelar
                  </button>
                )}
              </div>
            </div>
          )}

          {/* Error Message */}
          {error && (
            <div className="error-message">
              <div className="error-content">
                <strong>Erro:</strong> {error}
              </div>
            </div>
          )}
        </div>

        {/* Changelog */}
        <div className="changelog-card">
          <div className="changelog-header">
            <div className="changelog-title">
              <h2>Registro de Alterações</h2>
              {changes.length > 0 && (
                <span className="changes-count">
                  {changes.length}
                </span>
              )}
            </div>
            
            <div className="changelog-controls">
              {changes.length > 0 && (
                <button
                  onClick={clearChanges}
                  className="control-btn"
                  title="Limpar histórico"
                >
                  <Trash2 className="icon" />
                </button>
              )}
              <button
                onClick={() => setShowChangelog(!showChangelog)}
                className="control-btn"
                title={showChangelog ? "Ocultar" : "Mostrar"}
              >
                {showChangelog ? <ChevronUp className="icon" /> : <ChevronDown className="icon" />}
              </button>
            </div>
          </div>

          {showChangelog && (
            <div className="changelog-content">
              {changes.length === 0 ? (
                <div className="empty-state">
                  <div className="empty-icon">
                    <RefreshCw className="icon-large" />
                  </div>
                  <h3 className="empty-title">Nenhuma alteração detectada</h3>
                  <p className="empty-subtitle">
                    {isMonitoring 
                      ? "Aguardando mudanças nos arquivos..." 
                      : "Inicie o monitoramento para ver as alterações"
                    }
                  </p>
                </div>
              ) : (
                <div className="changes-list">
                  {changes.map((change, index) => (
                    <div key={index} className="change-item">
                      <div className="change-header">
                        <span className="change-timestamp">
                          {change.timestamp}
                        </span>
                      </div>
                      <div className="change-details">
                        {change.changes.map((c, i) => (
                          <div key={i} className="change-line">
                            {c}
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;