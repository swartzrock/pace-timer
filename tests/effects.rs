use pace::{effects::Effects, model::TimerDetails};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

const EFFECTS: [&str; 8] = [
    "bar",
    "pie",
    "dots",
    "sweep",
    "colorwheel",
    "circles",
    "sine",
    "lava",
];

fn draw(effect: &mut Effects, name: &str, elapsed: f64, area: Rect) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 40));
    effect.draw(
        name,
        TimerDetails {
            elapsed,
            duration: 60.0,
        },
        area,
        &mut buf,
    );
    buf
}

#[test]
fn all_geometry_effects_produce_visible_content() {
    let mut effects = Effects::new();
    for name in EFFECTS {
        let buf = draw(&mut effects, name, 15.0, Rect::new(0, 0, 80, 30));
        assert!(
            buf.content.iter().any(|cell| cell.symbol() != " "),
            "{name}"
        );
    }
}

#[test]
fn effects_clip_to_tiny_and_offset_viewports() {
    let mut effects = Effects::new();
    for name in EFFECTS {
        for area in [
            Rect::new(9, 7, 1, 1),
            Rect::new(9, 7, 0, 0),
            Rect::new(9, 7, 14, 7),
        ] {
            let buf = draw(&mut effects, name, 15.0, area);
            for y in 0..40 {
                for x in 0..100 {
                    if !area.contains((x, y).into()) {
                        assert_eq!(buf[(x, y)].symbol(), " ", "{name} escaped {area:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn intermediate_frames_do_not_change_animation_phase_or_random_choices() {
    let mut effects = Effects::new();
    let area = Rect::new(5, 3, 80, 30);
    for name in EFFECTS {
        let expected = draw(&mut effects, name, 21.25, area);
        for elapsed in [0.0, 0.1, 4.99, 19.9, 20.0, 20.1, 21.0] {
            draw(&mut effects, name, elapsed, area);
        }
        assert_eq!(expected, draw(&mut effects, name, 21.25, area), "{name}");
        assert_eq!(
            expected,
            draw(&mut effects, name, 21.25, area),
            "paused {name}"
        );
    }
}

#[test]
fn pie_is_a_strict_radius_disk_with_doubled_columns() {
    let mut effects = Effects::new();
    let buf = draw(&mut effects, "pie", 30.0, Rect::new(0, 0, 40, 20));
    // radius = floor(min(20,40/2)/2)-2 = 8; chart is 34 by 17.
    assert_eq!(buf[(3, 1)].symbol(), " ");
    assert_eq!(buf[(19, 9)].symbol(), "█");
    for y in 1..18 {
        for x in (3..37).step_by(2) {
            assert_eq!(buf[(x, y)], buf[(x + 1, y)]);
        }
    }
}

#[test]
fn source_symbols_and_progress_colors_are_preserved() {
    let mut effects = Effects::new();
    let area = Rect::new(0, 0, 80, 30);
    for (elapsed, color) in [(30.0, 46), (48.0, 154), (57.0, 196)] {
        let bar = draw(&mut effects, "bar", elapsed, area);
        assert!(
            bar.content
                .iter()
                .any(|cell| cell.symbol() == "█" && cell.fg == Color::Indexed(color))
        );
        assert!(
            bar.content
                .iter()
                .any(|cell| cell.symbol() == "░" && cell.fg == Color::Indexed(239))
        );
        let dots = draw(&mut effects, "dots", elapsed, area);
        assert!(
            dots.content
                .iter()
                .any(|cell| cell.symbol() == "●" && cell.fg == Color::Indexed(color))
        );
        assert!(
            dots.content
                .iter()
                .any(|cell| cell.symbol() == "●" && cell.fg == Color::Indexed(232))
        );
        assert!(dots.content.iter().any(|cell| cell.symbol() == "╔"));
    }
}

#[test]
fn sine_retains_a_four_second_period_at_sixty_fps() {
    let mut effects = Effects::new();
    let mut a = Buffer::empty(Rect::new(0, 0, 80, 30));
    let mut b = a.clone();
    // Keep progress fixed at 50% while advancing the wave by its original 40 ticks.
    effects.draw(
        "sine",
        TimerDetails {
            elapsed: 10.025,
            duration: 20.05,
        },
        a.area,
        &mut a,
    );
    effects.draw(
        "sine",
        TimerDetails {
            elapsed: 14.025,
            duration: 28.05,
        },
        b.area,
        &mut b,
    );
    assert_eq!(a, b);
}

#[test]
fn sine_moves_between_frames_less_than_a_legacy_tick_apart() {
    let mut effects = Effects::new();
    let area = Rect::new(0, 0, 80, 32);
    let mut a = Buffer::empty(area);
    let mut b = Buffer::empty(area);
    effects.draw(
        "sine",
        TimerDetails {
            elapsed: 4.0,
            duration: 8.0,
        },
        area,
        &mut a,
    );
    effects.draw(
        "sine",
        TimerDetails {
            elapsed: 4.05,
            duration: 8.1,
        },
        area,
        &mut b,
    );
    // The 17-row field starts at y=7; this cell is one row above its origin.
    assert_eq!(a[(0, 14)].symbol(), " ");
    assert_eq!(b[(0, 14)].symbol(), "•");
}
