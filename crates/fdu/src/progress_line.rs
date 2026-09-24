//! The one-line progress indicator: whether a run may draw it, and what a frame says.
//!
//! The engine reports how much work a run has done; this module owns every presentation
//! decision about showing that, the way `cli` owns color. Two decisions are pure and live
//! here so tests can pin them without a terminal: [`should_draw`], from the `--progress`
//! flag and the [`TerminalFacts`] the process read once, and [`render_frame`], from a
//! snapshot's facts, the elapsed time, the spinner step, the width, and the color flag.
//!
//! Nothing here writes to a stream. The ticker in `progress_ticker` takes a
//! [`ProgressPlan`] resolved before the report starts, redraws through [`render_frame`],
//! and clears the line before anything else is written.

use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::ValueEnum;
use clap::builder::styling::{AnsiColor, Style as AnsiStyle};

use fdu_core::report_format::{human_bytes, human_count};

use crate::cli::paint;

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
    /// Whether an unset `TERM` still allows drawing.
    ///
    /// True on Windows, whose consoles (cmd, PowerShell, Windows Terminal) set no `TERM`;
    /// there whether the console accepts escape sequences is the terminal test. `dumb`
    /// still means non-interactive everywhere.
    pub term_may_be_unset: bool,
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
            term_may_be_unset: cfg!(windows),
            ci: env::var_os("CI"),
            vt_enabled: virtual_terminal_enabled(stderr_is_terminal),
        }
    }

    /// Whether a person is watching stderr at a terminal that will render a frame.
    ///
    /// Stderr must be a terminal, `TERM` must be set and not `dumb` (or, on Windows,
    /// may be unset), `CI` must be unset or empty, and the console must accept escape
    /// sequences. A `TERM` set to nothing is an unset one: the two are the same thing
    /// everywhere a shell exports variables, so they get the same answer on every
    /// platform. An agent shell observed in this repository runs with `TERM=dumb` and
    /// no terminal, so no agent detection is needed beyond this rule.
    pub fn is_interactive(&self) -> bool {
        let term_allows = match self.term.as_deref().filter(|term| !term.is_empty()) {
            None => self.term_may_be_unset,
            Some(term) => term != "dumb",
        };
        self.stderr_is_terminal
            && term_allows
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

/// The phase a run is in, as the frame names it.
///
/// A local mirror of the engine's phase, so the renderer is a pure function the
/// integration maps a snapshot onto rather than a consumer of the engine's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Reading a snapshot from the cache.
    Loading,
    /// Walking the tree cold.
    Scanning,
    /// Verifying a loaded snapshot against the tree.
    Revalidating,
    /// Reading file bodies.
    Analyzing,
    /// Writing the snapshot.
    Saving,
    /// Assembling the index once the walk is over.
    Indexing,
    /// Building the answer from the index.
    Summarizing,
}

impl Phase {
    /// The longest phase word, which every other one is padded to.
    const PADDED_WIDTH: usize = 12;

    const fn word(self) -> &'static str {
        match self {
            Self::Loading => "Loading",
            Self::Scanning => "Scanning",
            Self::Revalidating => "Revalidating",
            Self::Analyzing => "Analyzing",
            Self::Saving => "Saving",
            Self::Indexing => "Indexing",
            Self::Summarizing => "Summarizing",
        }
    }
}

/// The facts a frame is drawn from: a local mirror of the engine's progress snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameFacts {
    /// The phase most recently entered.
    pub phase: Phase,
    /// Directories walked so far.
    pub directories: u64,
    /// Files walked so far.
    pub files: u64,
    /// Bytes walked so far.
    pub bytes: u64,
    /// Content files analyzed, and the candidates known when analysis began.
    pub analysis: Option<(u64, u64)>,
}

/// The braille dots spinner, one cell per redraw.
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// The narrowest a root is elided to before the facts start giving way.
const MIN_ROOT_COLUMNS: usize = 12;

/// Below this width only the spinner and the phase word are drawn.
const MIN_FULL_WIDTH: usize = 20;

const STYLE_SPINNER: AnsiStyle = AnsiColor::Cyan.on_default();
const STYLE_ROOT: AnsiStyle = AnsiColor::Cyan.on_default();
const STYLE_PHASE: AnsiStyle = AnsiStyle::new().bold();
const STYLE_DIM: AnsiStyle = AnsiColor::BrightBlack.on_default();

/// Elapsed time as the frame shows it: one decimal under a minute (`3.1 s`), then
/// `1 m 04 s`, then `1 h 02 m`.
///
/// Rounded at the unit shown and carried, so `59.96 s` is `1 m 00 s` rather than a
/// `60.0 s` the next frame would contradict.
pub(crate) fn human_elapsed(elapsed: Duration) -> String {
    let millis = elapsed.as_millis();
    let tenths = (millis + 50) / 100;
    if tenths < 600 {
        return format!("{}.{} s", tenths / 10, tenths % 10);
    }
    let seconds = (millis + 500) / 1000;
    if seconds < 3600 {
        return format!("{} m {:02} s", seconds / 60, seconds % 60);
    }
    let minutes = (millis + 30_000) / 60_000;
    format!("{} h {:02} m", minutes / 60, minutes % 60)
}

/// A whole percentage that never reaches 100 before `done` equals `total`.
///
/// Truncated rather than rounded, which is what keeps `99.9%` at `99%`. Nothing of
/// nothing is `100%`: the engine reports `(0, 0)` when the content sidecar answered
/// every candidate and no file needed reading, which is analysis with nothing left to
/// do, not analysis that has not begun. A count past its total is capped at the total.
fn whole_percent(done: u64, total: u64) -> u64 {
    if total == 0 {
        return 100;
    }
    let done = done.min(total);
    u64::try_from(u128::from(done) * 100 / u128::from(total)).unwrap_or(100)
}

/// Columns one character of a root path occupies: one for ASCII, two otherwise.
///
/// Deliberately an overestimate for accented letters, because a frame that reserves a
/// column it does not use is invisible and one that wraps is not.
const fn char_columns(c: char) -> usize {
    if c.is_ascii() { 1 } else { 2 }
}

fn path_columns(text: &str) -> usize {
    text.chars().map(char_columns).sum()
}

/// A root slot: its text, and the columns it occupies.
#[derive(Clone, Debug)]
struct RootSlot {
    text: String,
    columns: usize,
}

impl RootSlot {
    fn new(root: &str) -> Self {
        Self { text: root.to_string(), columns: path_columns(root) }
    }

    /// The root elided in the middle with `…` to at most `target` columns, which must
    /// be fewer than it occupies.
    ///
    /// Whole characters only, so a wide character that would straddle the budget is
    /// left out and the result may come up one column short rather than one over.
    fn elided(root: &str, target: usize) -> Self {
        let budget = target.saturating_sub(1);
        let tail_budget = budget / 2;
        let head_budget = budget - tail_budget;

        let mut head = String::new();
        let mut head_columns = 0;
        for c in root.chars() {
            if head_columns + char_columns(c) > head_budget {
                break;
            }
            head.push(c);
            head_columns += char_columns(c);
        }
        let mut tail = Vec::new();
        let mut tail_columns = 0;
        for c in root.chars().rev() {
            if tail_columns + char_columns(c) > tail_budget {
                break;
            }
            tail.push(c);
            tail_columns += char_columns(c);
        }
        let tail: String = tail.into_iter().rev().collect();
        Self { text: format!("{head}…{tail}"), columns: head_columns + 1 + tail_columns }
    }

    /// Columns beyond the character count: one per wide character.
    fn extra_columns(&self) -> usize {
        self.columns - self.text.chars().count()
    }
}

/// The facts slot, in the shape the shrink order works on.
#[derive(Clone, Debug)]
enum FactsSlot {
    None,
    Walk { files: String, dirs: Option<String>, bytes: Option<String> },
    Analysis { percent: String, done: String, total: String },
}

impl FactsSlot {
    fn drop_dirs(&mut self) {
        if let Self::Walk { dirs, .. } = self {
            *dirs = None;
        }
    }

    fn drop_bytes(&mut self) {
        if let Self::Walk { bytes, .. } = self {
            *bytes = None;
        }
    }
}

/// One frame's slots, before and after shrinking.
#[derive(Clone, Debug)]
struct Slots {
    spinner: char,
    root: Option<RootSlot>,
    phase: &'static str,
    padded: bool,
    facts: FactsSlot,
    elapsed: Option<String>,
}

impl Slots {
    fn full(root: &str, facts: &FrameFacts, elapsed: Duration, step: usize) -> Self {
        let facts_slot = match (facts.phase, facts.analysis) {
            // Indexing keeps the walk's final counts: the walk is over, and the phase word
            // is what changes.
            (Phase::Scanning | Phase::Revalidating | Phase::Indexing | Phase::Summarizing, _) => {
                FactsSlot::Walk {
                    files: human_count(facts.files),
                    dirs: Some(human_count(facts.directories)),
                    bytes: Some(human_bytes(facts.bytes)),
                }
            }
            (Phase::Analyzing, Some((done, total))) => FactsSlot::Analysis {
                percent: format!("{:>3}%", whole_percent(done, total)),
                done: human_count(done),
                total: human_count(total),
            },
            (Phase::Loading | Phase::Saving | Phase::Analyzing, _) => FactsSlot::None,
        };
        Self {
            spinner: SPINNER[step % SPINNER.len()],
            root: Some(RootSlot::new(root)),
            phase: facts.phase.word(),
            padded: true,
            facts: facts_slot,
            elapsed: Some(human_elapsed(elapsed)),
        }
    }

    /// The form for a terminal too narrow for anything else.
    fn minimal(phase: Phase, step: usize) -> Self {
        Self {
            spinner: SPINNER[step % SPINNER.len()],
            root: None,
            phase: phase.word(),
            padded: false,
            facts: FactsSlot::None,
            elapsed: None,
        }
    }

    /// The frame's text in order, each piece with the style it takes when colored.
    fn segments(&self) -> Vec<(String, Option<AnsiStyle>)> {
        let mut segments = vec![(self.spinner.to_string(), Some(STYLE_SPINNER))];
        segments.push((" ".to_string(), None));
        if let Some(root) = &self.root {
            segments.push((root.text.clone(), Some(STYLE_ROOT)));
            segments.push(("  ".to_string(), None));
        }
        segments.push((self.phase.to_string(), Some(STYLE_PHASE)));
        if self.padded {
            let padding = Phase::PADDED_WIDTH.saturating_sub(self.phase.len());
            segments.push((" ".repeat(padding), None));
        }
        match &self.facts {
            FactsSlot::None => {}
            FactsSlot::Walk { files, dirs, bytes } => {
                segments.push((format!("  {files}"), None));
                segments.push((" files".to_string(), Some(STYLE_DIM)));
                if let Some(dirs) = dirs {
                    segments.push((" · ".to_string(), Some(STYLE_DIM)));
                    segments.push((dirs.clone(), None));
                    segments.push((" dirs".to_string(), Some(STYLE_DIM)));
                }
                if let Some(bytes) = bytes {
                    segments.push((" · ".to_string(), Some(STYLE_DIM)));
                    segments.push((bytes.clone(), None));
                }
            }
            FactsSlot::Analysis { percent, done, total } => {
                segments.push((format!("  {percent}  {done}"), None));
                segments.push((" / ".to_string(), Some(STYLE_DIM)));
                segments.push((total.clone(), None));
                segments.push((" files".to_string(), Some(STYLE_DIM)));
            }
        }
        if let Some(elapsed) = &self.elapsed {
            segments.push(("  ".to_string(), None));
            segments.push((elapsed.clone(), Some(STYLE_DIM)));
        }
        segments
    }

    /// The frame as bytes, with adjacent same-style pieces painted as one run.
    fn render(&self, color: bool) -> String {
        let mut out = String::new();
        let mut run: Option<(String, Option<AnsiStyle>)> = None;
        for (text, style) in self.segments() {
            match &mut run {
                Some((current, current_style)) if *current_style == style => {
                    current.push_str(&text);
                }
                _ => {
                    if let Some((current, current_style)) = run.take() {
                        out.push_str(&paint_segment(&current, current_style, color));
                    }
                    run = Some((text, style));
                }
            }
        }
        if let Some((current, current_style)) = run {
            out.push_str(&paint_segment(&current, current_style, color));
        }
        out
    }

    /// Columns the frame occupies, measured without color codes.
    fn columns(&self) -> usize {
        let plain = self.render(false);
        plain.chars().count() + self.root.as_ref().map_or(0, RootSlot::extra_columns)
    }
}

fn paint_segment(text: &str, style: Option<AnsiStyle>, color: bool) -> String {
    match style {
        Some(style) => paint(text, style, color),
        None => text.to_string(),
    }
}

/// One frame, exactly as the Appearance section of the plan specifies it.
///
/// Slots in order: spinner, root, phase word padded to `Revalidating`, the phase's
/// facts, elapsed time. The frame is measured without its color codes, with each
/// non-ASCII character of the root counted as two columns, and always leaves the last
/// column empty; when it is wider than that it shrinks in order until it fits: the
/// phase padding goes, the root is elided in the middle with `…` down to 12 columns,
/// the `dirs` count goes, the bytes go, and below 20 columns only the spinner and the
/// phase word are drawn. A frame never wraps.
///
/// The string excludes the leading `\r\x1b[2K`; the ticker adds it, so a frame is the
/// same bytes whether it is drawn or pinned by a test.
pub(crate) fn render_frame(
    root: &str,
    facts: &FrameFacts,
    elapsed: Duration,
    step: usize,
    width: usize,
    color: bool,
) -> String {
    let limit = width.saturating_sub(1);
    let mut slots = Slots::full(root, facts, elapsed, step);

    if slots.columns() > limit {
        slots.padded = false;
    }
    if slots.columns() > limit {
        let excess = slots.columns() - limit;
        let occupied = slots.root.as_ref().map_or(0, |root| root.columns);
        let target = occupied.saturating_sub(excess).max(MIN_ROOT_COLUMNS);
        if target < occupied {
            slots.root = Some(RootSlot::elided(root, target));
        }
    }
    if slots.columns() > limit {
        slots.facts.drop_dirs();
    }
    if slots.columns() > limit {
        slots.facts.drop_bytes();
    }
    if width < MIN_FULL_WIDTH || slots.columns() > limit {
        slots = Slots::minimal(facts.phase, step);
    }
    if slots.columns() > limit {
        // Narrower than the phase word itself. Nobody reads a terminal this narrow, but
        // the rule is that a frame never wraps, so it is cut rather than allowed to.
        return slots.render(false).chars().take(limit).collect();
    }
    slots.render(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interactive() -> TerminalFacts {
        TerminalFacts {
            stderr_is_terminal: true,
            term: Some(OsString::from("xterm-256color")),
            term_may_be_unset: false,
            ci: None,
            vt_enabled: true,
        }
    }

    /// Every reading of the five facts, and whether that reading is interactive.
    fn every_terminal() -> Vec<(TerminalFacts, bool)> {
        let mut readings = Vec::new();
        for stderr_is_terminal in [false, true] {
            for term in [None, Some("dumb"), Some(""), Some("xterm-256color")] {
                for term_may_be_unset in [false, true] {
                    for ci in [None, Some(""), Some("true")] {
                        for vt_enabled in [false, true] {
                            let facts = TerminalFacts {
                                stderr_is_terminal,
                                term: term.map(OsString::from),
                                term_may_be_unset,
                                ci: ci.map(OsString::from),
                                vt_enabled,
                            };
                            let term_unset = term.is_none_or(str::is_empty);
                            let term_allows =
                                term == Some("xterm-256color") || (term_unset && term_may_be_unset);
                            let interactive = stderr_is_terminal
                                && term_allows
                                && ci != Some("true")
                                && vt_enabled;
                            readings.push((facts, interactive));
                        }
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
        assert_eq!(readings.len(), 96);
        // A real TERM with CI unset or empty, on either platform (4), or a TERM that is
        // unset or empty where the platform allows it, with CI unset or empty (4).
        assert_eq!(readings.iter().filter(|(_, interactive)| *interactive).count(), 8);
        let empty_term = TerminalFacts { term: Some(OsString::new()), ..interactive() };
        assert!(!empty_term.is_interactive(), "off Windows an empty TERM is an unset one");
        assert!(
            TerminalFacts { term_may_be_unset: true, ..empty_term }.is_interactive(),
            "on Windows an empty TERM is an unset one too"
        );
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

    const ROOT: &str = "~/wrk/github";
    /// 38 GiB and 256 MiB, which `human_bytes` shows without a decimal at that scale.
    const BYTES_38_GIB: u64 = 41_070_624_768;

    fn walk(phase: Phase) -> FrameFacts {
        FrameFacts {
            phase,
            directories: 12_041,
            files: 412_309,
            bytes: BYTES_38_GIB,
            analysis: None,
        }
    }

    fn analyzing(done: u64, total: u64) -> FrameFacts {
        FrameFacts {
            phase: Phase::Analyzing,
            analysis: Some((done, total)),
            ..walk(Phase::Analyzing)
        }
    }

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn every_phase_uses_the_same_slots_in_the_same_order() {
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Loading), ms(600), 2, 100, false),
            "⠹ ~/wrk/github  Loading       0.6 s"
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Scanning), ms(3_100), 4, 100, false),
            "⠼ ~/wrk/github  Scanning      412,309 files · 12,041 dirs · 38 GiB  3.1 s"
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Revalidating), ms(1_400), 4, 100, false),
            "⠼ ~/wrk/github  Revalidating  412,309 files · 12,041 dirs · 38 GiB  1.4 s"
        );
        assert_eq!(
            render_frame(ROOT, &analyzing(12_044, 50_110), ms(7_900), 7, 100, false),
            "⠧ ~/wrk/github  Analyzing      24%  12,044 / 50,110 files  7.9 s"
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Saving), ms(8_100), 9, 100, false),
            "⠏ ~/wrk/github  Saving        8.1 s"
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Indexing), ms(3_800), 3, 100, false),
            "⠸ ~/wrk/github  Indexing      412,309 files · 12,041 dirs · 38 GiB  3.8 s"
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Summarizing), ms(8_600), 5, 100, false),
            "⠴ ~/wrk/github  Summarizing   412,309 files · 12,041 dirs · 38 GiB  8.6 s"
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Scanning), ms(3_100), 4, 100, false),
            render_frame(ROOT, &walk(Phase::Scanning), ms(3_100), 4, 100, false),
            "a frame is a pure function of its inputs"
        );
    }

    #[test]
    fn a_colored_frame_paints_each_slot_in_the_report_palette() {
        const CYAN: &str = "\u{1b}[36m";
        const BOLD: &str = "\u{1b}[1m";
        const DIM: &str = "\u{1b}[90m";
        const RESET: &str = "\u{1b}[0m";

        assert_eq!(
            render_frame(ROOT, &walk(Phase::Loading), ms(600), 2, 100, true),
            format!(
                "{CYAN}⠹{RESET} {CYAN}~/wrk/github{RESET}  {BOLD}Loading{RESET}       \
                 {DIM}0.6 s{RESET}"
            )
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Scanning), ms(3_100), 4, 100, true),
            format!(
                "{CYAN}⠼{RESET} {CYAN}~/wrk/github{RESET}  {BOLD}Scanning{RESET}      412,309\
                 {DIM} files · {RESET}12,041{DIM} dirs · {RESET}38 GiB  {DIM}3.1 s{RESET}"
            )
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Revalidating), ms(1_400), 4, 100, true),
            format!(
                "{CYAN}⠼{RESET} {CYAN}~/wrk/github{RESET}  {BOLD}Revalidating{RESET}  412,309\
                 {DIM} files · {RESET}12,041{DIM} dirs · {RESET}38 GiB  {DIM}1.4 s{RESET}"
            )
        );
        assert_eq!(
            render_frame(ROOT, &analyzing(12_044, 50_110), ms(7_900), 7, 100, true),
            format!(
                "{CYAN}⠧{RESET} {CYAN}~/wrk/github{RESET}  {BOLD}Analyzing{RESET}      24%  \
                 12,044{DIM} / {RESET}50,110{DIM} files{RESET}  {DIM}7.9 s{RESET}"
            )
        );
        assert_eq!(
            render_frame(ROOT, &walk(Phase::Saving), ms(8_100), 9, 100, true),
            format!(
                "{CYAN}⠏{RESET} {CYAN}~/wrk/github{RESET}  {BOLD}Saving{RESET}        \
                 {DIM}8.1 s{RESET}"
            )
        );
    }

    /// The full Scanning frame over this root is 105 columns; each width below takes
    /// exactly one more step of the shrink order.
    #[test]
    fn a_frame_shrinks_in_the_specified_order_until_it_fits() {
        let root = "/Volumes/archive/projects/example/repository";
        let facts = walk(Phase::Scanning);
        let frame = |width| render_frame(root, &facts, ms(3_100), 4, width, false);

        let full = "⠼ /Volumes/archive/projects/example/repository  Scanning      \
                    412,309 files · 12,041 dirs · 38 GiB  3.1 s";
        assert_eq!(full.chars().count(), 105);
        assert_eq!(frame(106), full, "fits with the last column empty");
        // 1. The phase padding goes.
        assert_eq!(
            frame(105),
            "⠼ /Volumes/archive/projects/example/repository  Scanning  \
             412,309 files · 12,041 dirs · 38 GiB  3.1 s"
        );
        // 2. The root is elided in the middle, by exactly the excess...
        assert_eq!(
            frame(100),
            "⠼ /Volumes/archive/proj…s/example/repository  Scanning  \
             412,309 files · 12,041 dirs · 38 GiB  3.1 s"
        );
        // ...and no further than 12 columns.
        assert_eq!(
            frame(70),
            "⠼ /Volum…itory  Scanning  412,309 files · 12,041 dirs · 38 GiB  3.1 s"
        );
        // 3. The dirs count goes.
        assert_eq!(frame(69), "⠼ /Volum…itory  Scanning  412,309 files · 38 GiB  3.1 s");
        assert_eq!(frame(56), "⠼ /Volum…itory  Scanning  412,309 files · 38 GiB  3.1 s");
        // 4. The bytes go.
        assert_eq!(frame(55), "⠼ /Volum…itory  Scanning  412,309 files  3.1 s");
        assert_eq!(frame(47), "⠼ /Volum…itory  Scanning  412,309 files  3.1 s");
        // 5. Nothing else can give way, so only the spinner and the phase word remain.
        assert_eq!(frame(46), "⠼ Scanning");
        assert_eq!(frame(20), "⠼ Scanning");
    }

    #[test]
    fn below_twenty_columns_only_the_spinner_and_phase_word_are_drawn() {
        let facts = walk(Phase::Saving);
        assert_eq!(render_frame("~", &facts, ms(100), 0, 26, false), "⠋ ~  Saving        0.1 s");
        assert_eq!(render_frame("~", &facts, ms(100), 0, 20, false), "⠋ ~  Saving  0.1 s");
        assert_eq!(render_frame("~", &facts, ms(100), 0, 19, false), "⠋ Saving");
        assert_eq!(
            render_frame("~", &facts, ms(100), 0, 19, true),
            "\u{1b}[36m⠋\u{1b}[0m \u{1b}[1mSaving\u{1b}[0m"
        );
        let long = walk(Phase::Revalidating);
        assert_eq!(render_frame("~", &long, ms(100), 0, 15, false), "⠋ Revalidating");
        assert_eq!(render_frame("~", &long, ms(100), 0, 10, false), "⠋ Revalid");
        assert_eq!(render_frame("~", &long, ms(100), 0, 0, false), "");
    }

    #[test]
    fn analysis_facts_give_way_only_as_a_whole() {
        let facts = analyzing(12_044, 50_110);
        let frame = |width| render_frame(ROOT, &facts, ms(7_900), 7, width, false);
        assert_eq!(frame(65), "⠧ ~/wrk/github  Analyzing      24%  12,044 / 50,110 files  7.9 s");
        assert_eq!(frame(64), "⠧ ~/wrk/github  Analyzing   24%  12,044 / 50,110 files  7.9 s");
        assert_eq!(frame(62), "⠧ ~/wrk/github  Analyzing   24%  12,044 / 50,110 files  7.9 s");
        assert_eq!(frame(61), "⠧ Analyzing");
    }

    #[test]
    fn percentages_are_whole_and_never_complete_early() {
        let percent = |done, total| {
            let frame = render_frame(ROOT, &analyzing(done, total), ms(0), 0, 100, false);
            frame.split_once("Analyzing     ").expect("the facts follow the phase").1[..4]
                .to_string()
        };
        assert_eq!(percent(0, 50_110), "  0%");
        assert_eq!(percent(3_508, 50_110), "  7%");
        assert_eq!(percent(12_044, 50_110), " 24%");
        assert_eq!(percent(999, 1_000), " 99%");
        assert_eq!(percent(99_999, 100_000), " 99%");
        assert_eq!(percent(50_110, 50_110), "100%");
        assert_eq!(percent(0, 0), "100%", "the sidecar answered everything: nothing left to do");
        assert_eq!(whole_percent(0, 0), 100);
        assert_eq!(whole_percent(7, 5), 100);
        assert_eq!(whole_percent(u64::MAX - 1, u64::MAX), 99);
    }

    #[test]
    fn elapsed_has_one_decimal_under_a_minute_then_minutes_then_hours() {
        assert_eq!(human_elapsed(ms(0)), "0.0 s");
        assert_eq!(human_elapsed(ms(500)), "0.5 s");
        assert_eq!(human_elapsed(ms(3_100)), "3.1 s");
        assert_eq!(human_elapsed(ms(3_149)), "3.1 s");
        assert_eq!(human_elapsed(ms(3_150)), "3.2 s");
        assert_eq!(human_elapsed(ms(59_940)), "59.9 s");
        assert_eq!(human_elapsed(ms(59_960)), "1 m 00 s");
        assert_eq!(human_elapsed(ms(64_000)), "1 m 04 s");
        assert_eq!(human_elapsed(ms(3_599_400)), "59 m 59 s");
        assert_eq!(human_elapsed(ms(3_599_600)), "1 h 00 m");
        assert_eq!(human_elapsed(ms(3_725_000)), "1 h 02 m");
        assert_eq!(human_elapsed(Duration::from_secs(10 * 3600 + 59 * 60 + 59)), "11 h 00 m");
    }

    #[test]
    fn non_ascii_root_characters_count_as_two_columns() {
        let root = "~/Документы/проекты";
        let facts = walk(Phase::Saving);
        let frame = |width| render_frame(root, &facts, ms(8_100), 9, width, false);
        let full = "⠏ ~/Документы/проекты  Saving        8.1 s";
        // 42 characters, of which 16 in the root are wide: 58 columns.
        assert_eq!(full.chars().count(), 42);
        assert_eq!(path_columns(root), 35);
        assert_eq!(frame(59), full);
        assert_eq!(frame(58), "⠏ ~/Документы/проекты  Saving  8.1 s");
        // The elision keeps whole characters: a head budget of 6 columns holds `~/` and
        // two wide letters, a tail budget of 5 holds two wide letters, 11 columns in all.
        assert_eq!(frame(30), "⠏ ~/До…ты  Saving  8.1 s");
        assert_eq!(frame(29), "⠏ ~/До…ты  Saving  8.1 s");
        assert_eq!(frame(28), "⠏ Saving");
    }

    #[test]
    fn the_spinner_advances_one_cell_per_step_and_wraps() {
        let facts = walk(Phase::Saving);
        let spinner = |step| {
            render_frame(ROOT, &facts, ms(0), step, 100, false).chars().next().expect("a frame")
        };
        assert_eq!((0..10).map(spinner).collect::<String>(), "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");
        assert_eq!(spinner(10), '⠋');
        assert_eq!(spinner(23), '⠸');
        assert_eq!(spinner(usize::MAX), '⠴');
    }

    #[test]
    fn an_analyzing_frame_without_analysis_facts_shows_none() {
        let facts = FrameFacts { analysis: None, ..analyzing(0, 0) };
        assert_eq!(
            render_frame(ROOT, &facts, ms(600), 2, 100, false),
            "⠹ ~/wrk/github  Analyzing     0.6 s"
        );
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
