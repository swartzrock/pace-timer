use crate::{
    graphics::centered_text,
    model::{Clock, FrameSchedule, TimerDetails},
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

pub fn run(duration: Duration, name: &str) -> io::Result<()> {
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
    let result = run_loop(duration, name, &interrupted);
    #[cfg(unix)]
    for signal in signals {
        signal_hook::low_level::unregister(signal);
    }
    result
}

fn run_loop(duration: Duration, name: &str, interrupted: &AtomicBool) -> io::Result<()> {
    let mut session = Session::enter()?;
    let mut renderers = Renderers::new();
    let started = Instant::now();
    let mut clock = Clock::new(duration, started);
    let mut frames = FrameSchedule { next: started };
    let mut dirty = true;
    loop {
        let now = Instant::now();
        if interrupted.load(Ordering::Relaxed) {
            return Ok(());
        }
        if dirty || (!clock.is_paused() && now >= frames.next) {
            let details = clock.details(now);
            let paused = clock.is_paused();
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
                        area,
                        frame.buffer_mut(),
                    );
                })
                .map(|_| ());
            // Close the synchronized update even if drawing failed.
            let end = execute!(session.terminal.backend_mut(), EndSynchronizedUpdate);
            draw?;
            end?;
            frames.advance(Instant::now());
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

fn compose(
    renderers: &mut Renderers,
    name: &str,
    details: TimerDetails,
    paused: bool,
    area: Rect,
    buf: &mut Buffer,
) {
    let status = renderers.status(name, details);
    let show_status = area.height > 1 && (details.elapsed < 7.0 || !status.is_empty());
    let content = Rect {
        height: area.height.saturating_sub(u16::from(show_status)),
        ..area
    };
    renderers.render(name, details, content, buf);
    if show_status {
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
        let message = if status.is_empty() {
            format!(
                "Starting a {} timer with the '{name}' renderer. Press <space> to pause.",
                crate::model::format_time(details.duration as u64)
            )
        } else {
            status
        };
        let clipped: String = message.chars().take(usize::from(area.width)).collect();
        centered_text(buf, Rect::new(area.x, y, area.width, 1), 0, &clipped, 117);
    }
    if paused {
        paused_overlay(buf, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                area,
                &mut buffer,
            );
            let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
            assert!(text.contains("0:05"), "{height} rows: {text}");
        }
    }
}
