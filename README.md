# Pace, in Rust

A standalone Rust port of [Pace](https://github.com/swartzrock/pace), the visual
terminal countdown timer. The TypeScript source remains in the sibling
`pace-ts` folder; this project builds and runs independently.

The port targets 60 frames per second with buffered terminal updates. Animation
and countdown progress use elapsed time rather than frame counts, so delayed
frames do not speed up effects or lengthen the timer. Missed frames are skipped.

## Run

Start a timer with `pace DURATION [RENDERER] [OPTIONS]`. The renderer defaults to
`shuffle`; durations use minutes and seconds, such as `5m`, `3m10s`, or `2.5m`.

With Rust and Cargo installed, run these commands from this folder:

```sh
cargo run --release -- 5m
cargo run --release -- 3m10s pie
cargo run --release -- 2.5m lava
cargo run --release -- 1m sine
```

To build a standalone executable:

```sh
cargo build --release
./target/release/pace 5m shuffle
./target/release/pace --help
```

Run `pace preview [OPTIONS]` to print static samples of the renderers (excluding
shuffle) and exit. The compact grid uses one column in narrow terminals, two at
100 characters wide, and three at 150. Renderers appear alphabetically, left to
right and then down. Panels are 18–24 rows tall, with a small heading and muted
borders. To preview with a specific color depth:

```sh
./target/release/pace preview --color-depth 256
```

On Windows the executable is `target\release\pace.exe`. The countdown requires
an interactive terminal. Press **Space** to pause or resume and **Ctrl-C** to exit.
Fonts and color data are compiled into the executable; Node and external font
files are not required.

## Color and FPS

```sh
./target/release/pace 1m lava --show-fps
./target/release/pace 1m colorwheel --color-depth truecolor
./target/release/pace 1m pie --color-depth 256
./target/release/pace 1m sine --color-depth 16
./target/release/pace 1m bigtext --color-depth 1
```

`--color-depth auto|truecolor|256|16|1` defaults to `auto`. Use `1` for monochrome
output. Auto uses positive `COLORTERM=truecolor|24bit` or direct-color `TERM`
hints for RGB, a 256-color hint for indexed color, recognized ANSI terminals for
16 colors, and otherwise monochrome. `TERM=dumb` uses monochrome.
SSH and tmux can hide capability hints; force a depth when you know the full
terminal connection supports it.

An explicit color depth overrides automatic detection and `NO_COLOR`. Auto
honors `NO_COLOR`. Flags can appear before or after operands; each flag may
appear once. Both foreground and background use the selected depth.

The muted footer shows elapsed / total time on the left and `Space: pause` or
`Space: resume` in the center. Shuffle also shows the current renderer and the
next renderer during a transition. Narrow terminals shorten the time display and
hide hints or FPS when they would overlap; one-row terminals show only the timer.

`--show-fps` is optional and timer-only. Its bottom-right label measures successful
completed draws per second, including terminal I/O and scheduled waiting over a
roughly one-second window. It measures the draw loop rather than the terminal
emulator's screen painting. `FPS --` means warming up; `FPS paused` means paused.
Resuming starts a fresh window. Narrow terminals hide labels that cannot fit,
and a one-row terminal prioritizes the countdown over the footer.

## Renderers

All 12 original renderer names are available: `bar`, `bigtext`, `circles`,
`colorwheel`, `colossal`, `dots`, `lava`, `pie`, `shuffle`, `sine`, `slant`, `sweep`.
The default, `shuffle`, changes renderer every ten active seconds and wipes into
the next renderer over the last 2.5 seconds of each slot.

Pixel effects use foreground/background halfblocks for finer vertical detail.
Lava interpolates its height maps and warm palettes across broad lobes;
colorwheel rotates continuously;
pie and sweep have softer contours; circles add lighting and generation fades;
sine includes a shaded trailing wave. Bar and dots show fractional segments.
The three text renderers retain their fonts and stable sizing with richer shading
and soft shadows. Shuffle feathers backgrounds while keeping one complete countdown.

![Static RGB buffer previews of all twelve renderers](docs/visuals/renderer-preview.png)

New renderers use true-color half-block cells for smooth gradients and two
vertical samples per character, without a GPU or extra runtime. They work best
in a 24-bit-color terminal and are included in shuffle:

- `aurora` (Aurora Veil): mint and violet light curtains retreat from right to left.
- `tide`: an ocean recedes with broad horizontal swells and a little foam along
  the waterline.
- `interference` (Interference Field): ripples from two sources settle from right
  to left as time runs out. It opens in blue and pink, then changes among six
  color pairs in a shuffled order every 10 seconds of active time, with a
  2.5-second crossfade from the top left to the bottom right.

```sh
cargo run --release -- 5m aurora
cargo run --release -- 5m tide
cargo run --release -- 5m interference
```

Font layout is checked against 306 original FIGlet.js outputs. Source font notices
and the FIGlet.js license are preserved under `assets/`.

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
cargo test --release --lib measure_lava_output -- --ignored --nocapture
python3 -B tests/terminal_measure.py target/release/pace
```

The Rust measurements report render work, color adaptation and a comparison of
encoded lava bytes. The PTY measurement reports observed completed-write
cadence and output bytes, including a shuffle transition. Neither measures emulator
painting. Visible smoothness depends on terminal size, emulator,
and font settings. For a visual check, run `1m sine`, `1m lava`, and `2m shuffle`;
pause, resume, and resize the window while they run.

## License

ISC, as in the original Pace project. Bundled font notices and the MIT license
for the ported FIGlet.js layout rules are in `assets/`.
