use crate::{
    color::{OutputPolicy, adapt_buffer, indexed_rgb},
    fonts::{FONT_NAMES, Fonts},
    graphics::{centered_text, put},
    model::TimerDetails,
    palette::{self, COLOR_NAMES},
    renderers::{PREVIEW_RENDERERS, Renderers},
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use std::io::{self, Write};

pub fn preview(out: &mut impl Write, policy: OutputPolicy) -> io::Result<()> {
    let (width, height) = crossterm::terminal::size().unwrap_or((120, 40));
    let cell_width = (width / 2).saturating_sub(1).max(1);
    let cell_height = (height / 2).saturating_sub(1).max(1);
    let mut fonts = Fonts::new();
    writeln!(out, "\n")?;
    gradient_text(
        out,
        &fonts.render("ANSI Regular", "pace renderers"),
        palette::DOUBLE_GRADIENTS[6].1,
        policy,
    )?;
    let mut renderers = Renderers::new();
    for pair in PREVIEW_RENDERERS.chunks(2) {
        let bounds = Rect::new(0, 0, width.max(1), cell_height);
        let mut buf = Buffer::empty(bounds);
        for (i, name) in pair.iter().enumerate() {
            let area = Rect::new((i as u16) * (cell_width + 1), 0, cell_width, cell_height);
            renderers.render(
                name,
                TimerDetails {
                    elapsed: 199.9,
                    duration: 300.0,
                },
                area,
                &mut buf,
            );
            for x in 0..i32::from(area.width) {
                put(&mut buf, area, x, 0, "─", 239);
            }
            let title = format!(" {}{} ", name[..1].to_uppercase(), &name[1..]);
            centered_text(&mut buf, area, 0, &title, 15);
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
