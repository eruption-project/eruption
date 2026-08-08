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

extract_line_placeholders() {
  local line="$1"
  printf '%s\n' "$line" \
    | grep -oE '\{[[:space:]]*\$[a-zA-Z0-9_]+[[:space:]]*\}' 2>/dev/null \
    | sed -E 's/\{[[:space:]]*\$([a-zA-Z0-9_]+)[[:space:]]*\}/\1/' \
    | sort -u \
    | tr '\n' ',' \
    | sed 's/,$//' \
    || true
}

extract_key_placeholder_map() {
  local f="$1"
  grep -E '^[a-zA-Z0-9_-]+\s*=' "$f" 2>/dev/null | while IFS= read -r line; do
    key="$(printf '%s\n' "$line" | sed -E 's/^([a-zA-Z0-9_-]+)\s*=.*/\1/')"
    ph="$(extract_line_placeholders "$line")"
    printf '%s=%s\n' "$key" "$ph"
  done || true
}

extract_placeholder_set() {
  local map="$1"
  printf '%s\n' "$map" \
    | sed -E 's/^[^=]+=//' \
    | tr ',' '\n' \
    | sed '/^$/d' \
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

  en_map="$(extract_key_placeholder_map "$base_en")"
  de_map="$(extract_key_placeholder_map "$de_file")"

  while IFS= read -r key; do
    [[ -z "$key" ]] && continue

    en_ph="$(printf '%s\n' "$en_map" | grep -E "^${key}=" | sed -E 's/^[^=]+=//' || true)"
    de_ph="$(printf '%s\n' "$de_map" | grep -E "^${key}=" | sed -E 's/^[^=]+=//' || true)"

    if [[ "$en_ph" != "$de_ph" ]]; then
      echo "ERROR: placeholder mismatch for key '$key' between:"
      echo "  $base_en"
      echo "  $de_file"
      echo "  en-US: ${en_ph:-<none>}"
      echo "  de-DE: ${de_ph:-<none>}"
      failures=$((failures + 1))
    fi
  done <<< "$en_keys"

  # Keep coarse file-level placeholder parity as an extra guardrail.
  en_set="$(extract_placeholder_set "$en_map")"
  de_set="$(extract_placeholder_set "$de_map")"
  if [[ "$en_set" != "$de_set" ]]; then
    echo "ERROR: aggregate placeholder set mismatch between:"
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
