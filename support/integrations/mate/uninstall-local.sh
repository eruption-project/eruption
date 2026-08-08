#!/usr/bin/env bash
set -euo pipefail

target_py="$HOME/.local/bin/eruption-mate-indicator"
desktop_file="$HOME/.config/autostart/eruption-mate-indicator.desktop"

rm -f "$target_py"
rm -f "$desktop_file"

echo "MATE integration removed."
