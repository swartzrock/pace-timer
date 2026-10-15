use crate::{
    fonts::{FONT_NAMES, Fonts},
    graphics::{centered_text, put},
    model::TimerDetails,
    palette::{self, COLOR_NAMES},
    renderers::{PREVIEW_RENDERERS, Renderers},
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use std::io::{self, Write};

pub fn preview(out: &mut impl Write) -> io::Result<()> {
    let (width, height) = crossterm::terminal::size().unwrap_or((120, 40));
    let cell_width = (width / 2).saturating_sub(1).max(1);
    let cell_height = (height / 2).saturating_sub(1).max(1);
    let mut fonts = Fonts::new();
    writeln!(out, "\n")?;
    gradient_text(
        out,
        &fonts.render("ANSI Regular", "pace renderers"),
        palette::DOUBLE_GRADIENTS[6].1,
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
        write_buffer(out, &buf)?;
        writeln!(out)?;
    }
    Ok(())
}

pub fn all_fonts(out: &mut impl Write) -> io::Result<()> {
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
        )?;
        writeln!(out)?;
    }
    Ok(())
}

pub fn color_blocks(out: &mut impl Write) -> io::Result<()> {
    let mut fonts = Fonts::new();
    writeln!(out, "\n")?;
    gradient_text(
        out,
        &fonts.render("ANSI Regular", "Color Gradients"),
        palette::DOUBLE_GRADIENTS[6].1,
    )?;
    for (title, gradients) in [
        ("SINGLE COLOR GRADIENTS", palette::SINGLE_GRADIENTS),
        ("DOUBLE COLOR GRADIENTS", palette::DOUBLE_GRADIENTS),
    ] {
        gradient_text(
            out,
            &fonts.render("JS Stick Letters", title),
            &[46, 47, 48, 49, 50, 51],
        )?;
        writeln!(
            out,
            "NAME                                              GRADIENT"
        )?;
        for (name, colors) in gradients {
            write!(out, "{name:<50}")?;
            let block_width = if colors.len() == 12 { 6 } else { 12 };
            for color in *colors {
                write!(out, "\x1b[48;5;{color}m{:width$}", "", width = block_width)?;
            }
            writeln!(out, "\x1b[0m")?;
        }
    }
    gradient_text(
        out,
        &fonts.render("ANSI Regular", "Color Palettes"),
        palette::DOUBLE_GRADIENTS[6].1,
    )?;
    for (title, rows) in [
        ("BLUE-GREEN PALETTE", palette::BLUE_GREEN_PALETTE),
        ("PURPLE-GREEN PALETTE", palette::PURPLE_GREEN_PALETTE),
        ("RED-YELLOW PALETTE", palette::RED_YELLOW_PALETTE),
    ] {
        writeln!(out, "\n{title}\n")?;
        for col in 0..12 {
            for row in rows {
                color_block(out, row[col])?;
            }
            writeln!(out)?;
        }
    }
    writeln!(out, "\nMONOCHROME PALETTE\n")?;
    for row in palette::MONOCHROME.chunks(6) {
        for color in row {
            color_block(out, *color)?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn color_block(out: &mut impl Write, color: u8) -> io::Result<()> {
    let fg = if color < 16 {
        if color == 0 { 15 } else { 0 }
    } else if color > 231 {
        if color < 244 { 15 } else { 0 }
    } else if (color - 16) % 36 >= 12 {
        0
    } else {
        15
    };
    write!(
        out,
        "\x1b[38;5;{fg}m\x1b[48;5;{color}m{:<22}\x1b[0m ",
        COLOR_NAMES[color as usize]
    )
}

fn gradient_text(out: &mut impl Write, value: &str, gradient: &[u8]) -> io::Result<()> {
    let count = value.lines().count().max(1);
    for (row, line) in value.lines().enumerate() {
        writeln!(
            out,
            "\x1b[38;5;{}m{line}\x1b[0m",
            gradient[row * gradient.len() / count]
        )?;
    }
    Ok(())
}

fn write_buffer(out: &mut impl Write, buf: &Buffer) -> io::Result<()> {
    for y in buf.area.y..buf.area.bottom() {
        let mut fg = Color::Reset;
        for x in buf.area.x..buf.area.right() {
            let cell = &buf[(x, y)];
            if cell.fg != fg {
                match cell.fg {
                    Color::Indexed(index) => write!(out, "\x1b[38;5;{index}m")?,
                    _ => write!(out, "\x1b[39m")?,
                }
                fg = cell.fg;
            }
            write!(out, "{}", cell.symbol())?;
        }
        writeln!(out, "\x1b[0m")?;
    }
    Ok(())
}
