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

use fdu_core::content::AnalysisSet;
use fdu_core::control::ControlCoverage;
#[cfg(feature = "watch")]
use fdu_core::query::parse_when;
use fdu_core::query::{
    AxisNames, Delivery, IgnoredEntries, ReadSpec, ReportSource, Request, RequestError,
    RequestSpec, ViewSpec, WatchDelivery, parse_cache_policy,
};
use fdu_core::report_format;
use fdu_core::report_format::human_count;
use fdu_core::{CachePolicy, CacheScope, CacheState, default_cache_path};
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
#[cfg(feature = "watch")]
const STYLE_WATCH_RULE: AnsiStyle = AnsiColor::BrightBlack.on_default();
const CLI_STYLES: Styles = Styles::styled()
    .header(STYLE_HEADING)
    .usage(STYLE_HEADING)
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .error(STYLE_ERROR)
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Yellow.on_default());

/// The short starting point `--help` gives before handing off to `--docs`.
///
/// Help stays the flag reference it is supposed to be.  The guide it points at used to
/// be split across `before_help` and `after_help`, which put a page of prose *above* the
/// tool's own description — the reader met the examples before learning what the command
/// was.
const DOCS_POINTER: &str = r"Examples:
  fdu .                     directory sizes (metadata only)
  fdu . --exclude-ignored   omit entries covered by .gitignore
  fdu . --view=summary      one total for the tree
  fdu . --analyze=code      standard lines of code by language
  fdu . --kind dir --include .venv --modified-before 7d --long
  fdu . --kind dir --include node_modules --modified-before 30d --long
  fdu . --kind dir --include target --modified-before 30d --format paths

Run `fdu --docs` for more commands, cache behavior, and the full usage guide.";

/// The guide `--docs` prints: common questions, the two axes, cache behavior, and the
/// contracts worth knowing before automating against the output.
///
/// Composed per build, because the guide names only flags this binary has. A command
/// line built without `watch` has no `--watch` or `--interval`, and a guide that still
/// named them would send its reader to flags the parser rejects. Dropping them from
/// that build's guide, rather than keeping them as flags that only refuse, keeps its
/// guide, `--help`, and parser describing one command. The two arguments are the
/// watch example with its note, and the Mode axis's flags.
macro_rules! docs_guide {
    ($watch_composition:literal, $mode_flags:literal) => {
        concat!(
            r"fdu — a fast, incremental file roll-up engine.

START HERE
  A report requires a PATH. Use `.` for the current directory.

    fdu .                                      directory sizes (the default)
    fdu . --exclude-ignored                    omit entries covered by .gitignore
    fdu . --view=summary                       one total for the tree
    fdu . --view=languages                     languages by byte size
    fdu . --view=families,types,extensions     three file-kind breakdowns
    fdu . --view=recent --limit=10             ten most recently modified files
    fdu . --analyze=lines                      physical lines and raw words
    fdu . --analyze=lines --view=languages     those metrics by language
    fdu . --analyze=code                       standard lines of code by language
    fdu . --analyze=words                      prose volume by document type

  `fdu .` is metadata-only. It prints a tree in allocated bytes, largest first,
  to depth 2, with at most 10 children per directory. Hidden and ignored entries
  are included; .gitignore is read to label ignored shares, not to exclude them.

VIEWS AND ANALYSIS
  --view chooses the question the report answers. Several views share one scan
    and one requested analysis; adding a view does not run a second scan.
  --analyze opts into reading eligible file bodies. Without it, regular file
    contents are not opened. Compatible cached results avoid rereading unchanged
    bodies, so a repeated content analysis can be much cheaper.

  Naming analyzers selects a view that displays them: code selects languages,
  words selects documents, and lines or a multi-analyzer set selects families.
  Name --view for a different projection; it always wins.

  A view never turns on an analyzer, because choosing how to look at a result
  should not quietly authorize reading every file in the tree. If a selected
  view cannot display requested analysis, fdu still performs the analysis and
  prints a note. --view=full names any view it had to skip.

MORE COMPOSITIONS
  fdu ~/Downloads --view=extensions
  fdu . --view=types,families --format=json
  fdu . --analyze=words --view=documents
  fdu PATH --view=largest --limit=100                        the 100 largest files
  fdu PATH --view=files --kind=file --modified-since=1h      files changed lately
  fdu PATH --view=files --only-ignored --format=jsonl        what .gitignore covers
",
            $watch_composition,
            r"
  largest and recent are presets over files, not more views to learn:
    largest = files --sort size --limit 20, regular files only
    recent  = files --sort mtime --limit 20, regular files only
  --sort and --limit still override them. files alone is complete: every
  matching entry, in name order. full keeps its bounded digest, without list/files.

LIST FORMATS AND OLD BUILD DIRECTORIES
  The metadata default view is list; its default format is tree. These agree:
    fdu PATH
    fdu PATH --view list --format tree
  Tree keeps the current directory roll-ups, depth 2, ten children per directory.
  Files contribute to totals without new leaf rows. --depth all expands levels;
  --limit all removes row caps. Flat list limits apply to the whole result.

  --format paths gives matching paths only; --long adds size, age, and path.
  Flat lists are complete and size-ranked by default; --sort name lists by name.
  JSON, JSONL, and YAML give exact metrics. text keeps automatic human tables.
  Tree/paths/long require a single list view; use text or machine formats for
  grouped/mixed views and full. largest/recent accept paths/long, keeping file ranks.
  Legacy files keeps name order; legacy tree keeps structured tree output.
  Explicit paths/long overrides the legacy tree presentation. Format flags conflict.

  fdu PATH --kind dir --include .venv --modified-before 7d --long
  fdu PATH --kind dir --include node_modules --modified-before 30d --format long
  fdu PATH --kind dir --include target --modified-before 30d --format paths
  fdu PATH --kind dir --include .venv --include venv --include node_modules --include target --modified-before 30d --long --sort mtime --reverse
  fdu PATH --kind dir --include .venv --modified-before 30d --format json

  Kind, name/path, size, and age are filters. Repeated includes form a union.
  Directory sizes sum eligible regular-file contents, excluding inode/symlink bytes.
  Age uses the newest modification of the root or an eligible descendant, including
  directories and symlinks. Empty directories use their own time; future age is negative.
  This is modification activity, not access or last use. target is a naming convention.
  Exclusions win throughout the subtree before size/age filtering. Nested matching
  roots can overlap; aggregate views count the covered contents once. --size apparent
  selects logical bytes. Paths/long omit the footer and send bound notices to stderr.

SIX AXES, AND EVERY OPTION BELONGS TO EXACTLY ONE
  Scope      PATH, --scan-depth, --one-filesystem       what is scanned and cached
             --gitignore-budget, --gitignore-line-limit, --no-gitignore
  Content    --analyze none|lines|code|words|all        which file bodies are read
  Selection  --include, --exclude, --depth, --limit     which entries are considered
             --exclude-ignored, --only-ignored
  View       list,summary,tree,families,types,extensions,languages,documents,
             largest,recent,files,full
  Format     --format text|tree|paths|long|json|jsonl|yaml, --tree, --long, --color
  Mode       ",
            $mode_flags,
            r"

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
  Unchanged results are restored from a separate sidecar written by the same
  analyzer set; any other set, wider or narrower, reads the files again.
  --cache=only never opens source files and fails if requested content is absent.

CACHE BEHAVIOR
  No ordinary view requires a preexisting cache. Metadata-only one-shot reports
  still inspect current metadata; under --cache=auto they skip loading a snapshot
  that cannot make that work cheaper, though a complete indexed scan may write one.

  Content analysis is where repeated-run caching pays most. The first run reads
  eligible file bodies. A compatible later run reuses results for unchanged files
  and reads only changed or newly eligible bodies; the performance footer reports
  fresh and cached analysis separately. Repeat the same --analyze command to see it.

  --cache=only is different: it does no filesystem verification, requires a
  compatible snapshot and content sidecar for the requested analysis, and labels
  its answer stale. --cache=off neither reads nor writes fdu's cache.

IGNORE RULES
  Every report reads each .gitignore in the tree, and summary, tree, and extension
  rows end with how much of their size its rules ignore, as `(128 B ignored)`.
  A directory a rule ignores is ignored with everything below it. Unignored is not
  tracked: .git is unignored unless a rule names it. --exclude-ignored and
  --only-ignored select one side after the scan; they do not prune metadata work
  or content analysis. Sort and --min-size follow the size shown.
  --no-gitignore reads no rules and shows no share. Only per-directory .gitignore
  files apply, not core.excludesFile, .git/info/exclude, or a global ignore file,
  and matching is case-sensitive on every platform. An unreadable .gitignore makes
  the result partial, like any unreadable path. A .gitignore past --gitignore-budget
  or --gitignore-line-limit is refused whole and named in a note: sizes stay exact,
  ignored shares under that directory do not.

OUTPUT AND AUTOMATION
  Every machine report uses fdu.report/7; watch changes use fdu.stream/2.
  Cache status is its own document in every machine format: fdu.cache/2.
  Summary, tree, extension, and file rows carry `ignored`: null under --no-gitignore.
  Text language rows use canonical names; machine formats retain lowercase IDs.
  Metric rows include detection source, confidence, origin flags, and coverage.
  One-shot text reports end with a gray performance line; machine formats omit it.
  JSON numbers above 2^53 (fingerprints, option hashes, nanosecond timestamps)
  lose precision in IEEE 754 binary64 parsers such as JavaScript JSON.parse.
  Results go to stdout; warnings and errors go to stderr.
  The command never prompts, pages, or animates progress.
  Reports require an explicit PATH; bare `fdu` prints help and scans nothing.
  `fdu --skill` prints a portable agent skill describing this same surface.

EXIT STATUS
  0  Complete result, or a partial result accepted with --allow-partial
  1  Fatal filesystem or cache error
  2  Partial result, or command-line usage error
"
        )
    };
}

/// The guide for a command line that can watch.
#[cfg(feature = "watch")]
const DOCS: &str = docs_guide!(
    "  fdu --watch --view files --format jsonl PATH              a tail -f for a tree

  --interval throttles rendering only; change detection is event-driven and
  unaffected by it, so an idle tree costs nothing between changes.
  The duration uses the age grammar: `2s`, `200ms`, `1h30m`.
",
    "--cache, --watch, --analysis-workers"
);
/// The guide for a command line built without `watch`, which names neither of its flags.
#[cfg(not(feature = "watch"))]
const DOCS: &str = docs_guide!("", "--cache, --analysis-workers");

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

/// The default size metric, as the defaults table spells it.
///
/// Read from the model rather than written out here, so `--help` cannot say one metric
/// while the engine answers in another. The flag still has a clap default because the help
/// text states it; what it must not have is a default of its own.
const SIZE_DEFAULT: &str = Request::DEFAULTS.size.label();

/// The default analyzer set, as the grammar spells it.
///
/// Read from the model the same way, and for the same reason: the flag keeps a clap
/// default because `--help` prints it, and what it must not have is a default of its own.
/// The empty set is the one analyzer set a `const` can spell, so the assertion is what
/// makes this a reading of the table rather than a guess about it.
const ANALYZE_DEFAULT: &str = AnalysisSet::NONE_LABEL;
const _: () = assert!(
    !Request::DEFAULTS.content.is_enabled(),
    "--help prints the default analyzer set; a table that enables one needs a spelling here"
);

/// The request model's refusal, in the command line's words.
fn refused(error: &RequestError) -> anyhow::Error {
    anyhow::anyhow!(error.message(&AxisNames::FLAGS))
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

    /// Bytes of .gitignore rules to retain before refusing more files [default: 4MiB]. Accepts `all`, which also reads each .gitignore whole.
    ///
    /// `SIZE` rather than `SIZE|all` as the value name, as `--depth` and `--limit` do: the
    /// longer name pushes this heading's help onto separate lines.
    #[arg(long, value_name = "SIZE", help_heading = "SCOPE")]
    pub gitignore_budget: Option<String>,

    /// Longest .gitignore line to apply before refusing its file [default: 16KiB]. Accepts `all`.
    #[arg(long, value_name = "SIZE", help_heading = "SCOPE")]
    pub gitignore_line_limit: Option<String>,

    /// Read no .gitignore files: rows lose their ignored share, and the snapshot scope differs
    #[arg(long, action = ArgAction::SetTrue, help_heading = "SCOPE")]
    pub no_gitignore: bool,

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

    /// Report only entries no .gitignore rule ignores; sizes and ordering follow.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "SELECTION")]
    pub exclude_ignored: bool,

    /// Report only entries a .gitignore rule ignores.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "SELECTION")]
    pub only_ignored: bool,

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
    #[arg(long, value_name = "METRIC", default_value = SIZE_DEFAULT, help_heading = "SELECTION")]
    pub size: String,

    // ---- view: which roll-ups are reported ----
    /// Views: list, extensions, types, families, languages, documents, largest, recent,
    /// summary, or full; tree/files are compatibility presets. Defaults to list with no analysis, otherwise to a view
    /// that displays the requested analysis.
    #[arg(long, value_name = "LIST", help_heading = "VIEWS")]
    pub view: Option<String>,

    /// Analyzers to run: none, lines, code, words, or all.
    ///
    /// Anything but none reads each eligible file missing from a compatible content cache.
    #[arg(
        long,
        value_name = "LIST",
        default_value = ANALYZE_DEFAULT,
        help_heading = "CONTENT ANALYSIS"
    )]
    pub analyze: String,

    /// Content reader workers; zero selects available parallelism.
    #[arg(long, value_name = "N", default_value_t = 0, help_heading = "CONTENT ANALYSIS")]
    pub analysis_workers: usize,

    /// Logical words per derived document page.
    #[arg(
        long,
        value_name = "N",
        default_value_t = fdu_core::query::Request::DEFAULTS.words_per_page,
        help_heading = "VIEWS"
    )]
    pub words_per_page: u64,

    // ---- format: how the report is serialized ----
    /// Format: tree (list default), paths, long (size/age/path), json, jsonl, yaml, or automatic text.
    #[arg(long, value_name = "FORMAT", default_value = "text", help_heading = "OUTPUT")]
    pub format: String,

    /// Display the list as the default directory tree.
    #[arg(long, conflicts_with_all = ["format", "long"], help_heading = "OUTPUT")]
    pub tree: bool,

    /// Display flat matching paths with size and modification age.
    #[arg(long, conflicts_with_all = ["format", "tree"], help_heading = "OUTPUT")]
    pub long: bool,

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
    /// Cache policy: auto, refresh, read-only, only (unverified), or off.
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
    #[arg(long, value_name = "DUR", default_value_t = format!("{}s", WatchDelivery::DEFAULT_INTERVAL.as_secs()), help_heading = "EXECUTION")]
    pub interval: String,

    /// Print help.
    #[arg(short = 'h', long = "help", action = ArgAction::HelpShort, help_heading = "OTHER")]
    pub help: Option<bool>,

    /// Print version.
    #[arg(short = 'V', long = "version", action = ArgAction::Version, help_heading = "OTHER")]
    pub version: Option<bool>,

    /// Print common commands, cache behavior, and the complete usage guide.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "OTHER")]
    pub docs: bool,

    /// Print a portable agent skill to stdout.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "OTHER")]
    pub skill: bool,
}

/// Parse the scope a lifecycle flag applies to.
fn parse_cache_scope(value: &str, flag: &str) -> anyhow::Result<CacheScope> {
    CacheScope::parse(value).ok_or_else(|| {
        anyhow::anyhow!(
            "invalid {flag} {:?}: expected root or all",
            value.trim().to_ascii_lowercase()
        )
    })
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
        let path = self.path.as_deref().ok_or_else(|| {
            usage(&anyhow::anyhow!(
                "missing PATH: specify the directory to summarize, for example `fdu .`"
            ))
        })?;
        // One grammar, one defaults table, one set of rules: this command line hands the
        // model the words its caller typed and renders whatever comes back in flag names.
        // It used to parse each axis itself, which is how a default could differ between
        // the doors into the same engine.
        let request = self.request(path, SystemTime::now())?;
        let delivery = Delivery {
            cache: self.parse_cache_policy().map_err(|error| usage(&error))?,
            cache_path: default_cache_path(path),
            workers: fdu_core::query::Workers {
                analysis: self.analysis_workers,
                ..Default::default()
            },
            batch_size: fdu_core::ScanConfig::default().batch_size,
            order: fdu_core::ScanOrder::default(),
            watch: self.watch_delivery().map_err(|error| usage(&error))?,
            // What `--allow-partial` says: a partial answer is a success. Only this
            // command's exit mapping reads it today, and the execution plan model will.
            accept_partial: self.allow_partial,
        };
        // What a watch cannot carry -- a narrowed scan scope, content analysis nothing
        // re-reads, a snapshot nothing verified -- is the model's rule now, so a library
        // caller and a Python caller meet the same wall this command line has always been.
        request.validate_delivery(&delivery).map_err(|error| usage(&refused(&error)))?;

        #[cfg(feature = "watch")]
        if self.watch {
            let color = ColorContext::from_environment(
                self.color,
                self.machine_format(),
                self.skill,
                stdout_is_terminal,
            )
            .enabled();
            return Self::run_watch(out, diagnostic, format, &request, &delivery, color);
        }

        let report_started = Instant::now();
        let collect_scan_diagnostics =
            std::env::var_os(SCAN_DIAGNOSTICS_ENV).is_some_and(|value| value == OsStr::new("1"));
        let (report, pending_save, performance, scan_diagnostics) = if collect_scan_diagnostics {
            prepare_report_with_scan_diagnostics(&request, &delivery)?
        } else {
            let (report, pending_save, performance) = prepare_report(&request, &delivery)?;
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
        let rendered_text = matches!(format, report_format::Format::Text | report_format::Format::Tree)
            .then(|| report_format::render(&report, format, color));
        let (rendered_text, render_result): (Option<String>, anyhow::Result<()>) =
            match rendered_text {
                Some(Ok(rendered)) => {
                    let result = write!(out, "{rendered}").and_then(|()| out.flush()).map_err(Into::into);
                    (Some(rendered), result)
                }
                Some(Err(error)) => (None, Err(error.into())),
                None => (
                    None,
                    report_format::write(&report, format, color, out)
                        .and_then(|()| out.flush())
                        .map_err(Into::into),
                ),
            };

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

        if matches!(format, report_format::Format::Text | report_format::Format::Tree) {
            let rendered = rendered_text.as_deref().unwrap_or_default();
            if !rendered.is_empty() && !rendered.ends_with('\n') {
                writeln!(out)?;
            }
            writeln!(
                out,
                "{}",
                paint(
                    &performance_footer(
                        performance,
                        &report.ignore_rules,
                        report_started.elapsed()
                    ),
                    STYLE_PERFORMANCE,
                    color,
                )
            )?;
            for note in
                display_notes(&request.query.views, request.basis.content, performance.bytes_read)
            {
                writeln!(out, "{}", paint(&note, STYLE_PERFORMANCE, color))?;
            }
        }

        if matches!(format, report_format::Format::Paths | report_format::Format::Long) {
            for note in report_format::flat_diagnostics(&report) {
                writeln!(diagnostic, "{note}")?;
            }
        }
        if matches!(format, report_format::Format::Text | report_format::Format::Tree)
            && !report.status.complete
        {
            let color =
                ColorContext::from_environment(self.color, false, false, stderr_is_terminal)
                    .enabled();
            for error in &report.status.errors {
                let _ = writeln!(
                    diagnostic,
                    "{}",
                    paint(&format!("warning: {}", error.message), STYLE_WARNING, color)
                );
            }
        }

        let plan = fdu_core::plan(&request, &delivery, fdu_core::Route::OneShot)?;
        Ok(match plan.outcome(&report.status) {
            fdu_core::OutcomeClass::Success => RunOutcome::Complete,
            fdu_core::OutcomeClass::Partial => RunOutcome::Partial,
        })
    }

    /// Whether the requested format is a machine format, which is never colorized.
    fn machine_format(&self) -> bool {
        self.parse_format().is_ok_and(report_format::Format::is_machine)
    }

    /// Run the query continuously, streaming changes as they arrive.
    ///
    /// The initial report is exactly what a one-shot run would print, and every later
    /// render is the same query re-evaluated. Detection is event-driven throughout: an
    /// idle tree costs no filesystem work, and `--interval` throttles only how often
    /// aggregate views repaint.
    #[cfg(feature = "watch")]
    fn run_watch(
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        format: report_format::Format,
        request: &Request,
        delivery: &Delivery,
        color: bool,
    ) -> anyhow::Result<RunOutcome> {
        use fdu_core::query::ViewSpec;
        use fdu_core::watch_session::{ChangeKind, Session};

        // The repaint interval the delivery already carries, rather than a second reading
        // of `--interval`: `run` refused an unparseable one before it opened anything, and
        // a value parsed twice is a value that can mean two things.
        let interval = delivery
            .watch
            .expect("run() builds a watch delivery before it takes the watch path")
            .interval;

        let mut session = Session::start(request.clone(), delivery.clone())?;
        Self::persist_live(&mut session, diagnostic, color);

        // A streaming run keeps only the views it can render incrementally plus the
        // aggregates it repaints; both come from the same query, so nothing here is a
        // second grammar.
        let streams_changes = request.query.views.contains(&ViewSpec::Files)
            && !matches!(
                format,
                report_format::Format::Tree
                    | report_format::Format::Paths
                    | report_format::Format::Long
            );
        let has_aggregates =
            !streams_changes || request.query.views.iter().any(|view| *view != ViewSpec::Files);

        // The initial answer, identical to a one-shot run's.
        if format == report_format::Format::Yaml {
            write!(out, "{}", report_format::document_start(format))?;
        }
        let initial = session.report(SystemTime::now())?;
        report_format::write(&initial, format, color, out)?;
        if matches!(format, report_format::Format::Paths | report_format::Format::Long) {
            for note in report_format::flat_diagnostics(&initial) {
                writeln!(diagnostic, "{note}")?;
            }
        }
        out.flush()?;

        let mut dirty_since_render = false;
        let mut last_render = SystemTime::now();
        loop {
            let Some(batch) = session.next_batch(interval)? else {
                // Nothing arrived in the window. Repaint only if something is pending,
                // so a quiet tree produces no output and no work at all.
                if has_aggregates && dirty_since_render {
                    Self::render_live(out, diagnostic, &session, format, color)?;
                    dirty_since_render = false;
                    last_render = SystemTime::now();
                }
                // The idle branch is where a throttled save has to land. A change that
                // arrived too soon after the last save would otherwise wait for the next
                // change to persist it, and the next change may never come: a burst
                // followed by silence is the single most likely way a watch session ends.
                Self::persist_live(&mut session, diagnostic, color);
                continue;
            };

            Self::render_watch_changes(out, diagnostic, &batch.changes, format, streams_changes)?;
            out.flush()?;

            dirty_since_render |= batch.dirty;
            let elapsed = last_render.elapsed().unwrap_or_default();
            if has_aggregates && dirty_since_render && elapsed >= interval {
                Self::render_live(out, diagnostic, &session, format, color)?;
                dirty_since_render = false;
                last_render = SystemTime::now();
            }

            // Persist as we go rather than only at exit. A watch session ends by signal
            // far more often than it ends politely, and std offers no portable signal
            // handler, so an exit-time save would be the one that never runs. Throttled
            // to the render interval so a churny tree does not rewrite constantly; the
            // pending flag is what guarantees a throttled change still reaches disk once
            // the tree goes quiet.
            Self::persist_live(&mut session, diagnostic, color);
        }
    }

    /// Surface a persistence failure without interrupting the live answer.
    #[cfg(feature = "watch")]
    fn persist_live(
        session: &mut fdu_core::watch_session::Session,
        diagnostic: &mut dyn Write,
        color: bool,
    ) {
        if let fdu_core::watch_session::SaveOutcome::Failed(error) =
            session.persist_due(Instant::now())
        {
            let _ = writeln!(
                diagnostic,
                "{}",
                paint(&format!("warning: {error}"), STYLE_WARNING, color)
            );
        }
    }

    /// Preserve invalidation notices without putting diagnostic text in flat rows.
    #[cfg(feature = "watch")]
    fn render_watch_changes(
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        changes: &[fdu_core::watch_session::Change],
        format: report_format::Format,
        streams_changes: bool,
    ) -> std::io::Result<()> {
        for change in changes {
            if change.kind == fdu_core::watch_session::ChangeKind::Invalidate
                && matches!(format, report_format::Format::Paths | report_format::Format::Long)
            {
                writeln!(diagnostic, "{}", report_format::render_change(change, format))?;
            } else if streams_changes
                || change.kind == fdu_core::watch_session::ChangeKind::Invalidate
            {
                writeln!(out, "{}", report_format::render_change(change, format))?;
            }
        }
        Ok(())
    }

    #[cfg(feature = "watch")]
    /// Repaint the aggregate views after a change.
    ///
    /// Only ever called for a repaint — the first answer is written by the caller before
    /// the loop — so the rule below can be unconditional.
    fn render_live(
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        session: &fdu_core::watch_session::Session,
        format: report_format::Format,
        color: bool,
    ) -> anyhow::Result<()> {
        let generated_at = SystemTime::now();
        let report = session.report(generated_at)?;
        // A watch run has no final answer and so no performance footer, which left text
        // repaints with nothing between them: the last row of one and the first row of
        // the next were adjacent lines. A blank line alone would not do, because that is
        // already what separates two views inside a single report.
        if matches!(format, report_format::Format::Text | report_format::Format::Tree) {
            writeln!(
                out,
                "\n{}",
                paint(&report_format::watch_rule(generated_at), STYLE_WATCH_RULE, color)
            )?;
        } else if format == report_format::Format::Yaml {
            write!(out, "{}", report_format::document_start(format))?;
        }
        report_format::write(&report, format, color, out)?;
        if matches!(format, report_format::Format::Paths | report_format::Format::Long) {
            for note in report_format::flat_diagnostics(&report) {
                writeln!(diagnostic, "{note}")?;
            }
        }
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
            let scope = parse_cache_scope(scope, "--cache-clear").map_err(|e| usage(&e))?;
            match (scope, &cache_dir) {
                (CacheScope::All, Some(dir)) => {
                    // Echo the directory before acting, so a destructive flag always says
                    // where it is pointed.
                    writeln!(out, "Cache directory: {}", dir.display())?;
                    let removed = fdu_core::clear_all_caches(dir)?;
                    if removed.is_empty() {
                        writeln!(out, "Cache already empty.")?;
                    }
                    if removed.snapshots > 0 {
                        writeln!(
                            out,
                            "Cache cleared: {} {}.",
                            removed.snapshots,
                            plural(removed.snapshots, "snapshot", "snapshots")
                        )?;
                    }
                    // Said separately because it is a different fact: these are fdu's own
                    // files, and none of them was a snapshot anyone could have used.
                    if removed.leftovers > 0 {
                        writeln!(
                            out,
                            "Also reclaimed: {} {} fdu left behind.",
                            removed.leftovers,
                            plural(removed.leftovers, "file", "files")
                        )?;
                    }
                    // Clearing never removes what it cannot identify, so it says what it
                    // left rather than letting "cleared" imply an empty directory.
                    let remaining = fdu_core::list_caches(dir)?;
                    let left = remaining
                        .iter()
                        .filter(|status| status.state == CacheState::Unrecognized)
                        .count();
                    if left > 0 {
                        writeln!(
                            out,
                            "Left in place: {left} {}; fdu --cache-status=all lists {}.",
                            plural(
                                left,
                                "file that is not an fdu snapshot",
                                "files that are not fdu snapshots"
                            ),
                            plural(left, "it", "them")
                        )?;
                    }
                    // A staging file young enough to belong to a running writer is the one
                    // leftover a clear leaves, and saying so beats a silent survival.
                    let staging = remaining
                        .iter()
                        .filter(|status| matches!(status.state, CacheState::Leftover(_)))
                        .count();
                    if staging > 0 {
                        writeln!(
                            out,
                            "Left in place: {staging} staging {} another fdu may still be \
                             writing.",
                            plural(staging, "file", "files")
                        )?;
                    }
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
                    let left = match &path {
                        Some(path) => fdu_core::cache_status(path)?.state,
                        None => CacheState::Absent,
                    };
                    if left == CacheState::Unrecognized {
                        writeln!(out, "Left in place: the file is not an fdu snapshot.")?;
                    }
                }
                (CacheScope::All, None) => writeln!(out, "Cache already empty.")?,
            }
        }

        if let Some(scope) = &self.cache_status {
            let scope = parse_cache_scope(scope, "--cache-status").map_err(|e| usage(&e))?;
            let statuses = match (scope, &cache_dir) {
                (CacheScope::All, Some(dir)) => fdu_core::list_caches(dir)?,
                (CacheScope::All, None) => Vec::new(),
                (CacheScope::Root, _) => match fdu_core::default_cache_path(root) {
                    Some(path) => vec![fdu_core::cache_status(&path)?],
                    None => Vec::new(),
                },
            };
            self.write_cache_status(out, &statuses, scope)?;
        }

        Ok(RunOutcome::Complete)
    }

    /// Render cache status through the format axis, like any other output.
    fn write_cache_status(
        &self,
        out: &mut dyn Write,
        statuses: &[fdu_core::CacheStatus],
        scope: CacheScope,
    ) -> anyhow::Result<()> {
        let format = self.parse_format().map_err(|e| usage(&e))?;
        // Every format, human included, comes from the one renderer. While the CLI kept
        // the text layout to itself, no other caller could print what fdu prints.
        writeln!(out, "{}", report_format::render_cache_status(statuses, scope, format))?;
        Ok(())
    }

    /// The request this invocation asks for, built and validated by the one model.
    ///
    /// Every axis reaches the model as the caller wrote it, so the grammars, the defaults,
    /// and the rules that relate one axis to another are stated once for every surface, and
    /// the refusal that comes back is rendered here in flag names.
    fn request(&self, root: &Path, now: SystemTime) -> anyhow::Result<Request> {
        let typed = self.typed_values();
        let request = Request::build(&self.spec(root, &typed)?, now, &AxisNames::FLAGS)
            .map_err(|error| usage(&refused(&error)))?;
        // A view nothing analyzed cannot answer, and a selection by ignored state over a
        // scan that reads no rule, are both refused before anything is scanned.
        request.validate().map_err(|error| usage(&refused(&error)))?;
        Ok(request)
    }

    /// The flags clap already typed, rendered back into the words the model reads.
    ///
    /// A value the model parses is a value one grammar owns; handing it clap's `usize`
    /// instead would be a second grammar that agrees today. Through `Display` a
    /// disagreement shows up as a golden difference rather than as a silent one.
    fn typed_values(&self) -> TypedValues {
        TypedValues {
            scan_depth: self.scan_depth.map(|depth| depth.to_string()),
            words_per_page: self.words_per_page.to_string(),
        }
    }

    /// This invocation as a surface-neutral request spec.
    fn spec<'a>(
        &'a self,
        root: &'a Path,
        typed: &'a TypedValues,
    ) -> anyhow::Result<RequestSpec<'a>> {
        Ok(RequestSpec {
            root,
            scan_depth: typed.scan_depth.as_deref(),
            one_filesystem: self.one_filesystem,
            // The flag says what it turns off, so only its presence is an instruction; the
            // default belongs to the model.
            read_controls: self.no_gitignore.then_some(false),
            control_budget: self.gitignore_budget.as_deref(),
            control_line_limit: self.gitignore_line_limit.as_deref(),
            analyze: Some(&self.analyze),
            read: ReadSpec {
                views: self.view.as_deref(),
                format: Some(self.parse_format()?.label()),
                words_per_page: Some(&typed.words_per_page),
                include: &self.include,
                exclude: &self.exclude,
                min_size: self.min_size.as_deref(),
                modified_since: self.modified_since.as_deref(),
                modified_before: self.modified_before.as_deref(),
                kinds: self.kind.as_deref(),
                ignored: self.ignored_selection()?,
                depth: self.depth.as_deref(),
                limit: self.limit.as_deref(),
                sort: self.sort.as_deref(),
                reverse: self.reverse,
                size: Some(&self.size),
            },
        })
    }

    /// Whether this run is a watch, and how often it repaints.
    ///
    /// The interval is parsed here because a bad one is a usage error like any other, and
    /// because the delivery a request is validated against has to say what it is before
    /// anything is opened.
    #[cfg(feature = "watch")]
    fn watch_delivery(&self) -> anyhow::Result<Option<WatchDelivery>> {
        if !self.watch {
            return Ok(None);
        }
        Ok(Some(WatchDelivery { interval: parse_duration(&self.interval)? }))
    }

    /// A command line built without the watch feature delivers no watch.
    #[cfg(not(feature = "watch"))]
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    fn watch_delivery(&self) -> anyhow::Result<Option<WatchDelivery>> {
        Ok(None)
    }

    /// The ignored-state axis, which this surface spells as two switches.
    ///
    /// Naming both of them is a conflict between flags rather than an invalid value, so it
    /// is refused here; every other axis is one flag and one value the model reads.
    fn ignored_selection(&self) -> anyhow::Result<Option<&'static str>> {
        match (self.exclude_ignored, self.only_ignored) {
            (false, false) => Ok(None),
            (true, false) => Ok(Some(IgnoredEntries::Exclude.label())),
            (false, true) => Ok(Some(IgnoredEntries::Only.label())),
            (true, true) => Err(usage(&anyhow::anyhow!(
                "--exclude-ignored and --only-ignored select opposite entries; use one of them"
            ))),
        }
    }

    /// Translate the cache-policy flag.
    fn parse_cache_policy(&self) -> anyhow::Result<CachePolicy> {
        parse_cache_policy(&self.cache, AxisNames::FLAGS.cache).map_err(|error| refused(&error))
    }

    /// Translate the format flag, naming every accepted value on a miss.
    fn parse_format(&self) -> anyhow::Result<report_format::Format> {
        if self.tree {
            return Ok(report_format::Format::Tree);
        }
        if self.long {
            return Ok(report_format::Format::Long);
        }
        report_format::Format::parse(&self.format).ok_or_else(|| {
            anyhow::anyhow!(
                "invalid --format {:?}: expected one of {}",
                self.format,
                report_format::Format::ALL.join(", ")
            )
        })
    }

    /// The query this invocation asks for, as the model builds it.
    ///
    /// Tests go through this rather than assembling a `Query` of their own, so a change to
    /// the grammars or the defaults cannot pass the suite while changing what the command
    /// does.
    #[cfg(test)]
    fn resolved_query(&self) -> anyhow::Result<fdu_core::query::Query> {
        Ok(self.resolved_request()?.query)
    }

    /// [`Cli::request`] for a test, over a root no test reads.
    #[cfg(test)]
    fn resolved_request(&self) -> anyhow::Result<Request> {
        self.request(self.path.as_deref().unwrap_or(Path::new(".")), SystemTime::now())
    }
}

/// Flags clap typed, rendered back into the words [`Request::build`] reads.
struct TypedValues {
    scan_depth: Option<String>,
    words_per_page: String,
}

/// Format transient one-shot work without adding it to the machine-report schema.
///
/// The ignore-rule count sits beside the walk it was read during. It is also what tells a
/// reader apart two reports that show no ignored share: one whose rules ignore nothing,
/// and one that read no rules.
fn performance_footer(
    performance: PerformanceSummary,
    ignore_rules: &ControlCoverage,
    total: Duration,
) -> String {
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
    let rules = match ignore_rules {
        ControlCoverage::NotObserved => "no ignore rules".to_string(),
        ControlCoverage::Observed(observed) => {
            let refused = if observed.refused > 0 {
                format!(", {} refused", human_count(observed.refused))
            } else {
                String::new()
            };
            format!(
                "ignore rules {} {}{refused}",
                human_count(observed.applied),
                plural_u64(observed.applied, "file", "files")
            )
        }
    };
    format!(
        "Performance: walked {} {} / {}; {rules}; content read {}{}; analysis {fresh}, {cached}; {}; total {}",
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

/// Whether a view renders anything the content analyzers produce.
///
/// This is the check behind the "paid for nothing" note.  It is deliberately a match
/// rather than a property of `ViewSpec`, so adding a view forces a decision here about
/// whether it displays content metrics.
const fn view_displays_analysis(view: ViewSpec) -> bool {
    matches!(view, ViewSpec::Types | ViewSpec::Families | ViewSpec::Languages | ViewSpec::Documents)
}

/// Views to render, plus any `--view full` could not satisfy.
#[cfg(test)]
#[derive(Debug)]
struct ResolvedViews {
    selected: Vec<ViewSpec>,
    omitted: Vec<ViewSpec>,
}

/// Resolve the view axis against the content axis.
///
/// `full` expands to what the requested analyzers can answer rather than failing the
/// whole run over one unsatisfiable view, and reports what it dropped so the omission is
/// stated rather than hidden. The command builds its views through the request model; this
/// is what the tests of that axis call, one layer below it.
#[cfg(test)]
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
fn display_notes(views: &[ViewSpec], profile: AnalysisSet, bytes_read: u64) -> Vec<String> {
    // Only the note that needs telemetry. The omission note is a fact about the report and
    // travels on it, so every surface states it rather than just this one (fdu-x8u6).
    //
    // Never an error: warming the content sidecar so a later run is warm is a supported
    // use, and `--cache`-aware callers depend on it.  Silence would hide the cost instead.
    if profile.is_enabled() && !views.iter().any(|view| view_displays_analysis(*view)) {
        return vec![format!(
            "note: --analyze {} read {}; no selected view displays content metrics — try --view families, languages, or full",
            profile.labels().join(","),
            report_format::human_bytes(bytes_read),
        )];
    }
    Vec::new()
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
    finish(result, diagnostic, diagnostic_color)
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

fn finish(result: anyhow::Result<RunOutcome>, diagnostic: &mut dyn Write, color: bool) -> u8 {
    match result {
        Ok(RunOutcome::Complete) => 0,
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
    use fdu_core::EntryKind;
    use fdu_core::query::{Bound, ScopeAxis, SizeMetric, SortKey};
    #[cfg(feature = "watch")]
    use std::time::UNIX_EPOCH;

    /// The two flags select opposite partitions, so asking for both is a usage error.
    #[test]
    fn the_ignored_selection_flags_pick_one_side_and_refuse_both() {
        assert_eq!(
            cli().resolved_query().expect("parses").selection.ignored,
            IgnoredEntries::Include
        );
        let exclude = Cli { exclude_ignored: true, ..cli() }.resolved_query().expect("parses");
        assert_eq!(exclude.selection.ignored, IgnoredEntries::Exclude);
        let only = Cli { only_ignored: true, ..cli() }.resolved_query().expect("parses");
        assert_eq!(only.selection.ignored, IgnoredEntries::Only);
        assert_eq!(
            query_error(&Cli { exclude_ignored: true, only_ignored: true, ..cli() }),
            "--exclude-ignored and --only-ignored select opposite entries; use one of them"
        );
    }

    /// A selection by ignored state needs the rules `--no-gitignore` turns off, so the pair
    /// is a usage error naming both flags, raised before anything is scanned.
    #[test]
    fn no_gitignore_refuses_a_selection_by_ignored_state_before_scanning() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let args = [
            "fdu",
            "--no-gitignore",
            "--only-ignored",
            "/nonexistent-root-that-must-not-be-scanned",
        ]
        .map(OsString::from);
        let status = run_with_io(&args, &mut out, &mut err, false, false);
        assert_eq!(status, 2);
        assert!(out.is_empty());
        assert_eq!(
            String::from_utf8(err).expect("UTF-8 diagnostics"),
            "fdu: --only-ignored needs .gitignore classification, and --no-gitignore turned it \
             off; drop one of them\n"
        );
    }

    #[test]
    fn the_performance_line_counts_the_ignore_rules_it_read_or_says_it_read_none() {
        use fdu_core::control::{ControlObservation, ControlRefusalReason, RefusedControl};

        let performance = PerformanceSummary {
            walked_files: 7,
            walked_bytes: 269,
            ..PerformanceSummary::default()
        };
        let footer = |rules: &ControlCoverage| {
            performance_footer(performance, rules, Duration::from_millis(3))
        };
        assert_eq!(
            footer(&ControlCoverage::NotObserved),
            "Performance: walked 7 files / 269 B; no ignore rules; content read 0 B; analysis 0 fresh, 0 cached; cold scan; total 3.0 ms"
        );
        let observed = |applied, refusals: Vec<RefusedControl>| {
            ControlCoverage::Observed(ControlObservation {
                limits: fdu_core::ControlLimits::default(),
                applied,
                refused: u64::try_from(refusals.len()).expect("a handful"),
                refusals,
            })
        };
        assert!(footer(&observed(1, Vec::new())).contains("; ignore rules 1 file; "));
        let refused = RefusedControl {
            path: PathBuf::from(".gitignore"),
            reason: ControlRefusalReason::LineLimit,
        };
        assert!(
            footer(&observed(0, vec![refused])).contains("; ignore rules 0 files, 1 refused; ")
        );
    }

    /// Each `.gitignore` limit flag sets only its own limit, names itself when rejected, and
    /// reaches the cache scope, which every run observes unless `--no-gitignore` says not to;
    /// where nothing is observed, neither flag splits the snapshot scope.
    #[test]
    fn each_gitignore_limit_flag_sets_only_its_own_limit_and_joins_the_observed_scope() {
        let scan_config = |flags: &[&str]| {
            let args = std::iter::once("fdu").chain(flags.iter().copied()).chain(["."]);
            Cli::try_parse_from(args)
                .expect("parses")
                .resolved_request()
                .map(|request| request.basis.scope)
        };
        let defaults = fdu_core::ControlLimits::default();
        let unobserved = |config: &fdu_core::query::Scope| {
            fdu_core::query::Scope { read_controls: false, ..config.clone() }.scope()
        };
        let default = scan_config(&[]).expect("defaults");
        assert_eq!(default.control_limits, defaults);
        assert!(default.read_controls, "every run observes .gitignore unless told not to");

        for (flags, limits) in [
            (
                &["--gitignore-budget", "16MiB"][..],
                fdu_core::ControlLimits { budget: Some(16 * 1024 * 1024), ..defaults },
            ),
            (
                &["--gitignore-budget", "all"][..],
                fdu_core::ControlLimits { budget: None, ..defaults },
            ),
            (
                &["--gitignore-line-limit", "64KiB"][..],
                fdu_core::ControlLimits { line_limit: Some(64 * 1024), ..defaults },
            ),
            (
                &["--gitignore-line-limit", "all"][..],
                fdu_core::ControlLimits { line_limit: None, ..defaults },
            ),
        ] {
            let config = scan_config(flags).expect("a valid limit");
            assert_eq!(config.control_limits, limits, "{flags:?}");
            assert_ne!(config.scope(), default.scope(), "{flags:?} while observed");
            assert_eq!(unobserved(&config), unobserved(&default), "{flags:?} once unobserved");
        }

        for flag in ["--gitignore-budget", "--gitignore-line-limit"] {
            assert_eq!(
                scan_config(&[flag, "lots"]).expect_err("not a size").to_string(),
                format!(
                    "invalid {flag} \"lots\": expected a number before the unit, as in `10M`, \
                     or `all` for no bound"
                )
            );
        }
    }

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

    #[cfg(feature = "watch")]
    #[test]
    fn flat_watch_invalidations_are_visible_without_becoming_path_rows() {
        use fdu_core::watch_session::{Change, ChangeKind};
        let changes = [Change {
            path: "builds".into(),
            kind: ChangeKind::Invalidate,
            clock: 1,
            entry_kind: None,
            bytes: None,
            allocated: None,
            mtime_ns: None,
            ignored: None,
        }];
        for format in [report_format::Format::Paths, report_format::Format::Long] {
            let (mut out, mut diagnostic) = (Vec::new(), Vec::new());
            Cli::render_watch_changes(&mut out, &mut diagnostic, &changes, format, false)
                .expect("invalidation");
            assert!(out.is_empty(), "stdout is reserved for flat data rows");
            assert_eq!(
                String::from_utf8(diagnostic).expect("diagnostic"),
                format!("{}\n", report_format::render_change(&changes[0], format))
            );
        }
    }

    #[cfg(feature = "watch")]
    #[test]
    fn an_interval_parses_without_overflowing_any_platforms_clock() {
        assert_eq!(parse_duration("2s").expect("seconds"), Duration::from_secs(2));
        assert_eq!(parse_duration("1h30m").expect("compound"), Duration::from_secs(5_400));
        assert_eq!(parse_duration("1w").expect("weeks"), Duration::from_secs(604_800));
        assert_eq!(parse_duration("200ms").expect("milliseconds"), Duration::from_millis(200));
        assert!(parse_duration("0.2s").is_err(), "a fractional age stays rejected");
        assert!(parse_duration("banana").is_err(), "a non-duration must be rejected, not parsed");
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
            gitignore_budget: None,
            gitignore_line_limit: None,
            no_gitignore: false,
            include: Vec::new(),
            exclude: Vec::new(),
            min_size: None,
            modified_since: None,
            modified_before: None,
            kind: None,
            exclude_ignored: false,
            only_ignored: false,
            // None, as clap now leaves it: the default belongs to the view.
            depth: None,
            limit: None,
            sort: None,
            reverse: false,
            size: SIZE_DEFAULT.to_string(),
            view: Some("tree".to_string()),
            analyze: "none".to_string(),
            analysis_workers: 0,
            words_per_page: Request::DEFAULTS.words_per_page,
            format: "text".to_string(),
            tree: false,
            long: false,
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

    /// A scope this build cannot honour is a usage error here, not a failed operation.
    ///
    /// The kind, not only the sentence: the same request raises `ValueError` in Python, so
    /// reporting it as an engine error exited 1 where the other surface refused -- one
    /// request with two kinds of outcome, which the path-independence matrix reported as
    /// 35 cross-surface differences for `--one-filesystem` on Windows.
    ///
    /// Driven through the refusal rather than through argv because the axes this build
    /// cannot honour are unreachable from a flag: `--one-filesystem` is honoured wherever
    /// the goldens run, and `follow_symlinks` is an `open` and library axis with no flag at
    /// all. What the command line owns is this mapping, and this is it.
    #[test]
    fn a_scope_this_build_cannot_honour_exits_as_a_usage_error() {
        for (axis, printed) in [
            (ScopeAxis::OneFilesystem, "--one-filesystem requires platform device identity"),
            (
                ScopeAxis::FollowSymlinks,
                "follow_symlinks requires cycle, root-boundary, and filesystem-boundary semantics",
            ),
        ] {
            let refusal = RequestError::ScopeUnsupported { axis, reason: axis.reason() };
            let error = usage(&refused(&refusal));
            assert!(is_usage_error(&error), "{axis:?} must exit like the bad argument it is");

            let mut diagnostic = Vec::new();
            assert_eq!(finish(Err(error), &mut diagnostic, false), 2);
            assert_eq!(
                String::from_utf8(diagnostic).expect("diagnostics are UTF-8"),
                format!("fdu: unsupported scan configuration: {printed}\n")
            );
        }
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
    fn the_undocumented_readonly_cache_alias_is_rejected() {
        assert_eq!(
            Cli { cache: "read-only".to_string(), ..cli() }
                .parse_cache_policy()
                .expect("the canonical policy name parses"),
            CachePolicy::ReadOnly
        );
        let error = Cli { cache: "readonly".to_string(), ..cli() }
            .parse_cache_policy()
            .expect_err("an unreleased alias must not become a contract");
        assert_eq!(
            error.to_string(),
            "invalid --cache \"readonly\": expected one of auto, refresh, read-only, only, off"
        );
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
    ///
    /// The View row is compared whole rather than word by word. A membership check kept
    /// passing after the view total was renamed `full` and two presets were added,
    /// because every old name still appeared somewhere in the guide, while the row went
    /// on offering `all` to a parser that rejects it.
    #[test]
    fn the_guide_only_names_views_and_analyzers_that_parse() {
        let row = DOCS
            .split("\n  View ")
            .nth(1)
            .and_then(|rest| rest.split("\n  Format ").next())
            .expect("the guide should have a View axis row");
        let listed: Vec<&str> =
            row.split([',', ' ', '\n']).filter(|name| !name.is_empty()).collect();
        let vocabulary = ViewSpec::vocabulary();
        let accepted: Vec<&str> = vocabulary.split(", ").collect();
        assert_eq!(listed, accepted, "the View row should list exactly what --view accepts");
        for set in ["none", "lines", "code", "words", "all"] {
            assert!(DOCS.contains(set), "the guide should list the {set} analyzer value");
            AnalysisSet::parse(set).unwrap_or_else(|_| panic!("{set} must parse"));
        }
    }

    /// Every command the guide shows must resolve as written.
    ///
    /// The guide is where a reader copies commands from, so a value the parser rejects
    /// is a broken example: `--view all` stayed on the ladder's last rung after the view
    /// total became `full`, and the binary printing it exited 2 on it.
    #[test]
    fn every_command_the_guide_shows_resolves() {
        let mut checked = 0;
        for line in DOCS.lines().filter(|line| line.starts_with(' ')) {
            let Some(example) = line.trim_start().strip_prefix("fdu ") else { continue };
            // A ladder row continues with its question after a column gap.
            let example = example.split("   ").next().unwrap_or(example);
            let args = std::iter::once("fdu").chain(example.split_whitespace());
            let parsed = Cli::try_parse_from(args)
                .unwrap_or_else(|error| panic!("the guide shows `fdu {example}`: {error}"));
            if let Err(error) = parsed.resolved_query() {
                panic!("the guide shows `fdu {example}`: {error}");
            }
            checked += 1;
        }
        assert!(checked > 0, "the guide should show commands");
    }

    /// Every view and analyzer value the skill shows must be one the parsers accept.
    ///
    /// An agent copies an inline `--view full` as readily as a fenced command, so both
    /// count. Only the guide was checked, and the skill kept `--view all` and an
    /// analyzer vocabulary the content axis no longer has.
    #[test]
    fn the_skill_only_names_views_and_analyzers_that_parse() {
        let skill = compose_skill();
        let spans = skill.split('`').filter(|span| span.starts_with("--"));
        let commands = skill.lines().map(str::trim_start).filter(|line| line.starts_with("fdu "));
        let mut checked = 0;
        for text in spans.chain(commands) {
            let mut words = text.split_whitespace();
            while let Some(flag) = words.next() {
                if flag != "--view" && flag != "--analyze" {
                    continue;
                }
                let Some(value) = words.next() else { continue };
                // A table cell lists alternatives, escaped for Markdown: `none\|lines`.
                for choice in value.split(['\\', '|']).filter(|choice| !choice.is_empty()) {
                    let parsed = if flag == "--view" {
                        ViewSpec::resolve(Some(choice), AnalysisSet::ALL, flag).map(drop)
                    } else {
                        AnalysisSet::parse(choice).map(drop)
                    };
                    assert!(parsed.is_ok(), "the skill shows `{flag} {choice}`: {parsed:?}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "the skill should show views and analyzers");
    }

    /// The stale-schema bug, made unrepeatable.
    ///
    /// The bump to `fdu.report/3` left "metric summaries use fdu.report/2" in the help
    /// text, contradicting what the binary emits. It survived a vocabulary sweep and a
    /// full `make check`, because no test compared prose against the constants.
    #[test]
    fn no_surface_names_a_schema_the_binary_does_not_emit() {
        // Every schema family the prose may name, so a new one is checked the day it is
        // mentioned rather than the day someone remembers this test.
        const PREFIX: &str = "fdu.";
        let live = [
            report_format::REPORT_SCHEMA,
            report_format::CONTENT_REPORT_SCHEMA,
            report_format::CACHE_SCHEMA,
            report_format::STREAM_SCHEMA,
        ];
        for (surface, text) in [("--docs", DOCS.to_string()), ("--skill", compose_skill())] {
            let mut rest = text.as_str();
            let mut found = 0;
            while let Some(at) = rest.find(PREFIX) {
                rest = &rest[at..];
                // A schema string is the prefix, a family name, a slash, and a version;
                // whatever punctuation follows belongs to the sentence, not to the schema.
                let tail = &rest[PREFIX.len()..];
                let family = tail.chars().take_while(char::is_ascii_alphabetic).count();
                if !tail[family..].starts_with('/') {
                    // Not a schema string at all: `fdu.` also begins ordinary prose.
                    rest = &rest[PREFIX.len()..];
                    continue;
                }
                let digits = tail[family + 1..].chars().take_while(char::is_ascii_digit).count();
                let named = &rest[..PREFIX.len() + family + 1 + digits];
                assert!(
                    live.contains(&named),
                    "{surface} names {named}, but the binary emits {live:?}"
                );
                found += 1;
                rest = &rest[named.len()..];
            }
            assert!(found > 0, "{surface} should state which schema it emits");
        }
    }

    /// The defect this axis exists to fix: a request that reads every eligible file must
    /// not print a report identical to one that read nothing.
    #[test]
    fn requesting_analysis_selects_a_view_that_displays_it() {
        let cases = [
            (AnalysisSet::NONE, ViewSpec::List),
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
        let notes = display_notes(&unspent.selected, AnalysisSet::ALL, 1_200);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("no selected view displays content metrics"), "{notes:?}");
        assert!(notes[0].contains("1.1 KiB"), "the note quantifies what was read: {notes:?}");
        // The note is advice, so every view it suggests must be one --view accepts.
        let (_, suggested) = notes[0].split_once("try --view ").expect("the note suggests views");
        for view in suggested.split([',', ' ']).filter(|word| !word.is_empty() && *word != "or") {
            let resolved = ViewSpec::resolve(Some(view), AnalysisSet::ALL, "--view");
            assert!(resolved.is_ok(), "the note suggests --view {view}: {resolved:?}");
        }

        // A view that does display the metrics earns no note at all.
        let spent = resolve_views(Some("families"), AnalysisSet::ALL).expect("resolve");
        assert!(display_notes(&spent.selected, AnalysisSet::ALL, 1_200).is_empty());

        // Neither does a metadata-only run, which bought nothing to display.
        let plain = resolve_views(None, AnalysisSet::NONE).expect("resolve");
        assert!(display_notes(&plain.selected, AnalysisSet::NONE, 0).is_empty());

        // And the omission note is no longer the CLI's to make, in either direction.
        let omitted = resolve_views(Some("full"), AnalysisSet::NONE).expect("resolve");
        assert!(!omitted.omitted.is_empty(), "full without analyzers must drop documents");
        assert!(
            display_notes(&omitted.selected, AnalysisSet::NONE, 0).is_empty(),
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
            // Built rather than validated, because a view that needs content is refused
            // rather than answered: what this test pins is that naming it never turns an
            // analyzer on behind the caller's back.
            let typed = cli.typed_values();
            let built = Request::build(
                &cli.spec(Path::new("."), &typed).expect("the spec composes"),
                SystemTime::now(),
                &AxisNames::FLAGS,
            )
            .expect("every view parses");
            assert_eq!(
                built.basis.content,
                AnalysisSet::NONE,
                "--view {spec} must leave the content axis empty"
            );
        }
    }

    #[test]
    fn analysis_profile_workers_and_page_denominator_parse_before_io() {
        let parsed = Cli {
            analyze: "lines".to_string(),
            analysis_workers: 3,
            words_per_page: Request::DEFAULTS.words_per_page,
            ..cli()
        };
        let request = parsed.resolved_request().expect("request");
        assert_eq!(request.basis.content, AnalysisSet::NONE.with_lines());
        // The one axis of a request that is delivery: it reaches the engine beside the
        // request rather than inside it.
        assert_eq!(parsed.analysis_workers, 3);
        assert_eq!(request.query.words_per_page, Request::DEFAULTS.words_per_page);

        let invalid = query_error(&Cli { analyze: "deep".to_string(), ..cli() });
        assert!(invalid.contains("none, lines, code, words, all"), "{invalid}");
        assert!(invalid.contains("--analyze"), "the message names the flag: {invalid}");
        assert!(query_error(&Cli { words_per_page: 0, ..cli() }).contains("positive"));
    }

    #[test]
    fn real_documents_report_exposes_requested_lines_words_pages_and_current_schema() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("notes.md"), b"one two\n\nthree\n").expect("write");
        let command = Cli {
            path: Some(root.path().to_path_buf()),
            analyze: "lines,words".to_string(),
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
        assert!(output.contains("\"schema\": \"fdu.report/7\""), "{output}");
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
            footer.starts_with(
                "Performance: walked 2 files / 8 B; ignore rules 0 files; content read 8 B at "
            ),
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
            &ControlCoverage::NotObserved,
            Duration::from_millis(2_500),
        );

        assert_eq!(
            footer,
            "Performance: walked 12,345 files / 2.0 KiB; no ignore rules; content read 2.0 KiB at 1.0 KiB/s; analysis 3,000 fresh at 1.5k files/s, 2 cached / 4.0 KiB; warm revalidation; total 2.50 s"
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
            ("tree", report_format::Format::Tree),
            ("paths", report_format::Format::Paths),
            ("long", report_format::Format::Long),
            ("json", report_format::Format::Json),
            ("jsonl", report_format::Format::Jsonl),
            ("yaml", report_format::Format::Yaml),
        ] {
            let cli = Cli { format: value.to_string(), ..cli() };
            assert_eq!(cli.parse_format().expect("format parses"), expected);
            assert_eq!(cli.machine_format(), expected.is_machine());
        }
        let message = Cli { format: "xml".to_string(), ..cli() }
            .parse_format()
            .expect_err("rejected")
            .to_string();
        assert!(message.contains("text, tree, paths, long, json, jsonl, yaml"), "{message}");
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
        assert_eq!(finish(Ok(RunOutcome::Complete), &mut diagnostic, false), 0);
        assert_eq!(finish(Ok(RunOutcome::Partial), &mut diagnostic, false), 2);

        let broken_pipe =
            anyhow::Error::new(io::Error::new(io::ErrorKind::BrokenPipe, "reader closed"))
                .context("render output");
        assert_eq!(finish(Err(broken_pipe), &mut diagnostic, false), 0);
        assert!(diagnostic.is_empty());

        let args = [OsString::from("fdu"), OsString::from("--help")];
        assert_eq!(
            run_with_io(&args, &mut FailingWriter, &mut diagnostic, false, false),
            1,
            "a non-pipe help-output failure is fatal"
        );
    }
}
