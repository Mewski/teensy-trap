# teensytrap

USB HID mouse proxy on Teensy 4.1 (NXP iMXRT1062). Intercepts a mouse via USB host, clones its identity exactly, and re-presents it to a target PC via USB device — while accepting injection commands from a control PC over serial.

## Architecture

```
Control PC ──(USB Serial)──► Teensy 4.1 ──(USB Device @ 480Mbps HS)──► Target PC
                                 ▲
                          (USB Host @ 480Mbps HS)
                                 │
                               Mouse
```

The Teensy uses both independent USB controllers on the iMXRT1062:
- **USB2 (Host)**: Enumerates upstream mouse, polls interrupt IN endpoint at up to 8kHz (125µs microframes)
- **USB1 (Device)**: Presents as an identical HID mouse to the target PC

## Core Design Principles

### Exact 1:1 Descriptor Cloning
The proxy must be indistinguishable from the real mouse. This means:
- Raw device descriptor cloned byte-for-byte (VID, PID, bcdDevice, bcdUSB, all of it)
- Raw configuration descriptor cloned byte-for-byte (interface, endpoint, HID class descriptors)
- Raw HID report descriptor cloned byte-for-byte (report fields, usage pages, collections)
- String descriptors cloned byte-for-byte (manufacturer, product, serial number)
- Interrupt IN endpoint max packet size and bInterval match the upstream mouse exactly

`EnumeratedDevice` stores all raw descriptors. `ClonedDescriptors::from_upstream()` does a direct copy — no reconstruction, no reinterpretation. The target PC sees the same bytes that the real mouse would send during enumeration.

### High-Speed USB (480Mbps)
Both USB ports run at USB 2.0 High Speed. The EHCI periodic schedule is configured to poll every microframe (S-mask 0xFF = 8kHz). The iMXRT1062 has integrated High-Speed PHYs on both ports — no external ULPI chip needed.

### Bare Metal, No RTOS
Direct EHCI register access via `imxrt-ral`. No USB middleware, no HAL USB stack, no scheduler between our code and the hardware. Every 125µs microframe is ours to fill.

## Module Responsibilities

- `usb/` — EHCI host controller driver. QH/qTD data structures, periodic frame list for 8kHz interrupt polling, async schedule for control transfers during enumeration.
- `host/enumerate.rs` — USB enumeration state machine. Issues SET_ADDRESS, GET_DESCRIPTOR (device, config, string, HID report), SET_CONFIGURATION. Stores all raw descriptor bytes.
- `host/hid.rs` — HID report descriptor parser. Walks the descriptor byte stream to determine field layout (button count, axis widths, report IDs) so we can interpret incoming reports for injection merging.
- `host/mouse.rs` — Interprets raw HID report bytes into structured `MouseReport` values using the parsed field layout.
- `device/descriptors.rs` — Takes `EnumeratedDevice` and produces `ClonedDescriptors` — byte-for-byte copies of every upstream descriptor.
- `device/hid_device.rs` — USB HID device class. Presents the cloned descriptors to the target PC and streams HID reports out the interrupt IN endpoint.
- `proxy/forward.rs` — Receives parsed mouse reports and serializes them back into raw HID report bytes for the device side.
- `proxy/inject.rs` — Merges injection commands into mouse reports. Additive for dx/dy, override for buttons/wheel.
- `serial/command.rs` — Binary protocol parser for injection commands from the control PC over UART.

## Injection Protocol

Header-framed binary, 7 bytes per command:

| Byte | Field | Type | Notes |
|------|-------|------|-------|
| 0 | Header | u8 | Always 0xAA |
| 1-2 | dx | i16 LE | Added to mouse X |
| 3-4 | dy | i16 LE | Added to mouse Y |
| 5 | buttons | u8 | Override, or 0xFF = passthrough |
| 6 | wheel | i8 | Override, or 0x80 = passthrough |

## Build

```
cargo build --release
cargo objcopy --release -- -O ihex teensytrap.hex
teensy-loader-cli -w -v --mcu=imxrt1062 teensytrap.hex
```

## Conventions

- Bare metal Rust (`#![no_std]`, `#![no_main]`)
- `imxrt-ral` 0.5.x for register access (pinned by teensy4-bsp 0.5.x)
- `defmt` + `defmt-rtt` for logging (not USB serial — that's occupied by the device port)
- `heapless` for fixed-capacity collections (no allocator)
- 2-space indentation, 120 char max width (see `rustfmt.toml`)
- No doc-style comments on scaffold stubs — comments are added during implementation passes
- Dead code warnings are expected during scaffolding — do not add `#[allow(dead_code)]`
- All descriptors are raw byte arrays or `heapless::Vec<u8, N>` — never reconstructed from parsed fields

## Hardware

- **Board**: Teensy 4.1 (PJRC)
- **MCU**: NXP iMXRT1062 (ARM Cortex-M7 @ 600MHz, 1MB TCM RAM)
- **USB Host**: USB2 EHCI controller with integrated HS PHY (pin header on Teensy 4.1)
- **USB Device**: USB1 EHCI controller with integrated HS PHY (micro-USB connector)
- **Serial**: Any hardware UART (pins configurable) for control PC injection commands

## Toolchain

Nix flake with fenix provides the full toolchain. `direnv allow` to enter the dev shell.

Packages: Rust stable + thumbv7em-none-eabihf target, llvm-tools, rust-analyzer, gcc-arm-embedded, teensy-loader-cli, probe-rs-tools, cargo-binutils, Python 3 with pyserial.
