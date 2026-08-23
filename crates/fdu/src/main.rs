//! The `fdu` binary's deliberately thin process shim.

use std::process::ExitCode;

#[global_allocator]
static ALLOCATOR: fdu_core::counters::alloc::CountingAlloc<std::alloc::System> =
    fdu_core::counters::system_allocator();

fn main() -> ExitCode {
    let measurement = fdu_core::counters::Measurement::from_env();
    let exit = ExitCode::from(fdu::run_process(std::env::args_os()));
    if let Some(report) = measurement.finish() {
        eprint!("{report}");
    }
    exit
}
