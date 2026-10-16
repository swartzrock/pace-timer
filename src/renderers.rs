use crate::{
    effects::Effects,
    fonts::Fonts,
    graphics::{
        centered, centered_text, double_box, gradient as rgb_gradient, mix, progress_color,
        progress_gradient, put_color, text,
    },
    model::TimerDetails,
};
use rand::seq::SliceRandom;
use ratatui::style::Color;
use ratatui::{
    buffer::{Buffer, Cell},
    layout::Rect,
};

pub const PREVIEW_RENDERERS: &[&str] = &[
    "aurora",
    "pie",
    "bar",
    "bigtext",
    "circles",
    "colorwheel",
    "colossal",
    "dots",
    "interference",
    "lava",
    "sine",
    "slant",
    "sweep",
    "tide",
];

pub struct Renderers {
    effects: Effects,
    fonts: Fonts,
    order: Vec<&'static str>,
    transition: Buffer,
    bigtext_reference: Option<(i32, i32)>,
}

impl Default for Renderers {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderers {
    pub fn new() -> Self {
        let mut order = PREVIEW_RENDERERS.to_vec();
        order.shuffle(&mut rand::rng());
        Self {
            effects: Effects::new(),
            fonts: Fonts::new(),
            order,
            transition: Buffer::empty(Rect::default()),
            bigtext_reference: None,
        }
    }

    pub fn render(&mut self, name: &str, details: TimerDetails, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        if name != "shuffle" {
            self.draw_one(name, details, area, buf);
            return;
        }
        let (current, next, phase) = self.shuffle_state(details);
        self.draw_one(current, details, area, buf);
        if phase > 0.75 {
            // Keep one reusable transition buffer instead of allocating a full canvas per frame.
            let mut transition = std::mem::take(&mut self.transition);
            transition.resize(area);
            transition.reset();
            self.draw_one(next, details, area, &mut transition);
            let amount = (phase - 0.75) * 4.0;
            let band = (f64::from(area.width) * 0.06).clamp(2.0, 8.0);
            let edge = amount * (f64::from(area.width) + band * 2.0) - band;
            for y in area.y..area.bottom() {
                for x in area.x..area.right() {
                    let source = &mut buf[(x, y)];
                    let destination = &transition[(x, y)];
                    // A countdown uses one complete mask, never a dithered mixture of digits.
                    if countdown_cell(current, source) || countdown_cell(next, destination) {
                        let fg = transition_color(source.fg, destination.fg, amount);
                        let bg = transition_color(source.bg, destination.bg, amount);
                        if amount >= 0.5 {
                            *source = destination.clone();
                        }
                        source.set_fg(fg).set_bg(bg);
                    } else {
                        let local =
                            ((edge - f64::from(x - area.x)) / (band * 2.0) + 0.5).clamp(0.0, 1.0);
                        feather_cell(
                            source,
                            destination,
                            local * local * (3.0 - 2.0 * local),
                            x - area.x,
                            y - area.y,
                        );
                    }
                }
            }
            self.transition = transition;
        }
    }

    fn shuffle_state(&self, details: TimerDetails) -> (&'static str, &'static str, f64) {
        let index = (details.elapsed / 10.0).floor() as usize % self.order.len();
        (
            self.order[index],
            self.order[(index + 1) % self.order.len()],
            (details.elapsed % 10.0) / 10.0,
        )
    }

    pub fn status(&self, name: &str, details: TimerDetails) -> String {
        if name != "shuffle" {
            return String::new();
        }
        let (current, next, phase) = self.shuffle_state(details);
        if phase >= 0.8 {
            format!("Shuffle: loading the \"{next}\" renderer...")
        } else {
            format!(
                "Shuffle: showing the \"{current}\" renderer for {} more seconds...",
                (10.0 - details.elapsed % 10.0).ceil() as u32
            )
        }
    }

    fn draw_one(&mut self, name: &str, details: TimerDetails, area: Rect, buf: &mut Buffer) {
        match name {
            "bigtext" | "colossal" | "slant" => self.draw_font(name, details, area, buf),
            _ => {
                self.effects.draw(name, details, area, buf);
                let (fill, shadow, double) = match name {
                    "aurora" | "tide" | "interference" => ("█", " ", false),
                    "pie" | "colorwheel" => ("█", "░", false),
                    "sweep" => ("█", " ", false),
                    "circles" | "sine" => ("●", "⬤", true),
                    _ => return,
                };
                let gradient: &[u8] = match name {
                    "aurora" | "tide" | "interference" => &[15],
                    "pie" => &[123, 159, 153, 147, 141, 135, 129],
                    "colorwheel" => &[255, 15, 159],
                    "sine" => &[51, 45, 39, 33, 27, 21],
                    _ => progress_gradient(details.progress()),
                };
                self.draw_shadowed(
                    &details.remaining_text(),
                    area,
                    buf,
                    gradient,
                    fill,
                    shadow,
                    double,
                    matches!(name, "aurora" | "tide" | "interference"),
                );
            }
        }
    }

    fn draw_font(&mut self, name: &str, details: TimerDetails, area: Rect, buf: &mut Buffer) {
        let font = match name {
            "colossal" => "Colossal",
            "slant" => "Slant Relief",
            _ => "ANSI Regular",
        };
        let rendered = self.fonts.render(font, &details.remaining_text());
        let lines = trimmed_lines(&rendered);
        let width = lines.iter().map(Vec::len).max().unwrap_or(0) as i32;
        let height = lines.len() as i32;
        let mut scale = 1;
        if name == "bigtext" {
            let (reference_width, reference_height) =
                self.bigtext_reference.get_or_insert_with(|| {
                    let reference = trimmed_lines(&self.fonts.render("ANSI Regular", "55:55"));
                    (
                        reference.iter().map(Vec::len).max().unwrap_or(0) as i32,
                        reference.len() as i32,
                    )
                });
            while *reference_width * scale * 2 < i32::from(area.width.saturating_sub(4))
                && *reference_height * scale * 2 < i32::from(area.height.saturating_sub(4))
            {
                scale *= 2;
            }
        }
        let x = (i32::from(area.width) - width * scale) / 2;
        let y = (i32::from(area.height) - height * scale) / 2;
        if name == "colossal" {
            let box_width = (width + 6).min(i32::from(area.width)).max(0) as u16;
            let box_height = (height + 6).min(i32::from(area.height)).max(0) as u16;
            let bounds = centered(area, box_width, box_height);
            double_box(buf, bounds, progress_color(details.progress()));
        }
        let gradient: &[u8] = if name == "bigtext" {
            if details.progress() > 0.9 {
                &[160, 161, 162, 163, 164, 165]
            } else if details.progress() > 0.7 {
                &[184, 185, 186, 187, 188, 189]
            } else {
                &[21, 27, 33, 39, 45, 51]
            }
        } else {
            &[
                57, 63, 69, 75, 81, 87, 51, 45, 39, 33, 27, 21, 21, 27, 33, 39, 45, 51, 87, 81, 75,
                69, 63, 57,
            ]
        };
        // Shadows are separate layers; foreground cells retain the original font mask.
        for (offset, intensity) in [(2, 0.10), (1, 0.22), (0, 1.0)] {
            for (row, line) in lines.iter().enumerate() {
                for (col, ch) in line.iter().enumerate() {
                    let symbol = if name == "slant" && *ch == ' ' {
                        '_'
                    } else {
                        *ch
                    };
                    if symbol == ' ' && offset > 0 {
                        continue;
                    }
                    let position = row as f64 / f64::from((height - 1).max(1)) * 0.75
                        + col as f64 / f64::from((width - 1).max(1)) * 0.25;
                    let mut color = rgb_gradient(
                        gradient,
                        if name == "colossal" {
                            (position + details.elapsed * 0.12).rem_euclid(1.0)
                        } else {
                            position
                        },
                    );
                    if name == "slant" {
                        color = if symbol == '_' {
                            rgb_gradient(&[39, 45, 51], position)
                        } else {
                            mix(Color::White, color, 0.22)
                        };
                    }
                    if offset > 0 {
                        color = mix(Color::Black, color, intensity);
                    }
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let mut encoded = [0; 4];
                            put_text_layer(
                                buf,
                                area,
                                (
                                    x + col as i32 * scale + dx + offset,
                                    y + row as i32 * scale + dy + offset,
                                ),
                                if offset > 0 {
                                    "░"
                                } else {
                                    symbol.encode_utf8(&mut encoded)
                                },
                                color,
                                offset > 0,
                            );
                        }
                    }
                }
            }
        }
        if width > i32::from(area.width) || height > i32::from(area.height) {
            // Keep the countdown legible even when the large glyphs are clipped by a small terminal.
            centered_text(
                buf,
                area,
                i32::from(area.height) - 1,
                &details.remaining_text(),
                progress_color(details.progress()),
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_shadowed(
        &mut self,
        value: &str,
        area: Rect,
        buf: &mut Buffer,
        gradient: &[u8],
        fill: &str,
        shadow: &str,
        double: bool,
        dark_background: bool,
    ) {
        let rendered = self.fonts.render("ANSI Regular", value);
        let lines = trimmed_lines(&rendered);
        let width = lines.iter().map(Vec::len).max().unwrap_or(0) as i32;
        let height = lines.len() as i32;
        let scale =
            if double && width * 2 < i32::from(area.width) && height * 2 < i32::from(area.height) {
                2
            } else {
                1
            };
        let x = (i32::from(area.width) - width * scale) / 2;
        let y = (i32::from(area.height) - height * scale) / 2;
        if width > i32::from(area.width) || height > i32::from(area.height) {
            if dark_background {
                let left = (i32::from(area.width) - value.chars().count() as i32) / 2;
                for col in 0..value.chars().count() as i32 {
                    let x = left + col;
                    if x >= 0 && x < i32::from(area.width) {
                        buf[(area.x + x as u16, area.y + area.height / 2)]
                            .set_bg(Color::Rgb(0, 0, 0));
                    }
                }
            }
            centered_text(buf, area, i32::from(area.height) / 2, value, gradient[0]);
            return;
        }
        for (offset, intensity) in [(2, 0.08), (1, 0.18), (0, 1.0)] {
            for (row, line) in lines.iter().enumerate() {
                for (col, ch) in line.iter().enumerate().filter(|(_, c)| **c != ' ') {
                    let position = row as f64 / f64::from((height - 1).max(1)) * 0.75
                        + col as f64 / f64::from((width - 1).max(1)) * 0.25;
                    let color = rgb_gradient(gradient, position);
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let mut encoded = [0; 4];
                            let symbol = if offset > 0 {
                                shadow
                            } else if *ch == '█' {
                                fill
                            } else {
                                ch.encode_utf8(&mut encoded)
                            };
                            let px = x + col as i32 * scale + dx + offset;
                            let py = y + row as i32 * scale + dy + offset;
                            put_text_layer(
                                buf,
                                area,
                                (px, py),
                                symbol,
                                if offset > 0 {
                                    mix(Color::Black, color, intensity)
                                } else {
                                    color
                                },
                                offset > 0,
                            );
                            if dark_background
                                && px >= 0
                                && py >= 0
                                && px < i32::from(area.width)
                                && py < i32::from(area.height)
                                && let Some(cell) =
                                    buf.cell_mut((area.x + px as u16, area.y + py as u16))
                            {
                                cell.set_bg(Color::Rgb(0, 0, 0));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn put_text_layer(
    buf: &mut Buffer,
    area: Rect,
    (x, y): (i32, i32),
    symbol: &str,
    color: Color,
    shadow: bool,
) {
    if !shadow {
        put_color(buf, area, x, y, symbol, color);
    } else if x >= 0
        && y >= 0
        && x < i32::from(area.width)
        && y < i32::from(area.height)
        && let Some(cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16))
    {
        // Shade the existing lower half/background instead of cutting a black rectangle.
        let bg = if cell.bg == Color::Reset {
            Color::Reset
        } else {
            mix(cell.bg, Color::Black, 0.4)
        };
        cell.set_symbol(symbol).set_fg(color).set_bg(bg);
    }
}

fn countdown_cell(name: &str, cell: &Cell) -> bool {
    let symbol = cell.symbol();
    match name {
        "bigtext" | "colossal" | "slant" => symbol != " ",
        "aurora" | "tide" | "interference"
            if symbol == "█" && cell.fg == Color::Rgb(255, 255, 255) =>
        {
            true
        }
        "pie" | "colorwheel" | "sweep" if matches!(symbol, "█" | "░") => true,
        "circles" | "sine" if matches!(symbol, "●" | "⬤") => true,
        _ => symbol
            .chars()
            .any(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '|')),
    }
}

fn transition_color(source: Color, destination: Color, amount: f64) -> Color {
    if source == Color::Reset && destination == Color::Reset {
        Color::Reset
    } else {
        mix(source, destination, amount)
    }
}

fn feather_cell(source: &mut Cell, destination: &Cell, amount: f64, x: u16, y: u16) {
    if amount <= 0.0 {
        return;
    }
    if amount >= 1.0 {
        *source = destination.clone();
        return;
    }
    let fg = transition_color(source.fg, destination.fg, amount);
    let bg = transition_color(source.bg, destination.bg, amount);
    // Ordered coverage is stable while paused and chooses only an existing glyph.
    const COVERAGE: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    let threshold = (f64::from(COVERAGE[usize::from(y % 4)][usize::from(x % 4)]) + 0.5) / 16.0;
    if source.symbol() != destination.symbol() && amount > threshold {
        *source = destination.clone();
    }
    source.set_fg(fg).set_bg(bg);
}

fn trimmed_lines(rendered: &str) -> Vec<Vec<char>> {
    let mut lines: Vec<Vec<char>> = rendered.lines().map(|s| s.chars().collect()).collect();
    while lines
        .first()
        .is_some_and(|row| row.iter().all(|c| c.is_whitespace()))
    {
        lines.remove(0);
    }
    while lines
        .last()
        .is_some_and(|row| row.iter().all(|c| c.is_whitespace()))
    {
        lines.pop();
    }
    let left = lines
        .iter()
        .filter_map(|row| row.iter().position(|ch| !ch.is_whitespace()))
        .min()
        .unwrap_or(0);
    for line in &mut lines {
        *line = line.iter().skip(left).copied().collect();
        while line.last().is_some_and(|ch| ch.is_whitespace()) {
            line.pop();
        }
    }
    lines
}

pub fn paused_overlay(buf: &mut Buffer, area: Rect) {
    let width = 12.min(area.width);
    let height = 3.min(area.height);
    let bounds = centered(area, width, height);
    for y in bounds.y..bounds.bottom() {
        for x in bounds.x..bounds.right() {
            buf[(x, y)].reset();
        }
    }
    double_box(buf, bounds, 196);
    text(
        buf,
        bounds,
        (i32::from(width) - 6) / 2,
        i32::from(height) / 2,
        "PAUSED",
        196,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_shadows_keep_and_darken_the_effect_background() {
        let area = Rect::new(3, 2, 4, 3);
        let mut buf = Buffer::filled(
            area,
            Cell::default().set_bg(Color::Rgb(100, 150, 200)).clone(),
        );
        put_text_layer(&mut buf, area, (1, 1), "⬤", Color::Rgb(20, 30, 40), true);
        let cell = &buf[(4, 3)];
        assert_eq!(cell.symbol(), "⬤");
        assert!(
            matches!(cell.bg, Color::Rgb(r, g, b) if r > 0 && r < 100 && g > 0 && g < 150 && b > 0 && b < 200)
        );
        assert_eq!(buf[(3, 2)].bg, Color::Rgb(100, 150, 200));
    }

    #[test]
    fn feather_blends_both_channels_and_uses_only_source_glyphs() {
        for destination_symbol in ["▀", "▄"] {
            let mut source = Cell::default();
            source
                .set_symbol("▀")
                .set_fg(Color::Rgb(20, 40, 60))
                .set_bg(Color::Rgb(60, 40, 20));
            let mut destination = Cell::default();
            destination
                .set_symbol(destination_symbol)
                .set_fg(Color::Rgb(100, 120, 140))
                .set_bg(Color::Rgb(140, 120, 100));
            let original = source.clone();
            feather_cell(&mut source, &destination, 0.0, 2, 3);
            assert_eq!(source, original);
            for y in 0..4 {
                for x in 0..4 {
                    let mut blended = original.clone();
                    feather_cell(&mut blended, &destination, 0.25, x, y);
                    assert_eq!(blended.fg, Color::Rgb(40, 60, 80));
                    assert_eq!(blended.bg, Color::Rgb(80, 60, 40));
                    assert!(matches!(blended.symbol(), "▀" | "▄"));
                    let mut repeated = original.clone();
                    feather_cell(&mut repeated, &destination, 0.25, x, y);
                    assert_eq!(blended, repeated);
                }
            }
            feather_cell(&mut source, &destination, 1.0, 2, 3);
            assert_eq!(source, destination);
        }
    }

    #[test]
    fn feather_preserves_default_background_and_covers_incompatible_glyphs() {
        let mut source = Cell::default();
        source.set_symbol("a");
        let mut destination = Cell::default();
        destination.set_symbol("b");
        let mut destination_count = 0;
        for y in 0..4 {
            for x in 0..4 {
                let mut cell = source.clone();
                feather_cell(&mut cell, &destination, 0.5, x, y);
                assert_eq!(cell.fg, Color::Reset);
                assert_eq!(cell.bg, Color::Reset);
                destination_count += usize::from(cell.symbol() == "b");
            }
        }
        assert_eq!(destination_count, 8);
    }

    #[test]
    fn shuffle_keeps_a_complete_countdown_through_the_final_second_and_resize() {
        let mut renderers = Renderers::new();
        renderers.order = vec!["bigtext", "slant"];
        for area in [
            Rect::new(3, 2, 120, 40),
            Rect::new(1, 1, 12, 4),
            Rect::new(2, 3, 80, 24),
        ] {
            for elapsed in [8.0, 8.74, 8.75, 9.25, 9.99] {
                let details = TimerDetails {
                    elapsed,
                    duration: 10.0,
                };
                let mut actual = Buffer::empty(area);
                let mut expected = Buffer::empty(area);
                renderers.render("shuffle", details, area, &mut actual);
                renderers.render(
                    if elapsed < 8.75 { "bigtext" } else { "slant" },
                    details,
                    area,
                    &mut expected,
                );
                assert!(
                    actual
                        .content
                        .iter()
                        .zip(&expected.content)
                        .all(|(a, b)| a.symbol() == b.symbol()),
                    "countdown broke at {elapsed} in {area:?}"
                );
                assert_eq!(renderers.transition.area, area);
            }
        }
    }

    #[test]
    fn shuffle_keeps_embedded_countdown_labels_together() {
        let mut renderers = Renderers::new();
        renderers.order = vec!["bar", "dots"];
        let area = Rect::new(3, 2, 120, 40);
        let labels = |buf: &Buffer| -> Vec<_> {
            buf.content
                .iter()
                .enumerate()
                .filter(|(_, cell)| {
                    cell.symbol()
                        .chars()
                        .any(|ch| ch.is_ascii_alphanumeric() || ch == ':')
                })
                .map(|(i, cell)| (i, cell.symbol().to_owned()))
                .collect()
        };
        for elapsed in [8.0, 8.74, 8.75, 9.99] {
            let details = TimerDetails {
                elapsed,
                duration: 10.0,
            };
            let mut actual = Buffer::empty(area);
            let mut expected = Buffer::empty(area);
            renderers.render("shuffle", details, area, &mut actual);
            renderers.render(
                if elapsed < 8.75 { "bar" } else { "dots" },
                details,
                area,
                &mut expected,
            );
            assert_eq!(labels(&actual), labels(&expected));
        }
    }
}
