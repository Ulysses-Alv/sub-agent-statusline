#!/usr/bin/env pwsh
# Installation script for OpenCode Subagent Monitor CLI

param(
    [switch]$Global,
    [switch]$Uninstall
)

$exePath = Join-Path $PSScriptRoot "src-tauri\target\release\companion.exe"
$targetName = "opencode-companion.exe"

if ($Uninstall) {
    $installDir = if ($Global) { "$env:LOCALAPPDATA\Microsoft\WindowsApps" } else { "$env:USERPROFILE\.local\bin" }
    $targetPath = Join-Path $installDir $targetName
    
    if (Test-Path $targetPath) {
        Remove-Item $targetPath -Force
        Write-Host "Uninstalled $targetName from $installDir" -ForegroundColor Green
    } else {
        Write-Host "$targetName not found in $installDir" -ForegroundColor Yellow
    }
    exit 0
}

if (-not (Test-Path $exePath)) {
    Write-Host "Error: companion.exe not found at $exePath" -ForegroundColor Red
    Write-Host "Please build the app first: cd companion && npm run tauri build" -ForegroundColor Yellow
    exit 1
}

# Determine install location
if ($Global) {
    $installDir = "$env:LOCALAPPDATA\Microsoft\WindowsApps"
    # Requires admin privileges for global install
} else {
    $installDir = "$env:USERPROFILE\.local\bin"
    
    # Create .local/bin if it doesn't exist
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        # Add to PATH if not already there
        $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($userPath -notlike "*\.local\bin*") {
            [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
            Write-Host "Added $installDir to your PATH. Restart your terminal to use the command." -ForegroundColor Yellow
        }
    }
}

$targetPath = Join-Path $installDir $targetName

try {
    Copy-Item $exePath $targetPath -Force
    Write-Host "Installed opencode-companion to $targetPath" -ForegroundColor Green
    Write-Host "Run 'opencode-companion open' to start" -ForegroundColor Cyan
} catch {
    Write-Host "Error installing: $_" -ForegroundColor Red
    exit 1
}
