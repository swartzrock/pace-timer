use crate::model::RENDERER_NAMES;
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
}

pub fn parse(args: &[String]) -> Result<Command, String> {
    if args.is_empty() || args.iter().any(|s| s == "--help" || s == "-h") || args[0] == "help" {
        return Ok(Command::Help);
    }
    if args == ["--version"] {
        return Ok(Command::Version);
    }
    if args[0] == "tools" {
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
    let args = if args[0] == "timer" { &args[1..] } else { args };
    if args.is_empty() || args.len() > 2 {
        return Err("Expected: pace [timer] DURATION [RENDERER]".into());
    }
    let (duration, combined_renderer) = args[0]
        .split_once(':')
        .map_or((args[0].as_str(), None), |(d, r)| (d, Some(r)));
    let renderer = combined_renderer
        .or_else(|| args.get(1).map(String::as_str))
        .unwrap_or("shuffle");
    if !RENDERER_NAMES.contains(&renderer) {
        return Err(format!(
            "Unknown renderer '{renderer}'. Choose: {}",
            RENDERER_NAMES.join(", ")
        ));
    }
    Ok(Command::Timer {
        duration: parse_duration(duration)?,
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

pub fn help() -> String {
    format!(
        "Pace — visual countdown timer\n\nUSAGE\n  pace [timer] DURATION [RENDERER]\n  pace DURATION:RENDERER\n  pace tools <preview|allfonts|colorblocks|version>\n\nDURATION\n  Minutes and seconds: 5m, 3m10s, 30s, 2.5m\n\nRENDERERS\n  {}\n  Default: shuffle (switches every 10 seconds)\n\nCONTROLS\n  Space: pause/resume    Ctrl-C: exit\n\nEXAMPLES\n  pace 5m pie\n  pace timer 2.5m lava\n  pace 3m10s:shuffle\n  pace tools preview\n",
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
    fn accepts_original_command_forms() {
        for values in [
            vec!["3m10s", "pie"],
            vec!["timer", "3m10s", "pie"],
            vec!["3m10s:pie"],
        ] {
            assert_eq!(
                parse(&args(&values)).unwrap(),
                Command::Timer {
                    duration: Duration::from_secs(190),
                    renderer: "pie".into()
                }
            );
        }
        assert_eq!(
            parse(&args(&["2.5m"])).unwrap(),
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
