#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo_root"

failures=0

echo "[docs] Checking root redirect stubs..."
redirects=(
  DOCUMENTATION.md
  INSTALL.md
  LIBRARY.md
  MACROS.md
  NETFX.md
  SETUP_AUDIO.md
)

for file in "${redirects[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "ERROR: missing redirect file: $file"
    failures=$((failures + 1))
    continue
  fi

  if ! grep -q "This document has moved to" "$file"; then
    echo "ERROR: redirect marker missing in $file"
    failures=$((failures + 1))
  fi

done

echo "[docs] Checking local markdown links..."
while IFS= read -r md; do
  md_dir="$(dirname "$md")"

  while IFS= read -r target; do
    # Skip web and anchor links
    if [[ "$target" =~ ^https?:// ]] || [[ "$target" =~ ^# ]] || [[ "$target" =~ ^mailto: ]]; then
      continue
    fi

    target_no_fragment="${target%%#*}"
    if [[ -z "$target_no_fragment" ]]; then
      continue
    fi

    resolved="$(realpath -m "$md_dir/$target_no_fragment")"
    if [[ ! -e "$resolved" ]]; then
      echo "ERROR: broken link in $md -> $target"
      failures=$((failures + 1))
    fi
  done < <(grep -oE "\[[^]]+\]\(([^)]+)\)" "$md" | sed -E 's/.*\(([^)]+)\).*/\1/' || true)

done < <(git ls-files '*.md')

if [[ "$failures" -gt 0 ]]; then
  echo "[docs] FAILED with $failures issue(s)."
  exit 1
fi

echo "[docs] OK"
