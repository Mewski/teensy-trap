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
├── main.rs               entry point, init, main loop
├── usb/                  low-level USB controller drivers
│   ├── ehci.rs           host controller (USB2)
│   ├── device.rs         device controller (USB1)
│   ├── qh.rs             host Queue Head descriptors
│   ├── qtd.rs            host Transfer Descriptors
│   ├── periodic.rs       periodic frame list, 8kHz scheduling
│   ├── async_schedule.rs async schedule for control transfers
│   └── transaction.rs    transfer submission and completion
├── host/                 upstream mouse handling
│   ├── enumerate.rs      full device enumeration
│   ├── hid.rs            HID report descriptor parser
│   └── mouse.rs          mouse report interpretation
├── device/               downstream device to target PC
│   ├── descriptors.rs    1:1 descriptor cloning
│   ├── endpoint.rs       proxy endpoint (any type/direction)
│   └── proxy_device.rs   transparent proxy device
├── proxy/                forwarding and injection
│   ├── forward.rs        endpoint traffic routing
│   └── inject.rs         mouse movement injection
└── serial/               control PC communication
    └── command.rs        injection command parser
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
