//! The `fdu` binary's deliberately thin process shim.

use std::process::ExitCode;

#[global_allocator]
static ALLOCATOR: fdu::counters::alloc::CountingAlloc<std::alloc::System> =
    fdu::counters::system_allocator();

fn main() -> ExitCode {
    let measurement = fdu::counters::Measurement::from_env();
    let exit = ExitCode::from(fdu_cli::run_process(std::env::args_os()));
    if let Some(report) = measurement.finish() {
        eprint!("{report}");
    }
    exit
}
