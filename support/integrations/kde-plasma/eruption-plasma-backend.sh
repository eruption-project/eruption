#!/usr/bin/env bash
set -euo pipefail

cmd="${1:-status}"

require_eruptionctl() {
  if ! command -v eruptionctl >/dev/null 2>&1; then
    echo "ERROR: eruptionctl not found"
    exit 127
  fi
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

require_eruptionctl

case "$cmd" in
  status)
    profile="$(get_profile || true)"
    slot="$(get_slot || true)"
    brightness="$(get_brightness || true)"

    profile="${profile:-unavailable}"
    slot="${slot:-0}"
    brightness="${brightness:-0}"

    printf '{"profile":"%s","slot":%s,"brightness":%s}\n' "$(printf '%s' "$profile" | json_escape)" "$slot" "$brightness"
    ;;

  set-slot)
    slot="${2:-}"
    if [[ -z "$slot" ]]; then
      echo "ERROR: missing slot"
      exit 2
    fi
    eruptionctl switch slot "$slot" >/dev/null
    ;;

  set-brightness)
    brightness="${2:-}"
    if [[ -z "$brightness" ]]; then
      echo "ERROR: missing brightness"
      exit 2
    fi
    eruptionctl config brightness "$brightness" >/dev/null
    ;;

  open-ui)
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
