use crate::{
    color::{OutputPolicy, adapt_buffer},
    graphics::centered_text,
    model::{Clock, FrameRate, FrameSchedule, TimerDetails},
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
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};
use std::{
    io::{self, IsTerminal, Stdout},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

struct Session {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Session {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let result = (|| {
            let mut out = io::stdout();
            execute!(out, EnterAlternateScreen, Hide)?;
            Terminal::new(CrosstermBackend::new(out)).map(|terminal| Self { terminal })
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
    let mut session = Session::enter()?;
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
