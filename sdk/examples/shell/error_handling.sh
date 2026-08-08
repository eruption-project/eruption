#!/usr/bin/env bash
set -euo pipefail

# Error-handling flow for shell integration.

run_or_report() {
  local description="$1"
  shift

  if ! "$@"; then
    echo "FAILED: ${description}" >&2
    return 1
  fi

  return 0
}

if ! command -v eruptionctl >/dev/null 2>&1; then
  echo "eruptionctl not found in PATH" >&2
  exit 127
fi

# Example: invalid slot triggers a graceful failure path.
if ! run_or_report "switch invalid slot" eruptionctl switch slot 99; then
  echo "Recovering by switching to slot 1"
  eruptionctl switch slot 1
fi

# Example: continue only if daemon is reachable.
if run_or_report "query profile" eruptionctl status profile; then
  echo "Eruption daemon reachable"
else
  echo "Eruption daemon unavailable, skipping dependent actions" >&2
fi
