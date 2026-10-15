use crate::{
    color::{OutputPolicy, adapt_buffer},
    graphics::centered_text,
    model::{Clock, FrameRate, FrameSchedule, TimerDetails},
    output::OutputBackend,
    renderers::{Renderers, paused_overlay},
};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{
        self, BeginSynchronizedUpdate, EndSynchronizedUpdate, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{
    Terminal,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};
use std::{
    io::{self, IsTerminal, Stdout, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

struct Session {
    terminal: Terminal<OutputBackend<ColorWriter<Stdout>>>,
}

/// Crossterm encodes even named colors as indexed SGR. Basic terminals need ANSI SGR.
struct ColorWriter<W> {
    inner: W,
    basic: bool,
    pending: Vec<u8>,
}

impl<W: Write> Write for ColorWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.basic {
            return self.inner.write(bytes);
        }
        self.pending.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.basic {
            self.inner.write_all(&basic_ansi(&self.pending))?;
            self.pending.clear();
        }
        self.inner.flush()
    }
}

fn basic_ansi(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i..].starts_with(b"\x1b[") {
            let mut end = i + 2;
            while end < input.len() && (input[end].is_ascii_digit() || input[end] == b';') {
                end += 1;
            }
            if input.get(end) == Some(&b'm') {
                let parameters: Vec<_> = std::str::from_utf8(&input[i + 2..end])
                    .unwrap()
                    .split(';')
                    .collect();
                output.extend_from_slice(b"\x1b[");
                let mut p = 0;
                while p < parameters.len() {
                    if p > 0 {
                        output.push(b';');
                    }
                    let index = parameters
                        .get(p + 2)
                        .and_then(|index| index.parse::<u8>().ok())
                        .filter(|index| *index < 16);
                    if matches!(parameters[p], "38" | "48")
                        && parameters.get(p + 1) == Some(&"5")
                        && let Some(index) = index
                    {
                        write!(
                            output,
                            "{}",
                            crate::color::ansi_code(index, parameters[p] == "48")
                        )
                        .unwrap();
                        p += 3;
                    } else {
                        output.extend_from_slice(parameters[p].as_bytes());
                        p += 1;
                    }
                }
                output.push(b'm');
                i = end + 1;
                continue;
            }
        }
        output.push(input[i]);
        i += 1;
    }
    output
}

impl Session {
    fn enter(policy: OutputPolicy) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let result = (|| {
            let mut out = ColorWriter {
                inner: io::stdout(),
                basic: policy == OutputPolicy::Ansi16,
                pending: Vec::new(),
            };
            execute!(out, EnterAlternateScreen, Hide)?;
            Terminal::new(OutputBackend::new(out, policy == OutputPolicy::Ansi256))
                .map(|terminal| Self { terminal })
        })();
        if result.is_err() {
            restore();
        }
        result
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        restore();
    }
}

fn restore() {
    let _ = execute!(
        io::stdout(),
        EndSynchronizedUpdate,
        Show,
        LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
}

pub fn run(duration: Duration, name: &str, policy: OutputPolicy, show_fps: bool) -> io::Result<()> {
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        return Err(io::Error::other(
            "The timer needs an interactive terminal on stdin and stdout",
        ));
    }
    // The captured policy already resolves NO_COLOR and explicit overrides.
    crossterm::style::force_color_output(policy != OutputPolicy::Monochrome);
    let interrupted = Arc::new(AtomicBool::new(false));
    #[cfg(unix)]
    let signals = {
        let mut signals = Vec::new();
        for signal in [
            signal_hook::consts::SIGTERM,
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGHUP,
        ] {
            signals.push(signal_hook::flag::register(
                signal,
                Arc::clone(&interrupted),
            )?);
        }
        signals
    };
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        previous_hook(info);
    }));
    let result = run_loop(duration, name, policy, show_fps, &interrupted);
    #[cfg(unix)]
    for signal in signals {
        signal_hook::low_level::unregister(signal);
    }
    result
}

fn run_loop(
    duration: Duration,
    name: &str,
    policy: OutputPolicy,
    show_fps: bool,
    interrupted: &AtomicBool,
) -> io::Result<()> {
    let mut session = Session::enter(policy)?;
    let mut renderers = Renderers::new();
    let started = Instant::now();
    let mut clock = Clock::new(duration, started);
    let mut frames = FrameSchedule { next: started };
    let mut fps = FrameRate::default();
    let mut dirty = true;
    loop {
        let now = Instant::now();
        if interrupted.load(Ordering::Relaxed) {
            return Ok(());
        }
        if dirty || (!clock.is_paused() && now >= frames.next) {
            let details = clock.details(now);
            let paused = clock.is_paused();
            let fps_label = show_fps.then(|| fps.label());
            execute!(session.terminal.backend_mut(), BeginSynchronizedUpdate)?;
            let draw = session
                .terminal
                .draw(|frame| {
                    let area = frame.area();
                    compose(
                        &mut renderers,
                        name,
                        details,
                        paused,
                        fps_label.as_deref(),
                        area,
                        frame.buffer_mut(),
                    );
                    adapt_buffer(frame.buffer_mut(), policy);
                })
                .map(|_| ());
            // Close the synchronized update even if drawing failed.
            let end = execute!(session.terminal.backend_mut(), EndSynchronizedUpdate);
            draw?;
            end?;
            let completed = Instant::now();
            if show_fps {
                fps.record(completed);
            }
            frames.advance(completed);
            dirty = false;
            if clock.finished(now) {
                return Ok(());
            }
        }
        let now = Instant::now();
        let timeout = if clock.is_paused() {
            Duration::from_millis(100)
        } else {
            frames
                .next
                .saturating_duration_since(now)
                .min(Duration::from_millis(100))
        };
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return Ok(());
                    }
                    if key.code == KeyCode::Char(' ') {
                        clock.toggle_pause(Instant::now());
                        fps.set_paused(clock.is_paused());
                        dirty = true;
                        frames.next = Instant::now();
                    }
                }
                Event::Resize(_, _) => dirty = true,
                _ => {}
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compose(
    renderers: &mut Renderers,
    name: &str,
    details: TimerDetails,
    paused: bool,
    fps: Option<&str>,
    area: Rect,
    buf: &mut Buffer,
) {
    let status = renderers.status(name, details);
    let fps = fps.filter(|label| label.len() <= usize::from(area.width) && area.height > 1);
    let status_width = fps.map_or(area.width, |label| {
        area.width.saturating_sub(label.len() as u16 + 1)
    });
    let show_status =
        area.height > 1 && status_width >= 8 && (details.elapsed < 7.0 || !status.is_empty());
    let show_footer = show_status || fps.is_some();
    let content = Rect {
        height: area.height.saturating_sub(u16::from(show_footer)),
        ..area
    };
    renderers.render(name, details, content, buf);
    if paused {
        paused_overlay(buf, content);
    }
    if show_footer {
        let y = area.bottom() - 1;
        let bg = if details.elapsed < 3.5 {
            234
        } else {
            234 - (((details.elapsed - 3.5) / 3.5) * 3.0).clamp(0.0, 3.0) as u8
        };
        for x in area.x..area.right() {
            buf[(x, y)]
                .set_symbol(" ")
                .set_style(Style::default().bg(Color::Indexed(bg)));
        }
        if show_status {
            let message = if status.is_empty() {
                format!(
                    "Starting a {} timer with the '{name}' renderer. Press <space> to pause.",
                    crate::model::format_time(details.duration as u64)
                )
            } else {
                status
            };
            let clipped: String = message.chars().take(usize::from(status_width)).collect();
            centered_text(buf, Rect::new(area.x, y, status_width, 1), 0, &clipped, 117);
        }
        if let Some(label) = fps {
            let x = area.right() - label.len() as u16;
            crate::graphics::text(
                buf,
                Rect::new(x, y, label.len() as u16, 1),
                0,
                0,
                label,
                250,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_ansi_preserves_controls_and_handles_fragmented_combined_colors() {
        let mut writer = ColorWriter {
            inner: Vec::new(),
            basic: true,
            pending: Vec::new(),
        };
        writer.write_all(b"\x1b[?2026h\x1b[2;3H\x1b[38;").unwrap();
        writer
            .write_all("5;9;48;5;4m▀\x1b[39;49m\x1b[?2026l".as_bytes())
            .unwrap();
        writer.flush().unwrap();
        assert_eq!(
            writer.inner,
            "\x1b[?2026h\x1b[2;3H\x1b[91;44m▀\x1b[39;49m\x1b[?2026l".as_bytes()
        );
        assert!(writer.pending.is_empty());
        for (index, foreground) in [
            30, 31, 32, 33, 34, 35, 36, 37, 90, 91, 92, 93, 94, 95, 96, 97,
        ]
        .into_iter()
        .enumerate()
        {
            let source = format!("\x1b[38;5;{index};48;5;{index}m");
            let expected = format!("\x1b[{foreground};{}m", foreground + 10);
            assert_eq!(basic_ansi(source.as_bytes()), expected.as_bytes());
        }
    }

    #[test]
    fn fps_footer_is_right_aligned_and_keeps_countdown_in_content() {
        for width in [1, 8, 12, 80] {
            for height in [1, 2, 24] {
                for label in ["FPS --", "FPS 30.0", "FPS paused"] {
                    let area = Rect::new(3, 2, width, height);
                    let mut buf = Buffer::empty(Rect::new(0, 0, 90, 30));
                    compose(
                        &mut Renderers::new(),
                        "bigtext",
                        TimerDetails {
                            elapsed: 0.0,
                            duration: 5.0,
                        },
                        false,
                        Some(label),
                        area,
                        &mut buf,
                    );
                    let last: String = (area.x..area.right())
                        .map(|x| buf[(x, area.bottom() - 1)].symbol())
                        .collect();
                    if height > 1 && label.len() <= usize::from(width) {
                        assert!(last.ends_with(label), "{area:?}: {last:?}");
                    } else {
                        assert!(!last.contains("FPS"));
                    }
                    if width == 12 && height <= 2 {
                        let text: String = buf.content.iter().map(|cell| cell.symbol()).collect();
                        assert!(text.contains("0:05"), "{area:?}: {text}");
                    }
                    for y in 0..30 {
                        for x in 0..90 {
                            if !area.contains((x, y).into()) {
                                assert_eq!(buf[(x, y)].symbol(), " ");
                            }
                        }
                    }
                }
            }
        }
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        compose(
            &mut Renderers::new(),
            "sine",
            TimerDetails {
                elapsed: 10.0,
                duration: 60.0,
            },
            true,
            Some("FPS paused"),
            area,
            &mut buf,
        );
        let text: String = buf.content.iter().map(|c| c.symbol()).collect();
        assert!(text.contains("PAUSED") && text.contains("FPS paused"));
    }

    #[test]
    fn small_terminal_keeps_readable_countdown_alongside_status() {
        for height in [1, 4] {
            let area = Rect::new(0, 0, 12, height);
            let mut buffer = Buffer::empty(area);
            compose(
                &mut Renderers::new(),
                "bigtext",
                TimerDetails {
                    elapsed: 0.0,
                    duration: 5.0,
                },
                false,
                None,
                area,
                &mut buffer,
            );
            let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
            assert!(text.contains("0:05"), "{height} rows: {text}");
        }
    }
}
