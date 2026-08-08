#!/usr/bin/env python3
"""Eruption MATE companion indicator.

Provides quick slot switching and brightness control using eruptionctl.
"""

import subprocess
import shutil
from typing import Optional

import gi

gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, GLib


def run_cmd(args):
    try:
        return subprocess.check_output(args, text=True).strip()
    except Exception:
        return "unavailable"


def run_cmd_silent(args):
    try:
        subprocess.check_call(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return True
    except Exception:
        return False


class EruptionIndicator:
    def __init__(self):
        self.last_error: Optional[str] = None

        self.status_icon = Gtk.StatusIcon.new_from_icon_name("applications-system")
        self.status_icon.set_visible(True)
        self.status_icon.set_tooltip_text("Eruption")
        self.status_icon.connect("popup-menu", self.on_popup_menu)

        self.menu = Gtk.Menu()
        self.item_status = Gtk.MenuItem(label="Status: loading")
        self.item_status.set_sensitive(False)
        self.menu.append(self.item_status)

        self.menu.append(Gtk.SeparatorMenuItem())

        for slot in [1, 2, 3, 4]:
            item = Gtk.MenuItem(label=f"Switch to slot {slot}")
            item.connect("activate", self.on_set_slot, slot)
            self.menu.append(item)

        self.menu.append(Gtk.SeparatorMenuItem())

        for brightness in [20, 40, 60, 80, 100]:
            item = Gtk.MenuItem(label=f"Brightness {brightness}%")
            item.connect("activate", self.on_set_brightness, brightness)
            self.menu.append(item)

        self.menu.append(Gtk.SeparatorMenuItem())

        item_refresh = Gtk.MenuItem(label="Refresh")
        item_refresh.connect("activate", self.on_refresh)
        self.menu.append(item_refresh)

        item_open = Gtk.MenuItem(label="Open UI")
        item_open.connect("activate", self.on_open_ui)
        self.menu.append(item_open)

        item_quit = Gtk.MenuItem(label="Quit")
        item_quit.connect("activate", self.on_quit)
        self.menu.append(item_quit)

        self.menu.show_all()
        self.refresh_status()
        GLib.timeout_add_seconds(8, self.refresh_status)

    def is_eruptionctl_available(self) -> bool:
        return shutil.which("eruptionctl") is not None

    def parse_prefixed_value(self, raw: str, prefix: str) -> str:
        if not raw or raw == "unavailable":
            return ""

        if raw.startswith(prefix):
            return raw.replace(prefix, "", 1).strip()

        return ""

    def get_state(self):
        if not self.is_eruptionctl_available():
            self.last_error = "eruptionctl not found"
            return {
                "connected": False,
                "profile": "unavailable",
                "slot": "?",
                "brightness": "?",
            }

        profile_raw = run_cmd(["eruptionctl", "status", "profile"])
        slot_raw = run_cmd(["eruptionctl", "status", "slot"])
        brightness_raw = run_cmd(["eruptionctl", "config", "brightness"])

        profile = self.parse_prefixed_value(profile_raw, "Current profile:")
        slot = self.parse_prefixed_value(slot_raw, "Current slot:")
        brightness = self.parse_prefixed_value(brightness_raw, "Global brightness:")

        connected = bool(profile and slot and brightness)
        if connected:
            self.last_error = None
        else:
            self.last_error = "daemon unavailable or no response"

        return {
            "connected": connected,
            "profile": profile or "unavailable",
            "slot": slot or "?",
            "brightness": brightness or "?",
        }

    def run_action(self, args) -> bool:
        ok = run_cmd_silent(args)
        if not ok:
            self.last_error = "action failed"
        return ok

    def on_popup_menu(self, _icon, button, activate_time):
        self.menu.popup(None, None, None, None, button, activate_time)

    def on_set_slot(self, _item, slot):
        self.run_action(["eruptionctl", "switch", "slot", str(slot)])
        self.refresh_status()

    def on_set_brightness(self, _item, brightness):
        self.run_action(["eruptionctl", "config", "brightness", str(brightness)])
        self.refresh_status()

    def on_refresh(self, _item):
        self.refresh_status()

    def on_open_ui(self, _item):
        if run_cmd_silent(["sh", "-c", "command -v pyroclasm >/dev/null"]):
            subprocess.Popen(["pyroclasm"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            return
        if run_cmd_silent(["sh", "-c", "command -v eruption-gui-gtk3 >/dev/null"]):
            subprocess.Popen(["eruption-gui-gtk3"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            return

        self.last_error = "no UI executable found"
        self.refresh_status()

    def on_quit(self, _item):
        Gtk.main_quit()

    def refresh_status(self):
        state = self.get_state()

        label = f"Status: slot={state['slot']} brightness={state['brightness']}"
        if not state["connected"] and self.last_error:
            label += f" ({self.last_error})"

        self.item_status.set_label(label)

        tooltip = f"Eruption | profile={state['profile']} | slot={state['slot']}"
        if self.last_error:
            tooltip += f" | {self.last_error}"
        self.status_icon.set_tooltip_text(tooltip)

        return True


if __name__ == "__main__":
    EruptionIndicator()
    Gtk.main()
