# Guia de Build para Instalador MSI no Windows

Este documento descreve todos os passos necessários para configurar e buildar instaladores MSI para aplicações Tauri no Windows.

## 📋 Pré-requisitos

### Ferramentas Necessárias

1. **Node.js** (versão 16 ou superior)
2. **Rust** - Instalar via [rustup.rs](https://rustup.rs/)
3. **Visual Studio Build Tools** ou **Visual Studio Community**
   - Componentes necessários: MSVC v143, Windows 10/11 SDK
4. **WiX Toolset** - Será baixado automaticamente pelo Tauri

### Verificar Instalação

```bash
# Verificar Node.js
node --version

# Verificar Rust
rustc --version

# Verificar target Windows
rustup target add x86_64-pc-windows-msvc
```

## 🏗️ Estrutura do Projeto

O projeto Tauri possui a seguinte estrutura principal:

```
egadsync/
├── src/                    # Frontend (React/TypeScript)
├── src-tauri/             # Backend Rust
│   ├── src/
│   ├── icons/             # Ícones da aplicação
│   ├── tauri.conf.json    # Configuração principal
│   └── Cargo.toml
├── package.json           # Dependências e scripts
└── BUILD_WINDOWS.md       # Este arquivo
```

## ⚙️ Configuração

### 1. package.json - Scripts de Build

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "tauri": "tauri",
    "tauri:build": "tauri build",
    "tauri:build:windows": "tauri build --target x86_64-pc-windows-msvc",
    "tauri:build:msi": "tauri build --target x86_64-pc-windows-msvc --bundles msi",
    "tauri:build:nsis": "tauri build --target x86_64-pc-windows-msvc --bundles nsis",
    "tauri:build:portable": "tauri build --target x86_64-pc-windows-msvc --bundles app",
    "tauri:dev": "tauri dev",
    "build:installer": "npm run build && npm run tauri:build:nsis"
  }
}
```

### 2. tauri.conf.json - Configuração Windows

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "egadsync",
  "version": "0.1.0",
  "identifier": "com.egadsync",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": null,
      "wix": {
        "language": "pt-BR",
        "template": null,
        "fragmentPaths": [],
        "componentGroupRefs": [],
        "componentRefs": [],
        "featureGroupRefs": [],
        "featureRefs": [],
        "mergeRefs": [],
        "dialogImagePath": null,
        "bannerPath": null
      },
      "nsis": null
    }
  }
}
```

## 🚀 Processo de Build

### 1. Instalar Dependências

```bash
# Instalar dependências do projeto
npm install
```

### 2. Build do Frontend

```bash
# Compilar TypeScript e buildar com Vite
npm run build
```

### 3. Gerar Instalador MSI

```bash
# Buildar instalador MSI
npm run tauri:build:msi
```

### Saída Esperada

```
> egadsync@0.1.0 tauri:build:msi
> tauri build --target x86_64-pc-windows-msvc --bundles msi

     Running beforeBuildCommand `npm run build`
   Compiling egadsync v0.1.0
    Finished `release` profile [optimized] target(s) in 52.61s
       Built application at: ...\egadsync.exe
        Info Patching binary for type msi
        Info Verifying wix package
     Running candle for "main.wxs"
     Running light to produce ...\egadsync_0.1.0_x64_pt-BR.msi
    Finished 1 bundle at:
        C:\Users\...\src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\egadsync_0.1.0_x64_pt-BR.msi
```

## 📁 Arquivos Gerados

Após o build bem-sucedido, você encontrará:

- **Executável**: `src-tauri\target\x86_64-pc-windows-msvc\release\egadsync.exe`
- **Instalador MSI**: `src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\egadsync_0.1.0_x64_pt-BR.msi`

## 🛠️ Opções de Build Disponíveis

| Comando | Descrição | Arquivo Gerado |
|---------|-----------|----------------|
| `npm run tauri:build:msi` | Instalador MSI (Windows Installer) | `.msi` |
| `npm run tauri:build:nsis` | Instalador NSIS (executável) | `.exe` |
| `npm run tauri:build:portable` | Aplicação portável | `.exe` |
| `npm run tauri:build:windows` | Build padrão Windows | `.exe` + bundles |

## ❌ Resolução de Problemas

### Erro: "tsc: command not found"

**Solução**: Usar npx ou instalar TypeScript globalmente
```bash
npx tsc && npx vite build
# ou
npm install -g typescript
```

### Erro: "Cannot find module 'react'"

**Solução**: Instalar dependências
```bash
npm install
```

### Erro: "WiX not found"

**Solução**: O Tauri baixa automaticamente o WiX. Aguarde o download ou verifique a conexão de internet.

### Erro de Configuração NSIS

**Solução**: Simplificar a configuração no `tauri.conf.json`:
```json
"nsis": null
```

### Build Muito Lento

**Soluções**:
- Usar build em modo release: `--release`
- Verificar antivírus interferindo
- Fechar programas desnecessários

## 🔧 Customizações Avançadas

### Assinar Digitalmente (Código Signing)

Para produção, configure certificado digital:

```json
"windows": {
  "certificateThumbprint": "SEU_THUMBPRINT_AQUI",
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com"
}
```

### Personalizar WiX (MSI)

Para customizar o instalador MSI:

```json
"wix": {
  "language": "pt-BR",
  "template": "caminho/para/template.wxs",
  "dialogImagePath": "caminho/para/dialog.bmp",
  "bannerPath": "caminho/para/banner.bmp"
}
```

## 📝 Notas Importantes

1. **Primeiro Build**: Pode demorar devido ao download de dependências
2. **Tamanho**: O instalador MSI inclui o runtime necessário (~150MB)
3. **Compatibilidade**: Funciona no Windows 7+ (x64)
4. **Atualização**: Para atualizações automáticas, considere usar Tauri Updater

## 🎯 Comandos Rápidos

```bash
# Setup inicial
npm install

# Desenvolvimento
npm run tauri:dev

# Build de produção
npm run tauri:build:msi

# Verificar build
ls src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/
```

---

**Última atualização**: $(date)
**Tauri Version**: 2.x
**Target**: Windows x64