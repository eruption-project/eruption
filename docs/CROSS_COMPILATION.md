# Cross-compiling Eruption for Microsoft Windows

As of Eruption `0.5.0` there exists preliminary support for the Microsoft Windows operating system. To build Eruption from source, please follow the instructions listed below

## How is cross-compilation performed?

By using the `cross` build-tool, podman and a rust cross-compiler

## Install tooling

```sh
cargo install cross

```
Please refer to 
https://github.com/cross-rs/cross/wiki/Getting-Started for further information

## After Setup of build-time Dependencies

After everything is set-up, do the actual cross-compilation

```sh
make windows-installer
```

Please be patient...

After this is completed you may find the generated `Eruption.exe` installer in the `target/` subdirectory

## On Windows

Install Eruption using the `Eruption.exe` NSIS-based installer.

Bring up all required daemons by running `Eruption.bat`