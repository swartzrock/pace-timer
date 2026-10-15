fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(message) = run(&args) {
        eprintln!("pace: {message}\nTry 'pace --help'.");
        std::process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use pace::cli::Command;
    use std::io::{IsTerminal, Write};
    let parsed = pace::cli::parse(args)?;
    let policy = pace::color::OutputPolicy::resolve(
        parsed.options,
        std::io::stdout().is_terminal(),
        &pace::color::ColorEnvironment::capture(),
    );
    match parsed.command {
        Command::Help => print!("{}", pace::cli::help()),
        Command::Version => println!("{}", env!("CARGO_PKG_VERSION")),
        Command::Timer { duration, renderer } => pace::terminal::run(duration, &renderer)?,
        tool => {
            let mut out = std::io::BufWriter::new(std::io::stdout().lock());
            match tool {
                Command::Preview => pace::tools::preview(&mut out, policy)?,
                Command::AllFonts => pace::tools::all_fonts(&mut out, policy)?,
                Command::ColorBlocks => pace::tools::color_blocks(&mut out, policy)?,
                _ => unreachable!(),
            }
            out.flush()?;
        }
    }
    Ok(())
}
