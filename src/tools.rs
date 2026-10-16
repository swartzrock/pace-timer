use crate::{
    color::{OutputPolicy, adapt_buffer, indexed_rgb},
    fonts::{FONT_NAMES, Fonts},
    graphics::{centered_text, put, text},
    model::TimerDetails,
    palette::{self, COLOR_NAMES},
    renderers::{PREVIEW_RENDERERS, Renderers},
};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::Color,
};
use std::io::{self, Write};

pub fn preview(out: &mut impl Write, policy: OutputPolicy) -> io::Result<()> {
    preview_at_size(
        out,
        policy,
        crossterm::terminal::size().unwrap_or((120, 40)),
    )
}

fn preview_at_size(
    out: &mut impl Write,
    policy: OutputPolicy,
    (width, height): (u16, u16),
) -> io::Result<()> {
    let columns = (width / 50).clamp(1, 3);
    // Leave the final terminal column unused so a printed row cannot auto-wrap.
    let width = width.saturating_sub(1).max(1);
    let cell_width = (width - (columns - 1)) / columns;
    let cell_height = (height.saturating_sub(4) / 2).clamp(12, 16) * 3 / 2;
    let mut heading = Buffer::empty(Rect::new(0, 0, width, 3));
    let area = heading.area;
    text(&mut heading, area, 0, 0, "Pace renderers", 252);
    text(&mut heading, area, 0, 1, "Try one: pace 5m tide", 244);
    write_buffer(out, &heading, policy)?;
    let mut renderers = Renderers::new();
    let mut names = PREVIEW_RENDERERS.to_vec();
    names.sort_unstable();
    for row in names.chunks(usize::from(columns)) {
        let bounds = Rect::new(0, 0, width, cell_height);
        let mut buf = Buffer::empty(bounds);
        for (i, name) in row.iter().enumerate() {
            let area = Rect::new((i as u16) * (cell_width + 1), 0, cell_width, cell_height);
            renderers.render(
                name,
                TimerDetails {
                    elapsed: 199.9,
                    duration: 300.0,
                },
                area.inner(Margin::new(1, 1)),
                &mut buf,
            );
            for x in 0..i32::from(area.width) {
                put(&mut buf, area, x, 0, "─", 238);
            }
            if area.width >= 2 && area.height >= 2 {
                let right = i32::from(area.width) - 1;
                let bottom = i32::from(area.height) - 1;
                for x in 1..right {
                    put(&mut buf, area, x, bottom, "─", 238);
                }
                for y in 1..bottom {
                    put(&mut buf, area, 0, y, "│", 238);
                    put(&mut buf, area, right, y, "│", 238);
                }
                for (x, y, symbol) in [
                    (0, 0, "┌"),
                    (right, 0, "┐"),
                    (0, bottom, "└"),
                    (right, bottom, "┘"),
                ] {
                    put(&mut buf, area, x, y, symbol, 238);
                }
                let label = Rect::new(area.x + 1, area.y, area.width - 2, 1);
                centered_text(&mut buf, label, 0, &format!(" {name} "), 250);
            }
        }
        write_buffer(out, &buf, policy)?;
        writeln!(out)?;
    }
    Ok(())
}

pub fn all_fonts(out: &mut impl Write, policy: OutputPolicy) -> io::Result<()> {
    let mut fonts = Fonts::new();
    for (id, name) in FONT_NAMES {
        let width = (0..10)
            .map(|n| {
                fonts
                    .render(name, &format!("{n}{n}:{n}{n}"))
                    .lines()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);
        writeln!(out, ":: {id} (55:55 = {width} chars) ::")?;
        gradient_text(
            out,
            &fonts.render(name, "23 Quick foxes"),
            &[46, 47, 48, 49, 50, 51],
            policy,
        )?;
        writeln!(out)?;
    }
    Ok(())
}

pub fn color_blocks(out: &mut impl Write, policy: OutputPolicy) -> io::Result<()> {
    let mut fonts = Fonts::new();
    writeln!(out, "\n")?;
    gradient_text(
        out,
        &fonts.render("ANSI Regular", "Color Gradients"),
        palette::DOUBLE_GRADIENTS[6].1,
        policy,
    )?;
    for (title, gradients) in [
        ("SINGLE COLOR GRADIENTS", palette::SINGLE_GRADIENTS),
        ("DOUBLE COLOR GRADIENTS", palette::DOUBLE_GRADIENTS),
    ] {
        gradient_text(
            out,
            &fonts.render("JS Stick Letters", title),
            &[46, 47, 48, 49, 50, 51],
            policy,
        )?;
        writeln!(
            out,
            "NAME                                              GRADIENT"
        )?;
        for (name, colors) in gradients {
            write!(out, "{name:<50}")?;
            let block_width = if colors.len() == 12 { 6 } else { 12 };
            for color in *colors {
                if policy != OutputPolicy::Monochrome {
                    write_color(out, policy.adapt(indexed_rgb(*color)), true)?;
                }
                write!(out, "{:width$}", "", width = block_width)?;
            }
            reset(out, policy)?;
            writeln!(out)?;
        }
    }
    gradient_text(
        out,
        &fonts.render("ANSI Regular", "Color Palettes"),
        palette::DOUBLE_GRADIENTS[6].1,
        policy,
    )?;
    for (title, rows) in [
        ("BLUE-GREEN PALETTE", palette::BLUE_GREEN_PALETTE),
        ("PURPLE-GREEN PALETTE", palette::PURPLE_GREEN_PALETTE),
        ("RED-YELLOW PALETTE", palette::RED_YELLOW_PALETTE),
    ] {
        writeln!(out, "\n{title}\n")?;
        for col in 0..12 {
            for row in rows {
                color_block(out, row[col], policy)?;
            }
            writeln!(out)?;
        }
    }
    writeln!(out, "\nMONOCHROME PALETTE\n")?;
    for row in palette::MONOCHROME.chunks(6) {
        for color in row {
            color_block(out, *color, policy)?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn color_block(out: &mut impl Write, color: u8, policy: OutputPolicy) -> io::Result<()> {
    let fg = if color < 16 {
        if color == 0 { 15 } else { 0 }
    } else if color > 231 {
        if color < 244 { 15 } else { 0 }
    } else if (color - 16) % 36 >= 12 {
        0
    } else {
        15
    };
    if policy != OutputPolicy::Monochrome {
        write_color(out, policy.adapt(indexed_rgb(fg)), false)?;
        write_color(out, policy.adapt(indexed_rgb(color)), true)?;
    }
    write!(out, "{:<22}", COLOR_NAMES[color as usize])?;
    reset(out, policy)?;
    write!(out, " ")
}

fn gradient_text(
    out: &mut impl Write,
    value: &str,
    gradient: &[u8],
    policy: OutputPolicy,
) -> io::Result<()> {
    let count = value.lines().count().max(1);
    for (row, line) in value.lines().enumerate() {
        if policy != OutputPolicy::Monochrome {
            write_color(
                out,
                policy.adapt(indexed_rgb(gradient[row * gradient.len() / count])),
                false,
            )?;
        }
        write!(out, "{line}")?;
        reset(out, policy)?;
        writeln!(out)?;
    }
    Ok(())
}

fn reset(out: &mut impl Write, policy: OutputPolicy) -> io::Result<()> {
    if policy != OutputPolicy::Monochrome {
        write!(out, "\x1b[0m")?;
    }
    Ok(())
}

fn write_color(out: &mut impl Write, color: Color, background: bool) -> io::Result<()> {
    let channel = if background { 48 } else { 38 };
    match color {
        Color::Rgb(r, g, b) => write!(out, "\x1b[{channel};2;{r};{g};{b}m"),
        Color::Indexed(index) => write!(out, "\x1b[{channel};5;{index}m"),
        Color::Reset => write!(out, "\x1b[{}m", if background { 49 } else { 39 }),
        named => {
            let index = crate::color::ANSI_COLORS
                .iter()
                .position(|color| *color == named)
                .unwrap();
            let code = crate::color::ansi_code(index as u8, background);
            write!(out, "\x1b[{code}m")
        }
    }
}

/// Serialize final foreground and background with one transition per changed color.
pub fn write_buffer(out: &mut impl Write, buf: &Buffer, policy: OutputPolicy) -> io::Result<()> {
    let mut buf = buf.clone();
    adapt_buffer(&mut buf, policy);
    for y in buf.area.y..buf.area.bottom() {
        let (mut fg, mut bg) = (Color::Reset, Color::Reset);
        for x in buf.area.x..buf.area.right() {
            let cell = &buf[(x, y)];
            if cell.fg != fg {
                write_color(out, cell.fg, false)?;
                fg = cell.fg;
            }
            if cell.bg != bg {
                write_color(out, cell.bg, true)?;
                bg = cell.bg;
            }
            write!(out, "{}", cell.symbol())?;
        }
        if fg != Color::Reset || bg != Color::Reset {
            reset(out, policy)?;
        }
        writeln!(out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_borders_remain_gray_in_sixteen_color_output() {
        let mut output = Vec::new();
        preview_at_size(&mut output, OutputPolicy::Ansi16, (100, 35)).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("\x1b[90m┌"));
    }

    #[test]
    fn preview_grid_adapts_columns_and_keeps_panels_compact() {
        for (width, height, columns) in [
            (80, 24, 1),
            (99, 35, 1),
            (100, 35, 2),
            (149, 40, 2),
            (150, 40, 3),
            (200, 60, 3),
        ] {
            let mut output = Vec::new();
            preview_at_size(&mut output, OutputPolicy::Monochrome, (width, height)).unwrap();
            let output = String::from_utf8(output).unwrap();
            let lines: Vec<_> = output.lines().collect();
            assert_eq!(lines[0].trim_end(), "Pace renderers");
            assert_eq!(lines[1].trim_end(), "Try one: pace 5m tide");
            assert!(
                lines
                    .iter()
                    .all(|line| line.chars().count() < width as usize)
            );
            let tops: Vec<_> = lines
                .iter()
                .enumerate()
                .filter(|(_, line)| line.contains('┌'))
                .collect();
            assert_eq!(tops.len(), PREVIEW_RENDERERS.len().div_ceil(columns));
            let names: Vec<_> = tops
                .iter()
                .flat_map(|(_, line)| line.split_whitespace())
                .filter(|word| word.chars().all(|ch| ch.is_ascii_lowercase()))
                .collect();
            assert_eq!(names.len(), PREVIEW_RENDERERS.len());
            assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
            for (index, (top, line)) in tops.iter().enumerate() {
                let count = columns.min(PREVIEW_RENDERERS.len() - index * columns);
                assert_eq!(line.matches('┌').count(), count);
                assert_eq!(line.matches('┐').count(), count);
                let bottom = lines[*top + 1..]
                    .iter()
                    .position(|line| line.contains('└'))
                    .unwrap();
                assert!((18..=24).contains(&(bottom + 2)));
                let bottom_line = lines[*top + bottom + 1];
                assert_eq!(bottom_line.matches('└').count(), count);
                assert_eq!(bottom_line.matches('┘').count(), count);
            }
            for name in PREVIEW_RENDERERS {
                assert!(output.contains(&format!(" {name} ")), "{name}");
            }
            assert!(!output.contains('\x1b'));
        }
    }

    #[test]
    fn preview_grid_handles_tiny_terminals() {
        for size in [(0, 0), (1, 1), (2, 2), (15, 5)] {
            let mut output = Vec::new();
            preview_at_size(&mut output, OutputPolicy::Monochrome, size).unwrap();
            let output = String::from_utf8(output).unwrap();
            assert!(!output.is_empty());
            assert!(output.lines().all(|line| {
                line.chars().count() <= usize::from(size.0.saturating_sub(1).max(1))
            }));
        }
    }
}
