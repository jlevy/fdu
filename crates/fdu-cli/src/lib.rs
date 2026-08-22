//! The `fdu` command line, built on `fdu`'s public API.
//!
//! Lives in its own crate so that "the CLI invents nothing" is decided by the compiler
//! rather than asserted by review: `crate::` does not reach the library from here, so
//! anything the command line needs must be public API, and any future private need
//! becomes a visible act of making something public rather than a `pub(crate)` nobody
//! sees.
//!
//! A library as well as a binary, because the Python wheel's `fdu` console script runs
//! this command and so needs [`run_process`] callable rather than only executable.

mod cli;

pub use crate::cli::run_process;
