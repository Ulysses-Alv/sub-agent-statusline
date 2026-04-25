#!/usr/bin/env bash
# Installation script for OpenCode Subagent Monitor CLI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXE_PATH="$SCRIPT_DIR/src-tauri/target/release/companion"
TARGET_NAME="opencode-companion"
INSTALL_DIR="${HOME}/.local/bin"
UNINSTALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --global)
            INSTALL_DIR="/usr/local/bin"
            shift
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

if [ "$UNINSTALL" = true ]; then
    rm -f "$INSTALL_DIR/$TARGET_NAME"
    echo "Uninstalled $TARGET_NAME from $INSTALL_DIR"
    exit 0
fi

if [ ! -f "$EXE_PATH" ]; then
    echo "Error: companion executable not found at $EXE_PATH"
    echo "Please build first: cd companion && npm run tauri build"
    exit 1
fi

mkdir -p "$INSTALL_DIR"
cp "$EXE_PATH" "$INSTALL_DIR/$TARGET_NAME"
chmod +x "$INSTALL_DIR/$TARGET_NAME"

echo "Installed opencode-companion to $INSTALL_DIR/$TARGET_NAME"
echo "Run 'opencode-companion open' to start"
