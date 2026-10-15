use pace::{
    model::{RENDERER_NAMES, TimerDetails},
    renderers::{Renderers, paused_overlay},
};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect};

#[test]
fn every_renderer_handles_completion_resizes_and_offset_viewports() {
    let mut renderers = Renderers::new();
    for name in RENDERER_NAMES {
        for (width, height) in [(0, 0), (1, 1), (12, 4), (80, 24), (200, 60)] {
            let area = Rect::new(3, 2, width, height);
            let mut buf = Buffer::empty(Rect::new(0, 0, 210, 70));
            for elapsed in [0.0, 7.6, 9.99, 10.0, 19.9, 20.0, 59.99, 60.0] {
                buf.reset();
                renderers.render(
                    name,
                    TimerDetails {
                        elapsed,
                        duration: 60.0,
                    },
                    area,
                    &mut buf,
                );
                for y in 0..70 {
                    for x in 0..210 {
                        if !area.contains((x, y).into()) {
                            assert_eq!(buf[(x, y)].symbol(), " ", "{name} escaped {area:?}");
                        }
                    }
                }
                if width >= 80 {
                    assert!(buf.content.iter().any(|c| c.symbol() != " "), "{name}");
                }
            }
        }
    }
}

#[test]
fn pause_preserves_the_rendered_animation_and_adds_a_visible_overlay() {
    let mut renderers = Renderers::new();
    let mut terminal = Terminal::new(TestBackend::new(100, 35)).unwrap();
    let details = TimerDetails {
        elapsed: 12.345,
        duration: 60.0,
    };
    terminal
        .draw(|frame| {
            let area = frame.area();
            renderers.render("circles", details, area, frame.buffer_mut());
        })
        .unwrap();
    let before = terminal.backend().buffer().clone();
    terminal
        .draw(|frame| {
            let area = frame.area();
            renderers.render("circles", details, area, frame.buffer_mut());
        })
        .unwrap();
    assert_eq!(&before, terminal.backend().buffer());
    terminal
        .draw(|frame| {
            let area = frame.area();
            renderers.render("circles", details, area, frame.buffer_mut());
            paused_overlay(frame.buffer_mut(), area);
        })
        .unwrap();
    let paused = terminal.backend().buffer();
    let text: String = paused.content.iter().map(|cell| cell.symbol()).collect();
    assert!(text.contains("PAUSED"));
}

#[test]
fn shuffle_uses_ten_second_slots_and_a_two_and_a_half_second_transition() {
    let renderers = Renderers::new();
    let status = |elapsed| {
        renderers.status(
            "shuffle",
            TimerDetails {
                elapsed,
                duration: 60.0,
            },
        )
    };
    assert!(status(0.0).contains("10 more seconds"));
    assert!(status(7.5).contains("showing"));
    assert!(status(8.0).contains("loading"));
    assert!(status(10.0).contains("10 more seconds"));
    assert_ne!(status(0.0), status(10.0));
}

#[test]
#[ignore = "release-mode measurement, run with --release -- --ignored --nocapture"]
fn measure_frame_preparation() {
    use std::{hint::black_box, time::Instant};
    let mut renderers = Renderers::new();
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));
    for name in RENDERER_NAMES {
        renderers.render(
            name,
            TimerDetails {
                elapsed: 0.0,
                duration: 60.0,
            },
            buf.area,
            &mut buf,
        );
        let started = Instant::now();
        for frame in 0..600 {
            buf.reset();
            renderers.render(
                name,
                TimerDetails {
                    elapsed: frame as f64 / 60.0,
                    duration: 60.0,
                },
                buf.area,
                &mut buf,
            );
            black_box(&buf);
        }
        println!(
            "{name}: {:.3} ms/frame (120x40, 600 frames, warmed)",
            started.elapsed().as_secs_f64() * 1000.0 / 600.0
        );
    }
}

#[test]
fn bigtext_glyph_height_stays_stable_when_digits_change() {
    let mut renderers = Renderers::new();
    let area = Rect::new(0, 0, 100, 40);
    let heights: Vec<usize> = [11.0, 10.0]
        .into_iter()
        .map(|duration| {
            let mut buf = Buffer::empty(area);
            renderers.render(
                "bigtext",
                TimerDetails {
                    elapsed: 0.0,
                    duration,
                },
                area,
                &mut buf,
            );
            (0..area.height)
                .filter(|y| (0..area.width).any(|x| buf[(x, *y)].symbol() == "█"))
                .count()
        })
        .collect();
    assert_eq!(heights, vec![10, 10]);
}

#[test]
fn circle_and_sine_shadows_keep_the_original_black_foreground() {
    use ratatui::style::Color;
    let mut renderers = Renderers::new();
    let area = Rect::new(0, 0, 100, 35);
    for name in ["circles", "sine"] {
        let mut buf = Buffer::empty(area);
        renderers.render(
            name,
            TimerDetails {
                elapsed: 12.345,
                duration: 60.0,
            },
            area,
            &mut buf,
        );
        let shadows: Vec<_> = buf
            .content
            .iter()
            .filter(|cell| cell.symbol() == "⬤")
            .collect();
        assert!(!shadows.is_empty());
        assert!(
            shadows.iter().all(|cell| cell.fg == Color::Indexed(16)),
            "{name}"
        );
    }
}

#[test]
fn shuffle_wipe_composes_both_renderers_at_the_elapsed_split() {
    let area = Rect::new(0, 0, 120, 40);
    let mut renderers = Renderers::new();
    let status = renderers.status(
        "shuffle",
        TimerDetails {
            elapsed: 0.0,
            duration: 60.0,
        },
    );
    let current = status.split('"').nth(1).unwrap().to_owned();
    let status = renderers.status(
        "shuffle",
        TimerDetails {
            elapsed: 10.0,
            duration: 60.0,
        },
    );
    let next = status.split('"').nth(1).unwrap().to_owned();
    for elapsed in [7.5, 8.75, 9.375, 10.0] {
        let details = TimerDetails {
            elapsed,
            duration: 60.0,
        };
        let mut actual = Buffer::empty(area);
        let mut a = actual.clone();
        let mut b = actual.clone();
        renderers.render("shuffle", details, area, &mut actual);
        renderers.render(&current, details, area, &mut a);
        renderers.render(&next, details, area, &mut b);
        if elapsed == 7.5 {
            assert_eq!(actual, a);
        } else if elapsed == 10.0 {
            assert_eq!(actual, b);
        } else {
            let split = if elapsed == 8.75 { 60 } else { 90 };
            for y in 0..area.height {
                for x in 0..area.width {
                    if x < split {
                        assert_eq!(actual[(x, y)], b[(x, y)]);
                    } else if x == split {
                        assert_eq!(actual[(x, y)].symbol(), "░");
                    } else {
                        assert_eq!(actual[(x, y)], a[(x, y)]);
                    }
                }
            }
        }
    }
}
