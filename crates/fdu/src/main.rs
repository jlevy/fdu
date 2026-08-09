//! The `fdu` binary.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;

use fdu::cli::{Cli, RunOutcome};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let result = cli.run(&mut out).and_then(|outcome| {
        out.flush()?;
        Ok(outcome)
    });

    match result {
        Ok(RunOutcome::Complete) => ExitCode::SUCCESS,
        Ok(RunOutcome::Partial) if cli.allow_partial => ExitCode::SUCCESS,
        Ok(RunOutcome::Partial) => ExitCode::from(2),
        Err(err) => {
            // A closed pipe is what `fdu | head` looks like from in here. That is the
            // caller being done reading, not a failure worth a message or a nonzero
            // status.
            if let Some(io_err) = err.downcast_ref::<io::Error>() {
                if io_err.kind() == io::ErrorKind::BrokenPipe {
                    return ExitCode::SUCCESS;
                }
            }
            eprintln!("fdu: {err}");
            for cause in err.chain().skip(1) {
                eprintln!("  caused by: {cause}");
            }
            ExitCode::FAILURE
        }
    }
}
