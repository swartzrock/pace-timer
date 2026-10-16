#!/usr/bin/env python3
"""Exercise the actual binary in a Unix PTY; no terminal emulator or server needed."""
import fcntl
import os
import pathlib
import pty
import re
import select
import signal
import struct
import subprocess
import sys
import termios
import time


class Timer:
    def __init__(self, executable, *args, environment=None):
        self.master, self.slave = pty.openpty()
        self.resize(100, 35)
        self.original = termios.tcgetattr(self.slave)
        self.output = bytearray()
        self.started = time.monotonic()
        self.process = subprocess.Popen(
            [executable, *args], stdin=self.slave, stdout=self.slave,
            stderr=self.slave, start_new_session=True,
            env={**os.environ, "TERM": "xterm-256color", **(environment or {})},
        )

    def resize(self, width, height):
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", height, width, 0, 0))
        if hasattr(self, "process"):
            self.process.send_signal(signal.SIGWINCH)

    def drain(self, duration):
        until = time.monotonic() + duration
        while time.monotonic() < until:
            if select.select([self.master], [], [], max(0, until - time.monotonic()))[0]:
                self.output.extend(os.read(self.master, 65536))

    def finish(self):
        try:
            deadline = time.monotonic() + 4
            while self.process.poll() is None and time.monotonic() < deadline:
                self.drain(0.03)
            assert self.process.poll() is not None, "timer did not exit"
            self.drain(0.02)
            assert self.process.returncode == 0, bytes(self.output[-2000:])
            assert b"\x1b[?1049h" in self.output, "alternate screen never entered"
            assert b"\x1b[?1049l" in self.output, "alternate screen never restored"
            assert b"\x1b[?25h" in self.output, "cursor never restored"
            assert b"panicked" not in self.output
            assert termios.tcgetattr(self.slave) == self.original, "terminal mode not restored"
            return time.monotonic() - self.started
        finally:
            if self.process.poll() is None:
                self.process.kill()
                self.process.wait()
            os.close(self.master)
            os.close(self.slave)


def main():
    executable = str(pathlib.Path(sys.argv[1]).resolve())
    renderers = ["aurora", "bar", "bigtext", "circles", "colorwheel", "colossal", "dots", "lava", "pie", "shuffle", "sine", "slant", "sweep"]
    for renderer in renderers:
        timer = Timer(executable, "0.12s", renderer)
        elapsed = timer.finish()
        assert elapsed < 1.5, (renderer, elapsed)
    print(f"PASS: all {len(renderers)} renderers complete and restore terminal modes")

    timer = Timer(executable, "timer", "0.8s", "sine")
    timer.drain(0.2)
    os.write(timer.master, b" ")
    timer.drain(0.45)
    assert timer.process.poll() is None
    os.write(timer.master, b" ")
    elapsed = timer.finish()
    assert 1.15 < elapsed < 2.2, elapsed
    print("PASS: space pauses/resumes countdown and animation")

    timer = Timer(executable, "10s:shuffle")
    timer.drain(0.15)
    timer.resize(1, 1)
    timer.drain(0.12)
    timer.resize(160, 50)
    timer.drain(0.12)
    os.write(timer.master, b"\x03")
    timer.finish()
    print("PASS: resize to 1x1 and back, Ctrl-C restores terminal")

    timer = Timer(executable, "10s", "lava")
    timer.drain(0.2)
    timer.process.send_signal(signal.SIGTERM)
    timer.finish()
    print("PASS: SIGTERM restores terminal")

    timer = Timer(executable, "3s", "sine", "--show-fps", "--no-color")
    timer.drain(0.25)
    # The diff backend may encode the label's separating space as cursor movement.
    assert b"FPS" in timer.output and b"--" in timer.output, "FPS warmup not displayed"
    assert re.search(rb"\d{1,3}\.\d", timer.output), "measured FPS not displayed"
    os.write(timer.master, b" ")
    timer.drain(0.15)
    assert b"paused" in timer.output, "paused FPS state not displayed"
    timer.resize(12, 1)
    timer.drain(0.1)
    timer.resize(100, 35)
    timer.drain(0.1)
    mark = len(timer.output)
    os.write(timer.master, b" ")
    timer.drain(0.15)
    assert b"--" in timer.output[mark:], "resume did not reset FPS warmup"
    os.write(timer.master, b"\x03")
    timer.finish()
    assert not re.search(rb"\x1b\[(?:38|48);", timer.output), "monochrome timer emitted color"
    print("PASS: FPS warmup, measured cadence, pause/resume, tiny resize and monochrome")

    timer = Timer(executable, "0.12s", "pie")
    timer.finish()
    assert b"FPS" not in timer.output, "FPS should default off"

    for depth in ["truecolor", "256", "16"]:
        timer = Timer(executable, "0.12s", "pie", "--color-depth", depth,
                      environment={"NO_COLOR": "1"})
        timer.finish()
        if depth == "truecolor":
            assert b"38;2;" in timer.output, "NO_COLOR suppressed explicit RGB"
        elif depth == "256":
            assert b"38;5;" in timer.output, "NO_COLOR suppressed explicit indexed color"
        else:
            assert not re.search(rb"(?:38|48);[25];", timer.output), "16-color output used extended SGR"
            assert re.search(rb"\x1b\[(?:[0-9]+;)*(?:3[0-7]|9[0-7])(?:;[0-9]+)*m", timer.output), "basic ANSI color absent"
    timer = Timer(executable, "0.12s", "pie", "--color-depth", "truecolor", "--no-color",
                  environment={"NO_COLOR": "1"})
    timer.finish()
    assert not re.search(rb"(?:38|48);[25];", timer.output), "--no-color lost precedence"
    print("PASS: actual timer color depth and NO_COLOR override precedence")


if __name__ == "__main__":
    main()
