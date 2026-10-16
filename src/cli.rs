use crate::{
    color::{ColorDepth, OutputOptions},
    model::RENDERER_NAMES,
};
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum Command {
    Timer {
        duration: Duration,
        renderer: String,
    },
    Preview,
    AllFonts,
    ColorBlocks,
    Version,
    Help,
    ToolsHelp,
}

#[derive(Debug, PartialEq)]
pub struct Parsed {
    pub command: Command,
    pub options: OutputOptions,
}

pub fn parse(args: &[String]) -> Result<Parsed, String> {
    let mut options = OutputOptions::default();
    // Preserve the legacy help escape hatch, including otherwise invalid operands.
    if args.is_empty() || args.iter().any(|s| s == "--help" || s == "-h") || args[0] == "help" {
        return Ok(Parsed {
            command: match args {
                [first, ..] if first == "tools" => Command::ToolsHelp,
                [first, second, ..] if first == "help" && second == "tools" => Command::ToolsHelp,
                _ => Command::Help,
            },
            options,
        });
    }
    let mut operands = Vec::new();
    let mut depth_seen = false;
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        if arg == "--show-fps" {
            if options.show_fps {
                return Err("Repeated --show-fps".into());
            }
            options.show_fps = true;
        } else if arg == "--color-depth" || arg.starts_with("--color-depth=") {
            if depth_seen {
                return Err("Repeated --color-depth".into());
            }
            depth_seen = true;
            let value = if let Some(value) = arg.strip_prefix("--color-depth=") {
                value
            } else {
                index += 1;
                args.get(index)
                    .map(String::as_str)
                    .ok_or("Missing --color-depth value")?
            };
            options.color_depth = match value {
                "auto" => ColorDepth::Auto,
                "truecolor" => ColorDepth::Truecolor,
                "256" => ColorDepth::Ansi256,
                "16" => ColorDepth::Ansi16,
                "1" => ColorDepth::Monochrome,
                _ => {
                    return Err(format!(
                        "Invalid --color-depth '{value}'; choose auto, truecolor, 256, 16, or 1"
                    ));
                }
            };
        } else if arg.starts_with('-') && arg != "--version" {
            return Err(format!("Unknown option '{arg}'"));
        } else {
            operands.push(args[index].clone());
        }
        index += 1;
    }
    let command = parse_command(&operands)?;
    if options.show_fps && !matches!(command, Command::Timer { .. }) {
        return Err("--show-fps is only supported for timers".into());
    }
    Ok(Parsed { command, options })
}

fn parse_command(args: &[String]) -> Result<Command, String> {
    if args.is_empty() || args.iter().any(|s| s == "--help" || s == "-h") || args[0] == "help" {
        return Ok(Command::Help);
    }
    if args == ["--version"] {
        return Ok(Command::Version);
    }
    if args[0] == "tools" {
        if args.len() == 1 {
            return Ok(Command::ToolsHelp);
        }
        if args.len() != 2 {
            return Err("Expected: pace tools <preview|allfonts|colorblocks|version>".into());
        }
        return match args[1].as_str() {
            "preview" => Ok(Command::Preview),
            "allfonts" => Ok(Command::AllFonts),
            "colorblocks" => Ok(Command::ColorBlocks),
            "version" => Ok(Command::Version),
            other => Err(format!("Unknown tool '{other}'")),
        };
    }
    if args.len() > 2 {
        return Err("Expected: pace DURATION [RENDERER] [OPTIONS]".into());
    }
    let duration = parse_duration(&args[0])?;
    let renderer = args.get(1).map(String::as_str).unwrap_or("shuffle");
    if !RENDERER_NAMES.contains(&renderer) {
        return Err(format!(
            "Unknown renderer '{renderer}'. Choose: {}",
            RENDERER_NAMES.join(", ")
        ));
    }
    Ok(Command::Timer {
        duration,
        renderer: renderer.into(),
    })
}

/// Supports the documented integer and decimal minute/second forms, e.g. 3m10s and 2.5m.
pub fn parse_duration(input: &str) -> Result<Duration, String> {
    let invalid =
        || format!("Invalid duration '{input}'. Use minutes and seconds, e.g. 3m10s or 2.5m");
    let mut rest = input;
    let mut seconds = 0.0;
    let mut had_minutes = false;
    let mut had_seconds = false;
    while !rest.is_empty() {
        let end = rest
            .find(|ch: char| !ch.is_ascii_digit() && ch != '.')
            .ok_or_else(invalid)?;
        let value = rest[..end].parse::<f64>().map_err(|_| invalid())?;
        if !value.is_finite() || value < 0.0 {
            return Err(invalid());
        }
        match rest.as_bytes()[end] {
            b'm' if !had_minutes && !had_seconds => {
                had_minutes = true;
                seconds += value * 60.0;
            }
            b's' if !had_seconds => {
                had_seconds = true;
                seconds += value;
            }
            _ => return Err(invalid()),
        }
        rest = &rest[end + 1..];
    }
    if !had_minutes && !had_seconds {
        return Err(invalid());
    }
    Duration::try_from_secs_f64(seconds).map_err(|_| invalid())
}

const TOOL_DESCRIPTIONS: &str =
    "  preview      Preview all renderers in framed pairs (excluding shuffle)
  allfonts     Display FIGlet font samples and timer widths
  colorblocks  Display color gradients and palettes
  version      Print the Pace version";

pub fn tools_help() -> String {
    format!(
        "Pace tools — explore renderers, fonts, and colors

USAGE
  pace tools <preview|allfonts|colorblocks|version>

TOOLS
{TOOL_DESCRIPTIONS}

OUTPUT
  --color-depth auto|truecolor|256|16|1  Colors (default: auto; 1: monochrome)
  Redirected output is monochrome by default.

EXAMPLES
  pace tools preview
  pace tools allfonts
  pace tools colorblocks --color-depth 256
  pace tools version
"
    )
}

pub fn help() -> String {
    format!(
        "Pace — visual countdown timer

USAGE
  pace DURATION [RENDERER] [OPTIONS]

DURATION
  Minutes and seconds: 5m, 3m10s, 30s, 2.5m

RENDERERS
  {}
  Default: shuffle (switches every 10 seconds)

OPTIONS
  --help, -h                             Show help
  --version                              Print the Pace version
  --color-depth auto|truecolor|256|16|1  Colors (default: auto; 1: monochrome)
  --show-fps                             Show frame rate in the footer
  Flags may precede or follow operands; each flag may appear once.

CONTROLS
  Space: pause/resume    Ctrl-C: exit

EXAMPLES
  pace 5m
  pace 5m pie
  pace 2.5m lava
  pace 3m10s shuffle
  pace 1m interference --show-fps
",
        RENDERER_NAMES.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn accepts_duration_and_optional_renderer() {
        assert_eq!(
            parse(&args(&["3m10s", "pie"])).unwrap().command,
            Command::Timer {
                duration: Duration::from_secs(190),
                renderer: "pie".into()
            }
        );
        assert_eq!(
            parse(&args(&["2.5m"])).unwrap().command,
            Command::Timer {
                duration: Duration::from_secs(150),
                renderer: "shuffle".into()
            }
        );
    }

    #[test]
    fn validates_complete_duration_instead_of_ignoring_garbage() {
        for value in [
            "", "5", "-1s", "1h", "1mfoo", "1s2m", "1m2m", "1..5s", "1e3s",
        ] {
            assert!(parse_duration(value).is_err(), "{value}");
        }
        assert_eq!(parse_duration("0s").unwrap(), Duration::ZERO);
        assert_eq!(parse_duration("0.05s").unwrap(), Duration::from_millis(50));
    }
}
