# Table of Contents

- [Table of Contents](#table-of-contents)
  - [Eruption - FAQs](#eruption---faqs)
    - [Q: I have a problem with getting Eruption to build from source. How to correctly build the project?](#q-i-have-a-problem-with-getting-eruption-to-build-from-source-how-to-correctly-build-the-project)
    - [Q: What can I do to help get new hardware supported](#q-what-can-i-do-to-help-get-new-hardware-supported)
    - [Q: My device is listed as `supported`, but Eruption does not bind a driver to it](#q-my-device-is-listed-as-supported-but-eruption-does-not-bind-a-driver-to-it)
    - [Q: Is a Linux kernel patch required?](#q-is-a-linux-kernel-patch-required)

## Eruption - FAQs

Below you find a list of the most frequently asked questions and their respective answers.

### Q: I have a problem with getting Eruption to build from source. How to correctly build the project?

Please ensure that you are using the latest versions of Rust.

MSRV (Minimum supported Rust version) `Rust 1.69`

### Q: What can I do to help get new hardware supported

Please find more information in the [Eruption Wiki](https://github.com/eruption-project/eruption/wiki)

### Q: My device is listed as `supported`, but Eruption does not bind a driver to it

Maybe the driver for your device is still tagged as experimental? Please edit `/etc/eruption/eruption.conf` to allow experimental drivers

### Q: Is a Linux kernel patch required?

No! Eruption works entirely from user-space
