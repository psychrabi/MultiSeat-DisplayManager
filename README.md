# Display Manager

A native Windows desktop application for managing display settings in multi-user environments. Built with Tauri v2, Rust, and React.

Each user can configure their own display resolution, refresh rate, orientation, scale, and layout independently.

## Features

- **Monitor detection** — enumerates all active displays via Windows GDI
- **Per-monitor settings** — change resolution, refresh rate, orientation, and scale
- **Interactive layout preview** — drag monitors to reposition them with snap alignment
- **Per-user profiles** — save and restore different display configurations for each user
- **Auto-apply on login** — registers to run at startup so settings are applied per user automatically
- **Auto-updater** — checks GitHub Releases for new versions and installs them in-app
- **Light/Dark mode** — automatically follows OS color scheme preference

## Prerequisites

- **Rust** — install via https://rustup.rs (stable toolchain)
- **Bun** — install via `powershell -c "irm bun.sh/install.ps1 | iex"`
- **Visual C++ Build Tools** — from Visual Studio Installer, select "Desktop development with C++"
- **WebView2** — pre-installed on Windows 10 1803+ / Windows 11

## Setup

```bash
# Install JS dependencies
bun install

# Development (starts Tauri + Vite dev server)
bun run dev

# Production build (creates NSIS installer)
bun run build
```

The installer is at `src-tauri/target/release/bundle/nsis/`.

## Usage

1. Launch the app — it detects all connected monitors
2. Click a monitor in the layout preview to select it
3. Adjust resolution, refresh rate, orientation, or scale
4. Click **Apply** to commit changes
5. Drag monitors in the layout preview to reposition them, then click **Apply Changes**

### Per-User Profiles

1. Go to the **Profiles** page
2. Select a user and save their display configuration
3. Enable **Run at startup** in **Settings** so the profile applies automatically at login

## Project Structure

```
DisplayManager/
├── index.html                 # Vite entry HTML
├── src/
│   ├── App.jsx                # Root layout + sidebar + router
│   ├── main.jsx               # React entry point
│   ├── api.js                 # Tauri invoke wrapper
│   ├── js/utils.jsx           # Display utility functions
│   ├── hooks/
│   │   ├── useInitApp.js       # App initialization
│   │   ├── useMonitorActions.js # Monitor CRUD actions
│   │   └── useUpdater.js       # Auto-update check logic
│   ├── stores/
│   │   ├── useAppStore.js      # Global app state (Zustand)
│   │   ├── useDisplayStore.js  # Display/monitor state
│   │   └── useProfileStore.js  # User profile state
│   ├── components/
│   │   ├── Sidebar.jsx
│   │   ├── ToastContainer.jsx
│   │   ├── UpdateBanner.jsx    # Auto-update UI
│   │   ├── monitors/
│   │   │   ├── MonitorSettings.jsx  # Per-monitor settings form
│   │   │   ├── LayoutPreview.jsx    # Drag-to-snap layout
│   │   │   └── ConfirmationDialog.jsx
│   │   └── profiles/
│   │       └── ui.jsx
│   └── pages/
│       ├── Monitors.jsx
│       ├── Profiles.jsx
│       └── Settings.jsx
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json       # Plugin permissions
│   └── src/
│       ├── main.rs             # Entry point + --apply-profile flag
│       ├── lib.rs              # Tauri commands + plugin setup
│       ├── backend.rs          # Win32 display backend trait
│       └── profiles.rs         # Profile persistence + startup registry
├── .github/workflows/
│   └── publish-windows.yml     # CI/CD for Windows releases
└── package.json
```

## How It Works

### Display API
Uses `EnumDisplayDevicesW` + `EnumDisplaySettingsW` (Windows GDI) to enumerate adapters and supported modes. Applies settings via `ChangeDisplaySettingsExW`.

### Profiles
Saved to `%APPDATA%\DisplayManager\profiles.json`. Each profile maps a Windows display device to its desired mode, position, orientation, and scale.

### Auto-Start
Registers `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` with the `--apply-profile` flag, which applies the current user's saved profile headlessly on login.

### Auto-Updater
On launch, the app checks `https://github.com/psychrabi/MultiSeat-DisplayManager/releases/latest/download/latest.json` for a newer version. If found, a banner appears with download progress and an install button.

## Publishing a Release

1. Bump `version` in `src-tauri/tauri.conf.json`
2. Commit with `Release` in the commit message (e.g. `Release v1.0.1`)
3. Push to `main` — the workflow builds the app and creates a draft release
4. Go to **Releases** on GitHub, review, and publish

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri v2](https://v2.tauri.app) |
| Frontend | React 19 + React Router 7 |
| Styling | Tailwind CSS 4 + DaisyUI 5 |
| Form validation | react-hook-form + Zod |
| State management | Zustand |
| Icons | Lucide React |
| Backend | Rust (Win32 GDI API) |
| Bundler | Vite 8 |
| Package manager | Bun |
