#!/usr/bin/env bash
set -e

# Configuration
APP_LABEL="top.qiyuey.edge-copilot-helper"
INSTALL_DIR="$HOME/Library/Application Support/$APP_LABEL"
LOG_DIR="$HOME/Library/Logs/$APP_LABEL"
PLIST_NAME="$APP_LABEL.plist"
PLIST_DEST="$HOME/Library/LaunchAgents/$PLIST_NAME"

# Source files
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWIFT_SOURCE="$DIR/EdgeExitWatcher.swift"
FIX_SCRIPT_SOURCE="$DIR/fix-edge-copilot.sh"

# Destination files
EXECUTABLE_DEST="$INSTALL_DIR/EdgeExitWatcher"
FIX_SCRIPT_DEST="$INSTALL_DIR/fix-edge-copilot.sh"

echo "🔧 Setting up Edge Copilot Fixer Watcher..."

# 1. Check prerequisites
if ! command -v swiftc >/dev/null 2>&1; then
    echo "❌ Error: 'swiftc' is not found. Please install Xcode Command Line Tools."
    echo "   Run: xcode-select --install"
    exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "❌ Error: 'jq' is not found. Please install it (e.g., brew install jq)."
    exit 1
fi

if [ ! -f "$FIX_SCRIPT_SOURCE" ]; then
    echo "❌ Error: Fix script not found at $FIX_SCRIPT_SOURCE"
    exit 1
fi

# 2. Prepare directories
echo "📂 Preparing directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$LOG_DIR"

# 3. Compile and Install
echo "🔨 Compiling and installing..."
swiftc "$SWIFT_SOURCE" -o "$EXECUTABLE_DEST"
cp "$FIX_SCRIPT_SOURCE" "$FIX_SCRIPT_DEST"
chmod +x "$FIX_SCRIPT_DEST"

echo "✅ Installed binaries to: $INSTALL_DIR"

# 4. Create Launch Agent plist
echo "📝 Creating Launch Agent plist..."
cat > "$DIR/$PLIST_NAME" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$APP_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$EXECUTABLE_DEST</string>
        <string>$FIX_SCRIPT_DEST</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/service.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/service.err</string>
</dict>
</plist>
EOF

# 5. Install and Load
echo "📦 Installing Launch Agent to $PLIST_DEST..."

# Unload existing if present
if launchctl list | grep -q "$APP_LABEL"; then
    launchctl bootout gui/$(id -u) "$PLIST_DEST" 2>/dev/null || true
fi

mv "$DIR/$PLIST_NAME" "$PLIST_DEST"

# Load the new plist
launchctl load -w "$PLIST_DEST"

echo "✅ Service installed and loaded!"
echo "   Logs directory: $LOG_DIR"
echo "   Monitor with: tail -f $LOG_DIR/service.log"
