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
    assert_eq!(text.lines().filter(|s| s.starts_with(":: ")).count(), 289);
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
