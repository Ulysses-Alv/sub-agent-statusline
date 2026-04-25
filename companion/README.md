# OpenCode Subagent Monitor

A companion desktop app for monitoring OpenCode subagents in real-time.

## Features

- Real-time monitoring of subagent sessions
- Auto-refresh every 2 seconds
- Z-index intelligent window (stays on top of OpenCode, below other apps)
- Force clear for stuck agents
- CLI command for terminal usage

## Installation

### Option 1: Download Installer
Download `OpenCode Subagent Monitor_X.X.X_x64-setup.exe` from releases and run.

### Option 2: CLI Installation
After installation, add to PATH:

```powershell
# Copy to a directory in your PATH
Copy-Item "C:\path\to\companion.exe" "$env:USERPROFILE\.local\bin\opencode-companion.exe"
```

Or use the install script:
```powershell
.\install.ps1
```

## Usage

### GUI Mode (double-click)
Simply double-click `companion.exe` to launch. The app will auto-detect OpenCode instances.

### CLI Mode
```bash
opencode-companion open
```
Opens the companion and automatically closes when OpenCode desktop exits.

### Z-Index Behavior
- Companion stays **on top of OpenCode** when OpenCode is focused
- Companion falls **behind other apps** when you switch away
- Works automatically — no configuration needed

### Force Clear
If you see agents stuck in "running" state:
1. Click the ⚡ (force clear) button
2. Confirm the deletion
3. Agent is removed from state.json

## Troubleshooting

### Companion shows no agents
- Ensure OpenCode desktop is running with subagents
- Try refreshing with F5
- Check if state.json exists in temp directory

### Window doesn't appear on top
- Ensure Z-index feature is enabled (built by default)
- Try restarting the companion

## Build from Source

```bash
cd companion
npm install
npm run tauri build
```

Output: `companion/src-tauri/target/release/companion.exe`

## License

MIT
