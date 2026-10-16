#!/usr/bin/env python3
"""Measure drained PTY output cadence and bytes; this does not measure screen painting."""
import os
import pathlib
import select
import sys
import time

from terminal_smoke import Timer


class MeasuredTimer(Timer):
    first_completed = None

    def drain(self, duration):
        until = time.monotonic() + duration
        while time.monotonic() < until:
            if select.select([self.master], [], [], max(0, until - time.monotonic()))[0]:
                self.output.extend(os.read(self.master, 65536))
                if self.first_completed is None and b"\x1b[?2026l" in self.output:
                    self.first_completed = time.monotonic() - self.started


def main():
    executable = str(pathlib.Path(sys.argv[1]).resolve())
    cases = [(name, depth) for depth in ["truecolor", "256"]
             for name in ["bar", "bigtext", "colorwheel", "lava", "shuffle"]]
    cases += [("colorwheel", depth) for depth in ["16", "1"]]
    print("120x40 draining PTY; first completion includes process/setup; 1s observed window.")
    print("renderer depth first_ms completed/s KiB/s bytes/completion", flush=True)
    for name, depth in cases:
        flags = ["--color-depth", depth]
        timer = MeasuredTimer(executable, "60s", name, "--show-fps", *flags,
                              environment={"NO_COLOR": "", "COLORTERM": "truecolor"})
        timer.resize(120, 40)
        # Include the feathered transition in shuffle's output workload.
        timer.drain(8.0 if name == "shuffle" else 0.35)
        marker = len(timer.output)
        started = time.monotonic()
        timer.drain(1.0)
        elapsed = time.monotonic() - started
        output = timer.output[marker:]
        frames = output.count(b"\x1b[?2026l")
        os.write(timer.master, b"\x03")
        timer.finish()
        assert frames > 0, f"{name}: no completed terminal writes"
        print(f"{name:10} {depth:9} {timer.first_completed * 1000:8.2f} "
              f"{frames / elapsed:10.2f} {len(output) / elapsed / 1024:8.2f} "
              f"{len(output) / frames:8.0f}", flush=True)


if __name__ == "__main__":
    main()
