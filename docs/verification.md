# Local verification

Historical port verification on macOS with Rust 1.99.0. The independent sibling `pace-ts` checkout
remained clean on `main` at `9f49953`. This repository has no remote; nothing was
published.

## Checks

- `cargo test --all-targets`: 33 tests passed; the measurement test is deliberately
  ignored in the normal suite and was run separately in release mode.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo build --release`: passed with `--target-dir /tmp/pace-rs-target`.
- `python3 tests/terminal_smoke.py target/release/pace`: passed against the copied
  standalone release executable. Covered every renderer completing, repeated
  pause/resume, resizing down to 1x1 and back, Ctrl-C, SIGTERM, alternate screen,
  cursor restoration and exact termios restoration.
- Font tests compare 306 original FIGlet.js outputs, including all 289 catalog
  fonts, timer fonts, punctuation, multiline and extended/right-to-left glyphs.

The Codex sandbox blocked Cargo's temporary-directory cleanup inside the new
project, including after escalation. Building in `/tmp/pace-rs-target` succeeded;
the verified release executable was copied to `target/release/pace` (3.4 MiB).
This is a local build-environment limitation, not a Cargo configuration change.

## Adaptive color and FPS foundation — 2026-10-15

The first visual-improvement phase passes 46 Rust tests, formatting, strict
Clippy, release build and the expanded Unix PTY smoke test. Builds used
`--target-dir /tmp/pace-rs-target` for the local sandbox limitation above.
The original 306 FIGlet outputs still pass without font/parser/asset changes.

New coverage checks captured capability/override precedence, nearest palette
quantization, foreground/background serialization, all tool output paths,
paired pixels, monochrome shape preservation, successful-frame timing math,
and countdown/footer layout down to one-row and offset viewports. The PTY
exercises FPS warmup, measured cadence, pause/resume reset, tiny resize,
monochrome output, default-off behavior and terminal restoration.

Proof-first checks initially failed for the missing pixel and FPS APIs, then
passed after implementation. The legacy CLI was characterized before changing
its parser. Monochrome font art exposed two lines that resembled catalog
headers; the catalog assertion now recognizes the complete header format.

The Rust workflow runs test and release PTY checks on Linux and macOS and
formatting/Clippy on Linux. It accepts every pull-request base, including the
second branch of the visual-improvement stack. CI results belong to each PR's
current head; local verification does not substitute for those checks.

Review `20261015-133248-c5542576` confirmed three output-boundary defects;
all were reproduced and fixed before publication. Monochrome now distinguishes
empty dots and pie sectors. The timer configures Crossterm from the resolved
policy so explicit depth overrides `NO_COLOR`. A basic ANSI writer translates
Crossterm's indexed encoding of named colors into ordinary 16-color SGR.
PTY assertions now check actual timer bytes for all three forced color depths
with `NO_COLOR`, and verify that `--no-color` still wins. The writer regression
covers combined foreground/background, fragmented writes and cursor/sync controls.

## Renderer refinement — 2026-10-15

The combined phase passes 60 Rust tests, including all 306 unchanged FIGlet
fixtures, formatting, strict Clippy, release build and expanded PTY smoke.
Semantic tests cover sub-100ms motion, bilinear arithmetic, palette/generation
continuity, antialiased half-samples, fractional progress, clipping, deterministic
pause behavior, soft shadows and coherent countdowns during feathered shuffle.
The temporary review capture was removed after inspecting all 12 RGB renderers
and four representative effects at RGB/256/16/monochrome depths. These buffer
previews verify shape, color and text layout; native emulator appearance remains
a manual check. Bright monochrome pixel fields leave solid blocks for text contrast.

### Preparation before and after

Same local release environment, 120x40, 600 warmed frames spanning ten active
seconds. Before is `79d84fd`; after includes richer geometry and text layers.
The adaptation column includes composition plus the slowest selected color depth.
These measurements exclude terminal I/O and are subject to host timing noise.

| Renderer | Before ms/frame | After ms/frame | After including adaptation ms/frame |
| --- | ---: | ---: | ---: |
| bar | 0.009 | 0.005 | 0.032 |
| bigtext | 0.011 | 0.014 | 0.054 |
| circles | 0.020 | 0.416 | 0.640 |
| colorwheel | 0.032 | 0.141 | 0.287 |
| colossal | 0.008 | 0.020 | 0.055 |
| dots | 0.006 | 0.010 | 0.057 |
| lava | 0.011 | 0.168 | 0.321 |
| pie | 0.041 | 0.116 | 0.249 |
| shuffle | 0.028 | 0.191 | 0.335 |
| sine | 0.008 | 0.037 | 0.119 |
| slant | 0.007 | 0.102 | 0.096 |
| sweep | 0.018 | 0.102 | 0.206 |

The richer effects cost more computation while remaining below the 16.7ms frame
budget here. Individual runs vary, including the slant samples. Lava's first
render in this benchmark took 10.98ms to construct maps; warmed work excludes it.

The final simplification skips lighting calculations for uncovered circle samples.
Repeating the same release workload measured circles at 0.188ms/frame before
adaptation and 0.443ms/frame at the slowest depth, down from 0.416/0.640ms above.
The output pixels are unchanged; all 60 tests, strict Clippy and PTY checks still pass.

### Completed output and bytes

`python3 -B tests/terminal_measure.py /tmp/pace-rs-target/release/pace` drains a
120x40 PTY and counts completed synchronized-update markers in one-second windows.
It uses the actual release executable with FPS enabled; shuffle is sampled during
its transition. This is receiver-observed output cadence, with window/read timing
error, and does not establish monitor presentation rate or SSH throughput.

| Renderer | Truecolor completed/s | Truecolor KiB/s | 256 completed/s | 256 KiB/s |
| --- | ---: | ---: | ---: | ---: |
| bar | 59.96 | 5.16 | 60.00 | 3.02 |
| bigtext | 60.00 | 3.66 | 60.00 | 2.68 |
| colorwheel | 59.98 | 3989.64 | 59.97 | 628.75 |
| lava | 60.00 | 4657.18 | 58.92 | 1411.34 |
| shuffle | 59.89 | 2675.41 | 59.84 | 1101.31 |

Colorwheel measured 60.00 completed/s and 104.73 KiB/s at 16 colors, and 59.91
completed/s and 26.91 KiB/s in monochrome after the text-contrast adjustment.
First observed completions, including
process/setup, were 3.25–5.07ms for representative non-lava runs and 15.63–17.01ms
for lava. More color increases output volume; the optional FPS footer exposes
slower delivered loops on other terminal connections.

Phase-2 review `20261015-142123-7fc9bc92` found no actionable defects or unmet
requirements. Deterministic shuffle coverage does not force every combination of
text/geometric countdown classifications; native emulator appearance also remains
a manual check. Final-head CI results are recorded on the renderer PR.

### Lava correction after terminal feedback

The initial refinement still repeated a multicolor palette fourteen times across
the summed height field. This produced tightly packed rainbow bands even in RGB;
256-color quantization made the boundaries harsher, while unrelated palette
crossfades could become gray. Real-buffer captures reproduced the supplied image.
Lava now uses a lower spatial frequency and three related black/red/orange/gold
ramps, preserving elapsed-time motion, halfblock sampling and epoch continuity.
Other renderers and the global palette catalog are unchanged.

The regression fails before the fix at seed 0, six seconds: 320 of 4272 horizontal
half-sample neighbors differ by more than 95 in a channel after 256-color adaptation.
The corrected field stays below 1% across four seeds at 0, 6, 10, 19.9 and 20.1
seconds. A one-step palette jump is allowed: indexed color still has visible steps.
All 61 Rust tests, formatting, strict Clippy, release build and expanded PTY smoke
pass. Focused review `20261015-144350-1d2be19e` found no defects. The all-renderer
preview was refreshed; `visuals/lava-256-preview.png` shows the actual 256-color
buffer at six seconds. Appearance and motion in the user's emulator remain manual.

### Indexed output throughput at 210x78

Following a report of approximately 23 completed draws/s in macOS Terminal versus
60 in Warp, the output path was profiled at the reported 210-column, 78-row size.
CPU-only rendering, color adaptation, diffing and encoding took approximately
1–2ms/frame; this excludes terminal writes and emulator painting. The original
stream repeats both colors when either changes and prints solid pixels as Unicode
halfblocks. These increase the work sent to the terminal.

The Unix indexed path now emits only changed visible colors, uses background spaces
for solid paired pixels, uses shorter same-row cursor movements, and batches a
draw into one write request. RGB, basic ANSI, monochrome and styled-cell output
retain the Crossterm path. The reusable output buffer is bounded by one draw.
Other platforms retain Crossterm's native console handling for every color depth.
Scheduling, FPS accounting, renderer samples and chosen palettes are unchanged.

`cargo test --release --lib measure_lava_output -- --ignored --nocapture` compares
both encoders on the exact same 180 lava frames, including the initial full draw:

| Viewport | Original bytes/frame | Compact bytes/frame | Reduction |
| --- | ---: | ---: | ---: |
| 120x40 | 9914 | 5955 | 39.9% |
| 210x78 | 34369 | 19747 | 42.5% |

Palette choices vary by process, so absolute byte counts vary; each comparison
uses the same renderer and frame sequence for both encoders. The draining PTY
samples at 210x78 reached approximately 60 completed draws/s after the change;
they do not measure native Terminal painting. Launching native Terminal through
automation failed in this session. The user then retested the rebuilt binary in
the same 210x78 macOS Terminal window and reported 60 FPS, up from approximately
23, with the image unchanged. This confirms the improvement for that setup.

The output-budget regression first failed at 71599 bytes before the compact path
was implemented. New tests decode indexed cursor/SGR streams from both encoders
and compare visible glyphs and upper/lower color samples across all 12 renderers,
multiple times and two viewports. They also cover sparse row changes, complete
color reset, RGB/styled fallback, partial writes and error propagation. All 65
Rust tests, formatting, strict Clippy, release build and expanded PTY smoke pass.
Review `20261015-153400-e88cc103` found a Windows console fallback regression;
restricting the compact path to Unix preserves the existing Windows behavior.

## Historical port frame preparation

Release mode; 120x40 buffer; 600 frames per renderer after warming. Includes
buffer reset and renderer composition. Excludes terminal writes, emulator
painting, and initial font parsing/lava-map construction.

| Renderer | Milliseconds/frame |
| --- | ---: |
| bar | 0.013 |
| bigtext | 0.009 |
| circles | 0.015 |
| colorwheel | 0.025 |
| colossal | 0.007 |
| dots | 0.006 |
| lava | 0.011 |
| pie | 0.024 |
| shuffle | 0.010 |
| sine | 0.008 |
| slant | 0.008 |
| sweep | 0.018 |

These numbers establish frame-computation headroom against the 16.7ms budget.
They are not a measurement of visible 60 FPS or a benchmark against TypeScript.
Native Linux/Windows behavior and visible emulator smoothness were not manually
verified. Run the visual checks in the README on the intended terminal.

## Review and fixes

Completed Compound Engineering code review, run `20261014-213715-fb3de2d0`, with
correctness, testing, maintainability, security, performance, reliability and
adversarial lenses. Receipt: `/tmp/compound-engineering-501/ce-code-review/20261014-213715-fb3de2d0/review.json`.

All three confirmed P2 findings were applied inline and verified:

1. Stable bigtext scaling uses the original fixed `55:55` reference instead of
   changing scale with the displayed digits. Regression first failed with glyph
   heights `[20,10]`, then passed with `[10,10]`.
2. Terminal composition reserves space for the status bar and omits it in a
   one-row viewport. Regression first displayed only `Starting a 0`; the same
   composition test now finds the readable countdown at 12x1 and 12x4.
3. Circles/sine retain their original black text shadows. Exposed-shadow color
   assertions failed before the fix and pass afterward.

Added the review's suggested coverage for sub-100ms sine movement and shuffle
wipe composition before, during and after the transition. No actionable findings
remain. Manual emulator smoothness is the remaining verification limit.

The preceding simplification pass removed an unused helper, reused rectangle
centering and `mem::take`, hoisted scale-independent glyph work, and removed
unnecessary heap allocation/gradient work in the disk renderers. A proposed
second cache for trimmed glyph rows was omitted: warmed frame work was already
well below the budget, and another cache would duplicate state without a
measured need.
