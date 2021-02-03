# Table of Contents

- [Table of Contents](#table-of-contents)
  - [Known Issues](#known-issues)
  - [Feature Matrix](#feature-matrix)
    - [Keyboard Devices](#keyboard-devices)
    - [Mouse Devices](#mouse-devices)
  - [ROCCAT Vulcan 100/12x series keyboard](#roccat-vulcan-10012x-series-keyboard)
    - [Support status](#support-status)
    - [Remarks and known Issues](#remarks-and-known-issues)
  - [ROCCAT Vulcan Pro TKL series keyboard](#roccat-vulcan-pro-tkl-series-keyboard)
    - [Support status](#support-status-1)
    - [Remarks and known Issues](#remarks-and-known-issues-1)
  - [Other Devices](#other-devices)

## Known Issues

- Some keyboards may get into an inconsistent state when Eruption terminates while `Game Mode` is enabled. The state may be fixed manually or by a reboot/device hotplug

## Feature Matrix

### Keyboard Devices

| Vendor | Product        | Status           | Macro Keys | Easy Shift Key | Switch Profiles via F1-F4 Keys | Functions via F5-F8 Keys |
| ------ | -------------- | ---------------- | ---------- | -------------- | ------------------------------ | ------------------------ |
| ROCCAT | Vulcan 100/12x | 100%             | Yes        | Yes            | Yes                            | Yes                      |
| ROCCAT | Vulcan Pro TKL | 98%              | No         | Yes            | Yes (*inofficial)              | Yes (*inofficial)        |
| ROCCAT | Vulcan TKL     | work-in-progress | No         | Yes            | Yes (*inofficial)              | Yes (*inofficial)        |
| ROCCAT | Vulcan Pro     | unknown/pending  | Yes        | Yes            | Yes                            | Yes                      |

\* This feature is not supported/endorsed by the OEM and may be subject to change.

### Mouse Devices

| Vendor | Product         | Status | Macro Keys | Easy Shift Key |
| ------ | --------------- | ------ | ---------- | -------------- |
| ROCCAT | Kone Pure Ultra | 100%   | No         | No             |
| ROCCAT | Kone Aimo       | ??%    | No         | No             |
| ROCCAT | Kova AIMO       | ??%    | No         | No             |

\* This feature is not supported/endorsed by the OEM and may be subject to change.

## ROCCAT Vulcan 100/12x series keyboard

### Support status

Fully supported

### Remarks and known Issues

- Mute button will stay lit even if audio is muted
- The default `MODIFIER` key is the **`FN`** key. Use it to switch slots (with `F1-F4`) or execute macros (`M1-M6`).
- Use the `FN` key to access special function keys (`F5`-`F8`)
- Use the `FN` key to access media functions (`F9`-`F12`)
- Easy Shift+ may be activated by pressing `FN`+`Scroll Lock/GameMode` and then `CAPS LOCK`.
- You may want to set a different profile for each slot (`F1`-`F4`).

## ROCCAT Vulcan Pro TKL series keyboard

### Support status

Nearly fully supported, as of `0.1.19`

### Remarks and known Issues

- GUI support is incomplete
- NetworkFX shows garbled output, support for NetworkFX is still a TODO
- `F/FN` and `FN/Win` keys are not fully supported yet
- Setting of LED brightness via `FN + UP` and `FN + DOWN` is not fully supported yet
- Slots may currently only be switched via `FN` + `F1-F4`, not via `FN + LEFT` or `FN + RIGHT`
- The default `MODIFIER` key is the **`FN`** key. Use it to switch slots (with `F1-F4`).
- Neighbor topology tables are currently not fully correct, may lead to mis-rendering of some effects
- You can use the `FN` key to access special function keys (`F5`-`F8`) (*inofficial) like on the ROCCAT Vulcan Pro / ROCCAT Vulcan 100/12x
- Use the `FN` key too to access media functions (`F9`-`F12`)
- Easy Shift+ may be activated by pressing `FN`+`Page down/GameMode` and then `CAPS LOCK`.
- You may want to set a different profile for each slot (`F1`-`F4`).

## Other Devices

Support for more devices is being worked on! Please open up a feature request on GitHub, if you are willing to assist in getting your hardware supported.
