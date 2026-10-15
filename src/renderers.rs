use crate::{
    effects::Effects,
    fonts::Fonts,
    graphics::{centered, centered_text, double_box, progress_color, progress_gradient, put, text},
    model::TimerDetails,
};
use rand::seq::SliceRandom;
use ratatui::style::Color;
use ratatui::{buffer::Buffer, layout::Rect};

pub const PREVIEW_RENDERERS: &[&str] = &[
    "pie",
    "bar",
    "bigtext",
    "circles",
    "colorwheel",
    "colossal",
    "dots",
    "lava",
    "sine",
    "slant",
    "sweep",
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
            let split = ((phase - 0.75) * 4.0 * f64::from(area.width)) as u16;
            for y in area.y..area.bottom() {
                for x in area.x..area.x + split.min(area.width) {
                    buf[(x, y)] = transition[(x, y)].clone();
                }
                if split < area.width {
                    put(buf, area, i32::from(split), i32::from(y - area.y), "░", 129);
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
                    "pie" | "colorwheel" => ("█", "░", false),
                    "sweep" => ("█", " ", false),
                    "circles" | "sine" => ("●", "⬤", true),
                    _ => return,
                };
                let gradient: &[u8] = match name {
                    "pie" => &[123, 159, 153, 147, 141, 135, 129],
                    "colorwheel" => &[15],
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
        for (row, line) in lines.iter().enumerate() {
            let color = gradient[(row * gradient.len() / lines.len().max(1)
                + if name == "colossal" {
                    details.legacy_tick() as usize
                } else {
                    0
                })
                % gradient.len()];
            for (col, ch) in line.iter().enumerate() {
                let symbol = if name == "slant" && *ch == ' ' {
                    '_'
                } else {
                    *ch
                };
                let fg = if name == "slant" {
                    if symbol == '_' { 51 } else { 15 }
                } else {
                    color
                };
                for dy in 0..scale {
                    for dx in 0..scale {
                        put(
                            buf,
                            area,
                            x + col as i32 * scale + dx,
                            y + row as i32 * scale + dy,
                            symbol.encode_utf8(&mut [0; 4]),
                            fg,
                        );
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
            centered_text(buf, area, i32::from(area.height) / 2, value, gradient[0]);
            return;
        }
        for is_shadow in [true, false] {
            for (row, line) in lines.iter().enumerate() {
                let color = gradient[row * gradient.len() / lines.len().max(1)];
                for (col, ch) in line.iter().enumerate().filter(|(_, c)| **c != ' ') {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let offset = i32::from(is_shadow);
                            let mut encoded = [0; 4];
                            let symbol = if is_shadow {
                                shadow
                            } else if *ch == '█' {
                                fill
                            } else {
                                ch.encode_utf8(&mut encoded)
                            };
                            // Uncolored shadows retain the effect color; circles/sine use black.
                            let px = x + col as i32 * scale + dx + offset;
                            let py = y + row as i32 * scale + dy + offset;
                            if is_shadow
                                && px >= 0
                                && py >= 0
                                && px < i32::from(area.width)
                                && py < i32::from(area.height)
                            {
                                if let Some(cell) =
                                    buf.cell_mut((area.x + px as u16, area.y + py as u16))
                                {
                                    cell.set_symbol(symbol);
                                    if symbol == "⬤" {
                                        cell.set_fg(Color::Indexed(16));
                                    }
                                }
                            } else if !is_shadow {
                                put(buf, area, px, py, symbol, color);
                            }
                        }
                    }
                }
            }
        }
    }
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
