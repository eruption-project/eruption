#!/usr/bin/env bash
set -euo pipefail

target_dir="$HOME/.local/share/plasma/plasmoids/org.eruption.widget"
backend_target="$HOME/.local/bin/eruption-plasma-backend"

rm -rf "$target_dir"
rm -f "$backend_target"

echo "KDE Plasma integration removed."
