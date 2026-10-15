use std::time::{Duration, Instant};

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
        format_time((self.duration - self.elapsed).max(0.0).ceil() as u64)
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
