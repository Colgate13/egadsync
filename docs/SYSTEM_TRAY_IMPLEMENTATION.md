# Sistema de System Tray e Auto-start - Documentação Técnica Completa

## 📋 Visão Geral

Este documento detalha todas as modificações implementadas para adicionar funcionalidades de **System Tray** e **Auto-start** na aplicação EgadSync. A implementação permite que a aplicação rode em background, seja controlada via bandeja do sistema e inicie automaticamente com o sistema operacional.

---

## 🎯 Objetivos Alcançados

1. ✅ **System Tray**: Ícone na bandeja do sistema com menu contextual
2. ✅ **Minimizar para Tray**: Janela oculta ao invés de fechada
3. ✅ **Background Process**: Aplicação roda continuamente em background
4. ✅ **Auto-start**: Inicialização automática com o sistema operacional
5. ✅ **Controle via Interface**: Botões para controlar essas funcionalidades

---

## 🔧 Modificações por Arquivo

### 1. **`src-tauri/Cargo.toml`** - Dependências Rust

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-opener = "2"
tauri-plugin-autostart = "2"  # ← NOVA DEPENDÊNCIA
```

**Explicação:**
- `tauri-plugin-autostart = "2"`: Plugin oficial do Tauri v2 para gerenciar auto-start
- Fornece APIs cross-platform para Windows, Linux e macOS
- Integra-se nativamente com os sistemas de inicialização de cada OS

---

### 2. **`package.json`** - Dependências JavaScript

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-autostart": "^2",  // ← NOVA DEPENDÊNCIA
    "@tauri-apps/plugin-dialog": "^2.3.1",
    "@tauri-apps/plugin-opener": "^2"
  }
}
```

**Explicação:**
- `@tauri-apps/plugin-autostart`: Bindings JavaScript para o plugin autostart
- Permite controlar auto-start diretamente do frontend React
- Fornece funções `enable()`, `disable()`, `isEnabled()`

---

### 3. **`src-tauri/tauri.conf.json`** - Configuração do Tauri

```json
{
  "app": {
    "windows": [
      {
        "title": "egadsync",
        "width": 800,
        "height": 600,
        "visible": false  // ← MODIFICAÇÃO: Inicia oculta
      }
    ],
    "trayIcon": {  // ← NOVA SEÇÃO
      "iconPath": "icons/32x32.png",
      "iconAsTemplate": false,
      "menuOnLeftClick": false,
      "tooltip": "EgadSync - Sincronização de arquivos"
    }
  }
}
```

**Explicação Detalhada:**

- **`"visible": false`**: 
  - Janela inicia oculta por padrão
  - Usuário só vê a aplicação via system tray
  - Comportamento similar ao Discord/Steam

- **`trayIcon` seção**:
  - `iconPath`: Caminho para o ícone do tray (32x32px recomendado)
  - `iconAsTemplate`: `false` = ícone colorido; `true` = ícone monocromático
  - `menuOnLeftClick`: `false` = menu só no clique direito
  - `tooltip`: Texto exibido ao passar mouse sobre o ícone

---

### 4. **`src-tauri/capabilities/default.json`** - Permissões de Segurança

```json
{
  "permissions": [
    "core:default",
    "core:window:allow-show",        // ← NOVA: Mostrar janela
    "core:window:allow-hide",        // ← NOVA: Ocultar janela
    "core:window:allow-close",       // ← NOVA: Fechar janela
    "core:window:allow-set-skip-taskbar",  // ← NOVA: Controle taskbar
    "core:tray:default",             // ← NOVA: Funcionalidades do tray
    "opener:default",
    "dialog:default",
    "autostart:default"              // ← NOVA: Funcionalidades autostart
  ]
}
```

**Explicação do Sistema de Permissões:**
- Tauri v2 implementa um sistema rigoroso de permissões
- Cada funcionalidade deve ser explicitamente autorizada
- `core:tray:default`: Permite criar e gerenciar system tray
- `autostart:default`: Permite controlar inicialização automática
- Permissões de janela: Necessárias para show/hide/close programático

---

### 5. **`src-tauri/src/lib.rs`** - Implementação Principal (Rust)

#### 5.1 Imports e Estruturas

```rust
use tauri::{AppHandle, Emitter, Manager, WindowEvent, Window};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItem, MenuEvent};
```

**Explicação dos Imports:**
- `Manager`: Trait para acessar janelas e recursos da aplicação
- `Window`: Tipo para manipulação de janelas
- `TrayIconBuilder`: Construtor para ícones de system tray
- `TrayIconEvent`: Eventos do system tray (cliques, hover, etc.)
- `MenuBuilder/MenuItem`: Construção de menus contextuais

#### 5.2 Comandos Tauri para Controle de Janela

```rust
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
```

**Explicação Linha por Linha:**

1. `#[tauri::command]`: Macro que expõe função Rust para JavaScript
2. `async fn`: Função assíncrona (necessária para operações de UI)
3. `app: AppHandle`: Handle da aplicação para acessar recursos
4. `app.get_webview_window("main")`: Obtém referência da janela principal
5. `window.show()`: Torna a janela visível
6. `window.set_focus()`: Move foco para a janela (traz para frente)
7. `map_err(|e| e.to_string())`: Converte erro para String (JSON-serializável)

#### 5.3 Comandos para Auto-start

```rust
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
```

**Explicação Detalhada:**

1. `use tauri_plugin_autostart::ManagerExt`: Import do trait de extensão
2. `app.autolaunch()`: Obtém gerenciador de auto-start via trait
3. `manager.is_enabled()`: Verifica se auto-start está ativo
4. **Lógica de Toggle**:
   - Se habilitado → desabilita e retorna `false`
   - Se desabilitado → habilita e retorna `true`
5. `Result<bool, String>`: Retorna novo status ou erro

#### 5.4 Criação do Menu do System Tray

```rust
fn create_tray_menu(app: &AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Mostrar", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Ocultar", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    
    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &hide_item, &separator, &quit_item])
        .build()?;
    
    Ok(menu)
}
```

**Explicação dos Parâmetros:**

- `MenuItem::with_id(app, id, label, enabled, accelerator)`
  - `app`: Handle da aplicação
  - `id`: Identificador único ("show", "hide", "quit")
  - `label`: Texto exibido no menu ("Mostrar", "Ocultar", "Sair")
  - `enabled`: `true` = item ativo, `false` = desabilitado
  - `accelerator`: Atalho de teclado (None = sem atalho)

- `PredefinedMenuItem::separator()`: Linha divisória no menu
- `MenuBuilder::new(app).items(&[...]).build()`: Constrói menu final

#### 5.5 Manipulação de Eventos do System Tray

```rust
fn handle_tray_event(app: &AppHandle, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click { button, .. } => {
            if button == tauri::tray::MouseButton::Left {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        }
        _ => {}
    }
}
```

**Lógica de Comportamento:**

1. **Pattern Matching**: `match event` analisa tipo do evento
2. **Clique Esquerdo**: Detecta `TrayIconEvent::Click` com `MouseButton::Left`
3. **Toggle de Visibilidade**:
   - `window.is_visible()`: Verifica se janela está visível
   - Se visível → esconde (`window.hide()`)
   - Se oculta → mostra e foca (`window.show()` + `window.set_focus()`)
4. **Tratamento de Erro**: `let _ =` ignora erros (não-críticos)

#### 5.6 Manipulação de Eventos do Menu

```rust
fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
```

**Comportamento por ID:**

- `"show"`: Mostra janela e move foco
- `"hide"`: Oculta janela (mantém processo)
- `"quit"`: Termina aplicação completamente (`app.exit(0)`)
- `_ => {}`: Ignora IDs desconhecidos

#### 5.7 Interceptação de Fechamento de Janela

```rust
fn handle_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            window.hide().unwrap();
            api.prevent_close();
        }
        _ => {}
    }
}
```

**Funcionalidade Crítica:**

1. `WindowEvent::CloseRequested`: Intercepta tentativa de fechar janela
2. `window.hide()`: Oculta janela ao invés de fechar
3. `api.prevent_close()`: **Impede fechamento real da aplicação**
4. **Resultado**: Botão "X" minimiza para tray ao invés de fechar

#### 5.8 Configuração Principal da Aplicação

```rust
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
            let menu = create_tray_menu(app.handle())?;
            
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| handle_menu_event(app, event))
                .on_tray_icon_event(|tray, event| {
                    handle_tray_event(tray.app_handle(), event);
                })
                .build(app)?;

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
            show_window,        // ← NOVO
            hide_window,        // ← NOVO
            toggle_autostart,   // ← NOVO
            get_autostart_status // ← NOVO
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Explicação da Inicialização:**

1. **Plugin Autostart**:
   ```rust
   .plugin(tauri_plugin_autostart::init(
       tauri_plugin_autostart::MacosLauncher::LaunchAgent,
       Some(vec!["--minimized"]),
   ))
   ```
   - `MacosLauncher::LaunchAgent`: Tipo de launcher no macOS
   - `Some(vec!["--minimized"])`: Argumentos passados na inicialização

2. **Criação do System Tray**:
   ```rust
   let _tray = TrayIconBuilder::new()
       .menu(&menu)
       .on_menu_event(move |app, event| handle_menu_event(app, event))
       .on_tray_icon_event(|tray, event| {
           handle_tray_event(tray.app_handle(), event);
       })
       .build(app)?;
   ```
   - `.menu(&menu)`: Associa menu contextual
   - `.on_menu_event()`: Handler para cliques no menu
   - `.on_tray_icon_event()`: Handler para cliques no ícone
   - `.build(app)`: Constrói e registra o tray

3. **Event Handler Global**:
   ```rust
   .on_window_event(handle_window_event)
   ```
   - Intercepta TODOS os eventos de janela
   - Aplica comportamento de "minimizar para tray"

---

### 6. **`src/App.tsx`** - Interface React

#### 6.1 Novos Estados

```typescript
const [autostartEnabled, setAutostartEnabled] = useState<boolean>(false);
```

**Explicação:**
- Estado para controlar se auto-start está habilitado
- Usado para atualizar interface em tempo real
- Sincronizado com backend via `checkAutostartStatus()`

#### 6.2 Funções de Controle

```typescript
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
```

**Explicação das Funções:**

1. **`checkAutostartStatus()`**:
   - Chama comando Rust `get_autostart_status`
   - Atualiza estado local com status atual
   - Executada na inicialização do componente

2. **`toggleAutostart()`**:
   - Chama comando Rust `toggle_autostart`
   - Recebe novo status diretamente da função
   - Atualiza interface instantaneamente

3. **`hideToTray()`**:
   - Chama comando Rust `hide_window`
   - Oculta janela para system tray
   - Aplicação continua rodando em background

#### 6.3 Interface dos Controles

```tsx
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
    onClick={hideToTray}
    className="control-btn"
    title="Minimizar para bandeja do sistema"
  >
    <EyeOff className="icon" />
    Ocultar
  </button>
</div>
```

**Elementos da Interface:**

1. **Botão Auto-start**:
   - Classe condicional: `active` quando habilitado
   - Texto dinâmico: "ON" ou "OFF"
   - Tooltip explicativo para UX

2. **Botão Ocultar**:
   - Ícone `EyeOff` (lucide-react)
   - Funcionalidade equivalente ao "X" da janela
   - Permite ocultar via interface

---

### 7. **`src/App.css`** - Estilos

#### 7.1 Layout do Cabeçalho

```css
.header-content {
  display: flex;
  justify-content: space-between;  /* Logo à esquerda, controles à direita */
  align-items: center;
}

.header-controls {
  display: flex;
  gap: 8px;  /* Espaçamento entre botões */
}
```

#### 7.2 Estilos dos Botões de Controle

```css
.control-btn {
  background: none;
  border: 1px solid transparent;
  color: var(--gray-400);
  cursor: pointer;
  padding: var(--spacing-md);
  border-radius: var(--radius-lg);
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;           /* Espaço entre ícone e texto */
  font-size: 0.875rem;
  font-weight: 500;
}

.control-btn:hover {
  background: rgba(0, 0, 0, 0.05);
  color: var(--gray-600);
  transform: scale(1.05);  /* Efeito de crescimento no hover */
}

.control-btn.active {
  background: var(--success-50);    /* Verde claro */
  color: var(--success-600);        /* Verde escuro */
  border-color: var(--success-200); /* Borda verde */
}

.control-btn.active:hover {
  background: var(--success-100);
  color: var(--success-700);
}
```

**Sistema de Cores:**
- **Padrão**: Cinza neutro (`--gray-400`)
- **Hover**: Cinza mais escuro (`--gray-600`)
- **Ativo**: Verde (`--success-*`) para indicar estado habilitado
- **Transições**: `0.3s ease` para animações suaves

---

## 🔄 Fluxo de Funcionamento

### 1. **Inicialização da Aplicação**

```
1. Aplicação inicia (janela oculta por padrão)
2. System tray é criado com ícone e menu
3. Plugin autostart é inicializado
4. Status de autostart é verificado
5. Se monitoramento estava ativo, é retomado
```

### 2. **Interação via System Tray**

```
Clique Esquerdo no Ícone:
├── Janela visível? 
│   ├── SIM → Ocultar janela
│   └── NÃO → Mostrar janela + foco

Clique Direito no Ícone:
└── Exibir menu contextual
    ├── "Mostrar" → Mostrar janela + foco
    ├── "Ocultar" → Ocultar janela
    └── "Sair" → Fechar aplicação completamente
```

### 3. **Comportamento do Botão "X"**

```
Usuário clica em "X":
1. WindowEvent::CloseRequested é interceptado
2. window.hide() é chamado (oculta janela)
3. api.prevent_close() impede fechamento real
4. Aplicação continua rodando em background
5. Ícone permanece no system tray
```

### 4. **Sistema de Auto-start**

```
Toggle Auto-start:
1. Verifica status atual (enabled/disabled)
2. Se habilitado → desabilita
3. Se desabilitado → habilita
4. Retorna novo status para interface
5. Interface atualiza botão (ON/OFF + cor)
```

---

## 🌐 Cross-Platform: Como Funciona em Cada SO

### **Windows**
```
Auto-start: Registry key em HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
System Tray: Windows System Tray API
Ícone: .ico file (icons/icon.ico)
```

### **Linux**
```
Auto-start: .desktop file em ~/.config/autostart/
System Tray: XDG System Tray Protocol
Ícone: .png file (icons/32x32.png)
Observação: Alguns DEs podem exigir menu para mostrar ícone
```

### **macOS**
```
Auto-start: LaunchAgent plist em ~/Library/LaunchAgents/
System Tray: NSStatusBar API
Ícone: .icns file (icons/icon.icns)
Launcher: LaunchAgent (configurado no plugin init)
```

---

## 🔍 Decisões Técnicas Importantes

### 1. **Por que Tauri Plugin Autostart?**
- **Oficial**: Mantido pela equipe do Tauri
- **Cross-platform**: Suporte nativo para Win/Mac/Linux
- **Seguro**: Implementação correta para cada SO
- **Futuro**: Compatibilidade garantida com versões futuras

### 2. **Por que Interceptar CloseRequested?**
- **UX Familiar**: Comportamento esperado em apps de sistema
- **Não Invasivo**: Usuário pode sair via menu "Sair"
- **Background**: Mantém funcionalidades rodando (sync de arquivos)

### 3. **Por que Menu + Clique no Ícone?**
- **Flexibilidade**: Duas formas de acesso
- **Padrão**: Comportamento comum em apps similares
- **Acessibilidade**: Menu com labels claros

### 4. **Por que Janela Oculta por Padrão?**
- **Experiência**: App "invisível" até necessário
- **Performance**: Não ocupa espaço na taskbar
- **Profissional**: Comportamento de software de sistema

---

## 🐛 Possíveis Problemas e Soluções

### 1. **Ícone não Aparece no Linux**
```bash
# Solução: Garantir que sistema suporte system tray
sudo apt install libayatana-appindicator3-1
```

### 2. **Auto-start não Funciona**
```rust
// Verificar permissões no capabilities/default.json
"autostart:default"
```

### 3. **Janela não Responde após Ocultar**
```javascript
// Sempre usar show + focus juntos
await invoke("show_window");
```

### 4. **Menu Não Aparece**
```rust
// Verificar se tray foi criado corretamente
let _tray = TrayIconBuilder::new()
    .menu(&menu)  // ← Menu deve ser definido
    .build(app)?;
```

---

## 📈 Possíveis Melhorias Futuras

### 1. **Configurações Avançadas**
- Minimizar para tray ao iniciar
- Configurar atalhos de teclado
- Personalizar comportamento do clique

### 2. **Notificações**
- Mostrar notificações de mudanças via system tray
- Integração com sistema de notificações do OS

### 3. **Múltiplos Ícones**
- Ícones diferentes para estados (monitorando/parado)
- Animações no ícone durante sincronização

### 4. **Menu Dinâmico**
- Status do monitoramento no menu
- Atalhos para funcionalidades principais

---

## 📚 Recursos e Referências

### Documentação Oficial
- [Tauri System Tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri Autostart Plugin](https://v2.tauri.app/plugin/autostart/)
- [Tauri Window Management](https://v2.tauri.app/reference/javascript/api/namespacetray/)

### Códigos de Exemplo
- [Tauri Examples](https://github.com/tauri-apps/tauri/tree/dev/examples)
- [System Tray Example](https://github.com/tauri-apps/tauri/tree/dev/examples/api/src-tauri/src)

---

## ✅ Conclusão

A implementação combina múltiplas tecnologias para criar uma experiência profissional:

- **Rust**: Backend robusto com gerenciamento nativo do sistema
- **React**: Interface moderna e responsiva
- **Tauri Plugins**: Funcionalidades cross-platform confiáveis
- **CSS**: Estilização consistente e intuitiva

O resultado é uma aplicação que se comporta como software profissional, rodando discretamente em background e oferecendo controle total via system tray e interface gráfica.

**Total de linhas modificadas**: ~300
**Arquivos alterados**: 7
**Novas funcionalidades**: 4 principais
**Compatibilidade**: Windows, Linux, macOS

---

*Este documento serve como referência completa para entender, manter e expandir as funcionalidades implementadas.*