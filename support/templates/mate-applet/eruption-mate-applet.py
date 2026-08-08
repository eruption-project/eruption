#!/usr/bin/env python3
"""Minimal Eruption MATE applet starter.

This template intentionally keeps logic simple and focuses on structure.
"""

import subprocess

try:
    from gi.repository import Gtk
except Exception:  # pragma: no cover
    Gtk = None


def run_cmd(args):
    try:
        return subprocess.check_output(args, text=True).strip()
    except Exception:
        return "unavailable"


def build_window():
    window = Gtk.Window(title="Eruption Applet (Template)")
    window.set_default_size(260, 120)

    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
    box.set_margin_top(12)
    box.set_margin_bottom(12)
    box.set_margin_start(12)
    box.set_margin_end(12)

    status = Gtk.Label(label="Profile: " + run_cmd(["eruptionctl", "status"]))
    box.append(status)

    refresh = Gtk.Button(label="Refresh")
    refresh.connect(
        "clicked",
        lambda _btn: status.set_text("Profile: " + run_cmd(["eruptionctl", "status"])),
    )
    box.append(refresh)

    window.set_child(box)
    return window


def main():
    if Gtk is None:
        raise RuntimeError("GTK bindings are required")

    app = Gtk.Application(application_id="org.eruption.mateapplet.template")

    def on_activate(application):
        window = build_window()
        window.set_application(application)
        window.present()

    app.connect("activate", on_activate)
    app.run(None)


if __name__ == "__main__":
    main()
