use std::process::Command;

fn pace(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pace"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_lists_every_original_renderer_and_tools() {
    let output = pace(&["--help"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for name in pace::model::RENDERER_NAMES {
        assert!(text.contains(name), "{name}");
    }
    for name in ["preview", "allfonts", "colorblocks", "version"] {
        assert!(text.contains(name));
    }
    for option in [
        "--show-fps",
        "--no-color",
        "--color-depth auto|truecolor|256|16",
    ] {
        assert!(text.contains(option));
    }
}

#[test]
fn help_explains_the_original_typescript_tools() {
    for args in [
        vec![],
        vec!["--help"],
        vec!["tools"],
        vec!["tools", "--help"],
        vec!["help", "tools"],
    ] {
        let output = pace(&args);
        assert!(output.status.success(), "{args:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        for description in [
            "Preview all renderers",
            "FIGlet font samples and timer widths",
            "color gradients and palettes",
            "Print the Pace version",
        ] {
            assert!(text.contains(description), "{args:?}: {description}");
        }
    }
}

#[test]
fn preview_keeps_the_original_tools_panel_frames() {
    let output = pace(&["tools", "preview", "--no-color"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let panels = pace::renderers::PREVIEW_RENDERERS.len();
    for corner in ['┌', '┐', '└', '┘'] {
        assert_eq!(text.matches(corner).count(), panels, "missing {corner}");
    }
    assert!(text.contains('│'));
}

#[test]
fn version_tool_runs_without_a_terminal() {
    let output = pace(&["tools", "version"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "0.3.0");
}

#[test]
fn invalid_input_exits_with_a_useful_error_before_terminal_setup() {
    for args in [
        &["wat", "pie"][..],
        &["1s", "nope"],
        &["tools", "wat"],
        &["tools", "version", "extra"],
    ] {
        let output = pace(args);
        assert!(!output.status.success(), "{args:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("pace:"), "{error}");
        assert!(!output.stdout.contains(&0x1b));
    }
}

#[test]
fn timer_refuses_piped_output_without_changing_terminal_modes() {
    let output = pace(&["1s", "pie"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("terminal")
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn preview_contains_all_eleven_non_shuffle_renderers() {
    let output = pace(&["tools", "preview"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    for name in pace::renderers::PREVIEW_RENDERERS {
        let title = format!("{}{}", name[..1].to_uppercase(), &name[1..]);
        assert!(text.contains(&title), "{title}");
    }
}

#[test]
fn font_and_color_catalog_tools_preserve_original_catalogs() {
    let output = pace(&["tools", "allfonts"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        text.lines()
            .filter(|s| s.starts_with(":: ") && s.ends_with(" chars) ::"))
            .count(),
        289
    );
    assert!(text.contains("ANSI_REGULAR"));
    assert!(text.contains("SLANT_RELIEF"));
    let output = pace(&["tools", "colorblocks"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for heading in [
        "BLUE-GREEN PALETTE",
        "PURPLE-GREEN PALETTE",
        "RED-YELLOW PALETTE",
        "MONOCHROME PALETTE",
        "GREEN_1_TO_CYAN_1",
    ] {
        assert!(text.contains(heading), "{heading}");
    }
}

#[test]
fn legacy_parser_characterization() {
    use pace::cli::{self, Command as ParsedCommand};
    use std::time::Duration;
    for (args, renderer) in [
        (vec!["3m10s", "pie"], "pie"),
        (vec!["timer", "3m10s", "pie"], "pie"),
        (vec!["3m10s:pie"], "pie"),
        (vec!["3m10s:pie", "bar"], "pie"),
        (vec!["3m10s"], "shuffle"),
    ] {
        let args = args.into_iter().map(String::from).collect::<Vec<_>>();
        assert_eq!(
            cli::parse(&args).unwrap().command,
            ParsedCommand::Timer {
                duration: Duration::from_secs(190),
                renderer: renderer.into()
            }
        );
    }
    for (args, expected) in [
        (vec![], ParsedCommand::Help),
        (vec!["help"], ParsedCommand::Help),
        (vec!["garbage", "--help"], ParsedCommand::Help),
        (vec!["--version"], ParsedCommand::Version),
        (vec!["tools", "preview"], ParsedCommand::Preview),
        (vec!["tools", "allfonts"], ParsedCommand::AllFonts),
        (vec!["tools", "colorblocks"], ParsedCommand::ColorBlocks),
        (vec!["tools", "version"], ParsedCommand::Version),
    ] {
        let args = args.into_iter().map(String::from).collect::<Vec<_>>();
        assert_eq!(cli::parse(&args).unwrap().command, expected);
    }
}

#[test]
fn output_flags_parse_anywhere_and_preserve_timer_operands() {
    use pace::{
        cli,
        color::{ColorDepth, OutputOptions},
    };
    for args in [
        vec![
            "--color-depth",
            "truecolor",
            "timer",
            "3m10s",
            "pie",
            "--show-fps",
        ],
        vec![
            "timer",
            "--show-fps",
            "3m10s:pie",
            "--color-depth=truecolor",
        ],
    ] {
        let args = args.into_iter().map(String::from).collect::<Vec<_>>();
        let parsed = cli::parse(&args).unwrap();
        assert_eq!(
            parsed.options,
            OutputOptions {
                color_depth: ColorDepth::Truecolor,
                no_color: false,
                show_fps: true
            }
        );
        assert_eq!(
            parsed.command,
            cli::Command::Timer {
                duration: std::time::Duration::from_secs(190),
                renderer: "pie".into()
            }
        );
    }
    for args in [
        vec!["1s", "--color-depth"],
        vec!["1s", "--color-depth="],
        vec!["1s", "--color-depth=8"],
        vec!["1s", "--color-depth=16", "--color-depth=auto"],
        vec!["1s", "--no-color", "--no-color"],
        vec!["1s", "--show-fps", "--show-fps"],
        vec!["1s", "--wat"],
        vec!["tools", "preview", "--show-fps"],
    ] {
        let args = args.into_iter().map(String::from).collect::<Vec<_>>();
        assert!(cli::parse(&args).is_err(), "{args:?}");
    }
}

#[test]
fn every_tool_honors_pipe_explicit_depth_and_no_color() {
    for tool in ["preview", "allfonts", "colorblocks"] {
        for (flags, expected) in [
            (vec![], "mono"),
            (vec!["--color-depth=auto"], "mono"),
            (vec!["--color-depth=truecolor"], "rgb"),
            (vec!["--color-depth=256"], "256"),
            (vec!["--color-depth=16"], "16"),
            (vec!["--color-depth=truecolor", "--no-color"], "mono"),
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_pace"))
                .args(["tools", tool])
                .args(flags)
                .env("TERM", "xterm-256color")
                .env("COLORTERM", "truecolor")
                .env("NO_COLOR", "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{tool} {expected}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let text = String::from_utf8(output.stdout).unwrap();
            match expected {
                "mono" => assert!(!text.contains('\x1b'), "{tool}"),
                "rgb" => {
                    assert!(text.contains("38;2;"), "{tool}");
                    assert!(!text.contains("\x1b[38;5;") && !text.contains("\x1b[48;5;"));
                }
                "256" => {
                    assert!(text.contains("38;5;"), "{tool}");
                    assert!(!text.contains("\x1b[38;2;") && !text.contains("\x1b[48;2;"));
                }
                "16" => {
                    assert!(text.contains('\x1b'), "{tool}");
                    assert!(!text.contains("\x1b[38;"));
                    assert!(!text.contains("\x1b[48;"));
                }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn shader_fields_are_registered_for_selection_preview_and_shuffle() {
    use pace::{
        cli::{self, Command as CliCommand},
        renderers::PREVIEW_RENDERERS,
    };
    let output = pace(&["tools", "preview"]);
    assert!(output.status.success());
    let preview = String::from_utf8(output.stdout).unwrap();
    for name in ["aurora", "tide", "interference"] {
        assert!(PREVIEW_RENDERERS.contains(&name));
        for args in [
            vec!["5m".into(), name.into()],
            vec!["timer".into(), "5m".into(), name.into()],
            vec![format!("5m:{name}")],
        ] {
            assert!(
                matches!(cli::parse(&args).map(|parsed| parsed.command), Ok(CliCommand::Timer { renderer, .. }) if renderer == name)
            );
        }
        let title = format!("{}{}", name[..1].to_uppercase(), &name[1..]);
        assert!(preview.contains(&title), "{name}");
    }
}

#[test]
fn preview_preserves_true_color_and_half_block_backgrounds() {
    let output = pace(&["tools", "preview", "--color-depth=truecolor"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("\x1b[38;2;"), "missing RGB foregrounds");
    assert!(text.contains("\x1b[48;2;"), "missing RGB backgrounds");
    assert!(text.contains('▀'), "missing half-block detail");
}
