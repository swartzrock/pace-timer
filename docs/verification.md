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

The first visual-improvement phase passes 44 Rust tests, formatting, strict
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

## Frame preparation

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
