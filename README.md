# MultiSeat Display Manager

A native Windows desktop app (built with Tauri + Rust + React) to manage display resolution and refresh rates per user for ASTER multi-seat setups.

## Features

- **Detect all active monitors** — lists every display Windows reports as active
- **Change resolution & refresh rate** per monitor with a single click
- **Persist to registry** — settings survive reboots (uses `CDS_UPDATEREGISTRY | CDS_GLOBAL`)
- **Per-user profiles** — save different display configurations for each ASTER seat user
- **Auto-apply on login** — registers itself to HKCU\...\Run so settings are applied when each user logs in
- **`--apply-profile` CLI flag** — runs headlessly, applies the current user's profile, and exits (used by startup registration)

## Prerequisites

1. **Rust** — https://rustup.rs (install the `stable-x86_64-pc-windows-msvc` toolchain)
2. **Node.js** ≥ 18 — https://nodejs.org
3. **Tauri CLI v2** — installed via npm (see below)
4. **Visual C++ Build Tools** — install via Visual Studio Installer (select "Desktop development with C++")
5. **WebView2** — pre-installed on Windows 10 1803+ / Windows 11

## Setup & Build

```bash
# 1. Clone / extract the project
cd aster-display-manager

# 2. Install JS dependencies
npm install

# 3. Development (starts Tauri + the React dev server)
npm run dev

# 4. Production build (builds the React frontend and creates the installer)
npm run build
```

The installer will be at:
```
src-tauri/target/release/bundle/nsis/ASTER Display Manager_1.0.0_x64-setup.exe
```

## How It Works

### Monitor Detection
Uses `EnumDisplayDevicesW` + `EnumDisplaySettingsW` (Windows GDI) to enumerate all active adapters and their supported display modes.

### Applying Settings
Calls `ChangeDisplaySettingsExW` with the selected `DEVMODEW`. When "Persist" is enabled, passes `CDS_UPDATEREGISTRY | CDS_GLOBAL` flags so Windows writes the setting to the registry.

### Per-User Profiles
Profiles are saved to:
```
%APPDATA%\AsterDisplayManager\profiles.json
```

Each profile maps a Windows device name (e.g. `\\.\DISPLAY1`) to a desired mode.

### Auto-Apply on Login
When "Run at startup" is toggled on, a registry entry is added:
```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
  AsterDisplayManager = "C:\...\aster-display-manager.exe" --apply-profile
```

Each ASTER seat user should:
1. Open the app on their session
2. Set their desired resolution/refresh rate and hit **Apply & Save**
3. Enable **Run at startup** in Settings

This way, every user who logs in will automatically get their display configured.

## ASTER-Specific Notes

- Run the app **once per seat user** to configure their profile
- The `--apply-profile` mode is intentionally headless — it applies settings and exits immediately, so it won't interrupt the user's login experience
- If ASTER resets display assignments on reboot, enable "Persist to Registry" so Windows re-applies the correct mode before ASTER initializes

## Project Structure

```
aster-display-manager/
├── index.html              # Vite entry HTML + shared app styles
├── src/
│   ├── App.jsx             # React UI
│   ├── api.js              # Tauri invoke wrapper + browser fallback
│   └── main.jsx            # React entry point
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs         # Entry point + --apply-profile handling
│       ├── lib.rs          # Tauri commands
│       ├── display.rs      # Windows GDI display API
│       └── profiles.rs     # Profile persistence + startup registry
├── vite.config.js
└── package.json
```
