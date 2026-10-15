use crate::color::{color_rgb, indexed_rgb};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

/// Coordinates are relative to `area`; every primitive clips to that viewport.
pub fn put(buf: &mut Buffer, area: Rect, x: i32, y: i32, symbol: &str, fg: u8) {
    put_color(buf, area, x, y, symbol, indexed_rgb(fg));
}

pub fn put_color(buf: &mut Buffer, area: Rect, x: i32, y: i32, symbol: &str, fg: Color) {
    if x < 0 || y < 0 || x >= i32::from(area.width) || y >= i32::from(area.height) {
        return;
    }
    if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16)) {
        cell.set_symbol(symbol).set_style(Style::reset().fg(fg));
    }
}

/// Interpolate in RGB; terminal quantization happens after composition.
pub fn mix(from: Color, to: Color, amount: f64) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    if amount == 0.0 {
        return from;
    }
    if amount == 1.0 {
        return to;
    }
    let a = color_rgb(from).unwrap_or((0, 0, 0));
    let b = color_rgb(to).unwrap_or((0, 0, 0));
    let channel =
        |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * amount).round() as u8;
    Color::Rgb(channel(a.0, b.0), channel(a.1, b.1), channel(a.2, b.2))
}

/// Continuous position across legacy palette stops, in the range zero to one.
pub fn gradient(stops: &[u8], position: f64) -> Color {
    if stops.is_empty() {
        return Color::Reset;
    }
    let position = position.clamp(0.0, 1.0) * (stops.len() - 1) as f64;
    let lower = position.floor() as usize;
    mix(
        indexed_rgb(stops[lower]),
        indexed_rgb(stops[(lower + 1).min(stops.len() - 1)]),
        position.fract(),
    )
}

/// One terminal cell holds two vertical samples. None leaves that half transparent.
pub fn paired_pixels(
    buf: &mut Buffer,
    area: Rect,
    x: i32,
    y: i32,
    upper: Option<Color>,
    lower: Option<Color>,
) {
    if x < 0 || y < 0 || x >= i32::from(area.width) || y >= i32::from(area.height) {
        return;
    }
    let (symbol, fg, bg) = match (upper, lower) {
        (Some(upper), Some(lower)) => ("▀", upper, lower),
        (Some(upper), None) => ("▀", upper, Color::Reset),
        (None, Some(lower)) => ("▄", lower, Color::Reset),
        (None, None) => return,
    };
    if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16)) {
        cell.set_symbol(symbol)
            .set_style(Style::reset().fg(fg).bg(bg));
    }
}

pub fn text(buf: &mut Buffer, area: Rect, x: i32, y: i32, value: &str, fg: u8) {
    for (col, ch) in value.chars().enumerate() {
        put(
            buf,
            area,
            x + col as i32,
            y,
            ch.encode_utf8(&mut [0; 4]),
            fg,
        );
    }
}

pub fn centered_text(buf: &mut Buffer, area: Rect, y: i32, value: &str, fg: u8) {
    text(
        buf,
        area,
        (i32::from(area.width) - value.chars().count() as i32) / 2,
        y,
        value,
        fg,
    );
}

pub fn double_box(buf: &mut Buffer, area: Rect, fg: u8) {
    if area.width < 2 || area.height < 2 {
        return;
    }
    let right = i32::from(area.width) - 1;
    let bottom = i32::from(area.height) - 1;
    for x in 1..right {
        put(buf, area, x, 0, "═", fg);
        put(buf, area, x, bottom, "═", fg);
    }
    for y in 1..bottom {
        put(buf, area, 0, y, "║", fg);
        put(buf, area, right, y, "║", fg);
    }
    for (x, y, ch) in [
        (0, 0, "╔"),
        (right, 0, "╗"),
        (0, bottom, "╚"),
        (right, bottom, "╝"),
    ] {
        put(buf, area, x, y, ch, fg);
    }
}

pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width.min(area.width),
        height.min(area.height),
    )
}

pub fn progress_color(progress: f64) -> u8 {
    if progress > 0.9 {
        196
    } else if progress > 0.7 {
        154
    } else {
        46
    }
}

pub fn progress_gradient(progress: f64) -> &'static [u8] {
    if progress > 0.9 {
        &[160, 161, 162, 163, 164, 165]
    } else if progress > 0.7 {
        &[184, 185, 186, 187, 188, 189]
    } else {
        &[34, 35, 36, 37, 38, 39]
    }
}
