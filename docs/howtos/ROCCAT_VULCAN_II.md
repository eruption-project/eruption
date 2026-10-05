# Experimental ROCCAT Vulcan II support

This driver targets USB ID `1e7d:2f4e` (ROC-12-110), firmware **1.09**
and the **German ISO layout** (`0x01`). Other firmware/layout combinations
are rejected until verified. It is not the Vulcan II Max or Mini driver.

The driver requires the experimental driver maturity level. Testing does
not require installing Eruption or enabling a system service.

## Lighting lifecycle

Initialization explicitly selects firmware Direct mode, waits for readiness,
then enables direct transfer before sending RGB frames. Enabling transfer
alone was insufficient after changing the built-in lighting effect.

Shutdown disables direct transfer. Hardware testing confirmed an automatic
return to the **default effect**, not guaranteed restoration of the previous
effect or custom profile. Reading the mode feature report failed on the
verified firmware, so the driver cannot currently snapshot the old effect.

Fn + Left/Right selects a built-in effect afterwards if needed. This is
**not a factory reset**. No factory-reset shortcut is claimed here.

## Verified behavior

- Direct colors at the Escape, F1 and F2 addresses and six keyboard rows.
- Integrated daemon solid-color lighting and default-effect fallback.
- Native previous-track, play/pause and next-track press/release events,
  including while direct lighting was active without input grabbing.
- Main-keyboard forwarding through Eruption's virtual keyboard: left/right
  Shift and Ctrl each produced exactly one press and release in a bounded
  test with exclusive input grabbing.
- Left Shift+Ctrl and right Shift+Ctrl overlaps on the virtual keyboard,
  with balanced press/release counts and no held test keys at capture end.
- Right-arrow press/release and repeat events at the virtual keyboard;
  exact repeat rate and physical-to-virtual event equality remain unverified.
- Insert, Home, PageUp, Delete, End, PageDown, Left and Down forwarding,
  each with one balanced press/release pair in the navigation test.
- Guided suite: all 36 steps met the current completion criteria, including
  German ISO < and Y/Z positions, main/numpad Enter, numpad +/- and
  two-/three-modifier overlaps. Modifier repeat events also occur at the
  physical Linux input source without Eruption; their presence is not an
  Eruption-only anomaly. Exact physical/virtual repeat-rate equality remains
  unverified.
- All 26 letter positions and the ten top-row digit keys reached the virtual
  keyboard with balanced press/release counts in the guided blocks. This
  verifies Linux key events, not composed characters or desktop layout rules.
- Remaining function keys, German punctuation/umlaut positions, numpad
  digits, divide/multiply and Pause had balanced press/release counts. The
  numpad comma/Del key was user-confirmed but not captured successfully by
  the probe; its forwarding remains unmeasured.
- Caps, Num and Scroll Lock forwarding: two balanced press/release pairs
  each. The user visually confirmed indicator toggling and return to the
  prior indicator state; this was not an automated LED-state measurement.
- Standalone firmware brightness, effect selection and lighting on/off
  using the documented Fn combinations.

## Limitations

Media controls use a separate native input source from the main keyboard.
The special HID handler does not synthesize volume or media actions, to
avoid duplicate desktop actions. Media/macro/Fn Lua features remain disabled.

Fn + F6 produced ordinary F6 rather than a microphone-mute event in the
tested configuration. The default microphone's mute status did not change.
Do not bind ordinary F6 to microphone mute as a driver workaround.

Windows, menu, Print and host-visible Fn handling remain unverified, as do
general rollover, repeat-rate accuracy, hotplug, arbitrary effect restoration
and other firmware/layout variants. Successful
HID writes alone are not evidence that the firmware displayed the colors.

## Automated checks

The protocol/layout/session integration tests exercise packet framing,
mode selection, firmware gates, key mappings, Rust/Lua consistency,
brightness scaling and failure cleanup without accessing hardware.
The driver's hardware tests are ignored by default and require explicit
opt-in; the lighting lifecycle test changes the keyboard's lighting.

Development probes and generic lighting-only configuration are separate
from the device implementation. Raw diagnostic logs and packet captures
are not part of this contribution.

Protocol references: OpenRGB's RoccatVulcanKeyboardController (Mola19) and
the ROCCAT Vulcan II Quick Start Guide, page 3.
