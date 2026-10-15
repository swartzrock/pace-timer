use pace::model::{Clock, TimerDetails};
use std::time::{Duration, Instant};

#[test]
fn countdown_uses_elapsed_time_rather_than_rendered_frames() {
    let start = Instant::now();
    let clock = Clock::new(Duration::from_secs(10), start);
    let details = clock.details(start + Duration::from_millis(1750));
    assert_eq!(details.remaining_text(), "0:08");
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

#[test]
fn late_frames_are_skipped_instead_of_replayed() {
    use pace::model::{FRAME_INTERVAL, FrameSchedule};
    let now = Instant::now();
    let mut schedule = FrameSchedule { next: now };
    schedule.advance(now + Duration::from_millis(90));
    assert_eq!(
        schedule.next,
        now + Duration::from_millis(90) + FRAME_INTERVAL
    );
    let next = schedule.next;
    schedule.advance(next);
    assert_eq!(schedule.next, next + FRAME_INTERVAL);
}

#[test]
fn remaining_seconds_round_down_as_in_the_original_cli() {
    assert_eq!(
        TimerDetails {
            elapsed: 0.1,
            duration: 2.0
        }
        .remaining_text(),
        "0:01"
    );
    assert_eq!(
        TimerDetails {
            elapsed: 1.1,
            duration: 2.0
        }
        .remaining_text(),
        "0:00"
    );
}
#[test]
fn fps_measures_completed_intervals_including_slow_frames() {
    use pace::model::FrameRate;
    let start = std::time::Instant::now();
    let mut fps = FrameRate::default();
    assert_eq!(fps.label(), "FPS --");
    fps.record(start);
    assert_eq!(fps.label(), "FPS --");
    for frame in 1..=60 {
        fps.record(start + std::time::Duration::from_secs_f64(frame as f64 / 60.0));
    }
    assert!((fps.rate().unwrap() - 60.0).abs() < 0.01);
    fps.record(start + std::time::Duration::from_secs(3));
    assert!((fps.rate().unwrap() - 0.5).abs() < 0.001);
    fps.set_paused(true);
    fps.record(start + std::time::Duration::from_secs(4));
    assert_eq!(fps.label(), "FPS paused");
    fps.set_paused(false);
    assert_eq!(fps.label(), "FPS --");
    fps.record(start + std::time::Duration::from_secs(5));
    fps.record(start + std::time::Duration::from_secs(5));
    assert_eq!(fps.label(), "FPS --");
    fps.record(start + std::time::Duration::from_secs_f64(5.0 + 1.0 / 30.0));
    assert!((fps.rate().unwrap() - 30.0).abs() < 0.01);
}
