#!/usr/bin/env bash
set -euo pipefail

# Update flow example: switch slot and update global brightness.

if ! command -v eruptionctl >/dev/null 2>&1; then
  echo "eruptionctl not found in PATH" >&2
  exit 127
fi

slot="${1:-1}"
brightness="${2:-60}"

echo "Switching to slot ${slot}"
eruptionctl switch slot "$slot"

echo "Setting global brightness to ${brightness}%"
eruptionctl config brightness "$brightness"

echo "Updated state:"
eruptionctl status slot
eruptionctl config brightness
