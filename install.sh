#!/usr/bin/env bash
set -e

# Configuration
APP_LABEL="top.qiyuey.edge-copilot-helper"
INSTALL_DIR="$HOME/Library/Application Support/$APP_LABEL"
LOG_DIR="$HOME/Library/Logs/$APP_LABEL"
PLIST_NAME="$APP_LABEL.plist"
PLIST_DEST="$HOME/Library/LaunchAgents/$PLIST_NAME"
BINARY_NAME="edge-copilot-helper"

echo "🔧 Setting up Edge Copilot Helper (Rust)..."

# 1. Check prerequisites
if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Error: 'cargo' is not found. Please install Rust (https://rustup.rs/)."
    exit 1
fi

# 2. Build
echo "🔨 Building release binary..."
cargo build --release

# 3. Prepare directories
echo "📂 Preparing directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$LOG_DIR"

# 4. Install Binary
echo "📦 Installing binary..."
cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# 5. Create Launch Agent plist
echo "📝 Creating Launch Agent plist..."
cat > "$PLIST_NAME" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$APP_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/$BINARY_NAME</string>
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

# 6. Install and Load
echo "🚀 Installing Launch Agent..."

# Unload existing if present
if launchctl list | grep -q "$APP_LABEL"; then
    launchctl bootout gui/$(id -u) "$PLIST_DEST" 2>/dev/null || true
fi

mv "$PLIST_NAME" "$PLIST_DEST"

# Load the new plist
launchctl load -w "$PLIST_DEST"

echo "✅ Service installed and loaded!"
echo "   Logs directory: $LOG_DIR"
echo "   Monitor with: tail -f $LOG_DIR/service.log"

