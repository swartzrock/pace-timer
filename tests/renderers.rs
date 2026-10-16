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
    use pace::color::{OutputPolicy, adapt_buffer};
    use std::{hint::black_box, time::Instant};
    let mut renderers = Renderers::new();
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));
    for name in RENDERER_NAMES {
        let initial = Instant::now();
        renderers.render(
            name,
            TimerDetails {
                elapsed: 0.0,
                duration: 60.0,
            },
            buf.area,
            &mut buf,
        );
        let initial_ms = initial.elapsed().as_secs_f64() * 1000.0;
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
            "{name}: {:.3} ms/frame (120x40, 600 frames, warmed; first render {initial_ms:.3} ms)",
            started.elapsed().as_secs_f64() * 1000.0 / 600.0
        );
        for policy in [
            OutputPolicy::Truecolor,
            OutputPolicy::Ansi256,
            OutputPolicy::Ansi16,
            OutputPolicy::Monochrome,
        ] {
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
                adapt_buffer(&mut buf, policy);
                black_box(&buf);
            }
            println!(
                "  {policy:?} including adaptation: {:.3} ms/frame",
                started.elapsed().as_secs_f64() * 1000.0 / 600.0
            );
        }
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
fn circle_and_sine_shadows_are_soft_dark_layers_behind_crisp_glyphs() {
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
            shadows
                .iter()
                .all(|cell| matches!(cell.fg, Color::Rgb(r, g, b) if r < 90 && g < 90 && b < 90)),
            "{name}"
        );
    }
}

#[test]
fn text_gradients_are_rgb_and_preserve_masks_between_sub_tick_frames() {
    use ratatui::style::Color;
    use std::collections::HashSet;
    let mut renderers = Renderers::new();
    let area = Rect::new(3, 2, 120, 40);
    for name in ["bigtext", "colossal", "slant"] {
        let mut frames = Vec::new();
        for elapsed in [12.01, 12.02] {
            let mut buf = Buffer::empty(area);
            renderers.render(
                name,
                TimerDetails {
                    elapsed,
                    duration: 60.0,
                },
                area,
                &mut buf,
            );
            frames.push(buf);
        }
        assert!(
            frames[0]
                .content
                .iter()
                .zip(&frames[1].content)
                .all(|(a, b)| a.symbol() == b.symbol()),
            "{name} masks moved"
        );
        let colors: HashSet<_> = frames[0]
            .content
            .iter()
            .filter(|c| c.symbol() != " ")
            .map(|c| c.fg)
            .collect();
        assert!(
            colors.iter().all(|color| matches!(color, Color::Rgb(..))),
            "{name} must stay RGB"
        );
        assert!(
            colors.len() > 12,
            "{name} needs continuous shading, got {} colors",
            colors.len()
        );
        if name == "colossal" {
            assert_ne!(
                frames[0], frames[1],
                "colossal gradient should move below 100ms"
            );
        }
    }
}

#[test]
fn shuffle_feathers_colors_with_valid_symbols_and_exact_endpoints() {
    let area = Rect::new(3, 2, 120, 40);
    let mut renderers = Renderers::new();
    let renderer_at = |renderers: &Renderers, elapsed| {
        renderers
            .status(
                "shuffle",
                TimerDetails {
                    elapsed,
                    duration: 600.0,
                },
            )
            .split('"')
            .nth(1)
            .unwrap()
            .to_owned()
    };
    let mut blended_cells = 0;
    for slot in 0..11 {
        let start = f64::from(slot) * 10.0;
        let current = renderer_at(&renderers, start);
        let next = renderer_at(&renderers, start + 10.0);
        for fraction in [0.0, 0.5, 0.75, 1.0] {
            let details = TimerDetails {
                elapsed: start + 7.5 + fraction * 2.5,
                duration: 600.0,
            };
            let mut actual = Buffer::empty(area);
            let mut a = actual.clone();
            let mut b = actual.clone();
            renderers.render("shuffle", details, area, &mut actual);
            renderers.render(&current, details, area, &mut a);
            renderers.render(&next, details, area, &mut b);
            if fraction == 0.0 {
                assert_eq!(actual, a);
            } else if fraction == 1.0 {
                assert_eq!(actual, b);
            } else {
                let mut paused = Buffer::empty(area);
                renderers.render("shuffle", details, area, &mut paused);
                assert_eq!(actual, paused, "paused shuffle changed");
                for ((cell, a), b) in actual.content.iter().zip(&a.content).zip(&b.content) {
                    assert!(
                        cell.symbol() == a.symbol() || cell.symbol() == b.symbol(),
                        "invented transition glyph {} for {current}->{next}",
                        cell.symbol()
                    );
                    blended_cells += usize::from(
                        (cell.fg != a.fg && cell.fg != b.fg)
                            || (cell.bg != a.bg && cell.bg != b.bg),
                    );
                }
            }
        }
    }
    assert!(
        blended_cells > usize::from(area.height),
        "transition needs a color feather wider than one column"
    );
}
