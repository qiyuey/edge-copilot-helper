#!/bin/bash

# Edge Preferences file path
PREFS_FILE="$HOME/Library/Application Support/Microsoft Edge/Default/Preferences"
BACKUP_FILE="$PREFS_FILE.bak"

# Check existence
if [ ! -f "$PREFS_FILE" ]; then
    echo "Preferences file not found at: $PREFS_FILE"
    exit 1
fi

# Create backup
cp "$PREFS_FILE" "$BACKUP_FILE"
echo "Backup created at: $BACKUP_FILE"

# Modify region to SG using jq
tmp=$(mktemp)
jq '.browser.custom_services.region_search = "SG"' "$PREFS_FILE" > "$tmp" && mv "$tmp" "$PREFS_FILE"

echo "✅ Edge Copilot region fix applied (set to SG)."

