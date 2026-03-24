# teensytrap

Transparent USB mouse proxy on Teensy 4.1 (NXP iMXRT1062). Sits between a mouse and a target PC as an invisible pass-through — cloning the mouse's full USB identity and forwarding ALL traffic bidirectionally. The only thing we add is the ability to inject mouse movements from a control PC.

## Architecture

```
Control PC ──(USB Serial)──► Teensy 4.1 ──(USB Device @ 480Mbps HS)──► Target PC
                                 ▲
                          (USB Host @ 480Mbps HS)
                                 │
                               Mouse
```

The Teensy uses both independent USB controllers on the iMXRT1062:
- **USB2 (Host)**: Enumerates upstream mouse, forwards all endpoint traffic
- **USB1 (Device)**: Presents as an exact clone of the upstream mouse to the target PC

## Core Design Principles

### Exact 1:1 Transparent Proxy
The proxy must be completely indistinguishable from the real mouse. The target PC must not be able to tell that the proxy exists. This means:
- Raw device descriptor cloned byte-for-byte (VID, PID, bcdDevice, bcdUSB, all of it)
- Raw configuration descriptor cloned byte-for-byte — ALL interfaces, ALL endpoints, not just the mouse HID one
- Raw HID report descriptors cloned byte-for-byte for EVERY HID interface
- ALL string descriptors cloned byte-for-byte (manufacturer, product, serial number, and any others)
- ALL endpoints forwarded: interrupt IN/OUT, bulk IN/OUT — whatever the mouse exposes
- Control transfers from the target PC forwarded upstream to the real mouse (SET_REPORT, vendor requests, firmware update commands, everything)
- Only the mouse HID interrupt IN endpoint is intercepted for injection — everything else is pure passthrough

This means firmware updates from Razer Synapse / Logitech G Hub / SteelSeries GG work. DPI changes work. LED configuration works. Macro programming works. Profile switching works. The proxy is invisible.

### High-Speed USB (480Mbps)
Both USB ports run at USB 2.0 High Speed. The EHCI periodic schedule is configured to poll every microframe (S-mask 0xFF = 8kHz). The iMXRT1062 has integrated High-Speed PHYs on both ports — no external ULPI chip needed.

### Bare Metal, No RTOS
Direct EHCI register access via `imxrt-ral`. No USB middleware, no HAL USB stack, no scheduler between our code and the hardware. Every 125µs microframe is ours to fill.

## Module Responsibilities

- `usb/ehci.rs` — EHCI host controller driver. Initializes USB2 in host mode, manages port power/reset/detection.
- `usb/device.rs` — USB device controller driver. Initializes USB1 in device mode with dQH/dTD structures. Handles EP0 SETUP packets, dynamic descriptor serving, endpoint priming/stalling. This is a separate driver from the host EHCI — the iMXRT1062 device controller has its own register set and data structures.
- `usb/qh.rs`, `usb/qtd.rs` — Host-side EHCI Queue Head and Transfer Descriptor structures.
- `usb/periodic.rs` — Periodic frame list for 8kHz interrupt polling (S-mask 0xFF, every microframe).
- `usb/async_schedule.rs` — Async schedule for control transfers during enumeration and forwarding.
- `usb/transaction.rs` — Transfer submission and completion polling.
- `host/enumerate.rs` — USB enumeration state machine. Discovers ALL interfaces, ALL endpoints, ALL string descriptors (walks both device and config descriptors for string indices), ALL HID report descriptors. Stores everything as raw bytes.
- `host/hid.rs` — HID report descriptor parser. Walks the mouse HID report descriptor to determine field layout (button count, axis widths, report IDs) so we can interpret incoming reports for injection merging.
- `host/mouse.rs` — Interprets raw HID report bytes into structured `MouseReport` values using the parsed field layout.
- `device/descriptors.rs` — `ClonedDescriptors::from_upstream()` does byte-for-byte copies of everything: device desc, config desc, all HID report descs, all string descs.
- `device/endpoint.rs` — `ProxyEndpoint` handles any endpoint type (interrupt, bulk) in either direction.
- `device/proxy_device.rs` — `ProxyDevice` presents the full cloned device to the target PC. Handles control transfer forwarding to upstream mouse, all endpoint traffic, and identifies which endpoint is the mouse HID interrupt IN for injection.
- `proxy/forward.rs` — Routes endpoint traffic. Mouse HID interrupt IN goes through the injection pipeline. Everything else is byte-for-byte passthrough.
- `proxy/inject.rs` — Merges injection commands into mouse reports only. Additive for dx/dy, optional override for buttons/wheel.
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
- Traffic that isn't the mouse HID interrupt IN is forwarded byte-for-byte with zero interpretation

## Hardware

- **Board**: Teensy 4.1 (PJRC)
- **MCU**: NXP iMXRT1062 (ARM Cortex-M7 @ 600MHz, 1MB TCM RAM)
- **USB Host**: USB2 EHCI controller with integrated HS PHY (5-pin header on Teensy 4.1)
- **USB Device**: USB1 EHCI controller with integrated HS PHY (micro-USB connector)
- **Serial**: Any hardware UART (pins configurable) for control PC injection commands

## Toolchain

Nix flake with fenix provides the full toolchain. `direnv allow` to enter the dev shell.

Packages: Rust stable + thumbv7em-none-eabihf target, llvm-tools, rust-analyzer, gcc-arm-embedded, teensy-loader-cli, probe-rs-tools, cargo-binutils, Python 3 with pyserial.
