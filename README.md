# Pace, in Rust

A standalone Rust port of [Pace](https://github.com/swartzrock/pace), the visual
terminal countdown timer. The TypeScript source remains in the sibling
`pace-ts` folder; this project builds and runs independently.

The port targets 60 frames per second with buffered terminal updates. Animation
and countdown progress use elapsed time rather than frame counts, so delayed
frames do not speed up effects or lengthen the timer. Missed frames are skipped.

## Run

With Rust and Cargo installed, run these commands from this folder:

```sh
cargo run --release -- 5m shuffle
cargo run --release -- 3m10s pie
cargo run --release -- timer 2.5m lava
cargo run --release -- 1m:sine
```

To build a standalone executable:

```sh
cargo build --release
./target/release/pace 5m shuffle
```

On Windows the executable is `target\release\pace.exe`. The countdown requires
an interactive terminal. Press **Space** to pause or resume and **Ctrl-C** to exit.
Fonts and color data are compiled into the executable; Node and external font
files are not required.

## Renderers and tools

All 12 original renderer names are available: `bar`, `bigtext`, `circles`,
`colorwheel`, `colossal`, `dots`, `lava`, `pie`, `shuffle`, `sine`, `slant`, `sweep`.
The default, `shuffle`, changes renderer every ten active seconds and wipes into
the next renderer over the last 2.5 seconds of each slot.

```sh
cargo run --release -- tools preview
cargo run --release -- tools allfonts
cargo run --release -- tools colorblocks
cargo run --release -- tools version
cargo run --release -- --help
```

The font catalog includes all 289 original fonts. Its layout is checked against
306 original FIGlet.js outputs. Source font notices and the FIGlet.js license are
preserved under `assets/`.

The documented decimal duration form (`2.5m`) now means 150 seconds; the old
parser accidentally interpreted it as five minutes. Invalid duration text
produces a usage error. Remaining seconds retain the original rounding down.
Animation freezes while paused, including lava, which previously used wall time.

## Verify

```sh
cargo test --all-targets
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --release
```

On macOS or Linux, exercise the binary in a pseudo-terminal, including all
renderers, pause/resume, resize, Ctrl-C, SIGTERM, and terminal restoration:

```sh
python3 tests/terminal_smoke.py target/release/pace
```

Measure warmed frame preparation separately from terminal output:

```sh
cargo test --release --test renderers measure_frame_preparation -- --ignored --nocapture
```

The frame measurement excludes the emulator, output flushing, and initial font
or lava-map preparation. Visible smoothness depends on terminal size, emulator,
and font settings. For a visual check, run `1m sine`, `1m lava`, and `2m shuffle`;
pause, resume, and resize the window while they run.

## License

ISC, as in the original Pace project. Bundled font notices and the MIT license
for the ported FIGlet.js layout rules are in `assets/`.
