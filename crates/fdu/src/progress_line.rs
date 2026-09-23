//! The one-line progress indicator: whether a run may draw it.
//!
//! The engine reports how much work a run has done; this module owns every presentation
//! decision about showing that, the way `cli` owns color. The gate is pure and lives
//! here so tests can pin it without a terminal: [`should_draw`], from the `--progress`
//! flag and the [`TerminalFacts`] the process read once.
//!
//! Nothing here writes to a stream. The ticker that does (fdu-hjjj) takes a
//! [`ProgressPlan`] resolved before the report starts and clears the line before
//! anything else is written. Until it lands, the non-test build has no caller for the
//! gate, which the expectation below records; it stops compiling the moment the ticker
//! consumes it, which is the cue to remove it.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by the progress ticker, which is fdu-hjjj")
)]

use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

use clap::ValueEnum;

/// When the indicator may be drawn, as `--progress` spells it.
///
/// Mirrors `--color`'s three words, with one deliberate difference: no value draws into
/// a pipe, a file, a log, or CI. `always` lifts only the machine-format rule, for a
/// person who wants the line while watching JSON arrive at their own terminal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ProgressMode {
    /// Draw for a person at an interactive terminal, for human formats only.
    #[default]
    Auto,
    /// Draw for every format, still only at an interactive terminal.
    Always,
    /// Never draw.
    Never,
}

/// What the process learned about stderr, read once and passed in.
///
/// Read in `run_process` rather than wherever the answer is needed, so a test can say
/// what the terminal is instead of inheriting whatever the test runner's is. The
/// default is the least interactive reading, which is also what every existing test
/// ran under before these facts existed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerminalFacts {
    /// Whether stderr is a terminal.
    pub stderr_is_terminal: bool,
    /// `TERM` as set, or `None` when unset.
    pub term: Option<OsString>,
    /// `CI` as set, or `None` when unset.
    pub ci: Option<OsString>,
    /// Whether the console will interpret the escape sequences a frame is drawn with.
    ///
    /// Always true off Windows. On Windows it is whether virtual terminal processing
    /// could be enabled for the console; a legacy console that refuses would print the
    /// erase sequence as text, so it counts as non-interactive.
    pub vt_enabled: bool,
}

impl TerminalFacts {
    /// Read the facts from the process, once.
    ///
    /// On Windows this asks the console to enable virtual terminal processing, which
    /// `anstyle-query` does for stdout and stderr together: a run whose stdout is a pipe
    /// reads as non-interactive there even with stderr at a console. Off Windows it
    /// reads nothing but stderr and two variables.
    pub fn detect() -> Self {
        let stderr_is_terminal = io::stderr().is_terminal();
        Self {
            stderr_is_terminal,
            term: env::var_os("TERM"),
            ci: env::var_os("CI"),
            vt_enabled: virtual_terminal_enabled(stderr_is_terminal),
        }
    }

    /// Whether a person is watching stderr at a terminal that will render a frame.
    ///
    /// Stderr must be a terminal, `TERM` must be set and not `dumb`, `CI` must be unset
    /// or empty, and the console must accept escape sequences. An agent shell observed
    /// in this repository runs with `TERM=dumb` and no terminal, so no agent detection
    /// is needed beyond this rule.
    pub fn is_interactive(&self) -> bool {
        self.stderr_is_terminal
            && self.term.as_deref().is_some_and(|term| !term.is_empty() && term != "dumb")
            && self.ci.as_deref().is_none_or(OsStr::is_empty)
            && self.vt_enabled
    }
}

/// Whether the console will interpret escape sequences, asked only when it matters.
///
/// Off Windows every terminal does, and asking costs nothing. On Windows the question is
/// only worth a console-mode call when stderr is a terminal at all.
#[cfg(not(windows))]
fn virtual_terminal_enabled(_stderr_is_terminal: bool) -> bool {
    true
}

#[cfg(windows)]
fn virtual_terminal_enabled(stderr_is_terminal: bool) -> bool {
    stderr_is_terminal && anstyle_query::windows::enable_ansi_colors() == Some(true)
}

/// Whether a run draws the indicator at all.
///
/// A non-interactive run never draws, whatever the flag says; a command that walks
/// nothing (`--docs`, `--skill`, `--cache-status`, `--cache-clear`) has nothing to show.
/// On an interactive walking run, `auto` draws for the human formats and not for the
/// machine ones, because some agent harnesses run commands under a pseudo-terminal and a
/// redrawn line would pollute their captured stderr; `always` lifts only that rule.
pub(crate) fn should_draw(
    mode: ProgressMode,
    terminal: &TerminalFacts,
    format_is_machine: bool,
    command_walks: bool,
) -> bool {
    if !command_walks || !terminal.is_interactive() {
        return false;
    }
    match mode {
        ProgressMode::Never => false,
        ProgressMode::Always => true,
        ProgressMode::Auto => !format_is_machine,
    }
}

/// What a run resolved about its indicator before any work started.
///
/// The ticker takes this, not the flag and the facts, so the decision is made once in
/// `Cli::run` beside the color decision and never re-derived deeper in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProgressPlan {
    /// Whether the ticker draws at all.
    pub draw: bool,
    /// The root as the frame shows it.
    pub root: String,
    /// Whether the frame is colored, under the rule that colors fdu's warnings.
    pub color: bool,
}

/// The root as a frame shows it: under `~` when the home directory is a prefix,
/// otherwise the path as given.
///
/// A prefix is a path prefix, component by component, so a sibling of the home
/// directory that merely shares its spelling is left alone. The report shows paths as
/// given and has no abbreviation of its own; this one exists because the root is the
/// first thing on a line that has to fit.
pub(crate) fn display_root(root: &Path, home: Option<&Path>) -> String {
    if let Some(home) = home.filter(|home| !home.as_os_str().is_empty()) {
        if let Ok(rest) = root.strip_prefix(home) {
            if rest.as_os_str().is_empty() {
                return "~".to_string();
            }
            return Path::new("~").join(rest).display().to_string();
        }
    }
    root.display().to_string()
}

/// The home directory the frame abbreviates, from the variables the cache path uses.
pub(crate) fn home_directory() -> Option<PathBuf> {
    let home = env::var_os("HOME").filter(|value| !value.is_empty());
    #[cfg(windows)]
    let home = home.or_else(|| env::var_os("USERPROFILE").filter(|value| !value.is_empty()));
    home.map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interactive() -> TerminalFacts {
        TerminalFacts {
            stderr_is_terminal: true,
            term: Some(OsString::from("xterm-256color")),
            ci: None,
            vt_enabled: true,
        }
    }

    /// Every reading of the four facts, and whether that reading is interactive.
    fn every_terminal() -> Vec<(TerminalFacts, bool)> {
        let mut readings = Vec::new();
        for stderr_is_terminal in [false, true] {
            for term in [None, Some("dumb"), Some(""), Some("xterm-256color")] {
                for ci in [None, Some(""), Some("true")] {
                    for vt_enabled in [false, true] {
                        let facts = TerminalFacts {
                            stderr_is_terminal,
                            term: term.map(OsString::from),
                            ci: ci.map(OsString::from),
                            vt_enabled,
                        };
                        let interactive = stderr_is_terminal
                            && term == Some("xterm-256color")
                            && ci != Some("true")
                            && vt_enabled;
                        readings.push((facts, interactive));
                    }
                }
            }
        }
        readings
    }

    #[test]
    fn interactive_needs_a_terminal_a_real_term_no_ci_and_escape_support() {
        assert!(interactive().is_interactive());
        let readings = every_terminal();
        assert_eq!(readings.len(), 48);
        assert_eq!(readings.iter().filter(|(_, interactive)| *interactive).count(), 2);
        for (facts, interactive) in readings {
            assert_eq!(facts.is_interactive(), interactive, "{facts:?}");
        }
        assert!(!TerminalFacts::default().is_interactive());
    }

    #[test]
    fn gating_covers_every_combination_and_never_draws_non_interactively() {
        let modes = [ProgressMode::Auto, ProgressMode::Always, ProgressMode::Never];
        for (facts, interactive) in every_terminal() {
            for mode in modes {
                for machine in [false, true] {
                    for walks in [false, true] {
                        let expected = interactive
                            && walks
                            && match mode {
                                ProgressMode::Never => false,
                                ProgressMode::Always => true,
                                ProgressMode::Auto => !machine,
                            };
                        assert_eq!(
                            should_draw(mode, &facts, machine, walks),
                            expected,
                            "{facts:?} {mode:?} machine={machine} walks={walks}"
                        );
                        if !interactive {
                            assert!(!should_draw(mode, &facts, machine, walks));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn interactive_gating_draws_human_formats_under_auto_and_everything_under_always() {
        let facts = interactive();
        assert!(should_draw(ProgressMode::Auto, &facts, false, true));
        assert!(!should_draw(ProgressMode::Auto, &facts, true, true));
        assert!(should_draw(ProgressMode::Always, &facts, false, true));
        assert!(should_draw(ProgressMode::Always, &facts, true, true));
        assert!(!should_draw(ProgressMode::Never, &facts, false, true));
        assert!(!should_draw(ProgressMode::Never, &facts, true, true));
        for mode in [ProgressMode::Auto, ProgressMode::Always, ProgressMode::Never] {
            assert!(!should_draw(mode, &facts, false, false), "a non-walking command drew");
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn the_root_is_abbreviated_under_home_by_path_component_only() {
        let home = Some(Path::new("/Users/levy"));
        assert_eq!(display_root(Path::new("/Users/levy/wrk/github"), home), "~/wrk/github");
        assert_eq!(display_root(Path::new("/Users/levy"), home), "~");
        assert_eq!(display_root(Path::new("/Users/levy2/wrk"), home), "/Users/levy2/wrk");
        assert_eq!(display_root(Path::new("/srv/data"), home), "/srv/data");
        assert_eq!(display_root(Path::new("."), home), ".");
        assert_eq!(display_root(Path::new("wrk/github"), home), "wrk/github");
        assert_eq!(display_root(Path::new("/Users/levy/wrk"), None), "/Users/levy/wrk");
        assert_eq!(
            display_root(Path::new("/Users/levy/wrk"), Some(Path::new(""))),
            "/Users/levy/wrk"
        );
    }

    #[cfg(windows)]
    #[test]
    fn the_root_is_abbreviated_under_home_by_path_component_only() {
        let home = Some(Path::new(r"C:\Users\levy"));
        assert_eq!(display_root(Path::new(r"C:\Users\levy\wrk\github"), home), r"~\wrk\github");
        assert_eq!(display_root(Path::new(r"C:\Users\levy"), home), "~");
        assert_eq!(display_root(Path::new(r"C:\Users\levy2\wrk"), home), r"C:\Users\levy2\wrk");
        assert_eq!(display_root(Path::new(r"D:\data"), home), r"D:\data");
        assert_eq!(display_root(Path::new("."), home), ".");
        assert_eq!(display_root(Path::new(r"C:\Users\levy\wrk"), None), r"C:\Users\levy\wrk");
    }

    #[test]
    fn detect_reads_the_process_without_panicking() {
        let facts = TerminalFacts::detect();
        assert_eq!(facts.stderr_is_terminal, io::stderr().is_terminal());
        assert_eq!(facts.term, env::var_os("TERM"));
        assert_eq!(facts.ci, env::var_os("CI"));
        if !cfg!(windows) {
            assert!(facts.vt_enabled);
        }
    }
}
