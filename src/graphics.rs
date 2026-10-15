use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

/// Coordinates are relative to `area`; every primitive clips to that viewport.
pub fn put(buf: &mut Buffer, area: Rect, x: i32, y: i32, symbol: &str, fg: u8) {
    if x < 0 || y < 0 || x >= i32::from(area.width) || y >= i32::from(area.height) {
        return;
    }
    if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16)) {
        cell.set_symbol(symbol)
            .set_style(Style::default().fg(Color::Indexed(fg)));
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

pub fn inset(area: Rect, x: u16, y: u16) -> Rect {
    Rect::new(
        area.x + x.min(area.width),
        area.y + y.min(area.height),
        area.width.saturating_sub(x.saturating_mul(2)),
        area.height.saturating_sub(y.saturating_mul(2)),
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
