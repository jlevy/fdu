//! The `fdu` binary's deliberately thin process shim.

use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(fdu::cli::run_process(std::env::args_os()))
}
