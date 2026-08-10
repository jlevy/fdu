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
    let stderr = io::stderr();
    let mut diagnostic = stderr.lock();

    finish(result, cli.allow_partial, &mut diagnostic)
}

fn finish(
    result: anyhow::Result<RunOutcome>,
    allow_partial: bool,
    diagnostic: &mut dyn Write,
) -> ExitCode {
    match result {
        Ok(RunOutcome::Complete) => ExitCode::SUCCESS,
        Ok(RunOutcome::Partial) if allow_partial => ExitCode::SUCCESS,
        Ok(RunOutcome::Partial) => ExitCode::from(2),
        Err(err) => {
            // A closed pipe is what `fdu | head` looks like from in here. That is the
            // caller being done reading, not a failure worth a message or a nonzero
            // status.
            if is_broken_pipe(&err) {
                return ExitCode::SUCCESS;
            }
            let _ = writeln!(diagnostic, "fdu: {err}");
            for cause in err.chain().skip(1) {
                let _ = writeln!(diagnostic, "  caused by: {cause}");
            }
            ExitCode::FAILURE
        }
    }
}

fn is_broken_pipe(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|io_error| io_error.kind() == io::ErrorKind::BrokenPipe)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broken_pipe_is_success_without_a_diagnostic() {
        let broken_pipe =
            anyhow::Error::new(io::Error::new(io::ErrorKind::BrokenPipe, "reader closed"))
                .context("render output");
        let mut diagnostic = Vec::new();

        let code = finish(Err(broken_pipe), false, &mut diagnostic);

        assert_eq!(code, ExitCode::SUCCESS);
        assert!(diagnostic.is_empty());
    }

    #[test]
    fn run_outcomes_have_stable_exit_codes() {
        let mut diagnostic = Vec::new();

        assert_eq!(finish(Ok(RunOutcome::Complete), false, &mut diagnostic), ExitCode::SUCCESS);
        assert_eq!(finish(Ok(RunOutcome::Partial), false, &mut diagnostic), ExitCode::from(2));
        assert_eq!(finish(Ok(RunOutcome::Partial), true, &mut diagnostic), ExitCode::SUCCESS);
        assert!(diagnostic.is_empty());
    }
}
