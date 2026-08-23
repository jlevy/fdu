//! The `fdu` command line, and the engine it is built on.
//!
//! The command line lives here and depends on [`fdu_core`] the way any consumer does, so
//! `crate::` does not reach the engine: anything the command line needs is public API, and
//! the compiler decides that on every build rather than a reviewer deciding it in review.
//!
//! The engine is re-exported, so `cargo add fdu` gives a library caller the whole API and
//! there is one name to know for both installing the tool and depending on it.

mod cli;

pub use fdu_core::*;

pub use crate::cli::run_process;
