import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface Change {
  timestamp: string;
  folder: string;
  changes: string[];
}

interface FileDiffEvent {
  folder: string;
  changes: string[];
}

function App() {
  const [greetMsg, setGreetMsg] = useState<string>("");
  const [name, setName] = useState<string>("");
  const [monitoredFolder, setMonitoredFolder] = useState<string>("");
  const [isMonitoring, setIsMonitoring] = useState<boolean>(false);
  const [syncStatus, setSyncStatus] = useState<string>("Parado");
  const [changes, setChanges] = useState<Change[]>([]);
  const [error, setError] = useState<string>("");

  useEffect(() => {
    // Verifica o status inicial do monitoramento
    checkMonitoringStatus();

    // Listeners para eventos do backend
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
      setChanges(prev => [...prev, {
        timestamp: new Date().toLocaleTimeString(),
        folder: data.folder,
        changes: data.changes
      }]);
    });

    const unlistenSyncError = listen<string>("sync_error", (event) => {
      setError(event.payload);
      setSyncStatus("Erro");
      setIsMonitoring(false);
    });

    // Cleanup listeners
    return () => {
      unlistenSyncStarted.then(fn => fn());
      unlistenSyncStopped.then(fn => fn());
      unlistenFileDiffs.then(fn => fn());
      unlistenSyncError.then(fn => fn());
    };
  }, []);

  async function greet(): Promise<void> {
    try {
      const result = await invoke<string>("greet", { name });
      setGreetMsg(result);
    } catch (error) {
      console.error("Erro ao cumprimentar:", error);
    }
  }

  async function checkMonitoringStatus(): Promise<void> {
    try {
      const status = await invoke<boolean>("get_monitoring_status");
      setIsMonitoring(status);
      setSyncStatus(status ? "Monitorando" : "Parado");
    } catch (error) {
      console.error("Erro ao verificar status:", error);
    }
  }

  async function selectFolder(): Promise<void> {
    try {
      const selected = await invoke<string | null>("select_folder");

      if (selected) {
        setMonitoredFolder(selected);
        await invoke("set_monitored_folder", { folderPath: selected });
        setError("");
        setChanges([]); // Limpa mudanças anteriores
      }
    } catch (error) {
      setError(`Erro ao configurar pasta: ${error}`);
      console.error("Erro ao selecionar pasta:", error);
    }
  }

  async function stopMonitoring(): Promise<void> {
    try {
      await invoke("stop_monitoring");
      setMonitoredFolder("");
      setChanges([]);
      setError("");
    } catch (error) {
      setError(`Erro ao parar monitoramento: ${error}`);
      console.error("Erro ao parar monitoramento:", error);
    }
  }

  function clearChanges(): void {
    setChanges([]);
  }

  return (
    <main className="container">
      <h1>Welcome to EgadSync</h1>
      
      <div>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
      </div>

      {/* Seção de teste do greet */}
      <section className="section">
        <h2>Teste de Comunicação</h2>
        <form
          className="row"
          onSubmit={(e) => {
            e.preventDefault();
            greet();
          }}
        >
          <input
            id="greet-input"
            value={name}
            onChange={(e) => setName(e.currentTarget.value)}
            placeholder="Digite um nome..."
          />
          <button type="submit">Cumprimentar</button>
        </form>
        {greetMsg && <p>{greetMsg}</p>}
      </section>

      {/* Seção de monitoramento de arquivos */}
      <section className="section">
        <h2>Monitoramento de Arquivos</h2>
        
        <div className="status-bar">
          <span className={`status ${isMonitoring ? 'active' : 'inactive'}`}>
            Status: {syncStatus}
          </span>
          {monitoredFolder && (
            <span className="folder-path">
              Pasta: {monitoredFolder}
            </span>
          )}
        </div>

        <div className="controls">
          <button onClick={selectFolder} disabled={false}>
            {monitoredFolder ? "Alterar Pasta" : "Selecionar Pasta"}
          </button>
          
          {isMonitoring && (
            <button onClick={stopMonitoring} className="stop-btn">
              Parar Monitoramento
            </button>
          )}
          
          {changes.length > 0 && (
            <button onClick={clearChanges} className="clear-btn">
              Limpar Histórico
            </button>
          )}
        </div>

        {error && (
          <div className="error-message">
            <strong>Erro:</strong> {error}
          </div>
        )}

        {changes.length > 0 && (
          <div className="changes-section">
            <h3>Mudanças Detectadas ({changes.length})</h3>
            <div className="changes-list">
              {changes.slice(-10).reverse().map((change, index) => (
                <div key={index} className="change-item">
                  <div className="change-header">
                    <span className="timestamp">{change.timestamp}</span>
                    <span className="folder-name">{change.folder}</span>
                  </div>
                  <div className="change-details">
                    {change.changes.map((c, i) => (
                      <div key={i} className="change-line">{c}</div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </section>
    </main>
  );
}

export default App;