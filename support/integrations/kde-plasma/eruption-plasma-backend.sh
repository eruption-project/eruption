#!/usr/bin/env bash
set -euo pipefail

cmd="${1:-status}"

require_eruptionctl() {
  if ! command -v eruptionctl >/dev/null 2>&1; then
    return 1
  fi

  return 0
}

get_profile() {
  eruptionctl status profile 2>/dev/null | sed -E 's/^Current profile:[[:space:]]*//'
}

get_slot() {
  eruptionctl status slot 2>/dev/null | sed -E 's/^Current slot:[[:space:]]*//'
}

get_brightness() {
  eruptionctl config brightness 2>/dev/null | sed -E 's/^Global brightness:[[:space:]]*//; s/%//'
}

json_escape() {
  sed 's/\\/\\\\/g; s/"/\\"/g'
}

normalize_int() {
  local value="${1:-}"
  local default_value="${2:-0}"
  local min_value="${3:-0}"
  local max_value="${4:-100}"

  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "$default_value"
    return
  fi

  if (( value < min_value )); then
    echo "$min_value"
  elif (( value > max_value )); then
    echo "$max_value"
  else
    echo "$value"
  fi
}

status_json() {
  local connected="$1"
  local profile="$2"
  local slot="$3"
  local brightness="$4"
  local error_message="$5"

  printf '{"connected":%s,"profile":"%s","slot":%s,"brightness":%s,"error":"%s"}\n' \
    "$connected" \
    "$(printf '%s' "$profile" | json_escape)" \
    "$slot" \
    "$brightness" \
    "$(printf '%s' "$error_message" | json_escape)"
}

case "$cmd" in
  status)
    if ! require_eruptionctl; then
      status_json "false" "unavailable" "0" "0" "eruptionctl not found"
      exit 0
    fi

    profile="$(get_profile || true)"
    slot="$(get_slot || true)"
    brightness="$(get_brightness || true)"

    if [[ -z "$profile" || -z "$slot" || -z "$brightness" ]]; then
      status_json "false" "${profile:-unavailable}" "0" "0" "daemon unavailable or no response"
      exit 0
    fi

    slot="$(normalize_int "$slot" 0 0 4)"
    brightness="$(normalize_int "$brightness" 0 0 100)"

    status_json "true" "$profile" "$slot" "$brightness" ""
    ;;

  set-slot)
    require_eruptionctl || {
      echo "ERROR: eruptionctl not found"
      exit 127
    }

    slot="${2:-}"
    if [[ -z "$slot" || ! "$slot" =~ ^[1-4]$ ]]; then
      echo "ERROR: slot must be in range 1-4"
      exit 2
    fi

    eruptionctl switch slot "$slot" >/dev/null
    ;;

  set-brightness)
    require_eruptionctl || {
      echo "ERROR: eruptionctl not found"
      exit 127
    }

    brightness="${2:-}"
    if [[ -z "$brightness" || ! "$brightness" =~ ^[0-9]+$ ]]; then
      echo "ERROR: brightness must be an integer in range 0-100"
      exit 2
    fi

    brightness="$(normalize_int "$brightness" 0 0 100)"
    eruptionctl config brightness "$brightness" >/dev/null
    ;;

  open-ui)
    require_eruptionctl || {
      echo "ERROR: eruptionctl not found"
      exit 127
    }

    if command -v pyroclasm >/dev/null 2>&1; then
      nohup pyroclasm >/dev/null 2>&1 &
      exit 0
    fi

    if command -v eruption-gui-gtk3 >/dev/null 2>&1; then
      nohup eruption-gui-gtk3 >/dev/null 2>&1 &
      exit 0
    fi

    echo "ERROR: no UI executable found"
    exit 3
    ;;

  *)
    echo "ERROR: unknown command: $cmd"
    exit 2
    ;;
esac
