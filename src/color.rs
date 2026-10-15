//! Captured terminal capability and output-boundary color adaptation.
use ratatui::{buffer::Buffer, style::Color};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorDepth {
    #[default]
    Auto,
    Truecolor,
    Ansi256,
    Ansi16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutputOptions {
    pub color_depth: ColorDepth,
    pub no_color: bool,
    pub show_fps: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColorEnvironment {
    pub term: Option<String>,
    pub colorterm: Option<String>,
    pub no_color: Option<String>,
}

impl ColorEnvironment {
    pub fn capture() -> Self {
        Self {
            term: std::env::var("TERM").ok(),
            colorterm: std::env::var("COLORTERM").ok(),
            no_color: std::env::var_os("NO_COLOR")
                .map(|value| value.to_string_lossy().into_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputPolicy {
    Truecolor,
    Ansi256,
    Ansi16,
    Monochrome,
}

impl OutputPolicy {
    /// Pure resolution; context such as SSH, tmux, or TERM_PROGRAM is not RGB proof.
    pub fn resolve(options: OutputOptions, is_terminal: bool, env: &ColorEnvironment) -> Self {
        if options.no_color {
            return Self::Monochrome;
        }
        match options.color_depth {
            ColorDepth::Truecolor => return Self::Truecolor,
            ColorDepth::Ansi256 => return Self::Ansi256,
            ColorDepth::Ansi16 => return Self::Ansi16,
            ColorDepth::Auto => {}
        }
        if !is_terminal || env.no_color.as_ref().is_some_and(|value| !value.is_empty()) {
            return Self::Monochrome;
        }
        let term = env.term.as_deref().unwrap_or_default().to_ascii_lowercase();
        let colorterm = env
            .colorterm
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if term == "dumb" {
            Self::Monochrome
        } else if matches!(colorterm.as_str(), "truecolor" | "24bit")
            || ["-direct", "-truecolor", "-24bit"]
                .iter()
                .any(|suffix| term.ends_with(suffix))
        {
            Self::Truecolor
        } else if term.contains("256color") {
            Self::Ansi256
        } else if [
            "xterm", "screen", "tmux", "rxvt", "linux", "ansi", "vt100", "vt220",
        ]
        .iter()
        .any(|name| term == *name || term.starts_with(&format!("{name}-")))
        {
            Self::Ansi16
        } else {
            Self::Monochrome
        }
    }

    pub fn adapt(self, color: Color) -> Color {
        let Some(rgb) = color_rgb(color) else {
            return Color::Reset;
        };
        match self {
            Self::Truecolor => Color::Rgb(rgb.0, rgb.1, rgb.2),
            Self::Ansi256 => Color::Indexed(nearest_index(rgb)),
            Self::Ansi16 => ANSI_COLORS[nearest_ansi(rgb) as usize],
            Self::Monochrome => Color::Reset,
        }
    }
}

const ANSI_RGB: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (128, 0, 0),
    (0, 128, 0),
    (128, 128, 0),
    (0, 0, 128),
    (128, 0, 128),
    (0, 128, 128),
    (192, 192, 192),
    (128, 128, 128),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (0, 0, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];
const ANSI_COLORS: [Color; 16] = [
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::Gray,
    Color::DarkGray,
    Color::LightRed,
    Color::LightGreen,
    Color::LightYellow,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
    Color::White,
];
const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

pub fn indexed_rgb(index: u8) -> Color {
    let (r, g, b) = match index {
        0..=15 => ANSI_RGB[index as usize],
        16..=231 => {
            let n = usize::from(index - 16);
            (LEVELS[n / 36], LEVELS[n / 6 % 6], LEVELS[n % 6])
        }
        _ => {
            let gray = 8 + (index - 232) * 10;
            (gray, gray, gray)
        }
    };
    Color::Rgb(r, g, b)
}

pub fn color_rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Reset => None,
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(index) => color_rgb(indexed_rgb(index)),
        named => ANSI_COLORS
            .iter()
            .position(|color| *color == named)
            .map(|index| ANSI_RGB[index]),
    }
}

fn distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    [
        i32::from(a.0) - i32::from(b.0),
        i32::from(a.1) - i32::from(b.1),
        i32::from(a.2) - i32::from(b.2),
    ]
    .iter()
    .map(|delta| (delta * delta) as u32)
    .sum()
}

fn nearest_ansi(rgb: (u8, u8, u8)) -> u8 {
    (0..16)
        .min_by_key(|index| distance(rgb, ANSI_RGB[*index]))
        .unwrap() as u8
}

fn nearest_index(rgb: (u8, u8, u8)) -> u8 {
    let component = |value: u8| {
        (0..6)
            .min_by_key(|index| value.abs_diff(LEVELS[*index]))
            .unwrap() as u8
    };
    let cube = 16 + 36 * component(rgb.0) + 6 * component(rgb.1) + component(rgb.2);
    let sum = i32::from(rgb.0) + i32::from(rgb.1) + i32::from(rgb.2);
    // Round the mean to the nearest grayscale stop, preferring the lower index on ties.
    let gray = 232 + ((sum - 24 + 14) / 30).clamp(0, 23) as u8;
    [nearest_ansi(rgb), cube, gray]
        .into_iter()
        .min_by_key(|index| {
            (
                distance(rgb, color_rgb(indexed_rgb(*index)).unwrap()),
                *index,
            )
        })
        .unwrap()
}

/// Adapt final colors, retaining text and shape symbols.
pub fn adapt_buffer(buffer: &mut Buffer, policy: OutputPolicy) {
    for cell in &mut buffer.content {
        cell.fg = policy.adapt(cell.fg);
        cell.bg = policy.adapt(cell.bg);
    }
}
