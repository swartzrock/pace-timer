# Rust port of Pace

Preserve the 0.3.0 TypeScript CLI in a standalone sibling repository. The source
repository is `../pace-ts`, commit `9f49953`. Leave it untouched. Keep this port
local; do not create a remote repository or publish a pull request.

The user selected smoother animations as the primary goal and explicitly chose
to preserve all 12 renderers and existing CLI behavior. Preserve `pace DURATION
[RENDERER]`, `pace timer DURATION [RENDERER]`, `DURATION:RENDERER`, default shuffle,
space to pause/resume, Ctrl-C to exit, help, and tools preview/allfonts/colorblocks/version.
Preserve the original effects, gradients, fonts, ten-second shuffle period and
transition duration. Accept the documented minutes/seconds duration syntax.

One delivery phase on `feat/rust-timer-port`, with three implementation units:

1. Establish the Rust project, shared rendering contracts and time model.
2. Port the renderer collection and bundled font/tool support using independent
   file ownership, then integrate them with the terminal runtime.
3. Verify CLI parity, timing independent of frame rate, pauses, resizes, terminal
   restoration, all renderers at small/normal/large sizes, and frame preparation
   cost. Review, fix findings, and commit locally.

Use Rust with Ratatui/Crossterm: a synchronous event loop, monotonic countdown,
60 FPS target, elapsed-time animation and diffed terminal updates. Drop missed
frames rather than replaying them. Do not introduce a server, async runtime,
configuration system, or new timer product features.

Verification: cargo test, cargo fmt --check, cargo clippy --all-targets with
warnings denied, release build, CLI subprocess tests, and pseudo-terminal
integration tests. Record measured frame preparation separately from visible
terminal smoothness; the latter depends on the emulator and needs manual review.

