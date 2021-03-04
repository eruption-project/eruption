# Table of Contents

- [Table of Contents](#table-of-contents)
  - [Known Issues and Remarks](#known-issues-and-remarks)
  - [Feature Matrix](#feature-matrix)
    - [Keyboard Devices](#keyboard-devices)
    - [Mouse Devices](#mouse-devices)
  - [ROCCAT Vulcan 100/12x series keyboard](#roccat-vulcan-10012x-series-keyboard)
    - [Support status](#support-status)
    - [Remarks and known Issues](#remarks-and-known-issues)
  - [ROCCAT Vulcan Pro TKL series keyboard](#roccat-vulcan-pro-tkl-series-keyboard)
    - [Support status](#support-status-1)
    - [Remarks and known Issues](#remarks-and-known-issues-1)
  - [ROCCAT Vulcan TKL series keyboard](#roccat-vulcan-tkl-series-keyboard)
    - [Support status](#support-status-2)
    - [Remarks and known Issues](#remarks-and-known-issues-2)
  - [ROCCAT Vulcan Pro series keyboard](#roccat-vulcan-pro-series-keyboard)
    - [Support status](#support-status-3)
    - [Remarks and known Issues](#remarks-and-known-issues-3)
  - [Other Devices](#other-devices)

## Known Issues and Remarks

- You may want to set a different profile for each slot (`F1`-`F4`).
- Some keyboards may get into an inconsistent state when Eruption terminates while `Game Mode` is enabled. The state may be fixed manually or by a reboot/device hotplug

## Feature Matrix

### Keyboard Devices

| Vendor | Product        | Status                     | Macro Keys | Easy Shift Key | Switch Profiles via F1-F4 Keys | Special functions via F5-F8 Keys    | Media keys F9-F12 |
| ------ | -------------- | -------------------------- | ---------- | -------------- | ------------------------------ | ----------------------------------- | ----------------- |
| ROCCAT | Vulcan 100/12x | 100%                       | Yes        | Yes            | Yes                            | Yes                                 | Yes               |
| ROCCAT | Vulcan Pro TKL | 98%                        | No         | Yes            | Yes (*inofficial)              | No, but may be forced (*inofficial) | Yes               |
| ROCCAT | Vulcan TKL     | reportedly working         | No         | Yes            | Yes (*inofficial)              | No, but may be forced (*inofficial) | Yes               |
| ROCCAT | Vulcan Pro     | work-in-progress. untested | Yes        | Yes            | Yes                            | Yes                                 | Yes               |

\* This feature is not supported/endorsed by the OEM and may be subject to change.

### Mouse Devices

| Vendor | Product              | Status | Macro Keys | Easy Shift Key |
| ------ | -------------------- | ------ | ---------- | -------------- |
| ROCCAT | Kone Pure Ultra      | 100%   | N.a.       | N.a.           |
| ROCCAT | Kone Aimo            | 80%    | N.a.       | N.a.           |
| ROCCAT | Kone Aimo Remastered | 80%    | N.a.       | N.a.           |
| ROCCAT | Kova AIMO            | 80%    | N.a.       | N.a.           |

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

## ROCCAT Vulcan Pro TKL series keyboard

### Support status

Nearly fully supported, as of `0.1.19`

### Remarks and known Issues

- The default `MODIFIER` key is the **`FN`** key. Use it to switch slots (with `F1-F4`).
- Slots may currently only be switched via `FN` + `F1-F4`, switching via `FN + LEFT` or `FN + RIGHT` causes problems
- Neighbor topology tables are currently not fully correct, this may lead to mis-rendering of some effects
- You can use the `FN` key to access special function keys (`F5`-`F8`) (*inofficial) like on the ROCCAT Vulcan Pro / ROCCAT Vulcan 100/12x
- Use the `FN` key too to access media functions (`F9`-`F12`)
- Easy Shift+ may be activated by pressing `FN`+`Page Down/GameMode` and then `CAPS LOCK`.

## ROCCAT Vulcan TKL series keyboard

### Support status

Reported as working but not verified, as of `0.1.19`.

### Remarks and known Issues

- The default `MODIFIER` key is the **`FN`** key. Use it to switch slots (with `F1-F4`).
- Slots may currently only be switched via `FN` + `F1-F4`, switching via `FN + LEFT` or `FN + RIGHT` causes problems
- Neighbor topology tables are currently not fully correct, this may lead to mis-rendering of some effects
- You can use the `FN` key to access special function keys (`F5`-`F8`) (*inofficial) like on the ROCCAT Vulcan Pro / ROCCAT Vulcan 100/12x
- Use the `FN` key too to access media functions (`F9`-`F12`)
- Easy Shift+ may be activated by pressing `FN`+`Page Down/GameMode` and then `CAPS LOCK`.

## ROCCAT Vulcan Pro series keyboard

### Support status

Work-in-progress, completely untested. Probably no working, as of `0.1.20`

### Remarks and known Issues

- Mute button will stay lit even if audio is muted
- The default `MODIFIER` key is the **`FN`** key. Use it to switch slots (with `F1-F4`) or execute macros (`M1-M6`).
- Use the `FN` key to access special function keys (`F5`-`F8`)
- Use the `FN` key to access media functions (`F9`-`F12`)
- Easy Shift+ may be activated by pressing `FN`+`Scroll Lock/GameMode` and then `CAPS LOCK`.


## Other Devices

Support for more devices is being worked on! Please open up a feature request on GitHub, if you are willing to assist in getting your hardware supported.
