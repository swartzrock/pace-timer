use pace::{
    color::{OutputPolicy, adapt_buffer, indexed_rgb},
    graphics::{gradient, mix, paired_pixels, put_color},
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

#[test]
fn rgb_interpolation_preserves_endpoints_and_fills_between_stops() {
    let red = Color::Rgb(255, 0, 0);
    let blue = Color::Rgb(0, 0, 255);
    assert_eq!(mix(red, blue, 0.0), red);
    assert_eq!(mix(red, blue, 1.0), blue);
    assert_eq!(mix(red, blue, 0.5), Color::Rgb(128, 0, 128));
    assert_eq!(gradient(&[196, 21], 0.0), indexed_rgb(196));
    assert_eq!(gradient(&[196, 21], 0.5), Color::Rgb(128, 0, 128));
    assert_eq!(gradient(&[196, 21], 1.0), indexed_rgb(21));
}

#[test]
fn paired_pixels_encode_both_samples_and_clip_to_offset_viewports() {
    let outer = Rect::new(0, 0, 8, 6);
    let red = Color::Rgb(255, 0, 0);
    let blue = Color::Rgb(0, 0, 255);
    for area in [
        Rect::new(3, 2, 0, 0),
        Rect::new(3, 2, 1, 1),
        Rect::new(2, 1, 3, 3),
    ] {
        let mut buf = Buffer::empty(outer);
        for y in -1..=i32::from(area.height) {
            for x in -1..=i32::from(area.width) {
                paired_pixels(&mut buf, area, x, y, Some(red), Some(blue));
            }
        }
        for y in 0..outer.height {
            for x in 0..outer.width {
                let cell = &buf[(x, y)];
                if area.contains((x, y).into()) {
                    assert_eq!(cell.symbol(), "▀");
                    assert_eq!((cell.fg, cell.bg), (red, blue));
                } else {
                    assert_eq!(cell.symbol(), " ");
                }
            }
        }
    }
    let mut buf = Buffer::empty(outer);
    paired_pixels(&mut buf, outer, 0, 0, None, Some(blue));
    assert_eq!(buf[(0, 0)].symbol(), "▄");
    assert_eq!((buf[(0, 0)].fg, buf[(0, 0)].bg), (blue, Color::Reset));
    paired_pixels(&mut buf, outer, 1, 0, Some(red), Some(red));
    assert_eq!((buf[(1, 0)].fg, buf[(1, 0)].bg), (red, red));
    paired_pixels(&mut buf, outer, 2, 0, None, None);
    assert_eq!(buf[(2, 0)].symbol(), " ");
}

#[test]
fn monochrome_pixels_keep_structure_and_text_keeps_crisp_masks() {
    let area = Rect::new(0, 0, 4, 1);
    let mut buf = Buffer::empty(area);
    let bright = Color::Rgb(255, 255, 255);
    let dark = Color::Rgb(0, 0, 0);
    paired_pixels(&mut buf, area, 0, 0, Some(bright), Some(dark));
    paired_pixels(&mut buf, area, 1, 0, Some(dark), Some(bright));
    paired_pixels(&mut buf, area, 2, 0, Some(dark), Some(dark));
    put_color(&mut buf, area, 3, 0, "█", Color::Rgb(0, 0, 40));
    adapt_buffer(&mut buf, OutputPolicy::Monochrome);
    assert_eq!(buf[(0, 0)].symbol(), "▀");
    assert_eq!(buf[(1, 0)].symbol(), "▄");
    assert_eq!(buf[(2, 0)].symbol(), " ");
    assert_eq!(buf[(3, 0)].symbol(), "█");
    assert!(
        buf.content
            .iter()
            .all(|c| c.fg == Color::Reset && c.bg == Color::Reset)
    );
}

#[test]
fn monochrome_effects_retain_progress_outside_the_countdown() {
    use pace::{effects::Effects, model::TimerDetails};
    let area = Rect::new(0, 0, 80, 40);
    for name in ["dots", "pie"] {
        let mut effects = Effects::new();
        let mut states = Vec::new();
        for elapsed in [0.0, 30.0] {
            let mut buf = Buffer::empty(area);
            effects.draw(
                name,
                TimerDetails {
                    elapsed,
                    duration: 60.0,
                },
                area,
                &mut buf,
            );
            adapt_buffer(&mut buf, OutputPolicy::Monochrome);
            states.push(buf);
        }
        let count = |buffer: &Buffer| {
            buffer
                .content
                .iter()
                .map(|cell| match cell.symbol() {
                    "█" | "●" => 4,
                    "▓" | "◕" => 3,
                    "▒" | "▀" | "▄" | "◑" => 2,
                    "░" | "◔" => 1,
                    _ => 0,
                })
                .sum::<u32>()
        };
        assert!(
            count(&states[0]) < count(&states[1]),
            "{name} lost progress in monochrome"
        );
    }
}
