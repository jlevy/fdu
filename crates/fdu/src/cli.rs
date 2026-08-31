//! The `fdu` command line.
//!
//! The CLI serves two audiences from one binary, and neither is an afterthought:
//!
//! - **Humans** get colored, fixed-column tree output with percentage bars, sensible
//!   defaults, and `NO_COLOR` plus pipe detection so redirection degrades cleanly.
//! - **Agents** get `--help` as the complete source of truth, JSON whose schema is
//!   versioned with the tool, and meaningful exit codes — no pager, no prompts, no
//!   interactive surprises.

use std::ffi::{OsStr, OsString};
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use std::borrow::Cow;

use clap::builder::styling::{AnsiColor, Style as AnsiStyle, Styles};
use clap::{ArgAction, ColorChoice, CommandFactory, FromArgMatches, Parser, ValueEnum};

use fdu_core::content::{AnalysisRequest, AnalysisSet};
use fdu_core::query::{
    AxisNames, Bound, Pattern, Provenance, Query, ReportSource, Selection, SizeMetric, SortKey,
    ViewSpec, parse_size, parse_when, system_time_to_nanos,
};
use fdu_core::report_format;
use fdu_core::report_format::human_count;
use fdu_core::{
    CachePolicy, EntryKind, OpenConfig, ScanConfig, default_cache_path, open_with_pending_save,
};
use fdu_core::{PerformanceSummary, prepare_report, prepare_report_with_scan_diagnostics};

const SKILL_TEMPLATE: &str = include_str!("skills/SKILL.md");

/// Repository-measurement switch for the compact installed-command path.
///
/// This is intentionally not a user-facing CLI option. The performance harness owns
/// both ends of the versioned contract and requires the exact value `1`.
const SCAN_DIAGNOSTICS_ENV: &str = "FDU_SCAN_DIAGNOSTICS";
const SCAN_DIAGNOSTICS_PREFIX: &str = "__FDU_SCAN_DIAGNOSTICS__=";

// ---- the terminal styling system --------------------------------------------------
//
// Five roles, one colour each, and one case convention.  Every human surface — the report,
// the `--docs` guide, and clap's help — draws from this table rather than choosing its own
// colours, which is what makes them read as one tool.
//
//   heading      cyan bold      ALL CAPS, no trailing colon
//   warning      yellow bold
//   error        red bold
//   cause        dimmed         the chain under an error
//   telemetry    bright black   the performance footer, notes, watch rules
//
// Colour applies only when the destination is a live terminal, and never to a machine
// format or under NO_COLOR; `ColorContext` owns that decision and `paint` applies it.

/// The one header style every human surface uses.
///
/// Report view headers, the `--docs` section headers, and clap's help section headings
/// are the same kind of thing — a name introducing a block — so they share a style and a
/// case convention rather than each inventing one. `report_format` re-exports this as the
/// view-header style so there is a single definition to change.
const STYLE_HEADING: AnsiStyle = AnsiColor::Cyan.on_default().bold();
const STYLE_WARNING: AnsiStyle = AnsiColor::Yellow.on_default().bold();
const STYLE_ERROR: AnsiStyle = AnsiColor::Red.on_default().bold();
const STYLE_CAUSE: AnsiStyle = AnsiStyle::new().dimmed();
const STYLE_PERFORMANCE: AnsiStyle = AnsiColor::BrightBlack.on_default();

/// The rule that separates one watch repaint from the one before it.
///
/// Gray for the same reason the performance footer is: it is a frame around the report,
/// not part of the answer, and should not compete with the rows for attention.
const STYLE_WATCH_RULE: AnsiStyle = AnsiColor::BrightBlack.on_default();
const CLI_STYLES: Styles = Styles::styled()
    .header(STYLE_HEADING)
    .usage(STYLE_HEADING)
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .error(STYLE_ERROR)
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Yellow.on_default());

/// The one line `--help` spends on everything `--docs` covers.
///
/// Help stays the flag reference it is supposed to be.  The guide it points at used to
/// be split across `before_help` and `after_help`, which put a page of prose *above* the
/// tool's own description — the reader met the examples before learning what the command
/// was.
const DOCS_POINTER: &str = "Run `fdu --docs` for more help and important usage examples.";

/// The guide `--docs` prints: the ladder, the two axes, and the contracts worth knowing
/// before automating against the output.
const DOCS: &str = r"fdu — a fast, incremental file roll-up engine.

THE LADDER
  Every report is one command, and they form a ladder. Each rung costs more than
  the one above it and tells you more, so stop at the cheapest answer that
  settles your question.

    fdu --view summary PATH             how big is this tree?        no reads
    fdu PATH                            which folders are big?       no reads
    fdu --view types PATH               what kinds of files?         no reads
    fdu --view languages PATH           which languages?             no reads
    fdu --analyze code PATH             how much code?               READS FILES
    fdu --analyze words PATH            how much writing?            READS FILES
    fdu --analyze all --view all PATH   everything there is          READS FILES

TWO FLAGS DO ALL OF IT
  --analyze decides what gets read. Anything but `none` opens and reads every
    eligible file, which is the only setting that makes a run cost more than a
    single metadata walk.
  --view decides what gets printed. It is free: every view is a projection over
    one walk, so asking for more views never touches the filesystem again.

  You rarely need both. Naming analyzers selects a view that displays them, so
  `fdu --analyze code PATH` already prints language rows with lines of code.
  Name --view yourself for a different projection; it always wins.

  A view never turns on an analyzer, because choosing how to look at a result
  should not quietly authorize reading every file in the tree. So a --view that
  displays none of what you asked to read says how much was read for nothing,
  and --view all names any view it had to skip.

MORE COMPOSITIONS
  fdu --view extensions ~/Downloads
  fdu --view types,families --format json .
  fdu --analyze words --view documents .
  fdu --view files --min-size 10M --sort size -n 100 PATH   largest files
  fdu --view files --modified-since 1h --sort mtime PATH    recent changes
  fdu --watch --view files --format jsonl PATH              a tail -f for a tree

  --interval throttles rendering only; change detection is event-driven and
  unaffected by it, so an idle tree costs nothing between changes.

SIX AXES, AND EVERY OPTION BELONGS TO EXACTLY ONE
  Scope      PATH, --scan-depth                         what is scanned and cached
  Content    --analyze none|lines|code|words|all        which file bodies are read
  Selection  --include, --exclude, --depth, --limit     which entries are considered
  View       tree,extensions,types,families,languages,documents,files,summary,all
  Format     --format text|json|jsonl|yaml, --color
  Mode       --cache, --watch, --analysis-workers

CONTENT ANALYSIS
  none       metadata only; source files are never opened (default)
  lines      physical, blank, and nonblank lines plus raw word counts
  code       standard SLOC from the versioned common-language analyzer
  words      normalized and reader-visible word volume
  all        every shipped analyzer

  A comma-separated set: code,words runs both. none and all name the whole
  axis and cannot be combined. lines comes with any analyzer, free.
  languages is metadata-only by default; code adds standard LOC.
  documents requires any enabled analyzer.
  Analysis streams every eligible file through EOF; files are never size-truncated.
  --analysis-workers bounds concurrency.
  --words-per-page changes only report-time page derivation.
  Unchanged results are restored from a separate sidecar; a stored set answers
  any narrower request without re-reading.
  cache=only never opens source files and fails if requested content is absent.

OUTPUT AND AUTOMATION
  Metadata-only machine output remains fdu.report/4; metric summaries use fdu.report/5.
  Text language rows use canonical names; machine formats retain lowercase IDs.
  Metric rows include detection source, confidence, origin flags, and coverage.
  One-shot text reports end with a gray performance line; machine formats omit it.
  Results go to stdout; warnings and errors go to stderr.
  The command never prompts, pages, or animates progress.
  Reports require an explicit PATH; bare `fdu` prints help and scans nothing.
  `fdu --skill` prints a portable agent skill describing this same surface.

EXIT STATUS
  0  Complete result, or a partial result accepted with --allow-partial
  1  Fatal filesystem or cache error
  2  Partial result, or command-line usage error
";

/// When terminal styling should be enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen {
    /// Style output only when its destination is a terminal.
    #[default]
    Auto,
    /// Style output even when its destination is redirected.
    Always,
    /// Never style output.
    Never,
}

/// Marker attached to a rejected argument value.
///
/// Clap exits 2 for a malformed command line; a value clap accepted but this crate's own
/// grammar rejected is the same class of mistake, so it must exit the same way. Without
/// the marker these surfaced as exit 1, which tells a script "the filesystem failed"
/// when the truth is "fix your flag".
#[derive(Debug)]
struct UsageError(String);

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Carries the message rather than wrapping it: a context layer would make the
        // outermost error read "usage" and bury the grammar's suggestion under a
        // "caused by", which is exactly the text the user needs first.
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for UsageError {}

/// The result of one attempt to persist a watching session's index.
///
/// Named rather than folded into a `Result<bool>` at the decision site, because the three
/// cases update the loop's state differently and conflating any two of them has already
/// caused a defect: an early return that wrote nothing was once indistinguishable from a
/// completed write.
#[cfg(feature = "watch")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SaveOutcome {
    /// A snapshot reached disk.
    Written,
    /// Nothing was written: the index is not `Fresh` yet, or policy forbids writes.
    Skipped,
    /// The write was attempted and failed.
    Failed,
}

/// Whether a throttled save is due.
///
/// Pure so the throttle can be tested without a filesystem, a clock, or a watcher. The
/// case worth stating: a pending change that arrives inside the interval is *not* due
/// now, but stays pending, and the caller's idle path is what eventually saves it. Losing
/// that second half is what made a burst-then-quiet session never persist at all.
#[cfg(feature = "watch")]
fn save_is_due(pending: bool, since_last_save: Duration, interval: Duration) -> bool {
    pending && since_last_save >= interval
}

/// Whether a change still needs persisting after an attempt.
///
/// Only a completed write clears the flag. A skip and a failure both leave the change
/// unpersisted, and on a quiet tree the retry is the only thing that will ever save it.
#[cfg(feature = "watch")]
fn pending_after(outcome: SaveOutcome) -> bool {
    match outcome {
        SaveOutcome::Written => false,
        SaveOutcome::Skipped | SaveOutcome::Failed => true,
    }
}

/// Convert a parsed time bound to index nanoseconds, or reject the flag.
///
/// `system_time_to_nanos` returns `None` for an instant outside the range the index can
/// represent (roughly 1677-2262). Storing that `None` would leave the bound unset, so the
/// query would run with no time filter at all while the user believed one was active --
/// a silently wrong answer, which is worse than a rejected flag.
fn bound_nanos(input: &str, when: SystemTime, flag: &str) -> anyhow::Result<i64> {
    system_time_to_nanos(when).ok_or_else(|| {
        usage(&anyhow::anyhow!(
            "invalid {flag} \"{input}\": that time is outside the range fdu can represent \
             (about 1677 to 2262)"
        ))
    })
}

/// Re-tag an argument rejection so it exits like the usage error it is.
fn usage(error: &anyhow::Error) -> anyhow::Error {
    anyhow::Error::new(UsageError(error.to_string()))
}

/// Whether an error was raised by argument validation.
fn is_usage_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| cause.downcast_ref::<UsageError>().is_some())
}

/// Successful command outcome. Partial results are rendered before the caller returns
/// exit status 2, so scripts can opt into them without confusing them with complete data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunOutcome {
    /// Every path in scope was read successfully.
    Complete,
    /// Output was produced, but one or more filesystem paths could not be read.
    Partial,
}

/// Summarize directory trees: sizes, counts, recency, and file types, rolled up for
/// every directory at once.
#[derive(Parser, Debug)]
#[command(
    name = "fdu",
    // Set by build.rs: the package semver plus the git revision on dev builds, so a
    // binary built from a checkout never impersonates the published release.
    version = env!("FDU_BUILD_VERSION"),
    about,
    long_about = None,
    styles = CLI_STYLES,
    after_help = DOCS_POINTER,
    // Wrap at the terminal's width, never wider than this. clap takes the smaller of the
    // two, so a narrow terminal still narrows and a wide one stops at a readable measure
    // rather than running a description across the whole screen.
    max_term_width = 120,
    // `--help` renders the same compact block `-h` does. clap's long help puts every
    // description on its own line with a blank line between flags, which roughly doubles
    // the height and breaks a section into a list of islands; the prose that justified
    // that layout now lives in `--docs`.
    disable_help_flag = true,
    disable_version_flag = true,
    arg_required_else_help = true,
    override_usage = "fdu [OPTIONS] <PATH>\n       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]\n       fdu [PATH] --cache-clear[=<SCOPE>]\n       fdu --docs\n       fdu --skill"
)]
// A command line is a flat bag of independent switches. Folding these into enums to
// satisfy the lint would obscure the one thing this struct exists to mirror: the flags a
// user actually types.
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    // ---- scope: what the engine observes and retains ----
    /// Report root; optional only for the discovery and cache-lifecycle flags.
    #[arg(
        required_unless_present_any = ["docs", "skill", "cache_status", "cache_clear"],
        help_heading = "ARGUMENTS"
    )]
    pub path: Option<PathBuf>,

    /// Limit scanning and retention to N entry levels.
    #[arg(long, value_name = "N", help_heading = "SCOPE")]
    pub scan_depth: Option<usize>,

    /// Stay on the filesystem the root lives on.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "SCOPE")]
    pub one_filesystem: bool,

    // ---- selection: which retained entries this query considers ----
    /// Report only entries matching this glob; repeatable.
    #[arg(long, value_name = "GLOB", help_heading = "SELECTION")]
    pub include: Vec<String>,

    /// Exclude entries matching this glob; repeatable, and wins over --include.
    #[arg(long, value_name = "GLOB", help_heading = "SELECTION")]
    pub exclude: Vec<String>,

    /// Report only entries at least this large, as 512, 10M, or 1.5GiB.
    #[arg(long, value_name = "SIZE", help_heading = "SELECTION")]
    pub min_size: Option<String>,

    /// Report only entries modified at or after this time, as 2h or an RFC 3339 stamp.
    #[arg(long, value_name = "WHEN", help_heading = "SELECTION")]
    pub modified_since: Option<String>,

    /// Report only entries modified before this time.
    #[arg(long, value_name = "WHEN", help_heading = "SELECTION")]
    pub modified_before: Option<String>,

    /// Entry kinds to report: file, dir, symlink, other.
    #[arg(long, value_name = "LIST", help_heading = "SELECTION")]
    pub kind: Option<String>,

    /// Directory levels to show; does not limit scanning. Accepts `all` [tree default: 2].
    ///
    /// Optional for the same reason `--limit` is: the tree brings its own default from
    /// the library, so the CLI does not declare one and every surface agrees. Said in
    /// the help text rather than by clap, because it is the view's default and not the
    /// flag's -- only the tree renders a hierarchy for a depth to bound.
    #[arg(short, long, value_name = "N", help_heading = "SELECTION")]
    pub depth: Option<String>,

    /// Rows to show, per group. Accepts `all`.
    ///
    /// Each view brings its own default, because one number does not suit them all: a
    /// tree shows ten per directory, `largest` and `recent` show twenty, and `files`
    /// enumerates everything.
    #[arg(short = 'n', long, value_name = "N", help_heading = "SELECTION")]
    pub limit: Option<String>,

    /// Order results: size, count, mtime, or name.
    #[arg(long, value_name = "KEY", help_heading = "SELECTION")]
    pub sort: Option<String>,

    /// Reverse the ordering.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "SELECTION")]
    pub reverse: bool,

    /// Which size metric to report: allocated or apparent.
    #[arg(long, value_name = "METRIC", default_value = "allocated", help_heading = "SELECTION")]
    pub size: String,

    // ---- view: which roll-ups are reported ----
    /// Views: tree, extensions, types, families, languages, documents, largest, recent,
    /// files, summary, or full. Defaults to the view that displays what --analyze asked
    /// for.
    #[arg(long, value_name = "LIST", help_heading = "VIEWS")]
    pub view: Option<String>,

    /// Analyzers to run: none, lines, code, words, or all.
    ///
    /// Anything but none opens and reads every eligible file, which is the only setting
    /// that makes a run cost more than one metadata walk.
    #[arg(long, value_name = "LIST", default_value = "none", help_heading = "CONTENT ANALYSIS")]
    pub analyze: String,

    /// Content reader workers; zero selects available parallelism.
    #[arg(long, value_name = "N", default_value_t = 0, help_heading = "CONTENT ANALYSIS")]
    pub analysis_workers: usize,

    /// Logical words per derived document page.
    #[arg(long, value_name = "N", default_value_t = 250, help_heading = "VIEWS")]
    pub words_per_page: u64,

    // ---- format: how the report is serialized ----
    /// Output format: text, json, jsonl, or yaml.
    #[arg(long, value_name = "FORMAT", default_value = "text", help_heading = "OUTPUT")]
    pub format: String,

    /// Colorize human output: auto, always, or never.
    #[arg(
        long,
        value_name = "WHEN",
        default_value = "auto",
        hide_possible_values = true,
        help_heading = "OUTPUT"
    )]
    pub color: ColorWhen,

    // ---- mode: how the cache is used ----
    /// Cache policy: auto, refresh, read-only, only, or off.
    #[arg(long, value_name = "POLICY", default_value = "auto", help_heading = "EXECUTION")]
    pub cache: String,

    /// Accept operationally partial results, including filesystem or analysis failures.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "EXECUTION")]
    pub allow_partial: bool,

    /// Report cache contents instead of scanning: root (default) or all.
    #[arg(long, value_name = "SCOPE", num_args = 0..=1, require_equals = true, default_missing_value = "root", help_heading = "CACHE MANAGEMENT")]
    pub cache_status: Option<String>,

    /// Remove cached snapshots instead of scanning: root (default) or all.
    #[arg(long, value_name = "SCOPE", num_args = 0..=1, require_equals = true, default_missing_value = "root", help_heading = "CACHE MANAGEMENT")]
    pub cache_clear: Option<String>,

    /// Stream changes continuously instead of returning one report.
    #[cfg(feature = "watch")]
    #[arg(long, action = ArgAction::SetTrue, help_heading = "EXECUTION")]
    pub watch: bool,

    /// How often aggregate views re-render while watching, as a duration.
    ///
    /// Throttles rendering only; change detection is event-driven and unaffected.
    #[cfg(feature = "watch")]
    #[arg(long, value_name = "DUR", default_value = "2s", help_heading = "EXECUTION")]
    pub interval: String,

    /// Print help.
    #[arg(short = 'h', long = "help", action = ArgAction::HelpShort, help_heading = "OTHER")]
    pub help: Option<bool>,

    /// Print version.
    #[arg(short = 'V', long = "version", action = ArgAction::Version, help_heading = "OTHER")]
    pub version: Option<bool>,

    /// Print the usage guide: the report ladder, both axes, and the output contracts.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "OTHER")]
    pub docs: bool,

    /// Print a portable agent skill to stdout.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "OTHER")]
    pub skill: bool,
}

/// Which caches a lifecycle flag applies to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CacheScope {
    /// Only the snapshot for the resolved path.
    Root,
    /// Every snapshot in the cache directory.
    All,
}

impl CacheScope {
    fn parse(value: &str, flag: &str) -> anyhow::Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "root" => Ok(Self::Root),
            "all" => Ok(Self::All),
            other => anyhow::bail!("invalid {flag} {other:?}: expected root or all"),
        }
    }
}

impl Cli {
    /// Run the command, writing results to `out` and warnings to `diagnostic`.
    pub fn run(
        &self,
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        stdout_is_terminal: bool,
        stderr_is_terminal: bool,
    ) -> anyhow::Result<RunOutcome> {
        if self.docs {
            let color =
                ColorContext::from_environment(self.color, false, false, stdout_is_terminal)
                    .enabled();
            write!(out, "{}", style_guide(DOCS, color))?;
            return Ok(RunOutcome::Complete);
        }

        if self.skill {
            write!(out, "{}", compose_skill())?;
            return Ok(RunOutcome::Complete);
        }

        // Lifecycle flags run before scan validation, so they need no readable tree, and
        // they suppress the report entirely: a run that inspects or clears the cache is
        // not also a run that scans. Clear runs first so a combined invocation reports
        // the state it left behind.
        if self.cache_clear.is_some() || self.cache_status.is_some() {
            return self.run_cache_lifecycle(out);
        }

        // Parse the whole request before touching the filesystem, so a typo in a glob or a
        // time costs nothing and reports its own spelling rather than a scan's worth of
        // waiting followed by an error.
        let format = self.parse_format().map_err(|error| usage(&error))?;
        // The content axis is parsed first because the view axis defaults from it: a
        // request that pays to read files should display what it read.
        let analysis = self.parse_analysis().map_err(|error| usage(&error))?;
        let views =
            resolve_views(self.view.as_deref(), analysis.profile).map_err(|error| usage(&error))?;
        let query = self.parse_query(&views).map_err(|error| usage(&error))?;
        let path = self.path.as_deref().ok_or_else(|| {
            usage(&anyhow::anyhow!(
                "missing PATH: specify the directory to summarize, for example `fdu .`"
            ))
        })?;

        let policy = self.parse_cache_policy().map_err(|error| usage(&error))?;
        query
            .validate_analysis(analysis.profile)
            .map_err(|message| usage(&anyhow::anyhow!(message)))?;
        // Whether this invocation ever consumes ignore classification. The one-shot
        // report reads none of it, so a plain `fdu <dir>` performs no control-file I/O
        // and retains no control table -- reading and retaining every `.gitignore` was
        // both the control-budget abort on large homes and a measured share of the
        // whole-scan slowdown (fdu-etfj, fdu-pro1). Watching keeps control state
        // because a watch session maintains the ignored partitions incrementally and
        // cannot re-derive them later. The bit is part of the scan's semantic scope, so
        // a cache written either way is never served to the other.
        #[cfg(feature = "watch")]
        let read_controls = self.watch;
        #[cfg(not(feature = "watch"))]
        let read_controls = false;
        let config = OpenConfig {
            scan: ScanConfig {
                max_depth: self.scan_depth,
                one_filesystem: self.one_filesystem,
                read_controls,
                ..ScanConfig::default()
            },
            cache_path: default_cache_path(path),
            policy,
            analysis,
        };

        #[cfg(feature = "watch")]
        if self.watch && (self.scan_depth.is_some() || self.one_filesystem) {
            // Scope narrows what is observed, and a watcher cannot filter raw backend
            // events against that boundary yet. Selection flags stay legal with --watch
            // precisely because they filter the retained index instead, and the message
            // says so rather than only naming the conflict.
            return Err(usage(&anyhow::anyhow!(watch_scope_guidance())));
        }

        #[cfg(feature = "watch")]
        if self.watch && analysis.profile.is_enabled() {
            return Err(usage(&anyhow::anyhow!(
                "--analyze is not yet supported with --watch; use a one-shot report"
            )));
        }

        #[cfg(feature = "watch")]
        if self.watch {
            let color = ColorContext::from_environment(
                self.color,
                self.machine_format(),
                self.skill,
                stdout_is_terminal,
            )
            .enabled();
            return self.run_watch(out, diagnostic, format, query, &config, color);
        }

        let report_started = Instant::now();
        let collect_scan_diagnostics =
            std::env::var_os(SCAN_DIAGNOSTICS_ENV).is_some_and(|value| value == OsStr::new("1"));
        let (report, pending_save, performance, scan_diagnostics) = if collect_scan_diagnostics {
            prepare_report_with_scan_diagnostics(path, &config, &query)?
        } else {
            let (report, pending_save, performance) = prepare_report(path, &config, &query)?;
            (report, pending_save, performance, None)
        };
        if let Some(scan_diagnostics) = scan_diagnostics {
            writeln!(diagnostic, "{SCAN_DIAGNOSTICS_PREFIX}{}", scan_diagnostics.to_json())?;
        }

        let color = ColorContext::from_environment(
            self.color,
            self.machine_format(),
            self.skill,
            stdout_is_terminal,
        )
        .enabled();
        // The write is already running; rendering is the other reader. Whether output
        // finishes first or the save does, both complete -- and the rendered report is
        // flushed to the terminal *before* waiting on the save, or the overlap is only
        // nominal: the caller's writer is buffered, a default depth-2 tree fits inside
        // that buffer, and the user would see nothing until the snapshot's fsync and the
        // index teardown had finished (fdu-n75m). Same bytes in the same order; only
        // when they arrive changes.
        let rendered = report_format::render(&report, format, color);
        let render_result = write!(out, "{rendered}").and_then(|()| out.flush());

        // Joined before returning, and before the render error is raised: a broken pipe
        // must not abandon a finished scan's snapshot, because the next run would then
        // pay for a cold scan that this one had already done.
        if let Err(error) = pending_save.join() {
            let _ = writeln!(
                diagnostic,
                "{}",
                paint(&format!("warning: {error}"), STYLE_WARNING, stderr_is_terminal)
            );
        }
        render_result?;

        if format == report_format::Format::Text {
            if !rendered.is_empty() && !rendered.ends_with('\n') {
                writeln!(out)?;
            }
            writeln!(
                out,
                "{}",
                paint(
                    &performance_footer(performance, report_started.elapsed()),
                    STYLE_PERFORMANCE,
                    color,
                )
            )?;
            for note in display_notes(&views, analysis.profile, performance.bytes_read) {
                writeln!(out, "{}", paint(&note, STYLE_PERFORMANCE, color))?;
            }
        }

        if format == report_format::Format::Text && !report.complete {
            let color =
                ColorContext::from_environment(self.color, false, false, stderr_is_terminal)
                    .enabled();
            for error in &report.errors {
                let _ = writeln!(
                    diagnostic,
                    "{}",
                    paint(&format!("warning: {error}"), STYLE_WARNING, color)
                );
            }
        }

        Ok(if report.complete { RunOutcome::Complete } else { RunOutcome::Partial })
    }

    /// Whether the requested format is a machine format, which is never colorized.
    fn machine_format(&self) -> bool {
        !matches!(
            report_format::Format::parse(&self.format),
            None | Some(report_format::Format::Text)
        )
    }

    /// Run the query continuously, streaming changes as they arrive.
    ///
    /// The initial report is exactly what a one-shot run would print, and every later
    /// render is the same query re-evaluated. Detection is event-driven throughout: an
    /// idle tree costs no filesystem work, and `--interval` throttles only how often
    /// aggregate views repaint.
    #[cfg(feature = "watch")]
    fn run_watch(
        &self,
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        format: report_format::Format,
        query: Query,
        config: &OpenConfig,
        color: bool,
    ) -> anyhow::Result<RunOutcome> {
        use fdu_core::query::ViewSpec;
        use fdu_core::watch::WatchConfig;
        use fdu_core::watch_session::{ChangeKind, Session};

        let path = self.path.as_deref().expect("run() validates the report path first");
        let interval = parse_duration(&self.interval).map_err(|error| usage(&error))?;

        let scan_started_at = SystemTime::now();
        let (index, open_report, pending_save) = open_with_pending_save(path, config)?;
        if let Err(error) = pending_save.join() {
            let _ = writeln!(
                diagnostic,
                "{}",
                paint(&format!("warning: {error}"), STYLE_WARNING, color)
            );
        }

        // A streaming run keeps only the views it can render incrementally plus the
        // aggregates it repaints; both come from the same query, so nothing here is a
        // second grammar.
        let streams_changes = query.views.contains(&ViewSpec::Files);
        let has_aggregates = query.views.iter().any(|view| *view != ViewSpec::Files);

        // The save above was joined, so the writer has dropped its reference and this
        // is the only one left; the watch session needs the index by value.
        let index = std::sync::Arc::into_inner(index)
            .expect("the joined writer released the only other reference");
        let handle = fdu_core::IndexHandle::new(index);
        let mut session = Session::new(handle, config.scan.clone(), query, WatchConfig::default())?;

        // The initial answer, identical to a one-shot run's.
        let provenance = Provenance {
            scan_started_at: Some(scan_started_at),
            generated_at: SystemTime::now(),
            source: match open_report.path_taken {
                fdu_core::OpenPath::ColdScan => ReportSource::ColdScan,
                fdu_core::OpenPath::WarmRevalidate => ReportSource::WarmRevalidate,
                fdu_core::OpenPath::CacheOnly => ReportSource::CacheOnly,
            },
            complete: open_report.is_complete(),
            errors: open_report.error_messages(),
        };
        write!(out, "{}", report_format::render(&session.report(&provenance)?, format, color))?;
        out.flush()?;

        let mut dirty_since_render = false;
        let mut last_render = SystemTime::now();
        let mut last_save = SystemTime::now();
        let mut dirty_since_save = false;
        loop {
            let Some(batch) = session.next_batch(interval)? else {
                // Nothing arrived in the window. Repaint only if something is pending,
                // so a quiet tree produces no output and no work at all.
                if has_aggregates && dirty_since_render {
                    Self::render_live(out, &session, format, color)?;
                    dirty_since_render = false;
                    last_render = SystemTime::now();
                }
                // The idle branch is where a throttled save has to land. A change that
                // arrived too soon after the last save would otherwise wait for the next
                // change to persist it, and the next change may never come: a burst
                // followed by silence is the single most likely way a watch session ends.
                Self::save_if_pending(
                    &session,
                    config,
                    &mut dirty_since_save,
                    &mut last_save,
                    interval,
                    diagnostic,
                    color,
                );
                continue;
            };

            for change in &batch.changes {
                if change.kind == ChangeKind::Invalidate {
                    // Never dropped: an escalation says the consumer's view may have gaps.
                    writeln!(out, "{}", report_format::render_change(change, format))?;
                } else if streams_changes {
                    writeln!(out, "{}", report_format::render_change(change, format))?;
                }
            }
            out.flush()?;

            dirty_since_render |= batch.dirty;
            let elapsed = last_render.elapsed().unwrap_or_default();
            if has_aggregates && dirty_since_render && elapsed >= interval {
                Self::render_live(out, &session, format, color)?;
                dirty_since_render = false;
                last_render = SystemTime::now();
            }

            // Persist as we go rather than only at exit. A watch session ends by signal
            // far more often than it ends politely, and std offers no portable signal
            // handler, so an exit-time save would be the one that never runs. Throttled
            // to the render interval so a churny tree does not rewrite constantly; the
            // pending flag is what guarantees a throttled change still reaches disk once
            // the tree goes quiet.
            dirty_since_save |= batch.dirty;
            Self::save_if_pending(
                &session,
                config,
                &mut dirty_since_save,
                &mut last_save,
                interval,
                diagnostic,
                color,
            );
        }
    }

    /// Persist a pending change, if one is due.
    ///
    /// The pending flag clears only when a snapshot actually reached disk. An index that
    /// is not yet `Fresh`, a policy that forbids writes, and a failed write all leave the
    /// change unpersisted, and clearing the flag for any of them would mean the idle
    /// branch never retries -- which on a quiet tree is never at all.
    #[cfg(feature = "watch")]
    #[allow(clippy::too_many_arguments)]
    fn save_if_pending(
        session: &fdu_core::watch_session::Session,
        config: &OpenConfig,
        pending: &mut bool,
        last_save: &mut SystemTime,
        interval: Duration,
        diagnostic: &mut dyn Write,
        color: bool,
    ) {
        if !save_is_due(*pending, last_save.elapsed().unwrap_or_default(), interval) {
            return;
        }
        let outcome = match Self::save_live(session, config) {
            Ok(true) => SaveOutcome::Written,
            Ok(false) => SaveOutcome::Skipped,
            Err(error) => {
                Self::warn_save_failed(&error, diagnostic, color);
                SaveOutcome::Failed
            }
        };
        *pending = pending_after(outcome);
        // Throttled whether or not it worked, so a persistently failing save warns at the
        // interval rather than spinning.
        *last_save = SystemTime::now();
    }

    /// Warn about a failed save without disturbing the stream.
    ///
    /// A save failure costs the next run its warm start and nothing else, so it must not
    /// interrupt a watch that is otherwise working.
    #[cfg(feature = "watch")]
    fn warn_save_failed(error: &anyhow::Error, diagnostic: &mut dyn Write, color: bool) {
        let _ =
            writeln!(diagnostic, "{}", paint(&format!("warning: {error}"), STYLE_WARNING, color));
    }

    /// Persist a live session's index, when policy allows it.
    ///
    /// Keeps the warm cache current during a long watch instead of betting on a clean
    /// exit. A failure here is a warning: the stream is still correct, and only the next
    /// run's warmth is lost.
    #[cfg(feature = "watch")]
    fn save_live(
        session: &fdu_core::watch_session::Session,
        config: &OpenConfig,
    ) -> anyhow::Result<bool> {
        let (Some(cache_path), true) = (config.cache_path.as_deref(), config.policy.writes())
        else {
            return Ok(false);
        };
        let index = session.index_snapshot()?;
        if index.freshness() != fdu_core::Freshness::Fresh {
            // Only a trustworthy index is worth persisting; a partial one would be
            // served as fact on the next run. Reported as "not written" so the caller
            // keeps the change pending and tries again once the index settles.
            return Ok(false);
        }
        fdu_core::snapshot::save(&index, cache_path)?;
        Ok(true)
    }

    /// Re-render the aggregate views of a live session.
    #[cfg(feature = "watch")]
    /// Repaint the aggregate views after a change.
    ///
    /// Only ever called for a repaint — the first answer is written by the caller before
    /// the loop — so the rule below can be unconditional.
    fn render_live(
        out: &mut dyn Write,
        session: &fdu_core::watch_session::Session,
        format: report_format::Format,
        color: bool,
    ) -> anyhow::Result<()> {
        let provenance = session.live_provenance(SystemTime::now());
        let report = session.report(&provenance)?;
        // A watch run has no final answer and so no performance footer, which left text
        // repaints with nothing between them: the last row of one and the first row of
        // the next were adjacent lines. A blank line alone would not do, because that is
        // already what separates two views inside a single report.
        if format == report_format::Format::Text {
            writeln!(
                out,
                "\n{}",
                paint(&report_format::watch_rule(provenance.generated_at), STYLE_WATCH_RULE, color)
            )?;
        }
        write!(out, "{}", report_format::render(&report, format, color))?;
        out.flush()?;
        Ok(())
    }

    /// Run the cache lifecycle flags and report what they found or removed.
    fn run_cache_lifecycle(&self, out: &mut dyn Write) -> anyhow::Result<RunOutcome> {
        // Lifecycle commands do not scan. With no PATH they retain their existing
        // current-root meaning so `--cache-status=all` and `--cache-clear=all` remain
        // useful discovery/maintenance actions without weakening report safety.
        let root = self.path.as_deref().unwrap_or_else(|| Path::new("."));
        let cache_dir = fdu_core::default_cache_path(root)
            .and_then(|path| path.parent().map(Path::to_path_buf));

        if let Some(scope) = &self.cache_clear {
            let scope = CacheScope::parse(scope, "--cache-clear").map_err(|e| usage(&e))?;
            match (scope, &cache_dir) {
                (CacheScope::All, Some(dir)) => {
                    // Echo the directory before acting, so a destructive flag always says
                    // where it is pointed.
                    writeln!(out, "Cache directory: {}", dir.display())?;
                    let removed = fdu_core::clear_all_caches(dir)?;
                    writeln!(
                        out,
                        "{}",
                        if removed == 0 {
                            "Cache already empty.".to_string()
                        } else {
                            format!(
                                "Cache cleared: {removed} {}.",
                                plural(removed, "snapshot", "snapshots")
                            )
                        }
                    )?;
                }
                (CacheScope::Root, _) => {
                    let path = fdu_core::default_cache_path(root);
                    let removed = match &path {
                        Some(path) => fdu_core::clear_cache(path)?,
                        None => false,
                    };
                    if let Some(path) = &path {
                        writeln!(out, "Cache file: {}", path.display())?;
                    }
                    writeln!(
                        out,
                        "{}",
                        if removed { "Cache cleared." } else { "Cache already empty." }
                    )?;
                }
                (CacheScope::All, None) => writeln!(out, "Cache already empty.")?,
            }
        }

        if let Some(scope) = &self.cache_status {
            let scope = CacheScope::parse(scope, "--cache-status").map_err(|e| usage(&e))?;
            let statuses = match (scope, &cache_dir) {
                (CacheScope::All, Some(dir)) => fdu_core::list_caches(dir)?,
                (CacheScope::All, None) => Vec::new(),
                (CacheScope::Root, _) => match fdu_core::default_cache_path(root) {
                    Some(path) => vec![fdu_core::cache_status(&path)?],
                    None => Vec::new(),
                },
            };
            self.write_cache_status(out, &statuses)?;
        }

        Ok(RunOutcome::Complete)
    }

    /// Render cache status through the format axis, like any other output.
    fn write_cache_status(
        &self,
        out: &mut dyn Write,
        statuses: &[fdu_core::CacheStatus],
    ) -> anyhow::Result<()> {
        let format = self.parse_format().map_err(|e| usage(&e))?;
        // Every format, human included, comes from the one renderer. While the CLI kept
        // the text layout to itself, no other caller could print what fdu prints.
        writeln!(out, "{}", report_format::render_cache_status(statuses, format))?;
        Ok(())
    }

    /// Translate the cache-policy flag.
    fn parse_cache_policy(&self) -> anyhow::Result<CachePolicy> {
        match self.cache.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(CachePolicy::Auto),
            "refresh" => Ok(CachePolicy::Refresh),
            "read-only" | "readonly" => Ok(CachePolicy::ReadOnly),
            "only" => Ok(CachePolicy::Only),
            "off" => Ok(CachePolicy::Off),
            other => anyhow::bail!(
                "invalid --cache {other:?}: expected one of auto, refresh, read-only, only, off"
            ),
        }
    }

    /// Translate the format flag, naming every accepted value on a miss.
    fn parse_format(&self) -> anyhow::Result<report_format::Format> {
        report_format::Format::parse(&self.format).ok_or_else(|| {
            anyhow::anyhow!(
                "invalid --format {:?}: expected one of {}",
                self.format,
                report_format::Format::ALL.join(", ")
            )
        })
    }

    /// Translate every selection and view flag into the library's own types.
    ///
    /// This is all the CLI does: parse flags into `Query`, hand it to the library, and
    /// serialize what comes back. Any logic beyond that belongs in the library, where
    /// Rust and Python callers get it too.
    /// Resolve the content and view axes exactly as [`Cli::run`] does.
    ///
    /// Tests go through this rather than calling `parse_query` with a hand-written view
    /// list, so a change to the resolution order cannot pass the suite while breaking the
    /// command.
    #[cfg(test)]
    fn resolved_query(&self) -> anyhow::Result<Query> {
        let analysis = self.parse_analysis()?;
        let views = resolve_views(self.view.as_deref(), analysis.profile)?;
        self.parse_query(&views)
    }

    fn parse_query(&self, views: &ResolvedViews) -> anyhow::Result<Query> {
        let now = SystemTime::now();

        let mut selection = Selection {
            depth: self.depth.as_deref().map(|value| parse_bound(value, "--depth")).transpose()?,
            limit: self.limit.as_deref().map(|value| parse_bound(value, "--limit")).transpose()?,
            reverse: self.reverse,
            size: parse_size_metric(&self.size)?,
            ..Selection::default()
        };

        for pattern in &self.include {
            selection.include.push(Pattern::parse(pattern)?);
        }
        for pattern in &self.exclude {
            selection.exclude.push(Pattern::parse(pattern)?);
        }
        if let Some(min_size) = &self.min_size {
            selection.min_size = Some(parse_size(min_size)?);
        }
        if let Some(since) = &self.modified_since {
            selection.modified.since =
                Some(bound_nanos(since, parse_when(since, now)?, "--modified-since")?);
        }
        if let Some(before) = &self.modified_before {
            selection.modified.before =
                Some(bound_nanos(before, parse_when(before, now)?, "--modified-before")?);
        }
        if let Some(kinds) = &self.kind {
            selection.kinds = parse_list(kinds, "--kind", parse_kind)?;
        }
        if let Some(sort) = &self.sort {
            selection.sort = Some(parse_sort(sort)?);
        }

        let omitted_views = views.omitted.clone();
        let views = views.selected.clone();
        if self.words_per_page == 0 {
            anyhow::bail!("invalid --words-per-page 0: expected a positive integer");
        }
        // The command line names these axes with flags; a library caller names them with
        // fields, and the report's own diagnostics follow whichever asked.
        Ok(Query {
            selection,
            views,
            omitted_views,
            axes: AxisNames::FLAGS,
            words_per_page: self.words_per_page,
        })
    }

    fn parse_analysis(&self) -> anyhow::Result<AnalysisRequest> {
        let profile = AnalysisSet::parse_labeled(&self.analyze, "--analyze")
            .map_err(|message| anyhow::anyhow!(message))?;
        Ok(AnalysisRequest { profile, workers: self.analysis_workers })
    }
}

/// What each knob is called on the command line, given its name in the API.
const WATCH_SCOPE_VOCABULARY: [(&str, &str); 5] = [
    ("max_depth", "--scan-depth"),
    ("one_filesystem", "--one-filesystem"),
    ("modified_since", "--modified-since"),
    ("depth", "--depth"),
    ("include", "--include"),
];

/// The library's watch-scope rule, in the command line's vocabulary.
///
/// One rule, stated once, with only the knob names differing. It used to be two separate
/// messages -- the CLI's naming flags and the library's naming implementation -- which
/// drifted apart and which the parity harness could not tell were the same rule.
///
/// The substitution is an explicit whole-word map rather than a blind replace, which is
/// the bug fdu-7j6z was: `message.replace("analyze", "--analyze")` rewrote the user's own
/// token. Here every replacement is a field name that cannot appear inside a value, which
/// `the_watch_guidance_substitutes_whole_words_only` asserts, and the parity run verifies
/// the two surfaces stay equivalent.
fn watch_scope_guidance() -> String {
    // One pass over whole words, never re-scanning what was already substituted. A
    // sequential replace does re-scan: max_depth becomes --scan-depth, and then `depth`
    // matches inside it, giving `--scan---depth`. That is fdu-7j6z again, and it appeared
    // again here the moment the same shortcut was taken.
    fdu_core::scan::WATCH_SCOPE_GUIDANCE
        .split_inclusive(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .map(|piece| {
            let end = piece
                .find(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .unwrap_or(piece.len());
            let (word, tail) = piece.split_at(end);
            match WATCH_SCOPE_VOCABULARY.iter().find(|(field, _)| *field == word) {
                Some((_, flag)) => format!("{flag}{tail}"),
                None => piece.to_string(),
            }
        })
        .collect()
}

/// Format transient one-shot work without adding it to the machine-report schema.
fn performance_footer(performance: PerformanceSummary, total: Duration) -> String {
    let fresh = match (performance.fresh_files, performance.analysis_ns) {
        (0, _) | (_, 0) => format!("{} fresh", human_count(performance.fresh_files)),
        (files, elapsed_ns) => {
            format!("{} fresh at {} files/s", human_count(files), human_rate(files, elapsed_ns))
        }
    };
    let cached = if performance.cached_files == 0 {
        "0 cached".to_string()
    } else {
        format!(
            "{} cached / {}",
            human_count(performance.cached_files),
            report_format::human_bytes(performance.cached_bytes)
        )
    };
    let read_rate = if performance.bytes_read == 0 || performance.analysis_ns == 0 {
        String::new()
    } else {
        format!(
            " at {}/s",
            report_format::human_bytes(rate_per_second(
                performance.bytes_read,
                performance.analysis_ns,
            ))
        )
    };
    format!(
        "Performance: walked {} {} / {}; content read {}{}; analysis {fresh}, {cached}; {}; total {}",
        human_count(performance.walked_files),
        plural_u64(performance.walked_files, "file", "files"),
        report_format::human_bytes(performance.walked_bytes),
        report_format::human_bytes(performance.bytes_read),
        read_rate,
        performance_source(performance.source),
        human_duration(total),
    )
}

fn performance_source(source: ReportSource) -> &'static str {
    match source {
        ReportSource::ColdScan => "cold scan",
        ReportSource::WarmRevalidate => "warm revalidation",
        ReportSource::CacheOnly => "cache only",
    }
}

fn rate_per_second(units: u64, elapsed_ns: u64) -> u64 {
    if elapsed_ns == 0 {
        return 0;
    }
    let scaled = u128::from(units).saturating_mul(1_000_000_000);
    u64::try_from(scaled / u128::from(elapsed_ns)).unwrap_or(u64::MAX)
}

fn human_rate(units: u64, elapsed_ns: u64) -> String {
    let rate = rate_per_second(units, elapsed_ns);
    match rate {
        0..=999 => human_count(rate),
        1_000..=999_999 => format!("{}k", scaled_decimal(u128::from(rate), 1_000, 1)),
        1_000_000..=999_999_999 => {
            format!("{}M", scaled_decimal(u128::from(rate), 1_000_000, 1))
        }
        _ => format!("{}G", scaled_decimal(u128::from(rate), 1_000_000_000, 1)),
    }
}

fn human_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos < 1_000 {
        format!("{nanos} ns")
    } else if nanos < 1_000_000 {
        format!("{} µs", scaled_decimal(nanos, 1_000, 1))
    } else if nanos < 1_000_000_000 {
        format!("{} ms", scaled_decimal(nanos, 1_000_000, 1))
    } else {
        format!("{} s", scaled_decimal(nanos, 1_000_000_000, 2))
    }
}

/// Round an integer ratio to a fixed number of decimal places without losing precision.
fn scaled_decimal(value: u128, unit: u128, precision: u32) -> String {
    let factor = 10_u128.pow(precision);
    let scaled = value.saturating_mul(factor).saturating_add(unit / 2) / unit;
    let whole = scaled / factor;
    let fraction = scaled % factor;
    let width = usize::try_from(precision).expect("decimal precision fits usize");
    format!("{whole}.{fraction:0width$}")
}

fn plural_u64<'a>(count: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

/// Anchor for reading an interval as an age.
///
/// Far enough past the epoch that any interval worth writing subtracts cleanly, and near
/// enough to be representable everywhere. That second half is not theoretical: this was
/// once 2^40 seconds, about 34,865 years, which is fine where `SystemTime` counts seconds
/// and overflows on Windows, where it is 100-nanosecond FILETIME ticks. Roughly a
/// thousand years is astronomically larger than any render interval and comfortable on
/// every platform.
#[cfg(feature = "watch")]
const INTERVAL_ANCHOR_SECS: u64 = 1 << 35;

/// Parse a render interval, reusing the age half of the shared time grammar.
#[cfg(feature = "watch")]
fn parse_duration(value: &str) -> anyhow::Result<std::time::Duration> {
    // Expressed as an age before a fixed instant, so `2s` and `1h30m` mean here exactly
    // what they mean in --modified-since rather than being a fourth spelling.
    let anchor = SystemTime::UNIX_EPOCH + Duration::from_secs(INTERVAL_ANCHOR_SECS);
    let at = parse_when(value, anchor)
        .map_err(|error| anyhow::anyhow!("invalid --interval {value:?}: {error}"))?;
    anchor
        .duration_since(at)
        .map_err(|_| anyhow::anyhow!("invalid --interval {value:?}: expected a duration like `2s`"))
}

/// Pick the singular or plural noun for a count.
fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

/// Split a comma-delimited list of closed identifiers.
///
/// Closed vocabularies are comma lists and open pattern values are repeatable flags,
/// because glob brace syntax (`*.{rs,toml}`) contains commas and would be shredded by a
/// split. Duplicates are an error rather than a silent no-op: repeating a view is far
/// more likely to be a typo than an intention.
fn parse_list<T: PartialEq>(
    value: &str,
    flag: &str,
    parse: impl Fn(&str, &str) -> anyhow::Result<T>,
) -> anyhow::Result<Vec<T>> {
    let mut parsed = Vec::new();
    for token in value.split(',') {
        let token = token.trim();
        if token.is_empty() {
            anyhow::bail!("invalid {flag} {value:?}: empty entry in the list");
        }
        let item = parse(token, flag)?;
        if parsed.contains(&item) {
            anyhow::bail!("invalid {flag} {value:?}: {token:?} appears more than once");
        }
        parsed.push(item);
    }
    Ok(parsed)
}

/// Whether a view renders anything the content analyzers produce.
///
/// This is the check behind the "paid for nothing" note.  It is deliberately a match
/// rather than a property of `ViewSpec`, so adding a view forces a decision here about
/// whether it displays content metrics.
const fn view_displays_analysis(view: ViewSpec) -> bool {
    matches!(view, ViewSpec::Types | ViewSpec::Families | ViewSpec::Languages | ViewSpec::Documents)
}

/// Views to render, plus any `--view all` could not satisfy.
#[derive(Debug)]
struct ResolvedViews {
    selected: Vec<ViewSpec>,
    omitted: Vec<ViewSpec>,
}

/// Resolve the view axis against the content axis.
///
/// `all` expands to what the requested analyzers can answer rather than failing the
/// whole run over one unsatisfiable view, and reports what it dropped so the omission is
/// stated rather than hidden.
fn resolve_views(spec: Option<&str>, profile: AnalysisSet) -> anyhow::Result<ResolvedViews> {
    // The whole axis -- list grammar, `full` expansion, and the default -- lives in the
    // library, so the CLI and the Python API cannot disagree about what a spec means.
    let (selected, omitted) =
        ViewSpec::resolve(spec, profile, "--view").map_err(|message| anyhow::anyhow!(message))?;
    Ok(ResolvedViews { selected, omitted })
}

/// Notes that keep the display contract legible in human output.
///
/// Both are the same rule read in opposite directions: a run displays what it paid for,
/// and a view it could not render is named rather than quietly dropped.  Machine formats
/// carry neither, because the `reports` array already enumerates exactly which views were
/// produced — a consumer reads the omission from what is absent.
fn display_notes(views: &ResolvedViews, profile: AnalysisSet, bytes_read: u64) -> Vec<String> {
    // Only the note that needs telemetry. The omission note is a fact about the report and
    // travels on it, so every surface states it rather than just this one (fdu-x8u6).
    //
    // Never an error: warming the content sidecar so a later run is warm is a supported
    // use, and `--cache`-aware callers depend on it.  Silence would hide the cost instead.
    if profile.is_enabled() && !views.selected.iter().any(|view| view_displays_analysis(*view)) {
        return vec![format!(
            "note: --analyze {} read {}; no selected view displays content metrics — try --view families, languages, or all",
            profile.labels().join(","),
            report_format::human_bytes(bytes_read),
        )];
    }
    Vec::new()
}

/// Parse one `--kind` token.
fn parse_kind(token: &str, flag: &str) -> anyhow::Result<EntryKind> {
    match token.to_ascii_lowercase().as_str() {
        "file" => Ok(EntryKind::File),
        "dir" => Ok(EntryKind::Dir),
        "symlink" => Ok(EntryKind::Symlink),
        "other" => Ok(EntryKind::Other),
        _ => anyhow::bail!("invalid {flag} {token:?}: expected one of file, dir, symlink, other"),
    }
}

/// Parse a bound that accepts `all` for unbounded.
fn parse_bound(value: &str, flag: &str) -> anyhow::Result<Bound> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("all") {
        return Ok(Bound::All);
    }
    value
        .parse::<usize>()
        .map(Bound::Limit)
        .map_err(|_| anyhow::anyhow!("invalid {flag} {value:?}: expected a whole number or `all`"))
}

/// Parse the `--sort` key.
fn parse_sort(value: &str) -> anyhow::Result<SortKey> {
    match value.trim().to_ascii_lowercase().as_str() {
        "size" => Ok(SortKey::Size),
        "count" => Ok(SortKey::Count),
        "mtime" => Ok(SortKey::Mtime),
        "name" => Ok(SortKey::Name),
        other => {
            anyhow::bail!("invalid --sort {other:?}: expected one of size, count, mtime, name")
        }
    }
}

/// Parse the `--size` metric.
fn parse_size_metric(value: &str) -> anyhow::Result<SizeMetric> {
    match value.trim().to_ascii_lowercase().as_str() {
        "allocated" => Ok(SizeMetric::Allocated),
        "apparent" => Ok(SizeMetric::Apparent),
        other => anyhow::bail!("invalid --size {other:?}: expected allocated or apparent"),
    }
}

/// Run `fdu` through its real process boundary and return its stable numeric exit code.
///
/// This is shared by the native binary and the Python wheel's console entry point so
/// parsing, streams, color, diagnostics, broken pipes, and exit semantics cannot drift.
pub fn run_process<I, T>(args: I) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let stdout_is_terminal = stdout.is_terminal();
    let stderr_is_terminal = stderr.is_terminal();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut diagnostic = stderr.lock();

    run_with_io(&args, &mut out, &mut diagnostic, stdout_is_terminal, stderr_is_terminal)
}

fn run_with_io(
    args: &[OsString],
    out: &mut dyn Write,
    diagnostic: &mut dyn Write,
    stdout_is_terminal: bool,
    stderr_is_terminal: bool,
) -> u8 {
    // A bare invocation is the long-help discovery surface, byte for byte. Rewriting it
    // before parsing also gives it `--help`'s stdout and exit-0 behavior, rather than
    // Clap's shorter missing-argument rendering on stderr.
    let implicit_help;
    let args = if args.len() == 1 {
        implicit_help = vec![args[0].clone(), OsString::from("--help")];
        implicit_help.as_slice()
    } else {
        args
    };
    let requested_color = requested_color(args);
    let json_requested = flag_is_present(args, "--json");
    let skill_requested = flag_is_present(args, "--skill");
    let command = Cli::command().color(ColorChoice::Always);
    let matches = match command.try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => {
            let use_stderr = error.use_stderr();
            let destination_is_terminal =
                if use_stderr { stderr_is_terminal } else { stdout_is_terminal };
            let color = ColorContext::from_environment(
                requested_color,
                json_requested,
                skill_requested,
                destination_is_terminal,
            )
            .enabled();
            let rendered = error.render();
            let write_result = if use_stderr {
                write_styled(diagnostic, &rendered, color)
            } else {
                write_styled(out, &rendered, color)
            };
            if let Err(write_error) = write_result {
                return match write_error.kind() {
                    io::ErrorKind::BrokenPipe => 0,
                    _ => 1,
                };
            }
            return u8::try_from(error.exit_code()).unwrap_or(2);
        }
    };
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = write!(diagnostic, "{error}");
            return 2;
        }
    };

    let result =
        cli.run(out, diagnostic, stdout_is_terminal, stderr_is_terminal).and_then(|outcome| {
            out.flush()?;
            Ok(outcome)
        });
    let diagnostic_color = ColorContext::from_environment(
        cli.color,
        cli.machine_format(),
        cli.skill,
        stderr_is_terminal,
    )
    .enabled();
    finish(result, cli.allow_partial, diagnostic, diagnostic_color)
}

fn write_styled(
    out: &mut dyn Write,
    rendered: &clap::builder::StyledStr,
    color: bool,
) -> io::Result<()> {
    let rendered = if color { rendered.ansi().to_string() } else { rendered.to_string() };
    for line in rendered.split_inclusive('\n') {
        let (content, newline) =
            line.strip_suffix('\n').map_or((line, ""), |content| (content, "\n"));
        let content = content.trim_end_matches([' ', '\t']);
        let content = strip_heading_colon(content);
        out.write_all(content.as_bytes())?;
        out.write_all(newline.as_bytes())?;
    }
    Ok(())
}

/// Drop the colon clap appends to a section heading.
///
/// clap renders `help_heading` as `HEADING:` with no way to opt out, which would leave
/// help reading `SCOPE:` while the report reads `SUMMARY`. Rather than fight the
/// formatter, the one rendering path we already own removes it.
///
/// Only a standalone all-caps heading line qualifies. `Usage: fdu ...` keeps its colon
/// because it carries content after it, and an error line keeps its colon for the same
/// reason — the test is that the whole line is the heading.
fn strip_heading_colon(line: &str) -> Cow<'_, str> {
    let Some(without_colon) = line.strip_suffix(':').or_else(|| {
        // With colour the reset sequence trails the text, so the colon sits inside it.
        line.strip_suffix("\u{1b}[0m").and_then(|inner| inner.strip_suffix(':'))
    }) else {
        return Cow::Borrowed(line);
    };
    let visible = strip_ansi(without_colon);
    let is_heading = !visible.is_empty()
        && visible.chars().any(char::is_alphabetic)
        && visible.chars().all(|c| c.is_uppercase() || c == ' ' || !c.is_alphabetic());
    if !is_heading {
        return Cow::Borrowed(line);
    }
    // Remove just that colon, keeping any trailing reset sequence intact.
    let colon = line.rfind(':').expect("the suffix match found one");
    Cow::Owned(format!("{}{}", &line[..colon], &line[colon + 1..]))
}

/// The visible characters of a line, with any ANSI escape sequences removed.
fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for escape in chars.by_ref() {
                if escape.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn finish(
    result: anyhow::Result<RunOutcome>,
    allow_partial: bool,
    diagnostic: &mut dyn Write,
    color: bool,
) -> u8 {
    match result {
        Ok(RunOutcome::Complete) => 0,
        Ok(RunOutcome::Partial) if allow_partial => 0,
        Ok(RunOutcome::Partial) => 2,
        Err(error) if is_broken_pipe(&error) => 0,
        Err(error) if is_usage_error(&error) => {
            let _ = writeln!(diagnostic, "{} {error}", paint("fdu:", STYLE_ERROR, color));
            2
        }
        Err(error) => {
            let headline = error.to_string();
            let _ = writeln!(diagnostic, "{} {headline}", paint("fdu:", STYLE_ERROR, color));
            // `I/O error at {path}: {source}` embeds its own source, so the first link in
            // the chain repeated the sentence verbatim -- two lines where one carried the
            // information, on the most common failure there is. Only that one link is
            // elided, and only when the headline really does end with it: a plain
            // `contains` over the whole chain would silently swallow a deeper cause that
            // happened to be a substring of the headline, which is a different bug wearing
            // this fix's clothes (fdu-zppc).
            let mut chain = error.chain().skip(1).peekable();
            let embedded = chain.peek().is_some_and(|first| headline.ends_with(&first.to_string()));
            if embedded {
                chain.next();
            }
            for cause in chain {
                let cause = format!("  caused by: {cause}");
                let _ = writeln!(diagnostic, "{}", paint(&cause, STYLE_CAUSE, color));
            }
            1
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

fn requested_color(args: &[OsString]) -> ColorWhen {
    let mut arguments = args.iter().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--" {
            break;
        }
        if argument == "--color" {
            return arguments.next().and_then(|value| color_value(value)).unwrap_or_default();
        }
        if let Some(value) = argument.to_str().and_then(|value| value.strip_prefix("--color=")) {
            return color_value(OsStr::new(value)).unwrap_or_default();
        }
    }
    ColorWhen::Auto
}

fn color_value(value: &OsStr) -> Option<ColorWhen> {
    match value.to_str()? {
        "auto" => Some(ColorWhen::Auto),
        "always" => Some(ColorWhen::Always),
        "never" => Some(ColorWhen::Never),
        _ => None,
    }
}

fn flag_is_present(args: &[OsString], flag: &str) -> bool {
    args.iter().skip(1).take_while(|argument| *argument != "--").any(|argument| argument == flag)
}

#[derive(Clone, Copy)]
// Each boolean is an independent external input to the color contract. Naming them here
// is clearer than encoding unrelated facts into bit flags or positional arguments.
#[allow(clippy::struct_excessive_bools)]
struct ColorContext {
    when: ColorWhen,
    json: bool,
    skill: bool,
    no_color_env: bool,
    force_color_env: bool,
    destination_is_terminal: bool,
}

impl ColorContext {
    fn from_environment(
        when: ColorWhen,
        json: bool,
        skill: bool,
        destination_is_terminal: bool,
    ) -> Self {
        let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        let force_color_env =
            std::env::var_os("FORCE_COLOR").is_some_and(|value| !value.is_empty() && value != "0");
        Self { when, json, skill, no_color_env, force_color_env, destination_is_terminal }
    }

    fn enabled(self) -> bool {
        if self.json || self.skill {
            return false;
        }
        match self.when {
            ColorWhen::Always => true,
            ColorWhen::Never => false,
            ColorWhen::Auto if self.no_color_env => false,
            ColorWhen::Auto if self.force_color_env => true,
            ColorWhen::Auto => self.destination_is_terminal,
        }
    }
}

/// Paint the guide's section headers with the shared header style.
///
/// A header is a line that starts at column zero and is upper case — the same shape the
/// report's view headers take, which is what makes the two surfaces look like one tool.
/// Everything else passes through untouched, so redirecting to a file or setting `NO_COLOR`
/// yields exactly the bytes the golden pins.
fn style_guide(guide: &str, color: bool) -> String {
    if !color {
        return guide.to_string();
    }
    let mut out = String::with_capacity(guide.len());
    for line in guide.lines() {
        let is_header = !line.is_empty()
            && !line.starts_with(char::is_whitespace)
            && line.chars().any(char::is_alphabetic)
            && line.chars().filter(|c| c.is_alphabetic()).all(char::is_uppercase);
        if is_header {
            out.push_str(&paint(line, STYLE_HEADING, true));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn paint(text: &str, style: AnsiStyle, color: bool) -> String {
    if color { format!("{style}{text}{style:#}") } else { text.to_string() }
}

fn compose_skill() -> String {
    compose_skill_from(SKILL_TEMPLATE)
}

fn compose_skill_from(template: &str) -> String {
    // Git checkouts may translate the Markdown resource to CRLF on Windows. Keep the
    // public skill byte-stable across installation platforms before substituting the
    // reviewed package version.
    template.replace("\r\n", "\n").replace("__FDU_VERSION__", env!("CARGO_PKG_VERSION"))
}

#[cfg(any(unix, windows))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    /// Every `--watch` run parses an interval before anything else, so this must work on
    /// every platform the binary ships to.
    ///
    /// Regression test. The anchor used to sit about 34,865 years past the epoch, which
    /// `SystemTime` accepts where it counts seconds and rejects on Windows, where it
    /// counts 100-nanosecond FILETIME ticks. Building it panicked before any input was
    /// examined, so `fdu --watch` aborted on Windows regardless of arguments -- and
    /// nothing caught it, because both watch integration tests are Unix-only and the
    /// scope-validation goldens exit before the watch path is reached. A CLI golden
    /// driving a real watch session on Windows CI is what finally surfaced it.
    #[cfg(feature = "watch")]
    #[test]
    fn a_watch_rule_carries_the_instant_and_cannot_be_read_as_a_data_row() {
        // The separator has to be recognisable as a boundary at a glance and never
        // mistakable for a row: every report row this tool prints starts with a
        // right-aligned size, so a rule starting with a box-drawing run cannot collide.
        let rule = report_format::watch_rule(UNIX_EPOCH + Duration::from_secs(1_786_386_151));
        assert_eq!(rule, "──── 2026-08-10T18:22:31.000000000Z ────");
        assert!(rule.starts_with('─'), "{rule}");

        // Colour is a decoration over it, never a requirement for reading it.
        assert_eq!(paint(&rule, STYLE_WATCH_RULE, false), rule);
        assert!(paint(&rule, STYLE_WATCH_RULE, true).contains(&rule));
    }

    #[test]
    fn an_interval_parses_without_overflowing_any_platforms_clock() {
        assert_eq!(parse_duration("2s").expect("seconds"), Duration::from_secs(2));
        assert_eq!(parse_duration("1h30m").expect("compound"), Duration::from_secs(5_400));
        assert_eq!(parse_duration("1w").expect("weeks"), Duration::from_secs(604_800));
        assert!(parse_duration("banana").is_err(), "a non-duration must be rejected, not parsed");
    }

    /// The watch loop's save throttle, as a table over every state that reaches it.
    ///
    /// Two of the three defects review found on this branch were transitions in here, and
    /// the second was introduced by fixing the first. End-to-end tests could not catch
    /// either: they observe whether a file changed on disk, which cannot distinguish "not
    /// due yet" from "due and skipped", nor a cleared flag from a retained one.
    #[cfg(feature = "watch")]
    #[test]
    fn a_save_is_due_only_when_a_change_is_pending_and_the_throttle_has_elapsed() {
        let interval = Duration::from_secs(1);
        let cases = [
            // (pending, since last save, due, what this case is)
            (true, Duration::from_secs(2), true, "pending and past the interval"),
            (true, interval, true, "pending, exactly at the interval: inclusive"),
            // The R5 case. Not due *now* -- and the flag stays set, which is the half that
            // was missing: the idle path saves it once the interval passes.
            (true, Duration::from_millis(1), false, "pending but throttled"),
            (false, Duration::from_secs(60), false, "nothing pending, however long it has been"),
            (false, Duration::ZERO, false, "nothing pending and just saved"),
        ];

        for (pending, since, want, case) in cases {
            assert_eq!(save_is_due(pending, since, interval), want, "{case}");
        }
    }

    /// A throttled change must survive every outcome except a completed write.
    #[cfg(feature = "watch")]
    #[test]
    fn only_a_completed_write_clears_the_pending_change() {
        // The R7 case is Skipped and Failed: clearing the flag for either means the idle
        // path never retries, so on a quiet tree the change is never persisted at all.
        assert!(!pending_after(SaveOutcome::Written), "a completed write persists the change");
        assert!(
            pending_after(SaveOutcome::Skipped),
            "a skipped save wrote nothing, so the change is still owed to disk",
        );
        assert!(pending_after(SaveOutcome::Failed), "a failed save must be retried, not forgotten");
    }

    /// The sequence that defeated the feature in its most common shape.
    #[cfg(feature = "watch")]
    #[test]
    fn a_burst_then_a_quiet_tree_still_persists() {
        let interval = Duration::from_secs(1);

        // A change arrives too soon after the last save, so nothing is written yet.
        let mut pending = true;
        assert!(!save_is_due(pending, Duration::from_millis(50), interval));
        assert!(pending, "the throttle must not consume the change");

        // The tree goes quiet: no further batches will ever arrive. The idle path is the
        // only remaining caller, and once the interval passes the save must happen.
        assert!(save_is_due(pending, Duration::from_secs(3), interval));

        // A skip at that point keeps it pending for the next idle tick rather than
        // silently dropping the session's work.
        pending = pending_after(SaveOutcome::Skipped);
        assert!(pending);
        pending = pending_after(SaveOutcome::Written);
        assert!(!pending, "once written, the loop stops rewriting an unchanged index");
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("output failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// A CLI with every axis at its default, so a test can vary exactly one.
    fn cli() -> Cli {
        Cli {
            path: Some(PathBuf::from(".")),
            scan_depth: None,
            one_filesystem: false,
            include: Vec::new(),
            exclude: Vec::new(),
            min_size: None,
            modified_since: None,
            modified_before: None,
            kind: None,
            // None, as clap now leaves it: the default belongs to the view.
            depth: None,
            limit: None,
            sort: None,
            reverse: false,
            size: "allocated".to_string(),
            view: Some("tree".to_string()),
            analyze: "none".to_string(),
            analysis_workers: 0,
            words_per_page: 250,
            format: "text".to_string(),
            color: ColorWhen::Auto,
            cache: "off".to_string(),
            cache_status: None,
            cache_clear: None,
            #[cfg(feature = "watch")]
            watch: false,
            #[cfg(feature = "watch")]
            interval: "2s".to_string(),
            allow_partial: false,
            help: None,
            version: None,
            docs: false,
            skill: false,
        }
    }

    fn query_error(cli: &Cli) -> String {
        cli.resolved_query().expect_err("expected a rejection").to_string()
    }

    #[test]
    fn a_bare_invocation_prints_help_instead_of_scanning_the_current_directory() {
        let mut bare_out = Vec::new();
        let mut bare_err = Vec::new();
        let bare = [OsString::from("fdu")];
        let status = run_with_io(&bare, &mut bare_out, &mut bare_err, false, false);

        let mut help_out = Vec::new();
        let mut help_err = Vec::new();
        let help = [OsString::from("fdu"), OsString::from("--help")];
        let help_status = run_with_io(&help, &mut help_out, &mut help_err, false, false);

        assert_eq!(status, 0, "showing help is a successful discovery action");
        assert_eq!(help_status, 0);
        assert_eq!(bare_out, help_out, "bare fdu should be exactly the long-help surface");
        assert!(bare_err.is_empty(), "help belongs on stdout");
        assert!(help_err.is_empty());
        let bare_help = String::from_utf8(bare_out).expect("help is UTF-8");
        assert!(bare_help.contains("Usage: fdu"));
        assert!(
            bare_help.lines().all(|line| line.trim_end() == line),
            "help should not pad blank lines with invisible whitespace"
        );
    }

    #[test]
    fn report_options_do_not_make_the_scan_path_optional() {
        let error = Cli::command()
            .color(ColorChoice::Never)
            .try_get_matches_from(["fdu", "--view", "summary"])
            .expect_err("a report without PATH must never fall back to the current directory");

        assert_eq!(error.kind(), clap::error::ErrorKind::MissingRequiredArgument);
        assert_eq!(error.exit_code(), 2);
        assert!(error.use_stderr());
        assert!(error.to_string().contains("<PATH>"));
    }

    // ---- the five axes translate into library types, and nothing else ----

    #[test]
    fn views_parse_as_an_ordered_comma_list() {
        let parsed = Cli { view: Some("types,tree,summary".to_string()), ..cli() }
            .resolved_query()
            .expect("views parse");
        assert_eq!(parsed.views, vec![ViewSpec::Types, ViewSpec::Tree, ViewSpec::Summary]);
    }

    #[test]
    fn an_unknown_view_names_every_valid_value() {
        let message = query_error(&Cli { view: Some("bogus".to_string()), ..cli() });
        // The rejection is how the vocabulary is discovered, so every value must be in it.
        for view in ViewSpec::ALL {
            let label = view.label();
            assert!(message.contains(label), "{label} missing from: {message}");
        }
        assert!(message.contains("full"), "the total must be listed too: {message}");
    }

    #[test]
    fn a_repeated_view_is_a_typo_not_a_no_op() {
        let message = query_error(&Cli { view: Some("tree,tree".to_string()), ..cli() });
        assert!(message.contains("appears more than once"), "{message}");
    }

    #[test]
    fn an_empty_list_entry_is_rejected() {
        let message = query_error(&Cli { view: Some("tree,,types".to_string()), ..cli() });
        assert!(message.contains("empty entry"), "{message}");
    }

    #[test]
    fn bounds_accept_all_as_well_as_a_number() {
        let parsed = Cli { depth: Some("all".to_string()), limit: Some("3".to_string()), ..cli() }
            .resolved_query()
            .expect("bounds parse");
        assert_eq!(parsed.selection.depth, Some(Bound::All));
        assert_eq!(parsed.selection.limit, Some(Bound::Limit(3)));
        // du's meaning of depth 0 survives the rename from --max-depth.
        let zero = Cli { depth: Some("0".to_string()), ..cli() }.resolved_query().expect("parses");
        assert_eq!(zero.selection.depth, Some(Bound::Limit(0)));
    }

    /// The default the CLI used to declare itself now comes from the library, so every
    /// surface renders the same tree for the same request. While the CLI owned it, a
    /// Python caller leaving depth unset got an unbounded tree and no warning.
    /// The CLI's wording and the library's must stay one rule with different knob names,
    /// because that equivalence is what the parity harness verifies mechanically.
    ///
    /// Asserted against the constant and the vocabulary rather than by quoting prose. The
    /// first three versions of this test quoted phrases and went stale the moment the rule
    /// was reworded, which is a test measuring its own copy of the thing under test.
    #[test]
    fn the_watch_guidance_substitutes_whole_words_only() {
        let source = fdu_core::scan::WATCH_SCOPE_GUIDANCE;
        let text = watch_scope_guidance();

        // A sequential replace produced `--scan---depth`: max_depth became --scan-depth and
        // then `depth` matched inside the replacement. That is fdu-7j6z, and it reappeared
        // here the moment the same shortcut was taken.
        assert!(!text.contains("---"), "{text} re-substituted a replacement");

        // Whole words throughout, because `--scan-depth` contains `depth`. Checking with a
        // plain `contains` fails here for the same reason a sequential replace corrupts the
        // text -- which is the point, and is why this assertion is written the careful way.
        let names_word = |haystack: &str, word: &str| {
            // Hyphens stay inside the token, so `--scan-depth` is one word and not three.
            // Splitting on them is what made this assertion see a bare `depth` that is not
            // there -- the same tokenisation mistake, one layer up.
            haystack
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
                .any(|w| w == word)
        };
        for (field, flag) in WATCH_SCOPE_VOCABULARY {
            if names_word(source, field) {
                assert!(text.contains(flag), "{text} must name {flag} where the rule says {field}");
            }
            assert!(!names_word(&text, field), "{text} still names {field} untranslated");
        }

        // Substitution only: everything that is not a knob name survives untouched, so the
        // two surfaces state the same rule rather than two rules that happen to agree.
        let mut rebuilt = text.clone();
        for (field, flag) in WATCH_SCOPE_VOCABULARY {
            rebuilt = rebuilt.replace(flag, field);
        }
        assert_eq!(rebuilt, source, "the CLI wording must be the library's, knob names aside");
    }

    #[test]
    fn an_unnamed_depth_takes_the_view_default_rather_than_unbounded() {
        let parsed = cli().resolved_query().expect("parses");
        assert_eq!(parsed.selection.depth, None, "the CLI must not invent a default");
        assert_eq!(parsed.depth_for(ViewSpec::Tree), Bound::Limit(2));
        // Only the tree renders a hierarchy, so the question does not arise elsewhere.
        assert_eq!(parsed.depth_for(ViewSpec::Files), Bound::All);
    }

    #[test]
    fn patterns_are_repeatable_flags_so_brace_globs_survive() {
        // Comma-splitting these would shred `*.{rs,toml}`, which is why open-valued flags
        // are repeatable and only closed vocabularies are lists.
        let parsed = Cli {
            include: vec!["*.{rs,toml}".to_string(), "docs/**".to_string()],
            exclude: vec!["**/target/**".to_string()],
            ..cli()
        }
        .resolved_query()
        .expect("patterns parse");
        assert_eq!(parsed.selection.include.len(), 2);
        assert_eq!(parsed.selection.exclude.len(), 1);
    }

    #[test]
    fn value_grammars_reach_the_cli_with_their_suggestions_intact() {
        // The CLI must not restate the grammar; it hands the string to the library and
        // surfaces the library's own message, suggestion and all.
        let fractional = query_error(&Cli { modified_since: Some("1.5h".to_string()), ..cli() });
        assert!(fractional.contains("1h30m"), "{fractional}");
        let calendar = query_error(&Cli { modified_before: Some("3months".to_string()), ..cli() });
        assert!(calendar.contains("use days"), "{calendar}");
        let size = query_error(&Cli { min_size: Some("10X".to_string()), ..cli() });
        assert!(size.contains("unknown size unit"), "{size}");
    }

    #[test]
    fn the_modified_window_reaches_the_selection_as_nanoseconds() {
        let parsed = Cli {
            modified_since: Some("@1000".to_string()),
            modified_before: Some("@2000".to_string()),
            ..cli()
        }
        .resolved_query()
        .expect("window parses");
        assert_eq!(parsed.selection.modified.since, Some(1_000_000_000_000));
        assert_eq!(parsed.selection.modified.before, Some(2_000_000_000_000));
    }

    #[test]
    fn kinds_sort_and_size_translate_to_their_library_values() {
        let parsed = Cli {
            kind: Some("file,dir".to_string()),
            sort: Some("mtime".to_string()),
            size: "apparent".to_string(),
            reverse: true,
            ..cli()
        }
        .resolved_query()
        .expect("parses");
        assert_eq!(parsed.selection.kinds, vec![EntryKind::File, EntryKind::Dir]);
        assert_eq!(parsed.selection.sort, Some(SortKey::Mtime));
        assert_eq!(parsed.selection.size, SizeMetric::Apparent);
        assert!(parsed.selection.reverse);
    }

    /// Every flag the guide names must exist.
    ///
    /// tbd states this rule for its own docs surface as "the menu must only name
    /// selectors that exist", and it is worth a test rather than an intention: prose that
    /// advertises a flag the binary does not have is worse than prose that says nothing,
    /// because the reader spends their trust before finding out.
    #[test]
    fn the_guide_only_names_flags_that_exist() {
        let command = Cli::command();
        let known: Vec<String> = command
            .get_arguments()
            .filter_map(|arg| arg.get_long().map(|long| format!("--{long}")))
            .collect();
        let mut named = Vec::new();
        for token in DOCS.split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_')) {
            if token.starts_with("--") && token.len() > 2 {
                named.push(token.trim_end_matches('-').to_string());
            }
        }
        assert!(!named.is_empty(), "the guide should name some flags");
        for flag in &named {
            assert!(known.contains(flag), "--docs names {flag}, which is not a flag");
        }
        // And the pointer must name the flag that prints the guide.
        assert!(DOCS_POINTER.contains("--docs"), "{DOCS_POINTER}");
    }

    /// The vocabularies the guide lists must be the ones the parsers accept.
    #[test]
    fn the_guide_only_names_views_and_analyzers_that_parse() {
        for view in [
            "tree",
            "extensions",
            "types",
            "families",
            "languages",
            "documents",
            "files",
            "summary",
        ] {
            assert!(DOCS.contains(view), "the guide should list the {view} view");
            ViewSpec::parse(view).unwrap_or_else(|_| panic!("{view} must parse"));
        }
        for set in ["none", "lines", "code", "words", "all"] {
            assert!(DOCS.contains(set), "the guide should list the {set} analyzer value");
            AnalysisSet::parse(set).unwrap_or_else(|_| panic!("{set} must parse"));
        }
    }

    /// The stale-schema bug, made unrepeatable.
    ///
    /// The bump to `fdu.report/3` left "metric summaries use fdu.report/2" in the help
    /// text, contradicting what the binary emits. It survived a vocabulary sweep and a
    /// full `make check`, because no test compared prose against the constants.
    #[test]
    fn no_surface_names_a_schema_the_binary_does_not_emit() {
        const PREFIX: &str = "fdu.report/";
        let live = [report_format::REPORT_SCHEMA, report_format::CONTENT_REPORT_SCHEMA];
        for (surface, text) in [("--docs", DOCS.to_string()), ("--skill", compose_skill())] {
            let mut rest = text.as_str();
            let mut found = 0;
            while let Some(at) = rest.find(PREFIX) {
                rest = &rest[at..];
                // The version is the digit run after the prefix; whatever punctuation
                // follows belongs to the sentence, not to the schema string.
                let digits = rest[PREFIX.len()..].chars().take_while(char::is_ascii_digit).count();
                let named = &rest[..PREFIX.len() + digits];
                assert!(
                    live.contains(&named),
                    "{surface} names {named}, but the binary emits {live:?}"
                );
                found += 1;
                rest = &rest[PREFIX.len() + digits..];
            }
            assert!(found > 0, "{surface} should state which schema it emits");
        }
    }

    /// The defect this axis exists to fix: a request that reads every eligible file must
    /// not print a report identical to one that read nothing.
    #[test]
    fn requesting_analysis_selects_a_view_that_displays_it() {
        let cases = [
            (AnalysisSet::NONE, ViewSpec::Tree),
            (AnalysisSet::NONE.with_lines(), ViewSpec::Families),
            (AnalysisSet::NONE.with_code(), ViewSpec::Languages),
            (AnalysisSet::NONE.with_words(), ViewSpec::Documents),
            (AnalysisSet::ALL, ViewSpec::Families),
        ];
        for (profile, expected) in cases {
            assert_eq!(ViewSpec::default_for(profile), expected, "default view for {profile:?}");
            if profile.is_enabled() {
                assert!(
                    view_displays_analysis(ViewSpec::default_for(profile)),
                    "a paid-for run must default to a view that shows what it bought"
                );
            }
        }
    }

    /// An explicit view always wins; the derivation only supplies the default.
    #[test]
    fn an_explicit_view_overrides_the_derived_default() {
        let resolved = resolve_views(Some("tree"), AnalysisSet::ALL).expect("resolve");
        assert_eq!(resolved.selected, vec![ViewSpec::Tree]);
        assert!(resolved.omitted.is_empty());
    }

    #[test]
    fn view_full_expands_to_the_summary_views_the_analyzer_set_can_answer() {
        let bare = resolve_views(Some("full"), AnalysisSet::NONE).expect("resolve");
        assert_eq!(bare.omitted, vec![ViewSpec::Documents], "documents needs content");
        assert!(!bare.selected.contains(&ViewSpec::Documents));

        let analyzed = resolve_views(Some("full"), AnalysisSet::ALL).expect("resolve");
        assert!(analyzed.omitted.is_empty());
        // `full` is the summary views: both bounded presets are in, and the unbounded
        // enumeration is out, because an enumeration inside a digest destroys the digest.
        assert!(analyzed.selected.contains(&ViewSpec::Largest));
        assert!(analyzed.selected.contains(&ViewSpec::Recent));
        assert!(!analyzed.selected.contains(&ViewSpec::Files), "{:?}", analyzed.selected);
        assert_eq!(
            analyzed.selected,
            ViewSpec::ALL.into_iter().filter(|view| view.is_summary_view()).collect::<Vec<_>>(),
            "every summary view, in table order"
        );

        // `all` names the axis, so combining it with a view is a usage error rather than
        // a silently-widened request.
        let combined = resolve_views(Some("full,tree"), AnalysisSet::NONE)
            .expect_err("full cannot be combined")
            .to_string();
        assert!(combined.contains("cannot be combined"), "{combined}");
    }

    /// The display contract has two directions, and they now live in two places.
    ///
    /// What a request could not display is a fact about the report, so it travels on the
    /// report and every surface states it. What a request paid to read is telemetry about
    /// the run, which the report envelope deliberately excludes, so it stays here with the
    /// performance footer. Splitting them is what let the Python surface say the first
    /// (fdu-x8u6); this pins each to its own home.
    #[test]
    fn the_display_contract_reports_unspent_reads_from_the_cli() {
        let unspent = resolve_views(Some("tree"), AnalysisSet::ALL).expect("resolve");
        let notes = display_notes(&unspent, AnalysisSet::ALL, 1_200);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("no selected view displays content metrics"), "{notes:?}");
        assert!(notes[0].contains("1.1 KiB"), "the note quantifies what was read: {notes:?}");

        // A view that does display the metrics earns no note at all.
        let spent = resolve_views(Some("families"), AnalysisSet::ALL).expect("resolve");
        assert!(display_notes(&spent, AnalysisSet::ALL, 1_200).is_empty());

        // Neither does a metadata-only run, which bought nothing to display.
        let plain = resolve_views(None, AnalysisSet::NONE).expect("resolve");
        assert!(display_notes(&plain, AnalysisSet::NONE, 0).is_empty());

        // And the omission note is no longer the CLI's to make, in either direction.
        let omitted = resolve_views(Some("full"), AnalysisSet::NONE).expect("resolve");
        assert!(!omitted.omitted.is_empty(), "full without analyzers must drop documents");
        assert!(
            display_notes(&omitted, AnalysisSet::NONE, 0).is_empty(),
            "the omission travels on the report now"
        );
    }

    /// Principle 13, the direction that protects the user: no view, at any content
    /// setting, may cause a file body to be opened that `--analyze` did not authorize.
    #[test]
    fn no_view_enables_an_analyzer() {
        for view in ViewSpec::ALL {
            let spec = view.label();
            let cli = Cli { view: Some(spec.to_string()), ..cli() };
            let profile = cli.parse_analysis().expect("analysis").profile;
            assert_eq!(
                profile,
                AnalysisSet::NONE,
                "--view {spec} must leave the content axis empty"
            );
        }
    }

    #[test]
    fn analysis_profile_workers_and_page_denominator_parse_before_io() {
        let parsed =
            Cli { analyze: "lines".to_string(), analysis_workers: 3, words_per_page: 250, ..cli() };
        assert_eq!(
            parsed.parse_analysis().expect("analysis"),
            AnalysisRequest { profile: AnalysisSet::NONE.with_lines(), workers: 3 }
        );
        assert_eq!(parsed.resolved_query().expect("query").words_per_page, 250);

        let invalid = Cli { analyze: "deep".to_string(), ..cli() }
            .parse_analysis()
            .expect_err("invalid profile")
            .to_string();
        assert!(invalid.contains("none, lines, code, words, all"), "{invalid}");
        assert!(invalid.contains("--analyze"), "the message names the flag: {invalid}");
        assert!(query_error(&Cli { words_per_page: 0, ..cli() }).contains("positive"));
    }

    #[test]
    fn real_basic_documents_report_exposes_lines_words_pages_and_schema_two() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("notes.md"), b"one two\n\nthree\n").expect("write");
        let command = Cli {
            path: Some(root.path().to_path_buf()),
            analyze: "lines".to_string(),
            view: Some("documents".to_string()),
            format: "json".to_string(),
            size: "apparent".to_string(),
            ..cli()
        };
        let mut output = Vec::new();
        let outcome =
            command.run(&mut output, &mut Vec::new(), false, false).expect("run content report");
        assert_eq!(outcome, RunOutcome::Complete);
        let output = String::from_utf8(output).expect("UTF-8 JSON");
        assert!(output.contains("\"schema\": \"fdu.report/5\""), "{output}");
        assert!(output.contains("\"physical_lines\": 3"), "{output}");
        assert!(output.contains("\"raw_words\": 3"), "{output}");
        assert!(output.contains("\"words_per_page\": 250"), "{output}");
        assert!(output.contains("\"content-basic-v1\""), "{output}");
    }

    #[test]
    fn one_shot_text_ends_with_a_plain_or_dimmed_performance_footer() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("one.txt"), b"one\n").expect("write");
        std::fs::write(root.path().join("two.txt"), b"two\n").expect("write");
        let command = Cli {
            path: Some(root.path().to_path_buf()),
            analyze: "lines".to_string(),
            view: Some("summary".to_string()),
            size: "apparent".to_string(),
            ..cli()
        };

        let mut plain = Vec::new();
        command.run(&mut plain, &mut Vec::new(), false, false).expect("plain report");
        let plain = String::from_utf8(plain).expect("plain UTF-8");
        // `summary` displays no content metric, so this run also earns the paid-for-
        // nothing note; the footer is the line before it rather than the last line.
        let footer =
            plain.lines().find(|line| line.contains("Performance:")).expect("performance footer");
        assert!(
            footer.starts_with("Performance: walked 2 files / 8 B; content read 8 B at "),
            "{plain}"
        );
        assert!(footer.contains("2 fresh at "), "{footer}");
        assert!(footer.contains("0 cached"), "{footer}");
        assert!(footer.contains("cold scan; total "), "{footer}");
        assert!(!footer.contains('\u{1b}'), "color-disabled output must not contain ANSI");

        let mut colored = Vec::new();
        Cli { color: ColorWhen::Always, ..command }
            .run(&mut colored, &mut Vec::new(), false, false)
            .expect("colored report");
        let colored = String::from_utf8(colored).expect("colored UTF-8");
        let footer = colored
            .lines()
            .find(|line| line.contains("Performance:"))
            .expect("colored performance footer");
        assert!(
            footer.starts_with("\u{1b}[90mPerformance:"),
            "the footer must use terminal gray when color is active: {colored:?}"
        );
        // The notes are footer-adjacent telemetry and share its dimming, so a terminal
        // reading the report sees one quiet block rather than a bright interruption.
        let note =
            colored.lines().find(|line| line.contains("note: --analyze")).expect("colored note");
        assert!(note.starts_with("\u{1b}[90mnote:"), "notes share the footer style: {colored:?}");
    }

    #[test]
    fn performance_footer_names_units_cache_work_and_metadata_tier() {
        let footer = performance_footer(
            PerformanceSummary {
                walked_files: 12_345,
                walked_bytes: 2_048,
                fresh_files: 3_000,
                bytes_read: 2_048,
                analysis_ns: 2_000_000_000,
                cached_files: 2,
                cached_bytes: 4_096,
                source: ReportSource::WarmRevalidate,
            },
            Duration::from_millis(2_500),
        );

        assert_eq!(
            footer,
            "Performance: walked 12,345 files / 2.0 KiB; content read 2.0 KiB at 1.0 KiB/s; analysis 3,000 fresh at 1.5k files/s, 2 cached / 4.0 KiB; warm revalidation; total 2.50 s"
        );
    }

    #[test]
    fn machine_formats_omit_the_performance_footer() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("one.txt"), b"one\n").expect("write");
        for format in ["json", "jsonl", "yaml"] {
            let command = Cli {
                path: Some(root.path().to_path_buf()),
                view: Some("summary".to_string()),
                size: "apparent".to_string(),
                format: format.to_string(),
                ..cli()
            };
            let mut output = Vec::new();
            command.run(&mut output, &mut Vec::new(), false, false).expect("machine report");
            let output = String::from_utf8(output).expect("machine UTF-8");
            assert!(!output.contains("Performance:"), "{format}: {output}");
        }
    }

    #[test]
    fn formats_parse_and_machine_formats_are_never_colorized() {
        for (value, expected) in [
            ("text", report_format::Format::Text),
            ("json", report_format::Format::Json),
            ("jsonl", report_format::Format::Jsonl),
            ("yaml", report_format::Format::Yaml),
        ] {
            let cli = Cli { format: value.to_string(), ..cli() };
            assert_eq!(cli.parse_format().expect("format parses"), expected);
            assert_eq!(cli.machine_format(), value != "text");
        }
        let message = Cli { format: "xml".to_string(), ..cli() }
            .parse_format()
            .expect_err("rejected")
            .to_string();
        assert!(message.contains("text, json, jsonl, yaml"), "{message}");
    }

    #[test]
    fn an_unparseable_request_costs_no_filesystem_work() {
        // Parsing precedes open(), so a typo reports itself instead of arriving after a
        // scan of a large tree.
        let message = query_error(&Cli {
            path: Some(PathBuf::from("/nonexistent-root-that-should-not-be-scanned")),
            view: Some("bogus".to_string()),
            ..cli()
        });
        assert!(message.contains("expected one of"), "{message}");
    }

    /// clap appends a colon to every section heading with no way to opt out, so the one
    /// rendering path we own removes it. The test is that the whole line is the heading —
    /// anything carrying content after the colon keeps it.
    #[test]
    fn only_a_standalone_heading_loses_its_colon() {
        // Plain and coloured headings, where the colour puts the colon inside the reset.
        assert_eq!(strip_heading_colon("SCOPE:"), "SCOPE");
        assert_eq!(strip_heading_colon("CACHE MANAGEMENT:"), "CACHE MANAGEMENT");
        assert_eq!(
            strip_heading_colon("\u{1b}[1m\u{1b}[36mSCOPE:\u{1b}[0m"),
            "\u{1b}[1m\u{1b}[36mSCOPE\u{1b}[0m"
        );

        // Lines that merely contain a colon are untouched.
        for line in [
            "Usage: fdu [OPTIONS] <PATH>",
            "fdu: invalid --view \"bogus\": expected one of tree",
            "  Scope      PATH, --scan-depth",
            "note: omitted documents — requires content analysis",
            "",
            "  --scan-depth <N>  Limit scanning and retention to N entry levels",
        ] {
            assert_eq!(strip_heading_colon(line), line, "{line:?} must keep its shape");
        }
    }

    #[test]
    fn every_help_heading_is_upper_case_and_bare() {
        let mut rendered = Vec::new();
        write_styled(&mut rendered, &Cli::command().render_help(), false).expect("render");
        let rendered = String::from_utf8(rendered).expect("utf-8 help");
        let headings: Vec<&str> = rendered
            .lines()
            .filter(|line| {
                !line.is_empty()
                    && !line.starts_with(char::is_whitespace)
                    && line.chars().all(|c| c.is_uppercase() || c == ' ' || c == ':')
                    && line.chars().any(char::is_alphabetic)
            })
            .collect();
        assert!(headings.len() >= 8, "expected the section headings, got {headings:?}");
        for heading in headings {
            assert!(!heading.ends_with(':'), "{heading:?} still carries clap's colon");
            assert_eq!(heading, heading.to_uppercase(), "{heading:?} is not upper case");
        }
    }

    #[test]
    fn paint_is_a_no_op_when_color_is_off() {
        assert_eq!(paint("text", STYLE_HEADING, false), "text");
        assert!(paint("text", STYLE_HEADING, true).contains("\u{1b}["));
    }

    #[test]
    fn color_decision_has_stable_precedence_and_machine_output_is_plain() {
        let auto_terminal = ColorContext {
            when: ColorWhen::Auto,
            json: false,
            skill: false,
            no_color_env: false,
            force_color_env: false,
            destination_is_terminal: true,
        };

        assert!(auto_terminal.enabled());
        assert!(!ColorContext { destination_is_terminal: false, ..auto_terminal }.enabled());
        assert!(
            ColorContext { force_color_env: true, destination_is_terminal: false, ..auto_terminal }
                .enabled()
        );
        assert!(
            !ColorContext { no_color_env: true, force_color_env: true, ..auto_terminal }.enabled()
        );
        assert!(
            ColorContext {
                when: ColorWhen::Always,
                no_color_env: true,
                destination_is_terminal: false,
                ..auto_terminal
            }
            .enabled()
        );
        assert!(
            !ColorContext { when: ColorWhen::Never, force_color_env: true, ..auto_terminal }
                .enabled()
        );
        assert!(!ColorContext { json: true, ..auto_terminal }.enabled());
        assert!(!ColorContext { skill: true, ..auto_terminal }.enabled());
    }

    #[test]
    fn portable_skill_is_self_contained_and_exactly_versioned() {
        let skill = compose_skill();

        assert!(skill.starts_with("---\nname: fdu\n"));
        assert!(!skill.contains('\r'), "the public skill must use portable LF endings");
        assert!(skill.contains(&format!("uvx --from fdu=={} fdu", env!("CARGO_PKG_VERSION"))));
        assert!(!skill.contains("__FDU_VERSION__"));
        assert!(!skill.contains("uvx --from fdu fdu"));
        assert!(!skill.contains("fdu==latest"), "the runnable command must not float releases");
        assert_eq!(
            compose_skill_from("---\r\nversion: __FDU_VERSION__\r\n"),
            format!("---\nversion: {}\n", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn run_outcomes_and_broken_pipes_have_stable_exit_codes() {
        let mut diagnostic = Vec::new();
        assert_eq!(finish(Ok(RunOutcome::Complete), false, &mut diagnostic, false), 0);
        assert_eq!(finish(Ok(RunOutcome::Partial), false, &mut diagnostic, false), 2);
        assert_eq!(finish(Ok(RunOutcome::Partial), true, &mut diagnostic, false), 0);

        let broken_pipe =
            anyhow::Error::new(io::Error::new(io::ErrorKind::BrokenPipe, "reader closed"))
                .context("render output");
        assert_eq!(finish(Err(broken_pipe), false, &mut diagnostic, false), 0);
        assert!(diagnostic.is_empty());

        let args = [OsString::from("fdu"), OsString::from("--help")];
        assert_eq!(
            run_with_io(&args, &mut FailingWriter, &mut diagnostic, false, false),
            1,
            "a non-pipe help-output failure is fatal"
        );
    }
}
