//! The `fdu` binary's deliberately thin process shim.

use std::process::ExitCode;

#[global_allocator]
static ALLOCATOR: fdu_core::counters::alloc::CountingAlloc<std::alloc::System> =
    fdu_core::counters::system_allocator();

/// The machine form of the counters, behind a sentinel a test can find.
///
/// Same shape as `__FDU_SCAN_DIAGNOSTICS__=`: a versioned payload on stderr, outside the
/// report envelope, so a cost assertion never depends on the report's schema and a report
/// consumer never has to skip past instrumentation.
const COUNTERS_PREFIX: &str = "__FDU_COUNTERS__=";

fn main() -> ExitCode {
    let mut measurement = fdu_core::counters::Measurement::from_env();
    let exit = ExitCode::from(fdu::run_process(std::env::args_os()));
    if let Some((report, machine)) = measurement.close() {
        eprint!("{report}");
        eprintln!("{COUNTERS_PREFIX}{machine}");
    }
    exit
}
