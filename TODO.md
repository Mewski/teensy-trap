# TODO

## Hardware Feasibility Notes

The iMXRT1062 on the Teensy 4.1 has two independent USB controllers with integrated High-Speed PHYs:
- USB1 (micro-USB connector): device controller with 8 endpoint pairs (EP0-EP7, each IN + OUT = 14 non-control endpoints)
- USB2 (5-pin header): host controller, full EHCI

Both run simultaneously at 480Mbps. The transparent proxy architecture is viable with these constraints:
- Max 7 non-control endpoint pairs mirrored from upstream (EP1-EP7). Most mice use 1-3 endpoints — this is fine.
- Control transfer forwarding adds latency: device side must NAK while the host side round-trips to the real mouse. At 600MHz this is single-digit microseconds for the CPU work, but the USB bus round-trip itself is ~125µs worst case (one microframe). Vendor software (Synapse, G Hub) tolerates this — it's within USB spec.
- The device controller uses dQH/dTD structures, not the same QH/qTD as the host EHCI. Both drivers are independent.

## Phase 1: Blink and Board Bring-Up

- [ ] Flash `examples/blink.rs` to verify toolchain and board
- [ ] Verify defmt RTT logging works over debug probe
- [ ] Confirm clock configuration (600MHz core, USB PLL for 480Mbps)

## Phase 2: USB Host Controller

- [x] `usb/ehci.rs`: Controller reset sequence (USBCMD.RST, wait for HCHalted)
- [x] `usb/ehci.rs`: Set host mode (USBMODE.CM = 0b11)
- [x] `usb/ehci.rs`: Port power enable, device connection detection (PORTSC1)
- [x] `usb/ehci.rs`: Port reset (drive reset, wait, check port enabled)
- [x] `usb/ehci.rs`: Port speed detection (PORTSC1.PSPD)
- [x] `usb/ehci.rs`: Set PERIODICLISTBASE (via DEVICEADDR offset in host mode)
- [x] `usb/ehci.rs`: Set ASYNCLISTADDR
- [x] `usb/ehci.rs`: Enable/disable periodic and async schedules with status polling
- [x] `usb/ehci.rs`: Run/stop controller, interrupt enable, status polling/ack
- [x] `usb/ehci.rs`: Frame list size configuration
- [x] `usb/transaction.rs`: Poll qTD for completion with timeout, check error bits
- [x] `usb/transaction.rs`: Poll USBSTS for transfer complete with timeout
- [ ] USBPHY initialization (disable charge detect, enable PLL, clear power-down)
- [ ] Interrupt handler wiring (NVIC registration for USB2 IRQ)
- [ ] Hardware validation: detect a real USB mouse connection

## Phase 3: Device Enumeration

- [x] `usb/consts.rs`: All SETUP packet builders (GET_DESCRIPTOR, SET_ADDRESS, SET_CONFIGURATION, SET_IDLE, etc.)
- [x] `usb/consts.rs`: USB descriptor type constants, request constants, HID constants
- [x] `host/enumerate.rs`: `parse_config_descriptor()` — walks raw bytes, builds InterfaceInfo vec with endpoints and HID report desc lengths
- [x] `host/enumerate.rs`: `string_indices_referenced()` — scans both device and config descriptors for string indices
- [x] `host/enumerate.rs`: `EnumeratedDevice` with raw storage for all descriptors
- [ ] `host/enumerate.rs`: `enumerate_device()` — wire up the full sequence using SETUP packets and control transfers:
  - GET_DESCRIPTOR(device) at addr 0, 8 bytes
  - Port reset
  - SET_ADDRESS
  - GET_DESCRIPTOR(device) full 18 bytes
  - GET_DESCRIPTOR(config) 9 bytes then full wTotalLength
  - GET_DESCRIPTOR(string) for each referenced index
  - SET_CONFIGURATION
  - GET_DESCRIPTOR(HID report) for each HID interface
  - SET_IDLE for each HID interface
- [ ] Test: enumerate a real mouse, log all descriptors via defmt

## Phase 4: HID Report Parsing

- [x] `host/hid.rs`: Full HID report descriptor parser — walks item stream (main/global/local items)
- [x] `host/hid.rs`: Extracts button field offset/count, X/Y axis offset/width/signedness, wheel offset
- [x] `host/hid.rs`: Handles Report ID prefix
- [x] `host/hid.rs`: Handles both explicit usage list and usage min/max range
- [x] `host/hid.rs`: Bit-level field extraction from raw report bytes (extract_x, extract_y, extract_buttons, extract_wheel)
- [ ] Test: parse report descriptors from known mice (Logitech, Razer, SteelSeries) — unit tests with known descriptor bytes

## Phase 5: USB Device Controller

- [x] `usb/device.rs`: Controller reset, set device mode (USBMODE.CM = 0b10, SLOM)
- [x] `usb/device.rs`: DqhList with proper 2048-byte alignment, dQH and dTD structures
- [x] `usb/device.rs`: dQH capabilities (max packet size, ZLT, interrupt on setup)
- [x] `usb/device.rs`: dTD init with buffer pointer page crossing, active/halted/bytes_remaining
- [x] `usb/device.rs`: Set endpoint list address (ASYNCLISTADDR in device mode)
- [x] `usb/device.rs`: Set device address with USBADRA advance bit
- [x] `usb/device.rs`: Configure endpoint type/direction (ENDPTCTRL registers, data toggle reset)
- [x] `usb/device.rs`: Endpoint prime (RX/TX), flush, stall
- [x] `usb/device.rs`: Setup tripwire mechanism (SUTW) for safe SETUP packet reads
- [x] `usb/device.rs`: Interrupt enable, status polling, endpoint complete readback
- [ ] EP0 control transfer handling — SETUP packet dispatch, dynamic descriptor serving
- [ ] Return cloned descriptors (device, config, string, HID report) from device side
- [ ] SET_ADDRESS handling with deferred address via USBADRA
- [ ] SET_CONFIGURATION handling — prime all non-control endpoints
- [ ] USBPHY initialization for USB1
- [ ] Interrupt handler wiring (NVIC registration for USB1 IRQ)
- [ ] Test: device enumerates on a PC, shows correct cloned VID/PID/strings in lsusb

## Phase 6: Transparent Proxy Loop

- [ ] Forward mouse HID interrupt IN reports from host to device (the hot path)
- [ ] Forward all other interrupt IN endpoint data unchanged
- [ ] Forward interrupt OUT data from device to host (LED commands, DPI changes)
- [ ] Forward bulk IN/OUT data from device to host and back (firmware updates)
- [ ] Control transfer forwarding: intercept SETUP on device side, forward to host side, relay response
- [ ] Handle SET_REPORT, GET_REPORT, vendor-specific control transfers transparently
- [ ] Handle SET_IDLE, SET_PROTOCOL passthrough
- [ ] Test: passthrough-only mode (no injection) — verify mouse works identically through proxy
- [ ] Test: Logitech G Hub detects mouse through proxy and can configure DPI/LEDs
- [ ] Test: Razer Synapse detects mouse through proxy and can update firmware
- [ ] Test: SteelSeries GG detects mouse through proxy

## Phase 7: Injection

- [ ] `serial/command.rs`: Wire UART RX interrupt to feed bytes into CommandParser
- [ ] `proxy/inject.rs`: Merge injection commands into parsed MouseReport
- [ ] `proxy/forward.rs`: Re-serialize modified MouseReport back to raw HID report bytes using MouseFieldLayout
- [ ] Test: inject constant dx/dy offset, verify cursor moves with added bias
- [ ] Test: inject while real mouse is moving, verify both inputs combine smoothly
- [ ] Test: injection at 8kHz rate from control PC, verify no dropped commands

## Phase 8: Performance and Polish

- [ ] Measure end-to-end proxy latency (host IN → device IN) with Cynthion/Packetry
- [ ] Verify 8kHz polling rate on host side with real 8kHz mouse
- [ ] Verify 8kHz output rate on device side
- [ ] Profile interrupt handler execution time — must complete within 125µs microframe
- [ ] Stress test: sustained 8kHz for hours, verify no drift/drops/hangs
- [ ] Test with multiple mouse brands and models
- [ ] Handle device disconnection and reconnection gracefully
- [ ] Handle target PC USB reset/suspend/resume

## Testing Strategy

Tests that run on-device (defmt output, verified by eye or script):
- Enumeration dumps: compare raw descriptors through proxy vs direct connection
- Loopback: inject known pattern, verify output matches expected
- Latency: toggle GPIO on host IN completion and device IN prime, measure with oscilloscope

Tests that run on host PC (Python + pyserial + Cynthion):
- `tools/inject.py` circle test — visual verification of injection
- Cynthion packet capture: compare USB traffic with and without proxy in path
- Vendor software tests: G Hub, Synapse, GG — configure device through proxy
- Firmware update test: update mouse firmware through proxy, verify success
