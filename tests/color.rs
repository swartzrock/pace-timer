use pace::color::{
    ColorDepth, ColorEnvironment, OutputOptions, OutputPolicy, adapt_buffer, color_rgb, indexed_rgb,
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

fn environment(
    term: Option<&str>,
    colorterm: Option<&str>,
    no_color: Option<&str>,
) -> ColorEnvironment {
    ColorEnvironment {
        term: term.map(String::from),
        colorterm: colorterm.map(String::from),
        no_color: no_color.map(String::from),
    }
}

#[test]
fn captured_policy_precedence_and_capabilities() {
    use OutputPolicy::*;
    let defaults = OutputOptions::default();
    for (term, colorterm, no_color, tty, expected) in [
        (
            Some("xterm-256color"),
            Some("truecolor"),
            None,
            true,
            Truecolor,
        ),
        (Some("xterm-256color"), Some("24bit"), None, true, Truecolor),
        (Some("xterm-direct"), None, None, true, Truecolor),
        (Some("xterm-256color"), None, None, true, Ansi256),
        (Some("screen-256color"), None, None, true, Ansi256),
        (Some("tmux"), None, None, true, Ansi16),
        (Some("xterm"), None, None, true, Ansi16),
        (Some("dumb"), Some("truecolor"), None, true, Monochrome),
        (None, None, None, true, Monochrome),
        (Some("unknown"), None, None, true, Monochrome),
        (
            Some("xterm-256color"),
            Some("truecolor"),
            Some(""),
            true,
            Truecolor,
        ),
        (
            Some("xterm-256color"),
            Some("truecolor"),
            Some("1"),
            true,
            Monochrome,
        ),
        (
            Some("xterm-256color"),
            Some("truecolor"),
            None,
            false,
            Monochrome,
        ),
    ] {
        assert_eq!(
            OutputPolicy::resolve(defaults, tty, &environment(term, colorterm, no_color)),
            expected,
            "{term:?} {colorterm:?} {no_color:?} tty={tty}"
        );
    }
    let hostile = environment(Some("dumb"), None, Some("1"));
    for (depth, expected) in [
        (ColorDepth::Truecolor, Truecolor),
        (ColorDepth::Ansi256, Ansi256),
        (ColorDepth::Ansi16, Ansi16),
    ] {
        let options = OutputOptions {
            color_depth: depth,
            ..defaults
        };
        assert_eq!(OutputPolicy::resolve(options, false, &hostile), expected);
        assert_eq!(
            OutputPolicy::resolve(
                OutputOptions {
                    no_color: true,
                    ..options
                },
                true,
                &hostile
            ),
            Monochrome
        );
    }
}

#[test]
fn legacy_palette_and_nearest_mapping_are_deterministic() {
    assert_eq!(indexed_rgb(0), Color::Rgb(0, 0, 0));
    assert_eq!(indexed_rgb(16), Color::Rgb(0, 0, 0));
    assert_eq!(indexed_rgb(196), Color::Rgb(255, 0, 0));
    assert_eq!(indexed_rgb(231), Color::Rgb(255, 255, 255));
    assert_eq!(indexed_rgb(232), Color::Rgb(8, 8, 8));
    assert_eq!(indexed_rgb(255), Color::Rgb(238, 238, 238));
    assert_eq!(
        OutputPolicy::Ansi256.adapt(Color::Rgb(13, 13, 13)),
        Color::Indexed(232)
    );
    for r in (0..=255).step_by(31) {
        for g in (0..=255).step_by(43) {
            for b in (0..=255).step_by(59) {
                let distance = |index: u8| {
                    let (pr, pg, pb) = color_rgb(indexed_rgb(index)).unwrap();
                    [
                        i32::from(r) - i32::from(pr),
                        i32::from(g) - i32::from(pg),
                        i32::from(b) - i32::from(pb),
                    ]
                    .iter()
                    .map(|d| d * d)
                    .sum::<i32>()
                };
                let expected = (0..=255)
                    .min_by_key(|index| (distance(*index), *index))
                    .unwrap();
                assert_eq!(
                    OutputPolicy::Ansi256.adapt(Color::Rgb(r, g, b)),
                    Color::Indexed(expected)
                );
            }
        }
    }
}

#[test]
fn final_buffer_and_serialization_adapt_both_channels() {
    let mut original = Buffer::empty(Rect::new(0, 0, 2, 1));
    for cell in &mut original.content {
        cell.set_symbol("x")
            .set_fg(Color::Rgb(12, 34, 56))
            .set_bg(Color::Rgb(210, 87, 32));
    }
    for policy in [
        OutputPolicy::Truecolor,
        OutputPolicy::Ansi256,
        OutputPolicy::Ansi16,
        OutputPolicy::Monochrome,
    ] {
        let mut buffer = original.clone();
        adapt_buffer(&mut buffer, policy);
        let mut out = Vec::new();
        pace::tools::write_buffer(&mut out, &original, policy).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("xx"));
        match policy {
            OutputPolicy::Truecolor => {
                assert_eq!(text.matches("\x1b[38;2;").count(), 1);
                assert_eq!(text.matches("\x1b[48;2;").count(), 1);
                assert!(text.contains("12;34;56m"));
                assert!(text.contains("210;87;32m"));
            }
            OutputPolicy::Ansi256 => {
                assert!(text.contains("38;5;"));
                assert!(text.contains("48;5;"));
                assert!(!text.contains(";2;"));
            }
            OutputPolicy::Ansi16 => {
                assert!(text.contains('\x1b'));
                assert!(!text.contains("38;"));
                assert!(!text.contains("48;"));
            }
            OutputPolicy::Monochrome => {
                assert_eq!(text, "xx\n");
                assert!(
                    buffer
                        .content
                        .iter()
                        .all(|cell| cell.fg == Color::Reset && cell.bg == Color::Reset)
                );
            }
        }
    }
}
