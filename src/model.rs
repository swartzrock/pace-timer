use std::time::{Duration, Instant};

pub const RENDERER_NAMES: &[&str] = &[
    "aurora",
    "bar",
    "bigtext",
    "circles",
    "colorwheel",
    "colossal",
    "dots",
    "lava",
    "pie",
    "shuffle",
    "sine",
    "slant",
    "sweep",
];

pub const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// A late frame is skipped; it never becomes a burst of catch-up renders.
pub struct FrameSchedule {
    pub next: Instant,
}

impl FrameSchedule {
    pub fn advance(&mut self, now: Instant) {
        self.next += FRAME_INTERVAL;
        if self.next <= now {
            self.next = now + FRAME_INTERVAL;
        }
    }
}

/// Animation phase and countdown progress use active elapsed time, never frame counts.
#[derive(Clone, Copy, Debug)]
pub struct TimerDetails {
    pub elapsed: f64,
    pub duration: f64,
}

impl TimerDetails {
    pub fn progress(self) -> f64 {
        if self.duration == 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    pub fn remaining_text(self) -> String {
        format_time((self.duration - self.elapsed).max(0.0).floor() as u64)
    }

    pub fn elapsed_text(self) -> String {
        format_time(self.elapsed.min(self.duration).floor() as u64)
    }

    /// Continuous equivalent of the original 10 Hz animation counter.
    pub fn legacy_tick(self) -> f64 {
        self.elapsed * 10.0
    }
}

pub fn format_time(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

pub struct Clock {
    duration: Duration,
    started: Instant,
    paused_at: Option<Instant>,
    paused_total: Duration,
}

impl Clock {
    pub fn new(duration: Duration, started: Instant) -> Self {
        Self {
            duration,
            started,
            paused_at: None,
            paused_total: Duration::ZERO,
        }
    }

    pub fn toggle_pause(&mut self, now: Instant) {
        if let Some(at) = self.paused_at.take() {
            self.paused_total += now.saturating_duration_since(at);
        } else {
            self.paused_at = Some(now);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        self.paused_at
            .unwrap_or(now)
            .saturating_duration_since(self.started)
            .saturating_sub(self.paused_total)
    }

    pub fn details(&self, now: Instant) -> TimerDetails {
        TimerDetails {
            elapsed: self.elapsed(now).min(self.duration).as_secs_f64(),
            duration: self.duration.as_secs_f64(),
        }
    }

    pub fn finished(&self, now: Instant) -> bool {
        self.elapsed(now) >= self.duration
    }
}
/// Successful draw completions, including scheduled waiting and terminal I/O.
#[derive(Default)]
pub struct FrameRate {
    completed: std::collections::VecDeque<Instant>,
    paused: bool,
}

impl FrameRate {
    pub fn record(&mut self, now: Instant) {
        if self.paused
            || self
                .completed
                .back()
                .is_some_and(|previous| now <= *previous)
        {
            return;
        }
        self.completed.push_back(now);
        // Keep the interval straddling the window, even when a draw took over a second.
        while self.completed.len() > 2
            && (now.duration_since(self.completed[1]) >= Duration::from_secs(1)
                || self.completed.len() > 256)
        {
            self.completed.pop_front();
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        if self.paused != paused {
            self.completed.clear();
            self.paused = paused;
        }
    }

    pub fn rate(&self) -> Option<f64> {
        if self.paused || self.completed.len() < 2 {
            return None;
        }
        let span = self
            .completed
            .back()?
            .duration_since(*self.completed.front()?)
            .as_secs_f64();
        Some((self.completed.len() - 1) as f64 / span)
    }

    pub fn label(&self) -> String {
        if self.paused {
            "FPS paused".into()
        } else if let Some(rate) = self.rate() {
            format!("FPS {rate:.1}")
        } else {
            "FPS --".into()
        }
    }
}
