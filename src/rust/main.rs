//! Entry point of the cURL Request Generator CLI.

use std::io::{self, Write};

use curlgenerator::{
    args::Args,
    help,
    run::{Output, run},
};

fn main() -> io::Result<()> {
    let args = Args::parse();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    if args.version {
        writeln!(writer, "{}", env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }

    if args.wants_help() {
        write!(writer, "{}", help::render(help::terminal_width()))?;
        return Ok(());
    }

    let code = run(&args, &Output::detect(), &mut writer)?;
    writer.flush()?;

    if code != 0 {
        std::process::exit(code);
    }

    Ok(())
}
