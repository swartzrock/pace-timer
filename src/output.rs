//! Compact indexed-color drawing; other terminal operations stay with Crossterm.
use ratatui::{
    backend::{Backend, ClearType, CrosstermBackend, WindowSize},
    buffer::Cell,
    layout::{Position, Size},
    style::Color,
};
use std::io::{self, Write};

pub(crate) struct OutputBackend<W: Write> {
    inner: CrosstermBackend<W>,
    compact: bool,
    bytes: Vec<u8>,
}

impl<W: Write> OutputBackend<W> {
    pub(crate) fn new(writer: W, compact: bool) -> Self {
        Self {
            inner: CrosstermBackend::new(writer),
            // Preserve Crossterm's native console fallback on other platforms.
            compact: compact && cfg!(unix),
            bytes: Vec::new(),
        }
    }
}

impl<W: Write> Write for OutputBackend<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.inner.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        Write::flush(&mut self.inner)
    }
}

impl<W: Write> Backend for OutputBackend<W> {
    type Error = io::Error;
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        if !self.compact {
            return self.inner.draw(content);
        }
        let cells: Vec<_> = content.collect();
        let indexed = |color| matches!(color, Color::Indexed(_) | Color::Reset);
        if cells.iter().any(|(_, _, cell)| {
            !cell.modifier.is_empty()
                || cell.underline_color != Color::Reset
                || !indexed(cell.fg)
                || !indexed(cell.bg)
        }) {
            return self.inner.draw(cells.into_iter());
        }
        self.bytes.clear();
        let (mut foreground, mut background) = (None, None);
        let mut cursor = None;
        for (x, y, cell) in cells {
            match cursor {
                Some((cx, cy)) if y == cy && x == cx => {}
                Some((cx, cy)) if y == cy && x > cx => write!(self.bytes, "\x1b[{}C", x - cx)?,
                _ => write!(self.bytes, "\x1b[{};{}H", y + 1, x + 1)?,
            }
            cursor = Some((x + 1, y));
            let solid =
                matches!(cell.symbol(), "▀" | "▄") && cell.fg == cell.bg && cell.fg != Color::Reset;
            let symbol = if solid { " " } else { cell.symbol() };
            let bg = cell.bg;
            // Spaces have no visible foreground. Keep the actual color for the next glyph.
            let set_fg = symbol != " " && foreground != Some(cell.fg);
            let set_bg = background != Some(bg);
            if set_fg || set_bg {
                self.bytes.extend_from_slice(b"\x1b[");
                if set_fg {
                    color_parameters(&mut self.bytes, cell.fg, false)?;
                    foreground = Some(cell.fg);
                }
                if set_bg {
                    if set_fg {
                        self.bytes.push(b';');
                    }
                    color_parameters(&mut self.bytes, bg, true)?;
                    background = Some(bg);
                }
                self.bytes.push(b'm');
            }
            self.bytes.extend_from_slice(symbol.as_bytes());
        }
        self.bytes.extend_from_slice(b"\x1b[0;59m");
        self.inner.write_all(&self.bytes)
    }
    fn hide_cursor(&mut self) -> io::Result<()> {
        self.inner.hide_cursor()
    }
    fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.show_cursor()
    }
    fn get_cursor_position(&mut self) -> io::Result<Position> {
        self.inner.get_cursor_position()
    }
    fn set_cursor_position<P: Into<Position>>(&mut self, pos: P) -> io::Result<()> {
        self.inner.set_cursor_position(pos)
    }
    fn clear(&mut self) -> io::Result<()> {
        self.inner.clear()
    }
    fn clear_region(&mut self, region: ClearType) -> io::Result<()> {
        self.inner.clear_region(region)
    }
    fn append_lines(&mut self, n: u16) -> io::Result<()> {
        self.inner.append_lines(n)
    }
    fn size(&self) -> io::Result<Size> {
        self.inner.size()
    }
    fn window_size(&mut self) -> io::Result<WindowSize> {
        self.inner.window_size()
    }
    fn flush(&mut self) -> io::Result<()> {
        Backend::flush(&mut self.inner)
    }
}

fn color_parameters(bytes: &mut Vec<u8>, color: Color, background: bool) -> io::Result<()> {
    match color {
        Color::Indexed(index) => write!(bytes, "{};5;{index}", if background { 48 } else { 38 }),
        Color::Reset => write!(bytes, "{}", if background { 49 } else { 39 }),
        _ => unreachable!("draw validates indexed colors before encoding"),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::{
        color::{OutputPolicy, adapt_buffer},
        model::{RENDERER_NAMES, TimerDetails},
        renderers::Renderers,
    };
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Color, Modifier},
    };

    fn streams(cells: &[(u16, u16, &Cell)], compact: bool) -> Vec<u8> {
        crossterm::style::force_color_output(true);
        let mut bytes = Vec::new();
        let mut backend = OutputBackend::new(&mut bytes, compact);
        backend.draw(cells.iter().copied()).unwrap();
        bytes
    }

    // Decode the emitted cursor/SGR subset into visible upper/lower samples and glyphs.
    // The oracle is Crossterm's stream, independent of the compact encoder.
    fn screen(bytes: &[u8], width: u16, height: u16) -> Vec<(String, Color, Color)> {
        let mut cells = vec![
            (" ".to_string(), Color::Reset, Color::Reset);
            usize::from(width) * usize::from(height)
        ];
        let (mut x, mut y) = (0, 0);
        let (mut fg, mut bg) = (Color::Reset, Color::Reset);
        let text = std::str::from_utf8(bytes).unwrap();
        let mut chars = text.chars();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                assert_eq!(chars.next(), Some('['));
                let mut args = String::new();
                let command = loop {
                    let c = chars.next().unwrap();
                    if c.is_ascii_alphabetic() {
                        break c;
                    }
                    args.push(c);
                };
                let p: Vec<u16> = args.split(';').map(|p| p.parse().unwrap()).collect();
                match command {
                    'H' => {
                        y = p[0] - 1;
                        x = p[1] - 1;
                    }
                    'C' => x += p[0],
                    'm' => {
                        let mut i = 0;
                        while i < p.len() {
                            match p[i] {
                                0 => {
                                    fg = Color::Reset;
                                    bg = Color::Reset;
                                }
                                39 => fg = Color::Reset,
                                49 => bg = Color::Reset,
                                59 => {} // underline color reset; no underlined cells in this path
                                channel @ (38 | 48) => {
                                    assert_eq!(p[i + 1], 5);
                                    let color = Color::Indexed(p[i + 2] as u8);
                                    if channel == 38 {
                                        fg = color;
                                    } else {
                                        bg = color;
                                    }
                                    i += 2;
                                }
                                other => panic!("unexpected SGR {other}"),
                            }
                            i += 1;
                        }
                    }
                    other => panic!("unexpected command {other}"),
                }
            } else {
                assert!(x < width && y < height, "cursor escaped: {x},{y}");
                let visible = match ch {
                    '▀' => ("pixel".to_string(), fg, bg),
                    '▄' => ("pixel".to_string(), bg, fg),
                    ' ' => ("pixel".to_string(), bg, bg),
                    _ => (ch.to_string(), fg, bg),
                };
                cells[usize::from(y) * usize::from(width) + usize::from(x)] = visible;
                x += 1;
            }
        }
        assert_eq!((fg, bg), (Color::Reset, Color::Reset), "styles must reset");
        cells
    }

    #[test]
    fn compact_indexed_output_preserves_rendered_pixels_and_glyphs() {
        let mut renderers = Renderers::new();
        for name in RENDERER_NAMES {
            for (w, h) in [(20, 8), (80, 24)] {
                let area = Rect::new(0, 0, w, h);
                let mut previous = Buffer::empty(area);
                for elapsed in [0.5, 6.0, 8.8, 20.0, 35.0] {
                    let mut frame = Buffer::empty(area);
                    renderers.render(
                        name,
                        TimerDetails {
                            elapsed,
                            duration: 60.0,
                        },
                        area,
                        &mut frame,
                    );
                    adapt_buffer(&mut frame, OutputPolicy::Ansi256);
                    let diff = previous.diff(&frame);
                    assert_eq!(
                        screen(&streams(&diff, true), w, h),
                        screen(&streams(&diff, false), w, h),
                        "{name} at {elapsed}s {w}x{h}"
                    );
                    previous = frame;
                }
            }
        }
    }

    #[test]
    fn compact_output_reduces_sparse_solid_pixel_commands() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 210, 78));
        for y in 1..77 {
            for x in (3..207).step_by(7) {
                let color = Color::Indexed(196 + x as u8 % 6);
                buf[(x, y)].set_symbol("▀").set_fg(color).set_bg(color);
            }
        }
        let old = Buffer::empty(buf.area);
        let diff = old.diff(&buf);
        let normal = streams(&diff, false);
        let compact = streams(&diff, true);
        assert_eq!(screen(&normal, 210, 78), screen(&compact, 210, 78));
        assert!(
            compact.len() * 100 < normal.len() * 70,
            "{} vs {} bytes",
            compact.len(),
            normal.len()
        );
    }

    #[test]
    fn styled_and_rgb_updates_keep_the_crossterm_path() {
        for (fg, modifier) in [
            (Color::Rgb(12, 34, 56), Modifier::empty()),
            (Color::Indexed(196), Modifier::BOLD),
        ] {
            let mut cell = Cell::default();
            cell.set_symbol("x");
            cell.fg = fg;
            cell.modifier = modifier;
            let diff = vec![(0, 0, &cell)];
            assert_eq!(streams(&diff, true), streams(&diff, false));
        }
    }

    #[test]
    fn compact_output_handles_partial_writes_and_propagates_errors() {
        struct ShortWriter(Vec<u8>);
        impl Write for ShortWriter {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                let n = bytes.len().min(3);
                self.0.extend_from_slice(&bytes[..n]);
                Ok(n)
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        struct BrokenWriter;
        impl Write for BrokenWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("broken terminal"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut cell = Cell::default();
        cell.set_symbol("▀")
            .set_fg(Color::Indexed(196))
            .set_bg(Color::Indexed(196));
        let cells = [(4, 3, &cell)];
        let mut short = ShortWriter(Vec::new());
        OutputBackend::new(&mut short, true)
            .draw(cells.into_iter())
            .unwrap();
        assert_eq!(short.0, streams(&cells, true));
        for compact in [false, true] {
            let error = OutputBackend::new(BrokenWriter, compact)
                .draw(cells.into_iter())
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::Other);
        }
    }

    #[test]
    #[ignore = "release-mode output measurement"]
    fn measure_lava_output() {
        let mut renderers = Renderers::new();
        for (w, h) in [(120, 40), (210, 78)] {
            let area = Rect::new(0, 0, w, h);
            let mut previous = Buffer::empty(area);
            let (mut normal_bytes, mut compact_bytes) = (0, 0);
            for index in 0..180 {
                let mut frame = Buffer::empty(area);
                renderers.render(
                    "lava",
                    TimerDetails {
                        elapsed: 0.5 + f64::from(index) / 60.0,
                        duration: 60.0,
                    },
                    area,
                    &mut frame,
                );
                adapt_buffer(&mut frame, OutputPolicy::Ansi256);
                let diff = previous.diff(&frame);
                normal_bytes += streams(&diff, false).len();
                compact_bytes += streams(&diff, true).len();
                previous = frame;
            }
            println!(
                "{w}x{h}, same 180 indexed lava frames: {} -> {} bytes/frame ({:.1}% fewer)",
                normal_bytes / 180,
                compact_bytes / 180,
                100.0 * (1.0 - compact_bytes as f64 / normal_bytes as f64)
            );
        }
    }
}
