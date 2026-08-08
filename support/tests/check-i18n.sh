#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo_root"

failures=0

extract_keys() {
  local f="$1"
  grep -E '^[a-zA-Z0-9_-]+\s*=' "$f" 2>/dev/null \
    | sed -E 's/^([a-zA-Z0-9_-]+)\s*=.*/\1/' \
    | sort -u \
    || true
}

extract_placeholders() {
  local f="$1"
  grep -oE '\{[[:space:]]*\$[a-zA-Z0-9_]+[[:space:]]*\}' "$f" 2>/dev/null \
    || true
}

normalize_placeholders() {
  sed -E 's/\{[[:space:]]*\$([a-zA-Z0-9_]+)[[:space:]]*\}/\1/' \
    | sort -u
}

echo "[i18n] Checking locale file pairs and key parity..."
while IFS= read -r base_en; do
  base_dir="$(dirname "$base_en")"
  module_dir="$(dirname "$base_dir")"
  rel_name="$(basename "$base_en")"
  de_file="$module_dir/de-DE/$rel_name"

  if [[ ! -f "$de_file" ]]; then
    echo "ERROR: missing de-DE pair for $base_en"
    failures=$((failures + 1))
    continue
  fi

  en_keys="$(extract_keys "$base_en")"
  de_keys="$(extract_keys "$de_file")"

  if [[ "$en_keys" != "$de_keys" ]]; then
    echo "ERROR: key mismatch between:"
    echo "  $base_en"
    echo "  $de_file"
    failures=$((failures + 1))
  fi

  en_ph="$(extract_placeholders "$base_en" | normalize_placeholders)"
  de_ph="$(extract_placeholders "$de_file" | normalize_placeholders)"

  if [[ "$en_ph" != "$de_ph" ]]; then
    echo "ERROR: placeholder mismatch between:"
    echo "  $base_en"
    echo "  $de_file"
    failures=$((failures + 1))
  fi

done < <(find . -type f -path '*/i18n/en-US/*.ftl' | sort)

if [[ "$failures" -gt 0 ]]; then
  echo "[i18n] FAILED with $failures issue(s)."
  exit 1
fi

echo "[i18n] OK"
