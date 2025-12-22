#!/usr/bin/env bash
set -e

# Configuration
APP_LABEL="top.qiyuey.edge-copilot-helper"
PLIST_NAME="$APP_LABEL.plist"
PLIST_DEST="$HOME/Library/LaunchAgents/$PLIST_NAME"
INSTALL_DIR="$HOME/Library/Application Support/$APP_LABEL"
LOG_DIR="$HOME/Library/Logs/$APP_LABEL"

echo "🗑️  Uninstalling Edge Copilot Fixer Watcher..."

# 1. Unload and remove Launch Agent
if launchctl list | grep -q "$APP_LABEL"; then
    echo "   Stopping service..."
    launchctl bootout gui/$(id -u) "$PLIST_DEST" 2>/dev/null || true
fi

if [ -f "$PLIST_DEST" ]; then
    echo "   Removing plist: $PLIST_DEST"
    rm "$PLIST_DEST"
fi

# 2. Remove Application Support files
if [ -d "$INSTALL_DIR" ]; then
    echo "   Removing files: $INSTALL_DIR"
    rm -rf "$INSTALL_DIR"
fi

# 3. Remove Logs
if [ -d "$LOG_DIR" ]; then
    echo "   Removing logs: $LOG_DIR"
    rm -rf "$LOG_DIR"
fi

echo "✅ Uninstallation complete."

