use pace::model::{Clock, TimerDetails};
use std::time::{Duration, Instant};

#[test]
fn countdown_uses_elapsed_time_rather_than_rendered_frames() {
    let start = Instant::now();
    let clock = Clock::new(Duration::from_secs(10), start);
    let details = clock.details(start + Duration::from_millis(1750));
    assert_eq!(details.remaining_text(), "0:09");
    assert_eq!(details.elapsed_text(), "0:01");
    assert!((details.progress() - 0.175).abs() < 0.0001);
    assert!(clock.finished(start + Duration::from_secs(11)));
}

#[test]
fn repeated_pauses_exclude_only_paused_time() {
    let start = Instant::now();
    let mut clock = Clock::new(Duration::from_secs(10), start);
    clock.toggle_pause(start + Duration::from_secs(2));
    assert!(clock.is_paused());
    assert_eq!(clock.details(start + Duration::from_secs(9)).elapsed, 2.0);
    clock.toggle_pause(start + Duration::from_secs(9));
    assert!(!clock.is_paused());
    clock.toggle_pause(start + Duration::from_secs(10));
    clock.toggle_pause(start + Duration::from_secs(20));
    assert_eq!(clock.details(start + Duration::from_secs(21)).elapsed, 4.0);
    assert!(!clock.finished(start + Duration::from_secs(26) - Duration::from_millis(1)));
    assert!(clock.finished(start + Duration::from_secs(27)));
}

#[test]
fn time_display_and_progress_clamp_at_completion() {
    let details = TimerDetails {
        elapsed: 90.0,
        duration: 80.0,
    };
    assert_eq!(details.remaining_text(), "0:00");
    assert_eq!(details.progress(), 1.0);
    assert_eq!(
        TimerDetails {
            elapsed: 0.0,
            duration: 0.0
        }
        .progress(),
        1.0
    );
}
