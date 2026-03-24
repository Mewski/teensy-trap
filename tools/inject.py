#!/usr/bin/env python3

import math
import serial
import struct
import sys
import time

HEADER = 0xAA
NO_BUTTONS = 0xFF
NO_WHEEL = 0x80


def pack_command(dx: int, dy: int, buttons: int = NO_BUTTONS, wheel: int = NO_WHEEL) -> bytes:
    return struct.pack("<BhhBb", HEADER, dx, dy, buttons, wheel)


def send_injection(ser: serial.Serial, dx: int, dy: int, buttons: int = NO_BUTTONS, wheel: int = NO_WHEEL):
    ser.write(pack_command(dx, dy, buttons, wheel))


def circle(ser: serial.Serial, radius: int = 50, steps: int = 360, delay: float = 0.001):
    for i in range(steps):
        angle = 2 * math.pi * i / steps
        dx = int(radius * math.cos(angle))
        dy = int(radius * math.sin(angle))
        ser.write(pack_command(dx, dy))
        time.sleep(delay)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"usage: {sys.argv[0]} <serial-port> [circle]")
        sys.exit(1)

    port = sys.argv[1]
    mode = sys.argv[2] if len(sys.argv) > 2 else "single"

    with serial.Serial(port, 115200, timeout=1) as ser:
        if mode == "circle":
            circle(ser)
        else:
            send_injection(ser, dx=100, dy=0)
