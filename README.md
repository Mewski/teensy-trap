# Teensy Trap

USB mouse interceptor and injector for Teensy 4.1

## Overview

Sits between a mouse and a target PC, forwarding HID reports at up to 8kHz while
accepting movement injection commands from a control PC over serial. Written in bare
metal Rust targeting the NXP iMXRT1062 EHCI controller directly.

```
Control PC ──(USB Serial)──► Teensy 4.1 ──(USB Device)──► Target PC
                                 ▲
                            (USB Host)
                                 │
                               Mouse
```

## Prerequisites

- Nix with flakes enabled, or manually install:
  - Rust toolchain with `thumbv7em-none-eabihf` target
  - `cargo-binutils` (for `cargo objcopy`)
  - `teensy-loader-cli`
  - `gcc-arm-embedded`
- Teensy 4.1 board
- USB mouse for testing

## Getting Started

1. Enter the dev shell:
   ```
   direnv allow
   ```

2. Build:
   ```
   cargo build --release
   ```

3. Convert to hex:
   ```
   cargo objcopy --release -- -O ihex teensytrap.hex
   ```

4. Flash (press the button on the Teensy):
   ```
   teensy-loader-cli -w -v --mcu=imxrt1062 teensytrap.hex
   ```

## Project Structure

```
src/
├── main.rs              entry point, init, main loop
├── usb/                 EHCI host controller driver
│   ├── ehci.rs          controller init, reset, port management
│   ├── qh.rs            Queue Head descriptors
│   ├── qtd.rs           Queue Transfer Descriptors
│   ├── periodic.rs      periodic frame list, microframe scheduling
│   ├── async_schedule.rs async schedule for control transfers
│   └── transaction.rs   submit/poll transfer helpers
├── host/                upstream mouse handling
│   ├── enumerate.rs     USB enumeration state machine
│   ├── hid.rs           HID report descriptor parser
│   └── mouse.rs         mouse report interpretation
├── device/              downstream HID device to target PC
│   ├── descriptors.rs   dynamic HID descriptor construction
│   ├── endpoint.rs      interrupt IN endpoint
│   └── hid_device.rs    USB HID device class
├── proxy/               forwarding and injection logic
│   ├── forward.rs       report forwarding
│   └── inject.rs        injection command merging
└── serial/              control PC communication
    └── command.rs       UART command parser
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
