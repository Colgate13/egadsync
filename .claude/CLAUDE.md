# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

EgadSync is a Tauri-based desktop application for real-time file monitoring and synchronization. The architecture combines a React TypeScript frontend with a Rust backend, communicating through Tauri's IPC system.

## Development Commands

### Core Development
```bash
npm run tauri:dev          # Development with hot reload (frontend + backend)
npm run dev                # Frontend only development
npm run build              # Frontend build (TypeScript + Vite)
```

### Production Builds
```bash
npm run tauri:build        # Production build for all platforms
npm run tauri:build:msi    # Windows MSI specific build
```

### Prerequisites for Windows MSI builds
```bash
rustup target add x86_64-pc-windows-msvc
# Additional Windows tools needed: mingw-w64, llvm
```

### GitHub Actions
Automated builds configured in `.github/workflows/build.yml`:
- Multi-platform builds (Windows, macOS, Linux)
- Automatic MSI generation on Windows runners
- Release artifact creation

## Architecture

### Backend (Rust) - `/src-tauri/src/`
- **`lib.rs`**: Entry point exposing Tauri commands (`setup`, `get_save_state`, `stop_monitoring`, etc.)
- **`file_tracker.rs`**: Core file monitoring using `walkdir` for directory scanning and change detection
- **`sync.rs`**: Background monitoring loop (60s intervals) with event emission to frontend
- **`config.rs`**: Centralized configuration (sync intervals, state persistence)
- **`error.rs`**: Custom error types for frontend communication
- **`logger.rs`**: Structured logging with `env_logger`

### Frontend (React) - `/src/`
- **`App.tsx`**: Main UI with real-time updates via Tauri event listeners
- **`App.css`**: Modern UI with glassmorphism effects and animations

### IPC Communication
**Commands** (Frontend → Backend):
- `setup(targetFolder)` - Initialize monitoring
- `get_monitoring_status()` - Check monitoring state
- `select_folder()` - Native folder picker

**Events** (Backend → Frontend):
- `sync_started/sync_stopped` - Monitoring status changes
- `file_diffs` - File changes with FileDiffPayload
- `sync_error` - Error notifications

## Key Technical Details

### File Monitoring
- Uses periodic directory scanning (not filesystem watchers)
- Tracks file metadata (modification time, size) for change detection
- State persisted to `./state.json`
- Changes filtered to files only (directories excluded from notifications)
- Memory limited to 100 recent changes

### Build Configuration
- **Tauri 2.x** with tray-icon feature
- **React 18+** with TypeScript and Vite 6
- **Development server**: Port 1420 (HMR on 1421)
- **Window**: 800x600 default, CSP disabled for development
- **MSI generation** configured with WiX toolset settings

### State Management
- Backend maintains monitoring state in `FileTracker` struct
- Frontend uses React state with Tauri event listeners
- Automatic cleanup on monitoring stop
- Persistent configuration through JSON serialization

## Development Notes

- The sync engine runs asynchronously with Tokio
- Error handling uses custom `SyncError` type for consistent frontend communication
- UI updates are event-driven through Tauri's emit system
- Cross-platform builds supported through Rust standard library