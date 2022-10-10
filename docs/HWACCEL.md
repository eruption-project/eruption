# Table of Contents

- [Table of Contents](#table-of-contents)
  - [Support for Hardware Accelerated Effects](#support-for-hardware-accelerated-effects)
  - [Environment Variables](#environment-variables)

## Support for Hardware Accelerated Effects

Eruption supports hardware accelerated effects since version `0.3.0`, using the Pixels library.

## Environment Variables

Pixels will default to selecting the most powerful GPU and most modern graphics API available on the system, and these choices can be overridden with environment variables. These are the same vars supported by the wgpu examples.

* __WGPU_BACKEND__: Select the backend (aka graphics API).
  Supported values: vulkan, metal, dx11, dx12, gl, webgpu
  The default depends on capabilities of the host system, with vulkan being preferred on Linux and Windows, and metal preferred on macOS.

* __WGPU_ADAPTER_NAME__: Select an adapter (aka GPU) with substring matching.
  E.g. 1080 will match NVIDIA GeForce 1080ti

* __WGPU_POWER_PREF__: Select an adapter (aka GPU) that meets the given power profile.
  Supported values: low, high
  The default is low. I.e. an integrated GPU will be preferred over a discrete GPU.
  Note that WGPU_ADAPTER_NAME and WGPU_POWER_PREF are mutually exclusive and that WGPU_ADAPTER_NAME takes precedence.
