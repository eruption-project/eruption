#!/usr/bin/env bash
set -euo pipefail

# Connect/query flow for shell-based automation around Eruption.

if ! command -v eruptionctl >/dev/null 2>&1; then
  echo "eruptionctl not found in PATH" >&2
  exit 127
fi

echo "Querying current state from Eruption..."
eruptionctl status profile
eruptionctl status slot
eruptionctl config brightness
