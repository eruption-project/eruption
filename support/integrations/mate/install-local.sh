#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../../.." && pwd)"
source_py="$repo_root/support/integrations/mate/eruption-mate-indicator.py"
target_py="$HOME/.local/bin/eruption-mate-indicator"
desktop_file="$HOME/.config/autostart/eruption-mate-indicator.desktop"

mkdir -p "$(dirname "$target_py")"
mkdir -p "$(dirname "$desktop_file")"

cp "$source_py" "$target_py"
chmod +x "$target_py"

cat > "$desktop_file" <<EOF
[Desktop Entry]
Type=Application
Name=Eruption MATE Indicator
Comment=Quick controls for Eruption
Exec=$target_py
Terminal=false
X-GNOME-Autostart-enabled=true
EOF

cat <<'EOF'
Installed local MATE indicator helper.

Next steps:
1. Re-login to start it automatically.
2. Or run manually: ~/.local/bin/eruption-mate-indicator
EOF
