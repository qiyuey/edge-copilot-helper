#!/usr/bin/env bash
set -e

# Get absolute path to the current directory
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWIFT_SOURCE="$DIR/EdgeExitWatcher.swift"
EXECUTABLE="$DIR/EdgeExitWatcher"
FIX_SCRIPT="$DIR/fix-edge-copilot.sh"
PLIST_NAME="com.yuchuan.edgecopilotfixer.plist"
PLIST_DEST="$HOME/Library/LaunchAgents/$PLIST_NAME"

echo "🔧 Setting up Edge Copilot Fixer Watcher..."

# 1. Check prerequisites
if ! command -v swiftc >/dev/null 2>&1; then
    echo "❌ Error: 'swiftc' is not found. Please install Xcode Command Line Tools."
    echo "   Run: xcode-select --install"
    exit 1
fi

if [ ! -f "$FIX_SCRIPT" ]; then
    echo "❌ Error: Fix script not found at $FIX_SCRIPT"
    exit 1
fi

chmod +x "$FIX_SCRIPT"

# 2. Compile the Swift watcher
echo "🔨 Compiling EdgeExitWatcher..."
swiftc "$SWIFT_SOURCE" -o "$EXECUTABLE"
if [ $? -eq 0 ]; then
    echo "✅ Compilation successful: $EXECUTABLE"
else
    echo "❌ Compilation failed."
    exit 1
fi

# 3. Create Launch Agent plist
echo "📝 Creating Launch Agent plist..."
cat > "$DIR/$PLIST_NAME" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.yuchuan.edgecopilotfixer</string>
    <key>ProgramArguments</key>
    <array>
        <string>$EXECUTABLE</string>
        <string>$FIX_SCRIPT</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/edgecopilotfixer.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/edgecopilotfixer.err</string>
</dict>
</plist>
EOF

# 4. Install and Load
echo "📦 Installing to $PLIST_DEST..."

# Unload existing if present
if launchctl list | grep -q "com.yuchuan.edgecopilotfixer"; then
    launchctl bootout gui/$(id -u) "$PLIST_DEST" 2>/dev/null || true
fi

mv "$DIR/$PLIST_NAME" "$PLIST_DEST"

# Load the new plist
# launchctl load is deprecated in favor of bootstrap/bootout but still widely used. 
# For user agents, 'launchctl bootstrap gui/UID path' is the modern way, but 'launchctl load' is simpler for scripts.
# We'll use the modern 'bootstrap' if possible, or fallback.
# Actually, let's use the robust 'load -w'.
launchctl load -w "$PLIST_DEST"

echo "✅ Service installed and loaded!"
echo "   Logs can be found at: /tmp/edgecopilotfixer.log"
echo "   To monitor: tail -f /tmp/edgecopilotfixer.log"

