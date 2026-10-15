fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(message) = run(&args) {
        eprintln!("pace: {message}\nTry 'pace --help'.");
        std::process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use pace::cli::Command;
    use std::io::Write;
    match pace::cli::parse(args)? {
        Command::Help => print!("{}", pace::cli::help()),
        Command::Version => println!("{}", env!("CARGO_PKG_VERSION")),
        Command::Timer { duration, renderer } => pace::terminal::run(duration, &renderer)?,
        tool => {
            let mut out = std::io::BufWriter::new(std::io::stdout().lock());
            match tool {
                Command::Preview => pace::tools::preview(&mut out)?,
                Command::AllFonts => pace::tools::all_fonts(&mut out)?,
                Command::ColorBlocks => pace::tools::color_blocks(&mut out)?,
                _ => unreachable!(),
            }
            out.flush()?;
        }
    }
    Ok(())
}
