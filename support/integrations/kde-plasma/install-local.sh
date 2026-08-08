#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../../.." && pwd)"
source_dir="$repo_root/support/templates/kde-plasma-widget"
target_dir="$HOME/.local/share/plasma/plasmoids/org.eruption.widget"
backend_target="$HOME/.local/bin/eruption-plasma-backend"

mkdir -p "$target_dir"
mkdir -p "$(dirname "$backend_target")"

rm -rf "$target_dir"
cp -r "$source_dir" "$target_dir"
cp "$repo_root/support/integrations/kde-plasma/eruption-plasma-backend.sh" "$backend_target"
chmod +x "$backend_target"

cat <<'EOF'
Installed local Plasma widget files.

Next steps:
1. Restart plasmashell or re-login.
2. Add widget "Eruption" from Plasma widgets picker.
EOF
