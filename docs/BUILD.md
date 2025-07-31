# Build Instructions

## Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
2. **Node.js**: Install Node.js 18+ from [nodejs.org](https://nodejs.org/)

## Cross-Platform Build Setup

### For Windows MSI (from Linux)
Cross-compilation from Linux to Windows requires additional tools:

```bash
# Add Windows target
rustup target add x86_64-pc-windows-msvc

# Install required tools (Ubuntu/Debian)
sudo apt install mingw-w64 llvm

# Alternative: Use Docker or Windows VM for native builds
```

**Note**: Cross-compilation can be complex. For reliable MSI builds, consider:
- Building natively on Windows
- Using GitHub Actions with Windows runner (see `.github/workflows/build.yml`)
- Using Docker with Windows container

### GitHub Actions (Recommended)
The project includes automated builds for Windows MSI via GitHub Actions:
- Builds are triggered on push to main, PRs, and releases
- Supports Windows, macOS, and Linux builds
- Automatically creates release artifacts including MSI files

## Build Commands

### Development
```bash
npm run tauri:dev
```

### Production Build
```bash
npm run tauri:build        # Native platform
npm run tauri:build:linux  # Linux specific
npm run tauri:build:msi    # Windows MSI (requires Windows tools)
```

## Output Location

After building, the MSI file will be located at:
```
src-tauri/target/release/bundle/msi/egadsync_0.1.0_x64_en-US.msi
```

## Notes

- The MSI installer is unsigned by default
- For production, consider code signing the MSI
- The installer includes automatic uninstall functionality
- Default installation path: `%LOCALAPPDATA%\Programs\egadsync`