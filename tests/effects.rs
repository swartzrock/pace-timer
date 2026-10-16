use pace::{effects::Effects, model::TimerDetails};
use ratatui::{buffer::Buffer, layout::Rect};

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

fn pixels(buf: &Buffer) -> impl Iterator<Item = &ratatui::buffer::Cell> {
    buf.content
        .iter()
        .filter(|cell| matches!(cell.symbol(), "▀" | "▄"))
}

#[test]
fn disks_use_independent_half_samples_and_soft_contours() {
    let mut effects = Effects::new();
    for name in ["pie", "sweep", "colorwheel"] {
        let buf = draw(&mut effects, name, 30.0, Rect::new(0, 0, 40, 20));
        assert!(pixels(&buf).any(|cell| cell.fg != cell.bg), "{name}");
        assert_eq!(buf[(0, 0)].symbol(), " ");
        assert!(pixels(&buf).count() > 40, "{name} disk disappeared");
    }
}

#[test]
fn wheel_moves_before_the_next_legacy_tick() {
    let mut effects = Effects::new();
    let area = Rect::new(0, 0, 80, 30);
    assert_ne!(
        draw(&mut effects, "colorwheel", 4.0, area),
        draw(&mut effects, "colorwheel", 4.016, area)
    );
}

#[test]
fn progress_segments_keep_exact_endpoints_and_fractional_coverage() {
    let mut effects = Effects::new();
    let area = Rect::new(0, 0, 80, 30);
    let empty = draw(&mut effects, "bar", 0.0, area);
    assert_eq!(
        empty
            .content
            .iter()
            .filter(|cell| cell.symbol() == "░")
            .count(),
        61
    );
    let full = draw(&mut effects, "bar", 60.0, area);
    assert_eq!(
        full.content
            .iter()
            .filter(|cell| cell.symbol() == "█")
            .count(),
        61
    );
    let mut widths = Vec::new();
    for fraction in [0.25, 0.5, 0.75] {
        let buf = draw(&mut effects, "bar", fraction * 60.0 / 61.0, area);
        let partial = buf
            .content
            .iter()
            .find(|cell| matches!(cell.symbol(), "▏" | "▎" | "▍" | "▌" | "▋" | "▊" | "▉"))
            .expect("fractional bar cell");
        widths.push(
            "▏▎▍▌▋▊▉"
                .chars()
                .position(|ch| partial.symbol().starts_with(ch))
                .unwrap(),
        );
    }
    assert!(widths.windows(2).all(|pair| pair[0] < pair[1]));
    let empty = draw(&mut effects, "dots", 0.0, area);
    let full = draw(&mut effects, "dots", 60.0, area);
    assert!(empty.content.iter().any(|cell| cell.symbol() == "○"));
    assert!(!full.content.iter().any(|cell| cell.symbol() == "○"));
    let total = empty
        .content
        .iter()
        .filter(|cell| cell.symbol() == "○")
        .count();
    assert_eq!(
        full.content
            .iter()
            .filter(|cell| cell.symbol() == "●")
            .count(),
        total
    );
    let mut shades = Vec::new();
    for fraction in [0.25, 0.5, 0.75] {
        let mut buf = draw(&mut effects, "dots", fraction * 60.0 / total as f64, area);
        pace::color::adapt_buffer(&mut buf, pace::color::OutputPolicy::Monochrome);
        let partial = buf
            .content
            .iter()
            .find(|cell| matches!(cell.symbol(), "░" | "▒" | "▓"))
            .expect("fractional dot cell");
        shades.push(
            ["░", "▒", "▓"]
                .iter()
                .position(|symbol| *symbol == partial.symbol())
                .unwrap(),
        );
    }
    assert!(shades.windows(2).all(|pair| pair[0] < pair[1]));
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
fn sine_has_shaded_subcell_motion_and_elapsed_derived_trails() {
    let mut effects = Effects::new();
    let area = Rect::new(0, 0, 80, 32);
    let a = draw(&mut effects, "sine", 4.0, area);
    let b = draw(&mut effects, "sine", 4.016, area);
    assert_ne!(a, b);
    assert!(pixels(&a).any(|cell| cell.fg != cell.bg));
    let colors: std::collections::HashSet<_> =
        pixels(&a).flat_map(|cell| [cell.fg, cell.bg]).collect();
    assert!(
        colors.len() > 12,
        "wave needs shaded edges and faint trail samples"
    );
}

#[test]
fn lava_and_circles_are_continuous_at_generation_boundaries() {
    let mut effects = Effects::new();
    let area = Rect::new(0, 0, 80, 30);
    for name in ["lava", "circles"] {
        let before = draw(&mut effects, name, 19.9999, area);
        let after = draw(&mut effects, name, 20.0001, area);
        let difference: u64 = before
            .content
            .iter()
            .zip(&after.content)
            .map(|(a, b)| {
                [a.fg, a.bg]
                    .into_iter()
                    .zip([b.fg, b.bg])
                    .map(|(a, b)| {
                        let (ar, ag, ab) = pace::color::color_rgb(a).unwrap_or_default();
                        let (br, bg, bb) = pace::color::color_rgb(b).unwrap_or_default();
                        u64::from(ar.abs_diff(br))
                            + u64::from(ag.abs_diff(bg))
                            + u64::from(ab.abs_diff(bb))
                    })
                    .sum::<u64>()
            })
            .sum();
        assert!(
            difference < 4000,
            "{name} generation jumped by {difference}"
        );
    }
}

#[test]
fn disks_preserve_empty_full_states_and_sweep_hand_moves_smoothly() {
    let mut effects = Effects::new();
    let area = Rect::new(0, 0, 40, 20);
    let empty = draw(&mut effects, "pie", 0.0, area);
    let full = draw(&mut effects, "pie", 60.0, area);
    assert_eq!(empty[(10, 10)].fg, pace::color::indexed_rgb(233));
    assert_eq!(full[(10, 10)].fg, pace::color::indexed_rgb(196));
    assert!(
        pixels(&full).any(|cell| matches!(cell.fg, ratatui::style::Color::Rgb(1..=254, 0, 0))),
        "pie contour needs partial coverage"
    );
    assert!(pixels(&draw(&mut effects, "sweep", 0.0, area)).count() > 0);
    assert_eq!(pixels(&draw(&mut effects, "sweep", 60.0, area)).count(), 0);
    let a = draw(&mut effects, "sweep", 30.0, area);
    let b = draw(&mut effects, "sweep", 30.016, area);
    assert!(a != b, "sweep hand must move within a cell");
}

#[test]
fn circle_lighting_and_wave_trail_include_faint_samples() {
    let mut effects = Effects::new();
    let area = Rect::new(0, 0, 80, 32);
    let circles = draw(&mut effects, "circles", 5.0, area);
    let colors: std::collections::HashSet<_> = pixels(&circles)
        .flat_map(|cell| [cell.fg, cell.bg])
        .collect();
    assert!(colors.len() > 30, "circle edges and lighting need shades");
    let wave = draw(&mut effects, "sine", 4.0, area);
    // At x=0 the current wave meets its origin; only the earlier phase reaches here.
    let (r, g, b) = pace::color::color_rgb(wave[(0, 17)].fg).unwrap();
    assert!(r.max(g).max(b) > 0 && r.max(g).max(b) < 40);
}

#[test]
fn resize_and_output_adaptation_preserve_repeatability_and_visible_shapes() {
    let mut effects = Effects::new();
    let area = Rect::new(5, 3, 80, 30);
    for name in EFFECTS {
        let expected = draw(&mut effects, name, 15.0, area);
        draw(&mut effects, name, 15.0, Rect::new(9, 7, 14, 7));
        assert_eq!(
            expected,
            draw(&mut effects, name, 15.0, area),
            "resized {name}"
        );
        for policy in [
            pace::color::OutputPolicy::Ansi256,
            pace::color::OutputPolicy::Ansi16,
            pace::color::OutputPolicy::Monochrome,
        ] {
            let mut adapted = expected.clone();
            pace::color::adapt_buffer(&mut adapted, policy);
            assert!(
                adapted.content.iter().any(|cell| cell.symbol() != " "),
                "{name} {policy:?}"
            );
            if policy == pace::color::OutputPolicy::Monochrome {
                assert!(
                    adapted
                        .content
                        .iter()
                        .all(|cell| cell.fg == ratatui::style::Color::Reset
                            && cell.bg == ratatui::style::Color::Reset)
                );
            }
        }
    }
}
