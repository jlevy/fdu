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
#[cfg(test)]
use fdu_core::query::IgnoredEntries;
#[cfg(feature = "watch")]
use fdu_core::query::parse_when;
use fdu_core::query::{
    AxisNames, Delivery, ReadSpec, Report, ReportSource, Request, RequestError, RequestSpec,
    RootsRequest, SizeMetric, WatchDelivery, parse_cache_policy,
};
use fdu_core::report_format;
use fdu_core::report_format::human_count;
use fdu_core::{CachePolicy, CacheScope, CacheState, Progress, default_cache_path_in};
use fdu_core::{
    PerformanceSummary, RootsPrepared, prepare_roots_report, prepare_roots_report_with_progress,
    prepare_roots_report_with_scan_diagnostics,
};

use crate::progress_line::{
    ProgressMode, ProgressPlan, TerminalFacts, display_root, home_directory, should_draw,
};
use crate::progress_ticker::{ProgressIo, Ticker};
use crate::skill_install;

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
//   warning      yellow
//   error        red bold
//   cause        dimmed         the chain under an error
//   telemetry    bright black   the performance footer, notes, watch rules
//
// Colour applies only when the destination is a live terminal, and never to a machine
// format or under NO_COLOR; `ColorContext` owns that decision and `paint` applies it.

/// Presentation choices shared by the initial watch report and every repaint.
#[cfg(feature = "watch")]
#[derive(Clone, Copy)]
struct WatchPresentation {
    render: report_format::RenderOptions,
    diagnostic_color: bool,
    quiet: bool,
}

/// The one header style every human surface uses.
///
/// Report view headers, the `--docs` section headers, and clap's help section headings
/// are the same kind of thing — a name introducing a block — so they share a style and a
/// case convention rather than each inventing one. `report_format` re-exports this as the
/// view-header style so there is a single definition to change.
const STYLE_HEADING: AnsiStyle = report_format::STYLE_HEADING;
const STYLE_WARNING: AnsiStyle = AnsiColor::Yellow.on_default();
pub(crate) const STYLE_ERROR: AnsiStyle = AnsiColor::Red.on_default().bold();
const STYLE_CAUSE: AnsiStyle = AnsiStyle::new().dimmed();
const STYLE_PERFORMANCE: AnsiStyle = report_format::STYLE_DETAIL;

/// The rule that separates one watch repaint from the one before it.
///
/// Gray for the same reason the performance footer is: it is a frame around the report,
/// not part of the answer, and should not compete with the rows for attention.
#[cfg(feature = "watch")]
const STYLE_WATCH_RULE: AnsiStyle = report_format::STYLE_DETAIL;
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
const DOCS_POINTER: &str = r"Agent setup:
  uvx --no-build fdu@latest --install-skill

Examples:
  fdu .                         directory sizes (metadata only)
  fdu . --view=code,documents   lines of code by language, words by document type
  fdu . --ignored=exclude       omit entries covered by .gitignore
  fdu . --view=summary          one total for the tree
  fdu . --kind dir --include .venv --modified-before 7d --long
  fdu . --kind dir --include node_modules --modified-before 30d --long
  fdu . --kind dir --include target --modified-before 30d --format paths

Run `fdu --docs` for setup, libraries, more commands, cache behavior, and the full usage guide.";

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
    ($one_path_watch:literal, $watch_composition:literal, $mode_flags:literal) => {
        concat!(
            r"fdu — the fastest du replacement, with .gitignore-aware sizes and code and
document counts, for the command line, Python, and Rust.

  For every directory in a tree at once, fdu reports its size, file count,
  recency, and file kinds, and on request lines of code by language and words
  by document type.
  It walks the tree on several threads through each platform's native
  directory interface, and on a generated million-entry tree (875,000 files)
  it finished ahead of du and the seven other disk-usage tools measured on
  Linux and macOS.
  Content metrics are cached between runs, and every capability is also a
  Rust and Python API.
  Performance measurements: https://github.com/jlevy/fdu/blob/main/docs/performance-measurements.md

SET UP WITH ANY CODING AGENT
  Install fdu's self-contained skill for current and future agent sessions:

    uvx --no-build fdu@latest --install-skill

  Run it from the project root. The generated SKILL.md needs no prior session
  context. If fdu is on PATH, `fdu --install-skill` is equivalent.

INSTALL THE COMMAND LINE
  Run the latest release once, or keep it on PATH:

    uvx --no-build fdu@latest PATH
    uv tool install --no-build fdu
    fdu PATH

  The wheel requires GIL-enabled Python 3.12 or newer. uv normally selects a
  matching interpreter; if it selects a free-threaded build such as 3.14t,
  add `--python 3.14` to the uvx or uv tool command.

USE AS A LIBRARY
  Python: `uv add fdu` (or `pip install fdu`)
  Rust:   `cargo add fdu` (or `cargo add fdu-core` for the engine alone)

START HERE
  A report requires a PATH. Use `.` for the current directory.

    fdu .                                      directory sizes (the default)
    fdu . --view=code,documents                lines of code and words, one scan
    fdu . --view=code                          standard lines of code by language
    fdu . --view=documents                     words and pages by document format
    fdu . --ignored=exclude                    omit entries covered by .gitignore
    fdu . --view=summary                       one total for the tree
    fdu . --view=languages                     languages by byte size
    fdu . --view=families,types,extensions     three file-kind breakdowns
    fdu . --view=recent --limit=10             ten most recently modified files
    fdu docs src                               several paths as one report, with a total

  `fdu .` is metadata-only. It prints a tree in allocated bytes, largest first,
  to depth 5, showing contents with at least 1% of the selected root size, each
  with its age: how long ago anything it counts last changed. Hidden
  and gitignored entries are included; .gitignore is read to label gitignored shares, not to exclude them.

  Several paths are what each would report, added: every size, row, share, and
  bound is over the union, once. The tree starts with a (total) row, each root
  is a row named as given, and flat paths are printed after their root's label.
  Every PATH is a directory; `fdu */` names only the directories here, and is
  refused where one is a symlink to another (lib64 -> lib). Each root is walked
  separately and pays a walk's fixed cost: 625 small roots took 1.72 s, one
  walk of their parent 0.23 s, about 7x (exp-216, macOS, uncontrolled host).
  One walk shows every entry one level down: `fdu --depth 1 --min-share 0% .`
  A root inside another, or the same directory twice, is refused (exit 2).
  --cache-status and --cache-clear take one PATH",
            $one_path_watch,
            r".

  code and documents read file contents; --analyze is the extra control for
  analysis a view does not imply:

    fdu . --analyze=code --view=languages      code lines in the language rows
    fdu . --analyze=lines --view=languages     physical lines and raw words by language

VIEWS AND ANALYSIS
  --view chooses the question the report answers. Several views share one scan
    and one analysis; adding a view does not run a second scan.
  code and documents are the views that read file contents: they show nothing
    without analysis, so naming one runs its analyzer, code or words. Every
    other view, full included, opens no regular file on its own.
  --analyze runs analyzers beyond what the views imply: code lines in the
    languages rows, physical lines in families and types, or a run that only
    warms the cache. Compatible cached results avoid rereading unchanged bodies,
    so a repeated content analysis can be much cheaper.

  Naming analyzers alone selects a view that displays them: code selects code,
  words selects documents, code,words selects both, and lines selects families.
  Name --view for a different projection; it always wins. Headers name views;
  columns name measurements: words is an analyzer, and documents is its view of
  prose and markup. --analyze words --view types counts words in every text type.

  A view with a metadata meaning never turns on an analyzer, because choosing how
  to look at a result should not quietly authorize reading every file in the
  tree. If a selected view cannot display requested analysis, fdu still performs
  the analysis and prints a note. --view=full names any view it had to skip.

MORE COMPOSITIONS
  fdu ~/Downloads --view=extensions
  fdu . --view=types,families --format=json
  fdu . --analyze=words --view=types
  fdu PATH --view tree --full --format json                 complete recursive tree
  fdu PATH --kind dir --full --format json                  recursive directory totals
  fdu PATH --kind file --full --format paths                find/fd-style file inventory
  fdu PATH --view=largest --limit=100                        the 100 largest files
  fdu PATH --view=files --kind=file --modified-since=1h      files changed lately
  fdu PATH --view=files --ignored=only --format=jsonl        what .gitignore covers
",
            $watch_composition,
            r"
  largest and recent are presets over files, not more views to learn:
    largest = files --sort size --limit 20, regular files only
    recent  = files --sort mtime --limit 20, regular files only
  --full expands to --depth=all --breadth=all --limit=all --min-share=0%.
  Explicit bounds override it regardless of order. It leaves view, scan scope,
  population, and analysis unchanged. --view=full chooses views instead.
  --sort and --limit still override them. files alone is complete: every
  matching entry, in name order. full combines applicable views, without list/files.

LIST FORMATS AND OLD BUILD DIRECTORIES
  The metadata default view is list; its default format is tree. These agree:
    fdu PATH
    fdu PATH --view list --format tree
  Tree shows directories and significant files to depth 5. --min-share 1% compares
  every row to the selected root total. --min-share 0% shows even zero-size rows.
  --breadth caps children per directory; --limit caps data rows per section.
  Both default to all. --depth all expands levels. Omission notes name each bound.
  Display bounds never change totals or limit the filesystem scan.
  A tree row's age is its newest counted change, of files, directories, and symlinks;
  unknown where a subtree was not listed. Ages read s, m, h, d, mo (30.44 days), and
  y (365.25 days); time bounds take days. --sort mtime orders rows by that age.

  --format paths gives matching paths only; --long adds size, age, and path.
  Flat lists are complete and size-ranked by default; --sort name lists by name.
  JSON, JSONL, and YAML give exact metrics. text keeps automatic human tables.
  Tree/paths/long require a single list view; use text or machine formats for
  grouped/mixed views and full. largest/recent accept paths/long, keeping file ranks.
  files keeps name order; the tree view keeps structured tree output.
  Explicit paths/long overrides the tree presentation. Format flags conflict.

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
  Scope      PATH..., --scan-depth, --one-filesystem    what is scanned and cached
             --gitignore-budget, --gitignore-line-limit, --no-gitignore, --ignored
  Content    --analyze none|lines|code|words|all        which file bodies are read
                                                        beyond what the views imply
  Selection  --include, --exclude, --depth, --limit     which entries are considered
             --breadth, --min-share
  View       list,summary,tree,families,types,extensions,languages,code,documents,
             largest,recent,files,full
  Format     --format text|tree|paths|long|json|jsonl|yaml, --tree, --long
             --color, --progress
  Mode       ",
            $mode_flags,
            r"

CONTENT ANALYSIS
  none       no analyzer beyond what the views imply (default)
  lines      physical, blank, and nonblank lines plus raw word counts
  code       standard SLOC from the versioned common-language analyzer
  words      normalized and reader-visible word volume
  all        every shipped analyzer

  A comma-separated set: code,words runs both. none and all name the whole
  axis and cannot be combined. code and words already include lines; adding lines
  explicitly changes neither measurements nor work. lines alone measures physical
  text volume without language-specific code counting or word normalization.
  languages is metadata-only by default; --view code runs code analysis itself.
  Code reports show source lines, language shares, population columns, and coverage.
  --view documents runs words analysis itself.
  Analysis streams every eligible file through EOF; files are never size-truncated.
  --workers bounds content-analysis concurrency; directory scanning uses its own pool.
  --words-per-page changes only report-time page derivation.
  Unchanged results are restored from a separate sidecar written by the same
  analyzer set; any other set, wider or narrower, reads the files again.
  --stale-ok never opens source files and fails if requested content is absent.

CACHE BEHAVIOR
  --cache=auto, the default, uses the cache only where the kind of run gains from
  it. A metadata report neither reads nor writes one: checking a snapshot costs as
  much as the scan it would save, and no later report reads it. Content analysis
  and long-lived sessions read, revalidate, and write it. --cache=on also writes
  after a one-shot report, leaving a snapshot for a later --stale-ok answer.

  Content analysis is where repeated-run caching pays most. The first run reads
  eligible file bodies. A compatible later run reuses results for unchanged files
  and reads only changed or newly eligible bodies; the performance footer reports
  fresh and cached analysis separately. Repeat a --view=code run to see it.

  --stale-ok answers from the snapshot alone: it does no filesystem verification,
  requires a compatible snapshot and content sidecar for the requested analysis,
  and labels its answer stale with a warn: line on stderr that --quiet keeps.
  --cache=off neither reads nor writes fdu's cache.

  macOS and Linux default to ~/.cache/fdu; Windows uses %LOCALAPPDATA%/fdu.
  --cache-dir overrides FDU_CACHE_DIR, then XDG_CACHE_HOME/fdu and native defaults.
  Each root has <key>.metadata.bin and optional <key>.analysis.bin. The latter
  stores derived metrics, not source bodies. Status and clear use the same directory.

IGNORE RULES
  Fresh scans read applicable .gitignore files by default; --stale-ok uses retained
  rules. Summary, tree, and extension rows show gitignored size as `(128 B gitignored)`.
  Ignoring a directory covers its descendants. Non-gitignored does not mean Git-tracked:
  .git is non-gitignored unless a rule names it. --ignored=include is default.
  --ignored=exclude prunes safely gitignored subtrees and skips gitignored body reads.
  --ignored=only discovers gitignored matches through ordinary ancestors, reading
  only gitignored bodies for analysis. Sort and --min-size follow the size shown.
  --no-gitignore reads no rules and shows no share. Only per-directory .gitignore
  files apply, not core.excludesFile, .git/info/exclude, or a global ignore file.
  Each is found as git opens it, so a .GITIGNORE counts on a case-insensitive
  volume; matching itself is case-sensitive on every platform. An unreadable
  .gitignore makes the result partial, like any unreadable path. A .gitignore past
  --gitignore-budget or --gitignore-line-limit is refused whole and named in a note:
  sizes stay exact, gitignored shares under that directory do not.

OUTPUT AND AUTOMATION
  Every machine report uses fdu.report/11; watch changes use fdu.stream/2.
  Cache status is its own document in every machine format: fdu.cache/3.
  Summary, tree, extension, and file rows carry `ignored`: null under --no-gitignore.
  Text language rows use canonical names; machine formats retain lowercase IDs.
  Metric rows include detection source, confidence, origin flags, and coverage.
  Tree remainder totals are shared by every format: recursive files, apparent and
  allocated bytes, and applicable reasons. Null means nothing hidden; unknown counts
  or sizes stay null. Text shows one root-level row, `… and N more files`, with the
  hidden share and size in the tree's columns.
  Text results hold only rows, column headings, and multi-view headers. Human
  diagnostics use note:, warn:, tip:, and perf: on stderr, in that order; one
  `tip: show more:` names the flags that lift every display limit that hid rows.
  One-shot text reports end with gray perf: on stderr; machine formats omit it.
  It counts ignore files and accepted rules, including repeated governing sources.
  Total files/s and binary GiB/s use the displayed elapsed duration. GiB/s represents
  walked file size; actual body-read throughput is reported separately.
  JSON numbers above 2^53 (fingerprints, option hashes, nanosecond timestamps)
  lose precision in IEEE 754 binary64 parsers such as JavaScript JSON.parse.
  Results go to stdout; all diagnostics, including warnings and errors, go to stderr.
  The command never prompts or pages. A progress line is drawn on stderr only for a
  person at an interactive terminal; --progress never draws into a pipe, a file, or CI.
  Reports require an explicit PATH; bare `fdu` prints help and scans nothing.
  `fdu --install-skill` writes a portable agent skill describing this same surface
  under the project root, and `fdu --skill` prints it.

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
    ", as does --watch",
    "  fdu --watch --view files --format jsonl PATH              a tail -f for a tree

  --interval throttles rendering only; change detection is event-driven and
  unaffected by it, so an idle tree costs nothing between changes.
  The duration uses the age grammar: `2s`, `200ms`, `1h30m`.
",
    "--cache, --watch, --workers"
);
/// The guide for a command line built without `watch`, which names neither of its flags.
#[cfg(not(feature = "watch"))]
const DOCS: &str = docs_guide!("", "", "--cache, --workers");

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

/// An error from a report's walks, in this command's words.
///
/// An overlap a walk found, entering another root through an alias, is the refusal the
/// same overlap found before the walk is, and exits 2 as that one does (review D1 on
/// #192): which check sees it depends only on whether the paths show it. Every other
/// error is the engine's, a root that changed between validation and its walk included.
fn walk_error(error: fdu_core::Error) -> anyhow::Error {
    match error {
        fdu_core::Error::InvalidRequest(refusal @ RequestError::RootReachedInside { .. }) => {
            usage(&refused(&refusal))
        }
        other => other.into(),
    }
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
    override_usage = "fdu [OPTIONS] <PATH>...\n       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]\n       fdu [PATH] --cache-clear[=<SCOPE>]\n       fdu --docs\n       fdu --skill\n       fdu --install-skill [--agent-base <DIR>]"
)]
// A command line is a flat bag of independent switches. Folding these into enums to
// satisfy the lint would obscure the one thing this struct exists to mirror: the flags a
// user actually types.
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    // ---- scope: what the engine observes and retains ----
    /// Report roots, disjoint; optional only for the discovery and cache-lifecycle flags.
    ///
    /// Several roots report as one, as if each were reported alone and the reports added.
    /// The engine validates them ([`Roots::resolve`]), so this only collects them in order.
    #[arg(
        value_name = "PATH",
        required_unless_present_any = ["docs", "skill", "install_skill", "cache_status", "cache_clear"],
        help_heading = "ARGUMENTS"
    )]
    pub paths: Vec<PathBuf>,

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

    /// Read no .gitignore files: rows lose their gitignored share, and the snapshot scope differs
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

    /// Ignored population: include, exclude, or only [default: include].
    #[arg(long, value_name = "MODE", help_heading = "SCOPE")]
    pub ignored: Option<String>,

    /// Directory levels to show; does not limit scanning. Accepts `all` [tree default: 5].
    ///
    /// Optional for the same reason `--limit` is: the tree brings its own default from
    /// the library, so the CLI does not declare one and every surface agrees. Said in
    /// the help text rather than by clap, because it is the view's default and not the
    /// flag's -- only the tree renders a hierarchy for a depth to bound.
    #[arg(short, long, value_name = "N", help_heading = "SELECTION")]
    pub depth: Option<String>,

    /// Maximum data rows per section. Accepts `all` [default: all; largest/recent: 20].
    ///
    /// Each view brings its own default, because one number does not suit them all: a
    /// tree and grouped views are unbounded, while `largest` and `recent` show twenty.
    #[arg(short = 'n', long, value_name = "N", help_heading = "SELECTION")]
    pub limit: Option<String>,

    /// Maximum children per directory. Accepts `all` [default: all].
    #[arg(long, value_name = "N", help_heading = "SELECTION")]
    pub breadth: Option<String>,

    /// Minimum percentage of the selected root measure [tree default: 1%].
    #[arg(long, value_name = "PERCENT", help_heading = "SELECTION")]
    pub min_share: Option<String>,

    /// Show all rows: --depth=all --breadth=all --limit=all --min-share=0%. Explicit bounds override this shorthand.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "SELECTION")]
    pub full: bool,

    /// Order results: size, count, mtime, name, or a requested metric such as `code_lines`.
    #[arg(long, value_name = "KEY", help_heading = "SELECTION")]
    pub sort: Option<String>,

    /// Reverse the ordering.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "SELECTION")]
    pub reverse: bool,

    /// Which size metric to report: allocated or apparent.
    #[arg(long, value_name = "METRIC", default_value = SIZE_DEFAULT, help_heading = "SELECTION")]
    pub size: String,

    // ---- view: which roll-ups are reported ----
    /// Views: list, tree, files, extensions, types, families, languages, code, documents,
    /// largest, recent, summary, or full. Defaults to list with no analysis, otherwise to
    /// a view that displays the requested analysis. code and documents read file contents:
    /// each runs its analyzer.
    #[arg(long, value_name = "LIST", help_heading = "VIEWS")]
    pub view: Option<String>,

    /// Analyzers to run beyond what the views imply: none, lines, code, words, or all.
    ///
    /// Any analyzer, named or implied by a view, reads each eligible file missing from a
    /// compatible content cache.
    #[arg(
        long,
        value_name = "LIST",
        default_value = ANALYZE_DEFAULT,
        help_heading = "CONTENT ANALYSIS"
    )]
    pub analyze: String,

    /// Content-analysis workers; zero selects available parallelism.
    #[arg(
        long = "workers",
        value_name = "N",
        default_value_t = 0,
        help_heading = "CONTENT ANALYSIS"
    )]
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

    /// Show a progress line on stderr while a report runs: auto, always, or never. Drawn only at an interactive terminal: always, unlike --color, never draws into a pipe or file
    #[arg(
        long,
        value_name = "WHEN",
        default_value = "auto",
        hide_possible_values = true,
        help_heading = "OUTPUT"
    )]
    pub progress: ProgressMode,

    /// Width of tree bars in cells (at most 4096); zero or a negative value hides the bar.
    #[arg(
        long,
        value_name = "N",
        default_value_t = 10,
        allow_hyphen_values = true,
        help_heading = "OUTPUT"
    )]
    pub bar_size: i64,

    /// Suppress notes, tips, performance summaries, and progress; keep warnings and errors.
    #[arg(short = 'q', long, action = ArgAction::SetTrue, help_heading = "OUTPUT")]
    pub quiet: bool,

    // ---- mode: how the cache is used ----
    /// Cache policy: auto (where it pays for this run), on, or off.
    #[arg(long, value_name = "POLICY", default_value = "auto", help_heading = "EXECUTION")]
    pub cache: String,

    /// Answer from the cached snapshot without scanning; the answer may be stale.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "EXECUTION")]
    pub stale_ok: bool,

    /// Cache directory; overrides `FDU_CACHE_DIR` and the platform default.
    #[arg(long, value_name = "DIR", help_heading = "DELIVERY")]
    pub cache_dir: Option<PathBuf>,

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

    /// Print setup, common commands, cache behavior, and the complete usage guide.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "OTHER")]
    pub docs: bool,

    /// Print a portable agent skill to stdout.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "OTHER")]
    pub skill: bool,

    /// Write that skill under the project root, to .agents/skills/fdu/ and .claude/skills/fdu/.
    #[arg(long, action = ArgAction::SetTrue, help_heading = "OTHER")]
    pub install_skill: bool,

    /// With --install-skill, write DIR/skills/fdu/SKILL.md instead: one agent's user scope, such as ~/.claude.
    #[arg(long, value_name = "DIR", requires = "install_skill", help_heading = "OTHER")]
    pub agent_base: Option<PathBuf>,
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
    ///
    /// `terminal` is what the process read about stderr once, in `run_process`; every
    /// stderr presentation decision, color and progress alike, is taken from it here
    /// rather than from the environment, so a test can say what the terminal is.
    /// `progress_io` is what a drawing run draws with, read there too, so a test can
    /// see the line's bytes and wait out no delay.
    pub fn run(
        &self,
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        stdout_is_terminal: bool,
        terminal: &TerminalFacts,
        progress_io: ProgressIo,
    ) -> anyhow::Result<RunOutcome> {
        let stderr_is_terminal = terminal.stderr_is_terminal;
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

        if self.install_skill {
            return self.run_install_skill(out);
        }

        // Lifecycle flags run before scan validation, so they need no readable tree, and
        // they suppress the report entirely: a run that inspects or clears the cache is
        // not also a run that scans. Clear runs first so a combined invocation reports
        // the state it left behind. A cache belongs to one root, so they take one PATH.
        if self.cache_clear.is_some() || self.cache_status.is_some() {
            let flag = if self.cache_clear.is_some() { "--cache-clear" } else { "--cache-status" };
            self.one_path_for(flag)?;
            return self.run_cache_lifecycle(out, stdout_is_terminal);
        }

        // Parse the whole request before touching the filesystem, so a typo in a glob or a
        // time costs nothing and reports its own spelling rather than a scan's worth of
        // waiting followed by an error.
        let format = self.parse_format().map_err(|error| usage(&error))?;
        let path = self.paths.first().ok_or_else(|| {
            usage(&anyhow::anyhow!(
                "missing PATH: specify the directory to summarize, for example `fdu .`"
            ))
        })?;
        // A watch keeps one tree current; several would need one watcher each and a
        // merged stream, which nothing builds yet.
        #[cfg(feature = "watch")]
        if self.watch {
            self.one_path_for("--watch")?;
        }
        // One grammar, one defaults table, one set of rules: this command line hands the
        // model the words its caller typed and renders whatever comes back in flag names.
        // It used to parse each axis itself, which is how a default could differ between
        // the doors into the same engine. Several roots share this one request; it names
        // the first, and the engine roots a copy at each.
        let request = self.request(path, SystemTime::now())?;
        let has_tree = request.query.views.iter().any(|view| {
            matches!(view, fdu_core::query::ViewSpec::List | fdu_core::query::ViewSpec::Tree)
        });
        if has_tree
            && matches!(format, report_format::Format::Text | report_format::Format::Tree)
            && self.bar_size() > report_format::MAX_BAR_SIZE
        {
            return Err(usage(&anyhow::anyhow!(
                "invalid --bar-size \"{}\": expected at most {} cells",
                self.bar_size,
                report_format::MAX_BAR_SIZE
            )));
        }
        let delivery = Delivery {
            cache: self.parse_cache_policy().map_err(|error| usage(&error))?,
            stale_ok: self.stale_ok,
            cache_path: None,
            // The directory every root's snapshot is named in, resolved here because this
            // command caches by default; the engine names each root's file in it, as this
            // command line once named one root's. A bad `--cache-dir` fails first, before
            // any root is resolved, as it always has.
            cache_dir: fdu_core::default_cache_dir(self.cache_dir.as_deref())?,
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
        // The roots, refused in the order one root's run has always been: a root that is
        // missing; then a delivery no route can carry -- what a watch cannot carry (a
        // narrowed scan scope, content analysis nothing re-reads, a snapshot nothing
        // verified) is the model's rule, so a library caller and a Python caller meet the
        // same wall -- then a root that is not a directory, or that overlaps another.
        let request =
            RootsRequest::resolve(&self.paths, request, &delivery).map_err(
                |error| match error {
                    fdu_core::Error::InvalidRequest(refusal) => usage(&refused(&refusal)),
                    other => self.root_error(other),
                },
            )?;

        // Resolved before any work starts, beside the color decision; the ticker takes
        // this rather than re-deriving it deeper in.
        let progress_plan = self.progress_plan(terminal, request.request());

        #[cfg(feature = "watch")]
        if self.watch {
            let color = ColorContext::from_environment(
                self.color,
                self.machine_format(),
                self.skill,
                stdout_is_terminal,
            )
            .enabled();
            let diagnostic_color = ColorContext::from_environment(
                self.color,
                false,
                false,
                terminal.stderr_is_terminal,
            )
            .enabled();
            let indicator = progress_plan.draw.then_some((progress_plan, progress_io));
            // A watch has one root, checked above, and keeps its snapshot where one root's
            // report would: the session names it in the delivery's directory.
            return Self::run_watch(
                out,
                diagnostic,
                format,
                request.request(),
                &delivery,
                indicator,
                WatchPresentation {
                    render: report_format::RenderOptions { color, bar_size: self.bar_size() },
                    diagnostic_color,
                    quiet: self.quiet,
                },
            );
        }

        let report_started = Instant::now();
        let collect_scan_diagnostics =
            std::env::var_os(SCAN_DIAGNOSTICS_ENV).is_some_and(|value| value == OsStr::new("1"));
        // Three doors into one engine route, and the answer is the same bytes through each.
        // The measurement door comes first because it is a measurement: a ticker thread
        // polling beside the walk would be part of what it measures. A run that does not
        // draw takes the plain door and pays nothing for the indicator, not even a handle.
        let (prepared, scan_diagnostics) = if collect_scan_diagnostics {
            prepare_roots_report_with_scan_diagnostics(&request, &delivery).map_err(walk_error)?
        } else if progress_plan.draw {
            // The one place the line is stopped on this route: right after the engine
            // returns, with a report or with an error, and before a byte reaches either
            // stream. Everything below -- the report, the save warning, the performance
            // line, the notes, the diagnostics, and the error `finish` prints -- comes
            // after this stop, and the ticker's drop repeats it on unwind.
            let progress = Progress::new();
            let mut ticker =
                Ticker::start(progress_plan, progress.clone(), report_started, progress_io);
            let prepared = prepare_roots_report_with_progress(&request, &delivery, &progress);
            ticker.stop();
            (prepared.map_err(walk_error)?, Vec::new())
        } else {
            (prepare_roots_report(&request, &delivery).map_err(walk_error)?, Vec::new())
        };
        let RootsPrepared { report, pending: pending_saves, performance } = prepared;
        // One line per root that walked, in the roots' order.
        for scan_diagnostics in scan_diagnostics.into_iter().flatten() {
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
        // nominal: the caller's writer is buffered, a compact tree can fit inside
        // that buffer, and the user would see nothing until the snapshot's fsync and the
        // index teardown had finished (fdu-n75m). Same bytes in the same order; only
        // when they arrive changes.
        let render_options = report_format::RenderOptions { color, bar_size: self.bar_size() };
        let rendered_text =
            matches!(format, report_format::Format::Text | report_format::Format::Tree)
                .then(|| report_format::render_with_options(&report, format, render_options));
        let (rendered_text, render_result): (Option<String>, anyhow::Result<()>) =
            match rendered_text {
                Some(Ok(rendered)) => {
                    let result =
                        write!(out, "{rendered}").and_then(|()| out.flush()).map_err(Into::into);
                    (Some(rendered), result)
                }
                Some(Err(error)) => (None, Err(error.into())),
                None => (
                    None,
                    report_format::write_with_options(&report, format, render_options, out)
                        .and_then(|()| out.flush())
                        .map_err(Into::into),
                ),
            };

        // Joined before returning, and before the render error is raised: a broken pipe
        // must not abandon a finished scan's snapshot, because the next run would then
        // pay for a cold scan that this one had already done.
        let diagnostic_color =
            ColorContext::from_environment(self.color, false, false, stderr_is_terminal).enabled();
        // Every root's save is joined. One root has always warned of its first failure;
        // over several, each failure is its own warning.
        let failures = pending_saves.join_all();
        let shown = if request.roots().is_several() { failures.len() } else { 1 };
        let save_warnings: Vec<String> =
            failures.into_iter().take(shown).map(|error| format!("warn: {error}")).collect();
        if render_result.is_err() {
            // A failed report write still joins the saves and tells the caller if one failed.
            for warning in &save_warnings {
                let _ = writeln!(diagnostic, "{}", paint(warning, STYLE_WARNING, diagnostic_color));
            }
        }
        render_result?;

        if matches!(format, report_format::Format::Text | report_format::Format::Tree) {
            let rendered = rendered_text.as_deref().unwrap_or_default();
            if !rendered.is_empty() && !rendered.ends_with('\n') {
                writeln!(out)?;
            }
        }
        out.flush()?;

        // The answer stays on stdout. Facts, suggestions, and operational diagnostics
        // share stderr; the human report's run summary closes that stream.
        write_report_diagnostics(
            diagnostic,
            &report,
            format,
            diagnostic_color,
            &save_warnings,
            self.quiet,
        )?;
        if !self.quiet
            && matches!(format, report_format::Format::Text | report_format::Format::Tree)
        {
            writeln!(
                diagnostic,
                "{}",
                paint(
                    &performance_footer(
                        &performance,
                        report.roots.as_deref(),
                        &report.ignore_rules,
                        report_started.elapsed(),
                        request.request().query.selection.size,
                        diagnostic_color,
                    ),
                    STYLE_PERFORMANCE,
                    diagnostic_color,
                )
            )?;
        }
        diagnostic.flush()?;

        let plan = fdu_core::plan(request.request(), &delivery, fdu_core::Route::OneShot)?;
        Ok(match plan.outcome(&report.status) {
            fdu_core::OutcomeClass::Success => RunOutcome::Complete,
            fdu_core::OutcomeClass::Partial => RunOutcome::Partial,
        })
    }

    /// Refuse a second PATH for a flag that acts on one root: a watch, and the cache
    /// lifecycle, whose snapshot belongs to one root. A command-line rule because no
    /// engine call can be asked otherwise: a watch session and `cache_status` take one root
    /// by type.
    fn one_path_for(&self, flag: &str) -> anyhow::Result<()> {
        match self.paths.len() {
            0 | 1 => Ok(()),
            given => Err(usage(&anyhow::anyhow!("{flag} takes one PATH; {given} were given"))),
        }
    }

    /// A root the engine refused, in this command's words.
    ///
    /// A PATH that names no directory says so by the name it was given and what to type
    /// instead, since `fdu *`, the way `du -sh *` is typed, meets a file in nearly every
    /// directory and fdu reports on directories (review A5 on #192); the exit status is
    /// still a filesystem error's. The message stays one line: what `fdu */` costs over
    /// many small directories, the one-walk alternative, and the symlinked sibling that
    /// refuses it in turn are in `--docs`, usage, and the skill (review D2). Every other
    /// error is the engine's.
    fn root_error(&self, error: fdu_core::Error) -> anyhow::Error {
        if let fdu_core::Error::Io { path, source } = &error {
            let given = (source.kind() == io::ErrorKind::NotADirectory)
                .then(|| {
                    self.paths
                        .iter()
                        .find(|given| given.canonicalize().is_ok_and(|resolved| resolved == *path))
                })
                .flatten();
            if let Some(given) = given {
                let what = if path.is_file() { "is a file" } else { "is not a directory" };
                return anyhow::anyhow!(
                    "{} {what}; fdu reports on directories (to name only the directories \
                     here: fdu */)",
                    given.display()
                );
            }
        }
        error.into()
    }

    /// Whether the requested format is a machine format, which is never colorized.
    fn machine_format(&self) -> bool {
        self.parse_format().is_ok_and(report_format::Format::is_machine)
    }

    /// Nonpositive widths hide the bar; checked conversion prevents wrap on 32-bit hosts.
    fn bar_size(&self) -> usize {
        usize::try_from(self.bar_size.max(0)).unwrap_or(usize::MAX)
    }

    /// What the progress ticker needs before its first frame: whether to draw at all,
    /// the root as the frame shows it, and whether the frame is colored.
    ///
    /// Resolved here beside the color decision because it is the same kind of decision:
    /// presentation, from the flag and the terminal, never from the engine. Whether the
    /// line is drawn is decided by `--progress`, the terminal, the format, and whether
    /// this command walks at all; whether it is colored follows the rule that colors
    /// warnings on stderr, `--color`, then `NO_COLOR`, then `FORCE_COLOR`, so neither
    /// setting can turn the other on or off. The ticker starts from this plan before
    /// the engine is called and stops before the first byte reaches either stream.
    fn progress_plan(&self, terminal: &TerminalFacts, request: &Request) -> ProgressPlan {
        let walks = !(self.docs
            || self.skill
            || self.install_skill
            || self.cache_status.is_some()
            || self.cache_clear.is_some());
        let home = home_directory();
        let roots = if self.paths.is_empty() {
            vec![display_root(Path::new("."), home.as_deref())]
        } else {
            self.paths.iter().map(|root| display_root(root, home.as_deref())).collect()
        };
        ProgressPlan {
            draw: !self.quiet && should_draw(self.progress, terminal, self.machine_format(), walks),
            roots,
            color: ColorContext::from_environment(
                self.color,
                false,
                false,
                terminal.stderr_is_terminal,
            )
            .enabled(),
            size: request.query.selection.size,
        }
    }

    /// Run the query continuously, streaming changes as they arrive.
    ///
    /// The initial report is exactly what a one-shot run would print, and every later
    /// render is the same query re-evaluated. Detection is event-driven throughout: an
    /// idle tree costs no filesystem work, and `--interval` throttles only how often
    /// aggregate views repaint.
    ///
    /// `indicator` is the progress plan and what it draws with, for a run that draws:
    /// the line runs during the initial scan and stops before the first paint, after
    /// which the repaint is the progress.
    #[cfg(feature = "watch")]
    fn run_watch(
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        format: report_format::Format,
        request: &Request,
        delivery: &Delivery,
        indicator: Option<(ProgressPlan, ProgressIo)>,
        presentation: WatchPresentation,
    ) -> anyhow::Result<RunOutcome> {
        use fdu_core::query::ViewSpec;

        let WatchPresentation { render, diagnostic_color, quiet } = presentation;

        // The repaint interval the delivery already carries, rather than a second reading
        // of `--interval`: `run` refused an unparseable one before it opened anything, and
        // a value parsed twice is a value that can mean two things.
        let interval = delivery
            .watch
            .expect("run() builds a watch delivery before it takes the watch path")
            .interval;

        let (mut session, initial) =
            Self::start_watch(request, delivery, format, render, indicator)?;
        Self::persist_live(&mut session, diagnostic, diagnostic_color);

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

        // The initial answer, identical to a one-shot run's, which the start above built
        // as a repaint so the session holds it as the answer every later repaint is
        // measured against.
        if format == report_format::Format::Yaml {
            write!(out, "{}", report_format::document_start(format))?;
        }
        report_format::write_with_options(&initial, format, render, out)?;
        out.flush()?;
        write_report_diagnostics(diagnostic, &initial, format, diagnostic_color, &[], quiet)?;

        let mut dirty_since_render = false;
        let mut last_render = SystemTime::now();
        loop {
            let Some(batch) = session.next_batch(interval)? else {
                // Nothing arrived in the window. Repaint only if something is pending,
                // so a quiet tree produces no output and no work at all.
                if has_aggregates && dirty_since_render {
                    Self::render_live(
                        out,
                        diagnostic,
                        &mut session,
                        format,
                        render,
                        diagnostic_color,
                        quiet,
                    )?;
                    dirty_since_render = false;
                    last_render = SystemTime::now();
                }
                // The idle branch is where a throttled save has to land. A change that
                // arrived too soon after the last save would otherwise wait for the next
                // change to persist it, and the next change may never come: a burst
                // followed by silence is the single most likely way a watch session ends.
                Self::persist_live(&mut session, diagnostic, diagnostic_color);
                continue;
            };

            Self::render_watch_changes(out, diagnostic, &batch.changes, format, streams_changes)?;
            out.flush()?;

            dirty_since_render |= batch.dirty;
            let elapsed = last_render.elapsed().unwrap_or_default();
            if has_aggregates && dirty_since_render && elapsed >= interval {
                Self::render_live(
                    out,
                    diagnostic,
                    &mut session,
                    format,
                    render,
                    diagnostic_color,
                    quiet,
                )?;
                dirty_since_render = false;
                last_render = SystemTime::now();
            }

            // Persist as we go rather than only at exit. A watch session ends by signal
            // far more often than it ends politely, and std offers no portable signal
            // handler, so an exit-time save would be the one that never runs. Throttled
            // to the render interval so a churny tree does not rewrite constantly; the
            // pending flag is what guarantees a throttled change still reaches disk once
            // the tree goes quiet.
            Self::persist_live(&mut session, diagnostic, diagnostic_color);
        }
    }

    /// Start the session and build its first answer, under the progress line when the
    /// run draws one.
    ///
    /// The one place the line is stopped on this route: once the first answer exists, or
    /// the start or the build has failed, and before the startup save's warning, that
    /// answer, or anything else reaches either stream. The build is under the line
    /// because it is part of the wait: a heavy view over a large tree takes seconds to
    /// build, and a line stopped when the start returned went blank for them
    /// (fdu-wku3). The answer is asked for as a repaint so the session holds it as the
    /// one every later repaint is measured against.
    #[cfg(feature = "watch")]
    fn start_watch(
        request: &Request,
        delivery: &Delivery,
        format: report_format::Format,
        render: report_format::RenderOptions,
        indicator: Option<(ProgressPlan, ProgressIo)>,
    ) -> anyhow::Result<(fdu_core::watch_session::Session, Report)> {
        use fdu_core::watch_session::Session;

        let (session, answer) = if let Some((plan, progress_io)) = indicator {
            let progress = Progress::new();
            let mut ticker = Ticker::start(plan, progress.clone(), Instant::now(), progress_io);
            let started =
                Session::start_with_progress(request.clone(), delivery.clone(), &progress)
                    .and_then(|mut session| {
                        let answer = session.changed_report_with_progress(
                            SystemTime::now(),
                            format,
                            render,
                            &progress,
                        )?;
                        Ok((session, answer))
                    });
            ticker.stop();
            started?
        } else {
            let mut session = Session::start(request.clone(), delivery.clone())?;
            let answer = session.changed_report(SystemTime::now(), format, render)?;
            (session, answer)
        };
        Ok((session, answer.expect("a session's first answer is always given")))
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
            let _ =
                writeln!(diagnostic, "{}", paint(&format!("warn: {error}"), STYLE_WARNING, color));
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
    /// the loop — so the rule below can be unconditional. A change that leaves the answer
    /// as the reader last saw it, such as a touch that moves no size, repaints nothing:
    /// the engine's session decides that (`Session::changed_report`), not this command
    /// line (fdu-wb5n). Python's `Watch.report` does not take that rule; it always
    /// answers.
    fn render_live(
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        session: &mut fdu_core::watch_session::Session,
        format: report_format::Format,
        render: report_format::RenderOptions,
        diagnostic_color: bool,
        quiet: bool,
    ) -> anyhow::Result<()> {
        let generated_at = SystemTime::now();
        let Some(report) = session.changed_report(generated_at, format, render)? else {
            return Ok(());
        };
        // A watch run has no final answer and so no performance footer, which left text
        // repaints with nothing between them: the last row of one and the first row of
        // the next were adjacent lines. A blank line alone would not do, because that is
        // already what separates two views inside a single report.
        if matches!(format, report_format::Format::Text | report_format::Format::Tree) {
            writeln!(
                out,
                "\n{}",
                paint(&report_format::watch_rule(generated_at), STYLE_WATCH_RULE, render.color)
            )?;
        } else if format == report_format::Format::Yaml {
            write!(out, "{}", report_format::document_start(format))?;
        }
        report_format::write_with_options(&report, format, render, out)?;
        out.flush()?;
        write_report_diagnostics(diagnostic, &report, format, diagnostic_color, &[], quiet)?;
        Ok(())
    }

    /// Write the skill where agents look for it, and say what each file needed.
    ///
    /// One line per file on stdout, `installed`, `updated`, or `unchanged`, with the
    /// path relative when it is under the current directory. A `SKILL.md` fdu did not
    /// generate is a usage error, exit 2, and nothing is written; a filesystem failure
    /// is exit 1 like any other, after the lines for whatever was finished before it.
    fn run_install_skill(&self, out: &mut dyn Write) -> anyhow::Result<RunOutcome> {
        let cwd = std::env::current_dir()
            .map_err(|error| anyhow::anyhow!("cannot read the current directory: {error}"))?;
        self.install_skill_from(out, &cwd)
    }

    /// `run_install_skill` with the current directory passed in, so a test can install
    /// at project scope without changing the process's directory.
    fn install_skill_from(&self, out: &mut dyn Write, cwd: &Path) -> anyhow::Result<RunOutcome> {
        let targets = skill_install::targets(cwd, self.agent_base.as_deref());
        let report = |out: &mut dyn Write, outcomes: &[skill_install::Outcome]| {
            for outcome in outcomes {
                writeln!(
                    out,
                    "{} {}",
                    outcome.action,
                    skill_install::display_path(&outcome.path, cwd)
                )?;
            }
            io::Result::Ok(())
        };
        match skill_install::install(&compose_skill(), &targets) {
            Ok(outcomes) => {
                report(out, &outcomes)?;
                Ok(RunOutcome::Complete)
            }
            Err(skill_install::InstallError::Foreign(path)) => Err(usage(&anyhow::anyhow!(
                "refusing to overwrite {}: fdu did not generate it (no `{}` marker); move it aside, then re-run fdu --install-skill",
                skill_install::display_path(&path, cwd),
                skill_install::GENERATED_MARKER_PREFIX,
            ))),
            Err(skill_install::InstallError::Io { path, source, completed }) => {
                // What was finished before the failure is said before the failure is:
                // flushed here so the lines reach stdout before the error reaches stderr.
                // Best effort: a stdout that cannot take them (a closed pipe, a full disk)
                // must not replace the install error, or a failed install would exit 0
                // through the broken-pipe rule with nothing on stderr.
                let _ = report(out, &completed).and_then(|()| out.flush());
                Err(anyhow::Error::new(source).context(format!(
                    "cannot install the skill at {}",
                    skill_install::display_path(&path, cwd)
                )))
            }
        }
    }

    /// Run the cache lifecycle flags and report what they found or removed.
    fn run_cache_lifecycle(
        &self,
        out: &mut dyn Write,
        stdout_is_terminal: bool,
    ) -> anyhow::Result<RunOutcome> {
        // Lifecycle commands do not scan. With no PATH they retain their existing
        // current-root meaning so `--cache-status=all` and `--cache-clear=all` remain
        // useful discovery/maintenance actions without weakening report safety.
        let root = self.paths.first().map_or_else(|| Path::new("."), PathBuf::as_path);
        let cache_dir = fdu_core::default_cache_dir(self.cache_dir.as_deref())?;

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
                            report_format::human_count_u128(removed.snapshots as u128),
                            plural(removed.snapshots, "snapshot", "snapshots")
                        )?;
                    }
                    // Said separately because it is a different fact: these are fdu's own
                    // files, and none of them was a snapshot anyone could have used.
                    if removed.leftovers > 0 {
                        writeln!(
                            out,
                            "Also reclaimed: {} {} fdu left behind.",
                            report_format::human_count_u128(removed.leftovers as u128),
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
                            "Left in place: {} {}; fdu --cache-status=all lists {}.",
                            report_format::human_count_u128(left as u128),
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
                            "Left in place: {} staging {} another fdu may still be \
                             writing.",
                            report_format::human_count_u128(staging as u128),
                            plural(staging, "file", "files")
                        )?;
                    }
                }
                (CacheScope::Root, _) => {
                    let path = default_cache_path_in(root, self.cache_dir.as_deref())?;
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
                (CacheScope::Root, _) => {
                    match default_cache_path_in(root, self.cache_dir.as_deref())? {
                        Some(path) => vec![fdu_core::cache_status(&path)?],
                        None => Vec::new(),
                    }
                }
            };
            self.write_cache_status(out, &statuses, scope, stdout_is_terminal)?;
        }

        Ok(RunOutcome::Complete)
    }

    /// Render cache status through the format axis, like any other output.
    fn write_cache_status(
        &self,
        out: &mut dyn Write,
        statuses: &[fdu_core::CacheStatus],
        scope: CacheScope,
        stdout_is_terminal: bool,
    ) -> anyhow::Result<()> {
        let format = self.parse_format().map_err(|e| usage(&e))?;
        let color = ColorContext::from_environment(
            self.color,
            format.is_machine(),
            false,
            stdout_is_terminal,
        )
        .enabled();
        // Every format, human included, comes from the one renderer. While the CLI kept
        // the text layout to itself, no other caller could print what fdu prints.
        writeln!(
            out,
            "{}",
            report_format::render_cache_status_with_options(
                statuses,
                scope,
                format,
                report_format::RenderOptions { color, ..Default::default() },
            )
        )?;
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
                ignored: self.ignored.as_deref(),
                depth: self.depth.as_deref().or(self.full.then_some("all")),
                limit: self.limit.as_deref().or(self.full.then_some("all")),
                breadth: self.breadth.as_deref().or(self.full.then_some("all")),
                min_share: self.min_share.as_deref().or(self.full.then_some("0%")),
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
        self.request(self.paths.first().map_or(Path::new("."), PathBuf::as_path), SystemTime::now())
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
///
/// Over several roots the counts are summed, and the tier names each root's when they
/// differ, `cold scan (docs), cache only (src)`, since one tier would misdescribe a run
/// whose roots were served differently.
fn performance_footer(
    parts: &[PerformanceSummary],
    roots: Option<&[fdu_core::query::NamedRoot]>,
    ignore_rules: &ControlCoverage,
    total: Duration,
    size: SizeMetric,
    color: bool,
) -> String {
    let performance = PerformanceSummary::sum(parts);
    let mut tiers: Vec<(ReportSource, Vec<String>)> = Vec::new();
    for (position, part) in parts.iter().enumerate() {
        let label = roots
            .and_then(|roots| roots.get(position))
            .map(|root| root.label.display().to_string());
        match tiers.iter_mut().find(|(source, _)| *source == part.source) {
            Some((_, labels)) => labels.extend(label),
            None => tiers.push((part.source, label.into_iter().collect())),
        }
    }
    let tier = match tiers.as_slice() {
        [(source, _)] => performance_source(*source).to_owned(),
        tiers => tiers
            .iter()
            .map(|(source, labels)| {
                format!("{} ({})", performance_source(*source), labels.join(", "))
            })
            .collect::<Vec<_>>()
            .join(", "),
    };
    // The walked bytes in the answer's own metric: a sparse disk image is terabytes
    // apparent and megabytes allocated, and the line sits right under the answer.
    let walked_bytes = match size {
        SizeMetric::Apparent => performance.walked_bytes,
        SizeMetric::Allocated => performance.walked_allocated,
    };
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
            "{} cached ({})",
            human_count(performance.cached_files),
            performance_bytes(performance.cached_bytes, color)
        )
    };
    let read_rate = if performance.bytes_read == 0 || performance.analysis_ns == 0 {
        String::new()
    } else {
        format!(
            " at {}/s",
            performance_bytes(
                rate_per_second(performance.bytes_read, performance.analysis_ns,),
                color
            )
        )
    };
    let rules = match ignore_rules {
        ControlCoverage::NotObserved => "gitignore not read".to_string(),
        ControlCoverage::Observed(observed) => {
            let refused = if observed.refused > 0 {
                format!(", {} refused", human_count(observed.refused))
            } else {
                String::new()
            };
            format!(
                "{} gitignore {} ({} {}){refused}",
                human_count(observed.rules),
                plural_u64(observed.rules, "rule", "rules"),
                human_count(observed.applied.saturating_add(observed.refused)),
                plural_u64(observed.applied.saturating_add(observed.refused), "file", "files")
            )
        }
    };
    let rates = performance.total_throughput(total, size);
    format!(
        "perf: took {} to walk {} {} ({}) at {rates}; {rules}; content read {}{}; analysis {fresh}, {cached}; {tier}",
        human_duration(total),
        human_count(performance.walked_files),
        plural_u64(performance.walked_files, "file", "files"),
        performance_bytes(walked_bytes, color),
        performance_bytes(performance.bytes_read, color),
        read_rate,
    )
}

/// Keep the diagnostic frame gray after a styled size resets its local ANSI style.
/// This also makes a GiB-scale size bold gray while leaving smaller sizes gray.
fn performance_bytes(bytes: u64, color: bool) -> String {
    let styled = report_format::styled_bytes(bytes, 0, color, true);
    if color { format!("{styled}{STYLE_PERFORMANCE}") } else { styled }
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
    if elapsed_ns == 0 {
        return "0".to_string();
    }
    report_format::human_count_u128(u128::from(units) * 1_000_000_000 / u128::from(elapsed_ns))
}

/// Performance duration: adaptive units and two-decimal seconds.
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

/// Progress shows seconds to one decimal at every duration, using perf's rounding rule.
pub(crate) fn progress_duration(duration: Duration) -> String {
    format!("{} s", scaled_decimal(duration.as_nanos(), 1_000_000_000, 1))
}

/// Round an integer ratio to a fixed number of decimal places without losing precision.
pub(crate) fn scaled_decimal(value: u128, unit: u128, precision: u32) -> String {
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

/// Views to render, plus any `--view full` could not satisfy.
#[cfg(test)]
#[derive(Debug)]
struct ResolvedViews {
    selected: Vec<fdu_core::query::ViewSpec>,
    omitted: Vec<fdu_core::query::ViewSpec>,
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
    let (selected, omitted) = fdu_core::query::ViewSpec::resolve(spec, profile, "--view")
        .map_err(|message| anyhow::anyhow!(message))?;
    Ok(ResolvedViews { selected, omitted })
}

/// Output-design boundary: flush result stdout before emitting notes, warnings, and
/// deduplicated tips to stderr. The caller appends `perf:` last for human one-shot
/// reports. Machine stdout receives no diagnostics or terminal escapes.
///
/// Warnings come from two owners: the engine's [`report_format::report_warnings`], which
/// describe the answer itself (a `--stale-ok` answer nothing verified), then this
/// frontend's operational ones (a failed save, each retained status issue). `--quiet`
/// keeps every warning, on every format, and drops only notes, tips, and `perf:`.
///
/// Use stderr's color decision: note/tip/perf are gray, warn is yellow without bold,
/// and fatal rendering in `finish` is red bold. Preserve paths and causes on warning
/// lines; details and remedies follow the shared `report_epilogue` contract.
fn write_report_diagnostics(
    diagnostic: &mut dyn Write,
    report: &Report,
    format: report_format::Format,
    color: bool,
    save_warnings: &[String],
    quiet: bool,
) -> io::Result<()> {
    let lines = if matches!(format, report_format::Format::Paths | report_format::Format::Long) {
        report_format::flat_diagnostic_lines(report)
    } else {
        report_format::diagnostic_lines(report)
    };
    if !quiet {
        for line in lines.notes {
            writeln!(diagnostic, "{}", paint(&line, STYLE_PERFORMANCE, color))?;
        }
    }
    for warning in report_format::report_warnings(report) {
        writeln!(diagnostic, "{}", paint(&warning, STYLE_WARNING, color))?;
    }
    for warning in save_warnings {
        writeln!(diagnostic, "{}", paint(warning, STYLE_WARNING, color))?;
    }
    if !report.status.complete {
        for warning in status_warnings(&report.status, report.roots.as_deref()) {
            writeln!(diagnostic, "{}", paint(&warning, STYLE_WARNING, color))?;
        }
    }
    if !quiet {
        for line in lines.tips {
            writeln!(diagnostic, "{}", paint(&line, STYLE_PERFORMANCE, color))?;
        }
    }
    diagnostic.flush()
}

/// The warnings for an incomplete report, one per retained issue.
///
/// The retention bound keeps the first [`fdu_core::MAX_RETAINED_ISSUES`] details, and the
/// last line says how many it dropped, so the terminal is never told less than the
/// machine formats' `errors_omitted`. An issue whose message does not name its path (a
/// content read failure carries only the operating system's text) is prefixed with it.
///
/// Over several roots a path is printed after its root's label, as every text path is, so
/// `docs/a.md` and `src/a.md` stay apart; a message that already names its path names it
/// absolutely, which needs no label.
fn status_warnings(
    status: &fdu_core::query::TreeStatus,
    roots: Option<&[fdu_core::query::NamedRoot]>,
) -> Vec<String> {
    let mut warnings: Vec<String> = status
        .errors
        .iter()
        .map(|detail| {
            let issue = &detail.issue;
            match &issue.path {
                Some(path) if !path.as_os_str().is_empty() && !names_path(&issue.message, path) => {
                    let shown = fdu_core::query::labelled_path(roots, detail.root, path);
                    format!("warn: {}: {}", shown.display(), issue.message)
                }
                _ => format!("warn: {}", issue.message),
            }
        })
        .collect();
    if status.errors_omitted > 0 {
        warnings.push(format!(
            "warn: {} more {} omitted; details are kept for the first {}",
            human_count(status.errors_omitted),
            if status.errors_omitted == 1 { "error" } else { "errors" },
            human_count(fdu_core::MAX_RETAINED_ISSUES as u64),
        ));
    }
    warnings
}

/// Whether `message` already names `path`, as a whole path rather than as a substring.
///
/// A short relative path such as `d` occurs inside most operating-system messages
/// ("Permission denied"), so a match must end the path at a separator or the start of
/// the message on the left and at a delimiter or the end of the message on the right.
fn names_path(message: &str, path: &Path) -> bool {
    let path = path.to_string_lossy();
    message.match_indices(&*path).any(|(start, found)| {
        let before = message[..start].chars().next_back();
        let after = message[start + found.len()..].chars().next();
        matches!(before, None | Some('/' | '\\' | ' ' | '"' | '\'' | '`'))
            && matches!(after, None | Some(':' | ' ' | '"' | '\'' | '`' | ')' | ','))
    })
}

/// Run `fdu` through its real process boundary and return its stable numeric exit code.
///
/// This is shared by the native binary and the Python wheel's console entry point so
/// parsing, streams, color, diagnostics, broken pipes, and exit semantics cannot drift.
///
/// A run that draws the progress indicator installs a process-wide Ctrl-C handler and
/// never removes it: from then on, Ctrl-C erases a visible line and ends the process by
/// the default interrupt action, even if the caller had set the signal to be ignored.
/// A process that embeds this function and keeps running afterwards should call it
/// only where that is acceptable; both callers above exit when it returns.
pub fn run_process<I, T>(args: I) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let stdout = io::stdout();
    let stdout_is_terminal = stdout.is_terminal();
    // Everything the run will ever ask about stderr, read here and only here.
    let terminal = TerminalFacts::detect();
    let mut out = io::BufWriter::new(stdout.lock());
    // Not locked for the run: the progress ticker draws to stderr from its own thread,
    // and a lock held here would make its first frame wait for the process to end. Each
    // diagnostic is one `writeln!`, which stderr writes under its lock as one piece, so
    // a line is still never interleaved with a frame; the ticker's own protocol -- stop
    // and erase before any write -- is what keeps the two in order.
    let mut diagnostic = io::stderr();

    run_with_io(
        &args,
        &mut out,
        &mut diagnostic,
        stdout_is_terminal,
        &terminal,
        ProgressIo::for_process(),
    )
}

fn run_with_io(
    args: &[OsString],
    out: &mut dyn Write,
    diagnostic: &mut dyn Write,
    stdout_is_terminal: bool,
    terminal: &TerminalFacts,
    progress_io: ProgressIo,
) -> u8 {
    let stderr_is_terminal = terminal.stderr_is_terminal;
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
        cli.run(out, diagnostic, stdout_is_terminal, terminal, progress_io).and_then(|outcome| {
            out.flush()?;
            Ok(outcome)
        });
    let diagnostic_color =
        ColorContext::from_environment(cli.color, false, false, stderr_is_terminal).enabled();
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

pub(crate) fn paint(text: &str, style: AnsiStyle, color: bool) -> String {
    report_format::paint(text, style, color)
}

fn compose_skill() -> String {
    compose_skill_from(SKILL_TEMPLATE)
}

fn compose_skill_from(template: &str) -> String {
    // Git checkouts may translate the Markdown resource to CRLF on Windows. Keep the
    // public skill byte-stable across installation platforms before substituting the
    // version. It is the build version `--version` prints, dev revision and all: the
    // skill tells its reader to re-run `--install-skill` when the two differ, which is
    // only a usable rule if the stamp is the same string.
    template.replace("\r\n", "\n").replace("__FDU_VERSION__", env!("FDU_BUILD_VERSION"))
}

#[cfg(any(unix, windows))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress_ticker::{ERASE_LINE, SharedBuffer, Timing};
    use fdu_core::EntryKind;
    use fdu_core::query::ViewSpec;
    use fdu_core::query::{Bound, ScopeAxis, SizeMetric, SortKey};
    #[cfg(feature = "watch")]
    use std::time::UNIX_EPOCH;

    /// A warning names the path its message leaves out, and the count the retention
    /// bound dropped closes the list (fdu-peil).
    #[test]
    fn status_warnings_name_missing_paths_and_the_omitted_count() {
        let rooted = |root, path: Option<&str>, message: &str| fdu_core::query::StatusIssue {
            root,
            issue: fdu_core::Issue {
                kind: fdu_core::IssueKind::ProviderFailure,
                path: path.map(PathBuf::from),
                message: message.to_string(),
                os_error: None,
            },
        };
        let issue = |path: Option<&str>, message: &str| rooted(0, path, message);
        let status = |errors, errors_omitted| fdu_core::query::TreeStatus {
            complete: false,
            coverage: fdu_core::Coverage::Partial(fdu_core::CoverageReason::Failed),
            errors,
            errors_omitted,
        };
        let complete = status_warnings(
            &status(
                vec![
                    issue(Some("docs/a.md"), "Permission denied (os error 13)"),
                    issue(Some("src"), "I/O error at /abs/src: Permission denied (os error 13)"),
                    issue(None, "content analysis results became stale"),
                    issue(Some("d"), "Permission denied (os error 13)"),
                ],
                0,
            ),
            None,
        );
        assert_eq!(
            complete,
            [
                "warn: docs/a.md: Permission denied (os error 13)",
                "warn: I/O error at /abs/src: Permission denied (os error 13)",
                "warn: content analysis results became stale",
                "warn: d: Permission denied (os error 13)",
            ]
        );
        let one = status_warnings(&status(vec![issue(None, "x")], 1), None);
        assert_eq!(
            one.last().map(String::as_str),
            Some("warn: 1 more error omitted; details are kept for the first 64")
        );
        let many = status_warnings(&status(Vec::new(), 1_234), None);
        assert_eq!(many, ["warn: 1,234 more errors omitted; details are kept for the first 64"]);

        // Over several roots, a path the message leaves out follows its root's label.
        let roots = [
            fdu_core::query::NamedRoot { label: "docs".into(), path: "/abs/docs".into() },
            fdu_core::query::NamedRoot { label: "src".into(), path: "/abs/src".into() },
        ];
        let labelled = status_warnings(
            &status(
                vec![
                    rooted(0, Some("a.md"), "Permission denied (os error 13)"),
                    rooted(1, Some("a.md"), "Permission denied (os error 13)"),
                ],
                0,
            ),
            Some(&roots),
        );
        // Joined as every text path is, so with `\` on Windows.
        let shown = |label: &str| Path::new(label).join("a.md").display().to_string();
        assert_eq!(
            labelled,
            [
                format!("warn: {}: Permission denied (os error 13)", shown("docs")),
                format!("warn: {}: Permission denied (os error 13)", shown("src")),
            ]
        );
    }

    /// One population axis parses all modes and rejects misspellings.
    #[test]
    fn the_ignored_population_axis_accepts_modes_and_rejects_bad_values() {
        assert_eq!(
            cli().resolved_query().expect("parses").selection.ignored,
            IgnoredEntries::Include
        );
        let exclude =
            Cli { ignored: Some("exclude".into()), ..cli() }.resolved_query().expect("parses");
        assert_eq!(exclude.selection.ignored, IgnoredEntries::Exclude);
        let only = Cli { ignored: Some("only".into()), ..cli() }.resolved_query().expect("parses");
        assert_eq!(only.selection.ignored, IgnoredEntries::Only);
        assert_eq!(
            query_error(&Cli { ignored: Some("invalid".into()), ..cli() }),
            "invalid --ignored \"invalid\": expected one of include, exclude, only"
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
            "--ignored=only",
            "/nonexistent-root-that-must-not-be-scanned",
        ]
        .map(OsString::from);
        let status = run_with_io(
            &args,
            &mut out,
            &mut err,
            false,
            &TerminalFacts::default(),
            ProgressIo::inert(),
        );
        assert_eq!(status, 2);
        assert!(out.is_empty());
        assert_eq!(
            String::from_utf8(err).expect("UTF-8 diagnostics"),
            "fdu: --ignored=only needs .gitignore classification, and --no-gitignore turned it \
             off; drop one of them\n"
        );
    }

    #[test]
    fn the_performance_line_counts_the_ignore_rules_it_read_or_says_it_read_none() {
        use fdu_core::control::{ControlObservation, ControlRefusalReason, RefusedControl};

        let performance = PerformanceSummary {
            walked_files: 7,
            walked_bytes: 269,
            walked_allocated: 28_672,
            ..PerformanceSummary::default()
        };
        let footer = |rules: &ControlCoverage| {
            performance_footer(
                &[performance],
                None,
                rules,
                Duration::from_millis(3),
                SizeMetric::Apparent,
                false,
            )
        };
        assert_eq!(
            footer(&ControlCoverage::NotObserved),
            "perf: took 3.0 ms to walk 7 files (269 B) at 2,333 files/s (0.000 GiB/s); gitignore not read; content read 0 B; analysis 0 fresh, 0 cached; cold scan"
        );
        // The walked bytes are the answer's metric, allocated unless `--size apparent`.
        assert!(
            performance_footer(
                &[performance],
                None,
                &ControlCoverage::NotObserved,
                Duration::from_millis(3),
                SizeMetric::Allocated,
                false,
            )
            .starts_with("perf: took 3.0 ms to walk 7 files (28 KiB) at ")
        );
        let observed = |applied, refusals: Vec<RefusedControl>| {
            ControlCoverage::Observed(ControlObservation {
                limits: fdu_core::ControlLimits::default(),
                applied,
                rules: 0,
                refused: u64::try_from(refusals.len()).expect("a handful"),
                refusals,
            })
        };
        assert!(footer(&observed(1, Vec::new())).contains("; 0 gitignore rules (1 file); "));
        let refused = RefusedControl {
            root: 0,
            path: PathBuf::from(".gitignore"),
            reason: ControlRefusalReason::LineLimit,
        };
        assert!(
            footer(&observed(0, vec![refused]))
                .contains("; 0 gitignore rules (1 file), 1 refused; ")
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

    /// A stdout whose reader is gone, as after `| head`.
    struct ClosedStdout;

    impl Write for ClosedStdout {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
    }

    /// A CLI with every axis at its default, so a test can vary exactly one.
    fn cli() -> Cli {
        Cli {
            paths: vec![PathBuf::from(".")],
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
            ignored: None,
            // None, as clap now leaves it: the default belongs to the view.
            depth: None,
            limit: None,
            breadth: None,
            min_share: None,
            full: false,
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
            progress: ProgressMode::Auto,
            bar_size: 10,
            quiet: false,
            cache: "off".to_string(),
            stale_ok: false,
            cache_dir: None,
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
            install_skill: false,
            agent_base: None,
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
        let terminal = TerminalFacts::default();
        let status =
            run_with_io(&bare, &mut bare_out, &mut bare_err, false, &terminal, ProgressIo::inert());

        let mut help_out = Vec::new();
        let mut help_err = Vec::new();
        let help = [OsString::from("fdu"), OsString::from("--help")];
        let help_status =
            run_with_io(&help, &mut help_out, &mut help_err, false, &terminal, ProgressIo::inert());

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

    /// Run `args` with inert progress, returning the exit status, stdout, and stderr.
    fn run_args(args: &[&OsStr]) -> (u8, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let args: Vec<OsString> = args.iter().map(|arg| (*arg).to_owned()).collect();
        let status = run_with_io(
            &args,
            &mut out,
            &mut err,
            false,
            &TerminalFacts::default(),
            ProgressIo::inert(),
        );
        let text = |bytes| String::from_utf8(bytes).expect("UTF-8");
        (status, text(out), text(err))
    }

    #[test]
    fn several_paths_parse_in_argument_order() {
        assert_eq!(
            parse(&["fdu", "src", "docs", "."]).paths,
            ["src", "docs", "."].map(PathBuf::from)
        );
        assert_eq!(parse(&["fdu", "--view", "files", "a", "--depth", "1", "b"]).paths.len(), 2);
    }

    /// A watch and the cache lifecycle act on one root, so a second PATH is a usage error
    /// naming the limit, raised before anything is read.
    #[test]
    fn a_second_path_with_watch_cache_status_or_cache_clear_is_a_usage_error() {
        let root = tempfile::tempdir().expect("tempdir");
        let (a, b) = (root.path().join("a"), root.path().join("b"));
        for flags in [
            &["--cache-status"][..],
            &["--cache-clear"],
            #[cfg(feature = "watch")]
            &["--watch"],
        ] {
            let mut args: Vec<&OsStr> = vec![OsStr::new("fdu")];
            args.extend(flags.iter().map(OsStr::new));
            args.extend([a.as_os_str(), b.as_os_str()]);
            let (status, out, err) = run_args(&args);
            assert_eq!(status, 2, "{flags:?}: {err}");
            assert!(out.is_empty(), "{flags:?}");
            assert!(err.contains(&format!("{} takes one PATH; 2 were given", flags[0])), "{err}");
        }
    }

    /// Overlapping roots are a refused request, exit 2, naming both labels as given; a
    /// missing root fails as one root always has, exit 1.
    #[test]
    fn overlapping_and_missing_roots_fail_as_one_root_does() {
        let root = tempfile::tempdir().expect("tempdir");
        let outer = root.path().join("a");
        std::fs::create_dir_all(outer.join("b")).expect("a/b");
        let inner = outer.join("b");
        let (status, out, err) = run_args(&[
            OsStr::new("fdu"),
            OsStr::new("--cache"),
            OsStr::new("off"),
            outer.as_os_str(),
            inner.as_os_str(),
        ]);
        assert_eq!(status, 2, "{err}");
        assert!(out.is_empty());
        assert!(
            err.contains(&format!(
                "{} is inside {}; name one or the other",
                inner.display(),
                outer.display()
            )),
            "{err}"
        );
        let missing = root.path().join("missing");
        let (status, _, err) = run_args(&[
            OsStr::new("fdu"),
            OsStr::new("--cache"),
            OsStr::new("off"),
            outer.as_os_str(),
            missing.as_os_str(),
        ]);
        assert_eq!(status, 1, "{err}");
        assert!(err.contains(&format!("I/O error at {}", missing.display())), "{err}");
    }

    /// An overlap a walk finds is refused as one found before the walk is, exit 2 (review
    /// D1 on #192). On macOS `/private` is a firmlink onto the data volume, whose own path
    /// never runs through `/System/Volumes/Data`, so only the walk of the data volume, deep
    /// enough to enter `private`, finds it. Probed rather than assumed, so a host without
    /// the firmlink skips with a message.
    #[cfg(target_os = "macos")]
    #[test]
    fn an_overlap_the_walk_finds_exits_as_a_refused_request() {
        use std::os::unix::fs::MetadataExt;
        let identity = |path: &str| {
            std::fs::metadata(path).map(|metadata| (metadata.dev(), metadata.ino())).ok()
        };
        let (outer, inner) = ("/System/Volumes/Data", "/private");
        let reached = "/System/Volumes/Data/private";
        if identity(reached).is_none() || identity(reached) != identity(inner) {
            eprintln!("skipped: {inner} is not a firmlink into {outer} on this host");
            return;
        }
        let (status, out, err) =
            run_args(&["fdu", "--cache", "off", "--scan-depth", "2", outer, inner].map(OsStr::new));
        assert_eq!(status, 2, "{err}");
        assert!(out.is_empty(), "{out}");
        assert!(
            err.contains(&format!(
                "{inner} is inside {outer}, which reaches it as {reached}; name one or the other"
            )),
            "{err}"
        );
    }

    /// Several roots report as one: a total row, root rows by label, labelled paths.
    #[test]
    fn several_roots_print_a_total_and_labelled_paths() {
        let root = tempfile::tempdir().expect("tempdir");
        for (name, size) in [("a", 3_000), ("b", 1_000)] {
            std::fs::create_dir(root.path().join(name)).expect("root");
            std::fs::write(root.path().join(name).join("f.bin"), vec![b'.'; size]).expect("file");
        }
        let (a, b) = (root.path().join("a"), root.path().join("b"));
        let base = [OsStr::new("fdu"), OsStr::new("--cache"), OsStr::new("off"), OsStr::new("-q")];
        let mut tree: Vec<&OsStr> = base.to_vec();
        tree.extend([a.as_os_str(), b.as_os_str()]);
        let (status, out, err) = run_args(&tree);
        assert_eq!(status, 0, "{err}");
        assert!(out.lines().next().is_some_and(|line| line.contains("(total) 2 files")), "{out}");
        assert!(out.contains(&format!("{}/", a.display())), "{out}");
        let mut paths: Vec<&OsStr> = base.to_vec();
        paths.extend(["--view", "files", "--format", "paths", "--sort", "name"].map(OsStr::new));
        paths.extend([b.as_os_str(), a.as_os_str()]);
        let (status, out, _) = run_args(&paths);
        assert_eq!(status, 0);
        assert_eq!(
            out.lines().collect::<Vec<_>>(),
            [a.join("f.bin").display().to_string(), b.join("f.bin").display().to_string()]
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
    fn the_cache_policy_has_three_values_and_retired_ones_name_their_replacement() {
        let parse = |value: &str| Cli { cache: value.to_string(), ..cli() }.parse_cache_policy();
        assert_eq!(parse("on").expect("a policy"), CachePolicy::On);
        assert_eq!(
            parse("only").expect_err("retired").to_string(),
            "invalid --cache \"only\": answering from the snapshot alone is now --stale-ok"
        );
        assert_eq!(
            parse("readonly")
                .expect_err("an unreleased alias must not become a contract")
                .to_string(),
            "invalid --cache \"readonly\": expected one of auto, on, off"
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
    fn full_is_the_explicit_bounds_request_and_specific_flags_override_it() {
        let bounds = |query: fdu_core::query::Query| {
            (
                query.selection.depth,
                query.selection.breadth,
                query.selection.limit,
                query.selection.min_share,
            )
        };
        let shorthand = Cli::try_parse_from(["fdu", ".", "--full"]).expect("parse");
        let explicit = Cli::try_parse_from([
            "fdu",
            ".",
            "--depth=all",
            "--breadth=all",
            "--limit=all",
            "--min-share=0%",
        ])
        .expect("parse");
        assert_eq!(
            bounds(shorthand.resolved_query().expect("query")),
            bounds(explicit.resolved_query().expect("query"))
        );
        for args in [
            vec!["fdu", ".", "--full", "--depth=3", "--breadth=2", "--limit=8", "--min-share=2%"],
            vec!["fdu", ".", "--depth=3", "--breadth=2", "--limit=8", "--min-share=2%", "--full"],
        ] {
            let actual = Cli::try_parse_from(args).expect("parse").resolved_query().expect("query");
            let expected = Cli::try_parse_from([
                "fdu",
                ".",
                "--depth=3",
                "--breadth=2",
                "--limit=8",
                "--min-share=2%",
            ])
            .expect("parse")
            .resolved_query()
            .expect("query");
            assert_eq!(bounds(actual), bounds(expected));
        }
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
        assert_eq!(parsed.depth_for(ViewSpec::Tree), Bound::Limit(5));
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

    /// Every fdu flag the guide names must exist.
    ///
    /// tbd states this rule for its own docs surface as "the menu must only name
    /// selectors that exist", and it is worth a test rather than an intention: prose that
    /// advertises a flag the binary does not have is worse than prose that says nothing,
    /// because the reader spends their trust before finding out.
    #[test]
    fn the_guide_only_names_flags_that_exist() {
        let command = Cli::command();
        // The setup section also documents the two uv flags that make wheel-only
        // installation explicit. Keep that external vocabulary narrow and visible
        // here so a misspelled fdu flag still fails this test.
        let external_install_flags = ["--no-build", "--python"];
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
            if external_install_flags.contains(&flag.as_str()) {
                continue;
            }
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
    /// analyzer vocabulary the content axis no longer has. Both spellings count too:
    /// splitting on whitespace alone skipped every `--view=code,documents`.
    #[test]
    fn the_skill_only_names_views_and_analyzers_that_parse() {
        let skill = compose_skill();
        let spans = skill.split('`').filter(|span| span.starts_with("--"));
        let commands = skill.lines().map(str::trim_start).filter(|line| line.starts_with("fdu "));
        let mut checked = 0;
        for text in spans.chain(commands) {
            let mut words =
                text.split(|c: char| c.is_whitespace() || c == '=').filter(|word| !word.is_empty());
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

    /// Every command the skill shows must resolve as written, as the guide's must.
    ///
    /// An agent runs these lines verbatim, the `uvx` fallback included, so each line of
    /// a fenced `bash` block is parsed and its request built and validated, which is
    /// where every refusal that needs no filesystem happens. `<that>` stands for the
    /// timestamp the watermark example records.
    #[test]
    fn every_command_the_skill_shows_resolves() {
        let skill = compose_skill();
        let mut in_bash = false;
        let mut checked = 0;
        for line in skill.lines() {
            if let Some(fence) = line.strip_prefix("```") {
                in_bash = !in_bash && fence == "bash";
                continue;
            }
            let line = line.trim_start();
            let command = line
                .strip_prefix("uvx --no-build fdu@latest ")
                .or_else(|| line.strip_prefix("fdu "))
                .filter(|_| in_bash);
            let Some(command) = command else { continue };
            // A trailing comment says what the command is for.
            let command = command.split(" #").next().unwrap_or(command).trim_end();
            let args = std::iter::once("fdu".to_string())
                .chain(shell_words(command).into_iter().map(|word| word.replace("<that>", "@0")));
            let parsed = Cli::try_parse_from(args)
                .unwrap_or_else(|error| panic!("the skill shows `fdu {command}`: {error}"));
            if let Err(error) = parsed.resolved_request() {
                panic!("the skill shows `fdu {command}`: {error}");
            }
            checked += 1;
        }
        assert!(checked >= 30, "the skill should show its commands; found {checked}");
    }

    /// A command line split into words the way a POSIX shell splits the ones the skill
    /// shows: whitespace separates words, and single quotes keep a glob in one word.
    fn shell_words(command: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut word: Option<String> = None;
        let mut quoted = false;
        for c in command.chars() {
            match c {
                '\'' => {
                    quoted = !quoted;
                    word.get_or_insert_with(String::new);
                }
                c if c.is_whitespace() && !quoted => words.extend(word.take()),
                c => word.get_or_insert_with(String::new).push(c),
            }
        }
        words.extend(word);
        words
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
            (AnalysisSet::NONE.with_code(), ViewSpec::Code),
            (AnalysisSet::NONE.with_words(), ViewSpec::Documents),
            (AnalysisSet::ALL, ViewSpec::Code),
        ];
        for (profile, expected) in cases {
            assert_eq!(ViewSpec::default_for(profile), expected, "default view for {profile:?}");
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
        assert_eq!(
            bare.omitted,
            vec![ViewSpec::Code, ViewSpec::Documents],
            "content views need their analyzers"
        );
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
            ViewSpec::ALL
                .into_iter()
                .filter(|view| view.is_summary_view() && *view != ViewSpec::Languages)
                .collect::<Vec<_>>(),
            "every summary view, in table order"
        );

        // `all` names the axis, so combining it with a view is a usage error rather than
        // a silently-widened request.
        let combined = resolve_views(Some("full,tree"), AnalysisSet::NONE)
            .expect_err("full cannot be combined")
            .to_string();
        assert!(combined.contains("cannot be combined"), "{combined}");
    }

    /// Cost flows one way, the direction that protects the user: a view with a metadata
    /// meaning never causes a file body to be opened, and a content view opens exactly
    /// what its analyzer reads -- naming it is the request for that analysis, and the
    /// command line passes the flag through for the model to decide.
    #[test]
    fn only_a_content_view_enables_its_analyzer() {
        for view in ViewSpec::ALL {
            let spec = view.label();
            let cli = Cli { view: Some(spec.to_string()), ..cli() };
            let typed = cli.typed_values();
            let built = Request::build(
                &cli.spec(Path::new("."), &typed).expect("the spec composes"),
                SystemTime::now(),
                &AxisNames::FLAGS,
            )
            .expect("every view parses");
            let expected = match view {
                ViewSpec::Code => AnalysisSet::NONE.with_code(),
                ViewSpec::Documents => AnalysisSet::NONE.with_words(),
                _ => AnalysisSet::NONE,
            };
            assert_eq!(built.basis.content, expected, "--view {spec}");
            built.validate().expect("a view answers the basis it built");
        }
    }

    #[test]
    fn analysis_profile_workers_and_page_denominator_parse_before_io() {
        let parsed_cli = Cli::try_parse_from(["fdu", ".", "--workers=3"]).expect("worker flag");
        assert_eq!(parsed_cli.analysis_workers, 3);
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
            paths: vec![root.path().to_path_buf()],
            analyze: "lines,words".to_string(),
            view: Some("documents".to_string()),
            format: "json".to_string(),
            size: "apparent".to_string(),
            ..cli()
        };
        let mut output = Vec::new();
        let outcome = command
            .run(
                &mut output,
                &mut Vec::new(),
                false,
                &TerminalFacts::default(),
                ProgressIo::inert(),
            )
            .expect("run content report");
        assert_eq!(outcome, RunOutcome::Complete);
        let output = String::from_utf8(output).expect("UTF-8 JSON");
        assert!(output.contains("\"schema\": \"fdu.report/11\""), "{output}");
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
            paths: vec![root.path().to_path_buf()],
            analyze: "lines".to_string(),
            view: Some("summary".to_string()),
            size: "apparent".to_string(),
            ..cli()
        };

        let (mut plain, mut diagnostics) = (Vec::new(), Vec::new());
        command
            .run(
                &mut plain,
                &mut diagnostics,
                false,
                &TerminalFacts::default(),
                ProgressIo::inert(),
            )
            .expect("plain report");
        let plain = String::from_utf8(plain).expect("plain UTF-8");
        let diagnostics = String::from_utf8(diagnostics).expect("plain diagnostics");
        assert!(!plain.contains("perf:"), "the answer stays on stdout: {plain}");
        assert!(!plain.contains("note:"), "facts stay on stderr: {plain}");
        assert!(!plain.contains("tip:"), "suggestions stay on stderr: {plain}");
        let lines: Vec<&str> = diagnostics.lines().collect();
        let note = lines.iter().position(|line| line.starts_with("note:")).expect("display fact");
        let tip =
            lines.iter().position(|line| line.starts_with("tip:")).expect("display suggestion");
        assert!(note < tip && tip < lines.len() - 1, "diagnostic category order: {diagnostics}");
        let footer = diagnostics.lines().last().expect("performance footer");
        assert!(footer.starts_with("perf: took "), "{plain}");
        assert!(footer.contains("2 fresh at "), "{footer}");
        assert!(footer.contains("0 cached"), "{footer}");
        assert!(footer.ends_with("cold scan"), "{footer}");
        assert!(!footer.contains('\u{1b}'), "color-disabled output must not contain ANSI");

        let (mut colored, mut colored_diagnostics) = (Vec::new(), Vec::new());
        Cli { color: ColorWhen::Always, ..command }
            .run(
                &mut colored,
                &mut colored_diagnostics,
                false,
                &TerminalFacts::default(),
                ProgressIo::inert(),
            )
            .expect("colored report");
        let colored = String::from_utf8(colored).expect("colored UTF-8");
        let colored_diagnostics =
            String::from_utf8(colored_diagnostics).expect("colored diagnostics");
        assert!(!colored.contains("perf:"), "the answer stays on stdout: {colored}");
        for prefix in ["note:", "tip:"] {
            let line =
                colored_diagnostics.lines().find(|line| line.contains(prefix)).expect(prefix);
            assert!(line.starts_with("\u{1b}[90m"), "gray {prefix}: {colored_diagnostics:?}");
        }
        let footer = colored_diagnostics.lines().last().expect("colored performance footer");
        assert!(
            footer.starts_with("\u{1b}[90mperf:"),
            "the footer must use terminal gray when color is active: {colored:?}"
        );
    }

    #[test]
    fn quiet_preserves_the_answer_and_suppresses_notes_tips_and_perf() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("one.txt"), b"one\n").expect("write");
        let command = |quiet| Cli {
            paths: vec![root.path().to_path_buf()],
            analyze: "lines".to_string(),
            view: Some("summary".to_string()),
            size: "apparent".to_string(),
            quiet,
            ..cli()
        };
        let run = |quiet| {
            let (mut out, mut diagnostic) = (Vec::new(), Vec::new());
            command(quiet)
                .run(
                    &mut out,
                    &mut diagnostic,
                    false,
                    &TerminalFacts::default(),
                    ProgressIo::inert(),
                )
                .expect("report");
            (out, diagnostic)
        };
        let (normal_out, normal_diagnostic) = run(false);
        let (quiet_out, quiet_diagnostic) = run(true);
        assert_eq!(normal_out, quiet_out, "quiet leaves result stdout intact");
        assert!(String::from_utf8_lossy(&normal_diagnostic).contains("note:"));
        assert!(String::from_utf8_lossy(&normal_diagnostic).contains("tip:"));
        assert!(String::from_utf8_lossy(&normal_diagnostic).contains("perf:"));
        assert!(quiet_diagnostic.is_empty(), "quiet diagnostic: {quiet_diagnostic:?}");

        let (mut json_out, mut json_diagnostic) = (Vec::new(), Vec::new());
        Cli {
            format: "json".to_string(),
            bar_size: 5_000, // Machine output ignores visual bar bounds.
            ..command(true)
        }
        .run(
            &mut json_out,
            &mut json_diagnostic,
            false,
            &TerminalFacts::default(),
            ProgressIo::inert(),
        )
        .expect("quiet JSON report");
        assert!(json_diagnostic.is_empty());
        let json = String::from_utf8(json_out).expect("UTF-8 JSON");
        assert!(json.trim().starts_with('{') && json.trim().ends_with('}'));
        assert!(json.contains("\"schema\": \"fdu.report/11\""));
        assert!(json.contains("\"view\": \"summary\""));
    }

    #[test]
    fn quiet_keeps_warnings_on_the_diagnostic_stream() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("one.txt"), b"one\n").expect("write");
        let command = Cli { paths: vec![root.path().to_path_buf()], quiet: true, ..cli() };
        let request = command.request(root.path(), SystemTime::now()).expect("request");
        let (report, pending_save, _) =
            fdu_core::prepare_report(&request, &Delivery::new(CachePolicy::Off, None))
                .expect("report");
        pending_save.join().expect("no cache save");
        let mut diagnostic = Vec::new();
        write_report_diagnostics(
            &mut diagnostic,
            &report,
            report_format::Format::Tree,
            false,
            &["warn: snapshot could not be saved".to_owned()],
            true,
        )
        .expect("diagnostics");
        assert_eq!(
            String::from_utf8(diagnostic).expect("UTF-8"),
            "warn: snapshot could not be saved\n"
        );
    }

    /// A `--stale-ok` answer is marked on stderr by a warning that `--quiet` keeps, on the
    /// plain text report and every other format, and a verified answer carries no such
    /// line. Before this, plain text said only `cache only` at the end of the `perf:`
    /// footer, and `--quiet` removed that too (fdu-mdop).
    #[test]
    fn quiet_keeps_the_stale_answer_warning_and_a_verified_answer_has_none() {
        const WARNING: &str = "warn: stale answer: served from the snapshot without filesystem \
                               verification; drop --stale-ok for a fresh answer\n";
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("one.txt"), b"one\n").expect("write");
        let cache = tempfile::tempdir().expect("cache dir");
        let command = |view: &str, format: &str, cache_policy: &str, stale_ok, quiet| Cli {
            paths: vec![root.path().to_path_buf()],
            view: Some(view.to_string()),
            format: format.to_string(),
            size: "apparent".to_string(),
            cache: cache_policy.to_string(),
            cache_dir: Some(cache.path().to_path_buf()),
            stale_ok,
            quiet,
            ..cli()
        };
        let run = |command: Cli| {
            let (mut out, mut diagnostic) = (Vec::new(), Vec::new());
            command
                .run(
                    &mut out,
                    &mut diagnostic,
                    false,
                    &TerminalFacts::default(),
                    ProgressIo::inert(),
                )
                .expect("report");
            (
                String::from_utf8(out).expect("UTF-8 stdout"),
                String::from_utf8(diagnostic).expect("UTF-8 stderr"),
            )
        };

        // `on` leaves the snapshot a stale answer reads; its own answer is verified.
        let (_, seeded) = run(command("tree", "text", "on", false, false));
        assert!(!seeded.contains("warn:"), "a cold scan is not stale: {seeded}");
        let (fresh_out, fresh) = run(command("summary", "text", "auto", false, false));
        assert!(!fresh.contains("warn:"), "a verified answer is not stale: {fresh}");
        let (_, fresh_quiet) = run(command("summary", "text", "auto", false, true));
        assert!(fresh_quiet.is_empty(), "nothing to keep: {fresh_quiet:?}");

        let (stale_out, stale) = run(command("summary", "text", "auto", true, false));
        assert_eq!(stale_out, fresh_out, "an unchanged tree gives the same answer");
        let warning = stale.find(WARNING).unwrap_or_else(|| panic!("stale marker: {stale}"));
        let perf = stale.find("perf:").expect("a text report closes with perf:");
        assert!(warning < perf, "warnings precede the performance footer: {stale}");
        assert!(stale.trim_end().ends_with("cache only"), "{stale}");

        let (quiet_out, quiet) = run(command("summary", "text", "auto", true, true));
        assert_eq!(quiet_out, stale_out, "quiet leaves result stdout intact");
        assert_eq!(quiet, WARNING, "quiet keeps the stale marker and nothing else");
        for (view, format) in [("summary", "json"), ("list", "paths")] {
            let (_, quiet) = run(command(view, format, "auto", true, true));
            assert_eq!(quiet, WARNING, "{format} keeps the same marker under quiet");
            let (_, fresh_quiet) = run(command(view, format, "auto", false, true));
            assert!(!fresh_quiet.contains("warn:"), "{format}: {fresh_quiet}");
        }
    }

    /// The footer's walked size is measured as the answer is, so it reads as the summary
    /// row's own total under the default and under `--size apparent` alike. (On Windows
    /// allocated falls back to apparent, and the two readings agree.)
    #[test]
    fn the_performance_footer_measures_walked_bytes_as_the_answer_does() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("one.txt"), b"one\n").expect("write");
        std::fs::write(root.path().join("two.txt"), b"two\n").expect("write");
        for size in [SIZE_DEFAULT, "apparent"] {
            let command = Cli {
                paths: vec![root.path().to_path_buf()],
                view: Some("summary".to_string()),
                size: size.to_string(),
                ..cli()
            };
            let (mut out, mut diagnostic) = (Vec::new(), Vec::new());
            command
                .run(
                    &mut out,
                    &mut diagnostic,
                    false,
                    &TerminalFacts::default(),
                    ProgressIo::inert(),
                )
                .expect("summary report");
            let out = String::from_utf8(out).expect("UTF-8");
            let diagnostic = String::from_utf8(diagnostic).expect("UTF-8 diagnostics");
            let row = out.lines().next().expect("the summary row").trim_start();
            let answer = row.split("  ").next().expect("the row's size");
            let footer = diagnostic.lines().last().expect("the footer");
            assert!(
                footer.contains(&format!("to walk 2 files ({answer}) at ")),
                "--size {size}: {out}"
            );
        }
    }

    #[test]
    fn performance_footer_names_units_cache_work_and_metadata_tier() {
        let footer = performance_footer(
            &[PerformanceSummary {
                walked_files: 12_345,
                walked_bytes: 2_048,
                walked_allocated: 50_565_120,
                fresh_files: 3_000,
                bytes_read: 2_048,
                analysis_ns: 2_000_000_000,
                cached_files: 2,
                cached_bytes: 4_096,
                source: ReportSource::WarmRevalidate,
            }],
            None,
            &ControlCoverage::NotObserved,
            Duration::from_millis(2_500),
            SizeMetric::Apparent,
            false,
        );

        assert_eq!(
            footer,
            "perf: took 2.50 s to walk 12,345 files (2.0 KiB) at 4,938 files/s (0.000 GiB/s); gitignore not read; content read 2.0 KiB at 1.0 KiB/s; analysis 3,000 fresh at 1,500 files/s, 2 cached (4.0 KiB); warm revalidation"
        );
    }

    /// Over several roots the line sums the walk and names each root's tier when they
    /// differ; when they agree, it names the one tier.
    #[test]
    fn perf_line_sums_and_names_each_tier_when_roots_differ() {
        let part = |files, source| PerformanceSummary {
            walked_files: files,
            walked_bytes: files * 100,
            source,
            ..PerformanceSummary::default()
        };
        let roots = [
            fdu_core::query::NamedRoot { label: "docs".into(), path: "/abs/docs".into() },
            fdu_core::query::NamedRoot { label: "src".into(), path: "/abs/src".into() },
            fdu_core::query::NamedRoot { label: "tests".into(), path: "/abs/tests".into() },
        ];
        let footer = |parts: &[PerformanceSummary]| {
            performance_footer(
                parts,
                Some(&roots),
                &ControlCoverage::NotObserved,
                Duration::from_secs(1),
                SizeMetric::Apparent,
                false,
            )
        };
        let mixed = footer(&[
            part(2, ReportSource::ColdScan),
            part(3, ReportSource::CacheOnly),
            part(4, ReportSource::ColdScan),
        ]);
        assert!(mixed.starts_with("perf: took 1.00 s to walk 9 files (900 B) at "), "{mixed}");
        assert!(mixed.ends_with("; cold scan (docs, tests), cache only (src)"), "{mixed}");
        let agreed = footer(&[
            part(2, ReportSource::ColdScan),
            part(3, ReportSource::ColdScan),
            part(4, ReportSource::ColdScan),
        ]);
        assert!(agreed.ends_with("; cold scan"), "{agreed}");
    }

    #[test]
    fn large_performance_sizes_are_bold_gray_and_restore_gray_afterward() {
        let performance = PerformanceSummary {
            walked_files: 1,
            walked_bytes: 1 << 30,
            fresh_files: 1,
            bytes_read: 1 << 30,
            analysis_ns: 1_000_000_000,
            cached_files: 1,
            cached_bytes: 1 << 30,
            ..PerformanceSummary::default()
        };
        let plain = performance_footer(
            &[performance],
            None,
            &ControlCoverage::NotObserved,
            Duration::from_secs(1),
            SizeMetric::Apparent,
            false,
        );
        assert!(!plain.contains('\u{1b}'));
        assert!(plain.contains("to walk 1 file (1.0 GiB)"), "{plain}");
        assert!(plain.contains("1 cached (1.0 GiB)"), "{plain}");

        let colored = paint(
            &performance_footer(
                &[performance],
                None,
                &ControlCoverage::NotObserved,
                Duration::from_secs(1),
                SizeMetric::Apparent,
                true,
            ),
            STYLE_PERFORMANCE,
            true,
        );
        let bold_gray_size = report_format::styled_bytes(1 << 30, 0, true, true);
        assert!(bold_gray_size.contains('\u{1b}'), "{bold_gray_size:?}");
        assert_eq!(
            colored.matches(&format!("{bold_gray_size}{STYLE_PERFORMANCE}")).count(),
            4,
            "walked, read, read-rate, and cached sizes each restore gray: {colored:?}"
        );
    }

    #[test]
    fn machine_formats_omit_the_performance_footer() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("one.txt"), b"one\n").expect("write");
        for format in ["json", "jsonl", "yaml"] {
            let command = Cli {
                paths: vec![root.path().to_path_buf()],
                view: Some("summary".to_string()),
                size: "apparent".to_string(),
                format: format.to_string(),
                ..cli()
            };
            let (mut output, mut diagnostic) = (Vec::new(), Vec::new());
            command
                .run(
                    &mut output,
                    &mut diagnostic,
                    false,
                    &TerminalFacts::default(),
                    ProgressIo::inert(),
                )
                .expect("machine report");
            let output = String::from_utf8(output).expect("machine UTF-8");
            let diagnostic = String::from_utf8(diagnostic).expect("machine diagnostics");
            assert!(!output.contains("perf:"), "{format}: {output}");
            assert!(!diagnostic.contains("perf:"), "{format}: {diagnostic}");
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
            paths: vec![PathBuf::from("/nonexistent-root-that-should-not-be-scanned")],
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
            "note: full omits code, documents without analysis",
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

    /// The runner rule is the user's decision of 2026-09-24: an installed `fdu` first,
    /// else `uvx --no-build fdu@latest`, and the stamp is the build version so "re-run when
    /// `--version` differs" compares like with like. It replaced an exact pin, which
    /// a dev build could never satisfy.
    #[test]
    fn portable_skill_prefers_the_installed_command_and_names_its_own_build() {
        let skill = compose_skill();

        assert!(skill.starts_with("---\nname: fdu\n"));
        assert!(!skill.contains('\r'), "the public skill must use portable LF endings");
        assert!(skill.contains("complete fdu usage contract"));
        assert!(skill.contains("needs no setup chat or prior session"));
        assert!(skill.contains("command -v fdu"), "the installed command comes first");
        assert!(
            skill.contains("uvx --no-build fdu@latest "),
            "the fallback is the latest release and requires a wheel"
        );
        assert!(!skill.contains("--from fdu=="), "no exact pin is left in the skill");
        assert!(skill.contains("uv tool install --no-build fdu"));
        assert!(skill.contains("uv tool upgrade --no-build fdu"));
        assert!(skill.contains("cargo install --locked fdu"));
        assert!(skill.contains(&format!("Generated by fdu `{}`", env!("FDU_BUILD_VERSION"))));
        assert!(!skill.contains("__FDU_VERSION__"));

        // The generated-by marker sits right after the frontmatter, so it neither
        // disturbs the frontmatter nor gets lost among the body's comments.
        let frontmatter_end = skill[4..].find("\n---\n").expect("frontmatter closes") + 4 + 5;
        assert!(
            skill[frontmatter_end..].starts_with("<!-- generated by fdu"),
            "the marker follows the frontmatter"
        );
        assert_eq!(
            compose_skill_from("---\r\nversion: __FDU_VERSION__\r\n"),
            format!("---\nversion: {}\n", env!("FDU_BUILD_VERSION"))
        );
    }

    /// `--install-skill` through the process boundary: the reported lines, the exit
    /// codes, and the refusal, against a user-scope base so no test changes directory.
    #[test]
    fn install_skill_reports_each_file_and_refuses_a_foreign_one_with_exit_two() {
        let sandbox = tempfile::tempdir().expect("tempdir");
        let base = sandbox.path().join(".claude");
        let skill = base.join("skills").join("fdu").join("SKILL.md");
        let run = || {
            let mut out = Vec::new();
            let mut err = Vec::new();
            let args = [
                OsString::from("fdu"),
                OsString::from("--install-skill"),
                OsString::from("--agent-base"),
                base.clone().into_os_string(),
            ];
            let status = run_with_io(
                &args,
                &mut out,
                &mut err,
                false,
                &TerminalFacts::default(),
                ProgressIo::inert(),
            );
            (status, String::from_utf8(out).expect("utf-8"), String::from_utf8(err).expect("utf-8"))
        };

        let (status, out, err) = run();
        assert_eq!(status, 0);
        assert_eq!(out, format!("installed {}\n", skill.display()));
        assert!(err.is_empty(), "{err}");
        assert_eq!(std::fs::read_to_string(&skill).expect("read"), compose_skill());

        let (status, out, err) = run();
        assert_eq!(status, 0);
        assert_eq!(out, format!("unchanged {}\n", skill.display()));
        assert!(err.is_empty(), "{err}");

        std::fs::write(&skill, "---\nname: fdu\n---\n# Written by hand\n").expect("write foreign");
        let (status, out, err) = run();
        assert_eq!(status, 2, "a foreign file is refused as a usage error");
        assert!(out.is_empty(), "nothing is reported installed: {out}");
        assert_eq!(
            err,
            format!(
                "fdu: refusing to overwrite {}: fdu did not generate it (no `<!-- generated by fdu` marker); move it aside, then re-run fdu --install-skill\n",
                skill.display()
            )
        );
        assert_eq!(
            std::fs::read_to_string(&skill).expect("read"),
            "---\nname: fdu\n---\n# Written by hand\n",
            "the foreign file is untouched"
        );

        // `--agent-base` is meaningless without `--install-skill`, and says so.
        let mut out = Vec::new();
        let mut err = Vec::new();
        let args = ["fdu", "--agent-base", "x", "."].map(OsString::from);
        let status = run_with_io(
            &args,
            &mut out,
            &mut err,
            false,
            &TerminalFacts::default(),
            ProgressIo::inert(),
        );
        assert_eq!(status, 2);
        assert!(String::from_utf8(err).expect("utf-8").contains("--install-skill"));
    }

    /// A write that fails partway is exit 1 after the lines for what was finished, so
    /// a partial install is never reported as nothing. The failure is the staged
    /// sibling being a directory, which fails the write on every platform. Each sandbox
    /// is its own project root, so a temporary directory inside a checkout cannot send
    /// the install to that checkout.
    #[test]
    fn install_skill_says_what_it_installed_before_a_write_fails() {
        let sandbox = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(sandbox.path().join(".git")).expect("mark the project root");

        // Project scope, through the seam that takes the directory instead of reading
        // it from the process: the first target's line precedes the second's failure.
        let targets = skill_install::targets(sandbox.path(), None);
        let staged = skill_install::staged_path(targets[1].parent().expect("parent"));
        std::fs::create_dir_all(&staged).expect("block the second target's staged path");
        let mut out = Vec::new();
        let error = parse(&["fdu", "--install-skill"])
            .install_skill_from(&mut out, sandbox.path())
            .expect_err("the second write fails");
        assert_eq!(
            String::from_utf8(out).expect("utf-8"),
            "installed .agents/skills/fdu/SKILL.md\n"
        );
        assert!(!is_usage_error(&error), "a write failure is not a usage error");
        assert_eq!(error.to_string(), "cannot install the skill at .claude/skills/fdu/SKILL.md");

        // A stdout that cannot take the report (its reader gone, as after `| head`) must
        // not replace the install error: the broken-pipe rule would turn it into exit 0
        // with nothing on stderr.
        let rerun = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(rerun.path().join(".git")).expect("mark the project root");
        let targets = skill_install::targets(rerun.path(), None);
        std::fs::create_dir_all(skill_install::staged_path(targets[1].parent().expect("parent")))
            .expect("block the second target's staged path");
        let error = parse(&["fdu", "--install-skill"])
            .install_skill_from(&mut ClosedStdout, rerun.path())
            .expect_err("the second write fails");
        assert_eq!(
            error.to_string(),
            "cannot install the skill at .claude/skills/fdu/SKILL.md",
            "the install error survives a closed stdout"
        );
        // And nothing in its chain is a broken pipe, which `finish` would turn into a
        // silent exit 0 even with the headline intact.
        let mut diagnostic = Vec::new();
        assert_eq!(finish(Err(error), &mut diagnostic, false), 1, "exit 1, not the broken-pipe 0");
        assert!(
            String::from_utf8(diagnostic)
                .expect("utf-8")
                .starts_with("fdu: cannot install the skill at .claude/skills/fdu/SKILL.md\n")
        );

        // User scope, through the process boundary: exit 1 and the same headline.
        let base = sandbox.path().join("home");
        let skill_dir = base.join("skills").join("fdu");
        std::fs::create_dir_all(skill_install::staged_path(&skill_dir))
            .expect("block the staged path");
        let mut out = Vec::new();
        let mut err = Vec::new();
        let args = [
            OsString::from("fdu"),
            OsString::from("--install-skill"),
            OsString::from("--agent-base"),
            base.into_os_string(),
        ];
        let status = run_with_io(
            &args,
            &mut out,
            &mut err,
            false,
            &TerminalFacts::default(),
            ProgressIo::inert(),
        );
        assert_eq!(status, 1);
        assert!(out.is_empty(), "nothing was installed, so nothing is reported: {out:?}");
        let err = String::from_utf8(err).expect("utf-8");
        let headline =
            format!("fdu: cannot install the skill at {}\n", skill_dir.join("SKILL.md").display());
        assert!(err.starts_with(&headline), "{err}");
        assert!(!skill_dir.join("SKILL.md").exists(), "the failed target never became visible");
    }

    #[test]
    fn machine_result_format_does_not_disable_fatal_stderr_color() {
        let temp = tempfile::tempdir().expect("tempdir");
        for (choice, colored) in [("always", true), ("never", false)] {
            let args = [
                OsString::from("fdu"),
                temp.path().join("missing").into_os_string(),
                OsString::from("--format=json"),
                OsString::from(format!("--color={choice}")),
            ];
            let (mut out, mut err) = (Vec::new(), Vec::new());
            let status = run_with_io(
                &args,
                &mut out,
                &mut err,
                false,
                &TerminalFacts::default(),
                ProgressIo::inert(),
            );
            assert_eq!(status, 1);
            assert!(out.is_empty());
            assert_eq!(err.contains(&0x1b), colored);
        }
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
            run_with_io(
                &args,
                &mut FailingWriter,
                &mut diagnostic,
                false,
                &TerminalFacts::default(),
                ProgressIo::inert()
            ),
            1,
            "a non-pipe help-output failure is fatal"
        );
    }

    /// The terminal a person is watching, as the gating tests describe one.
    fn interactive_terminal() -> TerminalFacts {
        TerminalFacts {
            stderr_is_terminal: true,
            term: Some(OsString::from("xterm-256color")),
            term_may_be_unset: false,
            ci: None,
            vt_enabled: true,
        }
    }

    #[test]
    fn warnings_are_yellow_without_bold_and_fatal_errors_are_red_bold() {
        let sgr = |text: &str| -> Vec<String> {
            text.split("\u{1b}[")
                .skip(1)
                .filter_map(|part| part.split_once('m').map(|(code, _)| code))
                .flat_map(|code| code.split(';').map(str::to_string))
                .collect()
        };
        let warning = paint("warn: incomplete", STYLE_WARNING, true);
        let warning_codes = sgr(&warning);
        assert!(warning_codes.iter().any(|code| code == "33"), "yellow warning: {warning:?}");
        assert!(!warning_codes.iter().any(|code| code == "1"), "warning is not bold: {warning:?}");

        let mut diagnostic = Vec::new();
        assert_eq!(finish(Err(anyhow::anyhow!("failed")), &mut diagnostic, true), 1);
        let fatal = String::from_utf8(diagnostic).expect("UTF-8 diagnostic");
        let error_codes = sgr(&fatal);
        assert!(error_codes.iter().any(|code| code == "31"), "red error: {fatal:?}");
        assert!(error_codes.iter().any(|code| code == "1"), "bold error: {fatal:?}");
    }

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("the command line parses")
    }

    #[test]
    fn quiet_and_tree_bar_size_flags_parse_with_their_defaults() {
        assert!(!parse(&["fdu", "."]).quiet);
        assert!(parse(&["fdu", "--quiet", "."]).quiet);
        assert!(parse(&["fdu", "-q", "."]).quiet);
        assert_eq!(parse(&["fdu", "."]).bar_size(), 10);
        assert_eq!(parse(&["fdu", "--bar-size", "20", "."]).bar_size(), 20);
        assert_eq!(parse(&["fdu", "--bar-size=0", "."]).bar_size(), 0);
        assert_eq!(parse(&["fdu", "--bar-size", "-1", "."]).bar_size(), 0);
    }

    #[test]
    fn the_progress_flag_parses_like_color_and_defaults_to_auto() {
        assert_eq!(parse(&["fdu", "."]).progress, ProgressMode::Auto);
        assert_eq!(parse(&["fdu", "--progress", "always", "."]).progress, ProgressMode::Always);
        assert_eq!(parse(&["fdu", "--progress=never", "."]).progress, ProgressMode::Never);
        assert_eq!(parse(&["fdu", "--progress", "auto", "."]).progress, ProgressMode::Auto);

        let mut out = Vec::new();
        let mut err = Vec::new();
        let args = ["fdu", "--progress", "sometimes", "."].map(OsString::from);
        let status = run_with_io(
            &args,
            &mut out,
            &mut err,
            false,
            &TerminalFacts::default(),
            ProgressIo::inert(),
        );
        assert_eq!(status, 2, "a value outside the grammar is a usage error");
        assert!(out.is_empty());
        let err = String::from_utf8(err).expect("UTF-8 diagnostics");
        assert!(err.contains("invalid value 'sometimes' for '--progress <WHEN>'"), "{err}");
    }

    #[test]
    fn help_places_progress_beside_color_and_says_always_never_reaches_a_pipe() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let args = [OsString::from("fdu"), OsString::from("--help")];
        run_with_io(
            &args,
            &mut out,
            &mut err,
            false,
            &TerminalFacts::default(),
            ProgressIo::inert(),
        );
        let help = String::from_utf8(out).expect("help is UTF-8");
        let lines: Vec<&str> = help.lines().collect();
        let color = lines
            .iter()
            .position(|line| line.trim_start().starts_with("--color <WHEN>"))
            .expect("--color is in the help");
        assert!(
            lines[color + 1].trim_start().starts_with("--progress <WHEN>"),
            "--progress follows --color:\n{help}"
        );
        let output = help.split("OUTPUT\n").nth(1).expect("an OUTPUT section");
        let output = output.split("\n\n").next().expect("the section ends at a blank line");
        assert!(output.contains("--progress <WHEN>"), "{output}");
        // Read across clap's wrapping, which is the terminal's business, not the text's.
        let progress = output.split("--progress").nth(1).expect("the flag's help");
        let progress = progress.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(progress.contains("unlike --color, never draws into a pipe or file"), "{progress}");
        assert!(progress.contains("[default: auto]"), "{progress}");
        assert!(output.contains("--bar-size <N>"), "{output}");
        assert!(output.contains("-q, --quiet"), "{output}");
    }

    /// The gating decision, resolved from the flag, the terminal, the format, and the
    /// command, for the ticker to consume.
    #[test]
    fn the_progress_plan_draws_only_for_a_walking_run_at_an_interactive_terminal() {
        let interactive = interactive_terminal();
        let plan = progress_plan_of;

        assert!(plan(&["fdu", "."], &interactive).draw);
        assert!(plan(&["fdu", "--tree", "."], &interactive).draw);
        assert!(plan(&["fdu", "--long", "."], &interactive).draw);
        assert!(plan(&["fdu", "--format", "paths", "."], &interactive).draw);
        assert!(!plan(&["fdu", "--format", "json", "."], &interactive).draw);
        assert!(!plan(&["fdu", "--format", "jsonl", "."], &interactive).draw);
        assert!(!plan(&["fdu", "--format", "yaml", "."], &interactive).draw);
        assert!(plan(&["fdu", "--progress", "always", "--format", "json", "."], &interactive).draw);
        assert!(!plan(&["fdu", "--progress", "never", "."], &interactive).draw);
        assert!(!plan(&["fdu", "--quiet", "."], &interactive).draw);
        assert!(!plan(&["fdu", "-q", "--progress", "always", "."], &interactive).draw);

        for command in [
            &["fdu", "--docs"][..],
            &["fdu", "--skill"],
            &["fdu", "--install-skill"],
            &["fdu", "--cache-status"],
            &["fdu", "--cache-clear", "."],
            &["fdu", "--progress", "always", "--docs"],
            &["fdu", "--progress", "always", "--install-skill", "--agent-base", "x"],
            &["fdu", "--progress", "always", "--cache-status=all", "."],
        ] {
            assert!(!plan(command, &interactive).draw, "{command:?} walks nothing");
        }

        for terminal in [
            TerminalFacts::default(),
            TerminalFacts { stderr_is_terminal: false, ..interactive_terminal() },
            TerminalFacts { term: None, ..interactive_terminal() },
            TerminalFacts { term: Some(OsString::from("dumb")), ..interactive_terminal() },
            TerminalFacts { ci: Some(OsString::from("true")), ..interactive_terminal() },
            TerminalFacts { vt_enabled: false, ..interactive_terminal() },
        ] {
            for mode in ["auto", "always", "never"] {
                assert!(
                    !plan(&["fdu", "--progress", mode, "."], &terminal).draw,
                    "{terminal:?} drew under --progress {mode}"
                );
            }
        }
    }

    #[test]
    fn the_progress_plan_names_the_root_and_follows_the_color_rule() {
        let interactive = interactive_terminal();
        let plan = |args: &[&str]| progress_plan_of(args, &interactive);
        assert_eq!(plan(&["fdu", "."]).roots, ["."]);
        assert_eq!(
            plan(&["fdu", "--cache-status"]).roots,
            ["."],
            "a lifecycle command without a path reports on the current directory"
        );
        let home = home_directory().expect("the test runner has a home directory");
        let under_home = home.join("wrk").join("github");
        assert_eq!(
            plan(&["fdu", under_home.to_str().expect("Unicode")]).roots,
            [Path::new("~").join("wrk").join("github").display().to_string()]
        );
        assert_eq!(plan(&["fdu", "docs", "src"]).roots, ["docs", "src"], "every root in order");

        assert!(!plan(&["fdu", "--color", "never", "."]).color);
        assert!(plan(&["fdu", "--color", "always", "."]).color);
        assert!(
            progress_plan_of(&["fdu", "--color", "always", "."], &TerminalFacts::default()).color,
            "--color always colors a frame it will never draw; drawing is --progress's call"
        );
        assert!(!plan(&["fdu", "--color", "never", "--format", "json", "."]).draw);

        // The line's bytes are measured as the answer's are.
        assert_eq!(plan(&["fdu", "."]).size, SizeMetric::Allocated);
        assert_eq!(plan(&["fdu", "--size", "apparent", "."]).size, SizeMetric::Apparent);
    }

    /// The plan a command resolves, from the request it would run.
    fn progress_plan_of(args: &[&str], terminal: &TerminalFacts) -> ProgressPlan {
        let parsed = parse(args);
        let request = parsed.resolved_request().expect("the request resolves");
        parsed.progress_plan(terminal, &request)
    }

    /// An interactive run inside the first-frame delay prints its summary without a
    /// progress frame or erase sequence.
    #[test]
    fn an_interactive_run_inside_the_delay_draws_no_progress_frame() {
        let interactive = interactive_terminal();
        let mut out = Vec::new();
        let err = SharedBuffer::default();
        let shipped = || ProgressIo {
            out: Box::new(err.clone()),
            timing: Timing::default(),
            width: || 100,
            interrupt: |_, _| {},
        };
        let args = ["fdu", "--progress", "always", "--docs"].map(OsString::from);
        assert_eq!(
            run_with_io(&args, &mut out, &mut err.clone(), true, &interactive, shipped()),
            0
        );
        assert!(!out.is_empty());
        assert!(err.contents().is_empty(), "{:?}", err.text());

        let root = tempfile::tempdir().expect("tempdir");
        let mut out = Vec::new();
        let args = [
            "fdu",
            "--cache",
            "off",
            "--color",
            "never",
            "--progress",
            "always",
            root.path().to_str().expect("Unicode"),
        ]
        .map(OsString::from);
        assert_eq!(
            run_with_io(&args, &mut out, &mut err.clone(), true, &interactive, shipped()),
            0
        );
        assert!(!String::from_utf8(out).expect("UTF-8").contains("perf:"));
        assert!(err.text().lines().last().is_some_and(|line| line.starts_with("perf:")));
        assert!(!err.text().contains(ERASE_LINE), "{:?}", err.text());
    }

    /// A tree whose walk outlasts the ticker thread's start by a wide margin, so a
    /// ticker with no delay and a one-millisecond tick has drawn at least one frame
    /// by the time the run stops it. A thousand files across forty directories take
    /// milliseconds to walk; the thread takes microseconds to start.
    fn wide_tree() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("tempdir");
        for dir in 0..40 {
            let dir = root.path().join(format!("dir{dir}"));
            std::fs::create_dir(&dir).expect("a directory");
            for file in 0..25 {
                std::fs::write(dir.join(format!("file{file}.txt")), b"some bytes\n")
                    .expect("a file");
            }
        }
        root
    }

    /// The line drawn into `err` at once and redrawn every millisecond, into the same
    /// buffer the run's diagnostics go to, so the test sees the order a terminal would.
    fn drawing_io(err: &SharedBuffer) -> ProgressIo {
        ProgressIo {
            out: Box::new(err.clone()),
            timing: Timing { delay: Duration::ZERO, tick: Duration::from_millis(1) },
            width: || 100,
            interrupt: |_, _| {},
        }
    }

    fn drawn_frames(text: &str) -> usize {
        // One erase per frame, plus the one at the stop point.
        text.matches(ERASE_LINE).count().saturating_sub(1)
    }

    /// Run `run` until the ticker has drawn at least one frame into its buffer, a few
    /// times at most, and return the run's status, stdout, and stderr text.
    ///
    /// Whether the ticker thread is scheduled during a short walk is up to the OS. The
    /// orderings these tests assert are only meaningful once a frame exists, and a
    /// runner that never schedules it in five runs is a result worth failing on.
    fn run_until_drawn(
        mut run: impl FnMut(&SharedBuffer) -> (u8, Vec<u8>),
    ) -> (u8, Vec<u8>, String) {
        for _ in 0..5 {
            let err = SharedBuffer::default();
            let (status, out) = run(&err);
            let text = err.text();
            if drawn_frames(&text) >= 1 {
                return (status, out, text);
            }
        }
        panic!("the ticker drew no frame in five runs");
    }

    /// The frames drawn while the tree was walked, then the erase, then report
    /// diagnostics. No frame follows the erase.
    #[cfg(unix)]
    #[test]
    fn the_line_is_erased_before_the_first_warning() {
        use std::os::unix::fs::PermissionsExt;

        let root = wide_tree();
        let denied = root.path().join("denied");
        std::fs::create_dir(&denied).expect("a directory");
        std::fs::write(denied.join("hidden.txt"), b"hidden").expect("a file");
        std::fs::set_permissions(&denied, std::fs::Permissions::from_mode(0o000))
            .expect("deny reads");
        if std::fs::read_dir(&denied).is_ok() {
            // A privileged process reads the directory anyway, so there is no warning
            // to order against; the ordering is covered by the error test below.
            eprintln!("skipped: this process is not subject to Unix permission bits");
            return;
        }

        let args = [
            "fdu",
            "--cache",
            "off",
            "--color",
            "never",
            "--progress",
            "always",
            root.path().to_str().expect("Unicode"),
        ]
        .map(OsString::from);
        let (status, out, text) = run_until_drawn(|err| {
            let mut out = Vec::new();
            let status = run_with_io(
                &args,
                &mut out,
                &mut err.clone(),
                false,
                &interactive_terminal(),
                drawing_io(err),
            );
            (status, out)
        });
        std::fs::set_permissions(&denied, std::fs::Permissions::from_mode(0o755))
            .expect("restore permissions so the directory can be removed");
        assert_eq!(status, 2, "a partial scan");
        assert!(!String::from_utf8(out).expect("UTF-8").contains("perf:"));
        assert!(text.lines().last().is_some_and(|line| line.starts_with("perf:")));

        let warning = text.find("warn:").expect("a status warning on stderr");
        let first_diagnostic = ["note:", "warn:", "tip:", "perf:"]
            .iter()
            .filter_map(|prefix| text.find(prefix))
            .min()
            .expect("a diagnostic after the scan");
        assert!(
            text[..first_diagnostic].ends_with(ERASE_LINE),
            "the erase is the last thing before diagnostics:\n{text:?}"
        );
        assert!(text[..first_diagnostic].starts_with(&format!("{ERASE_LINE}⠋ ")), "{text:?}");
        assert!(
            !text[first_diagnostic..].contains(ERASE_LINE),
            "the line was drawn after diagnostics"
        );
        assert!(!text[first_diagnostic..].contains('\r'), "{text:?}");
        assert!(warning < text.rfind("perf:").expect("performance summary"));
    }

    /// The frames, then the erase, then the error `finish` prints: a run whose report
    /// cannot be written still leaves a clean line under the error.
    #[test]
    fn the_line_is_erased_before_an_error() {
        let root = wide_tree();
        let args = [
            "fdu",
            "--cache",
            "off",
            "--color",
            "never",
            "--progress",
            "always",
            root.path().to_str().expect("Unicode"),
        ]
        .map(OsString::from);
        let (status, _, text) = run_until_drawn(|err| {
            let status = run_with_io(
                &args,
                &mut FailingWriter,
                &mut err.clone(),
                false,
                &interactive_terminal(),
                drawing_io(err),
            );
            (status, Vec::new())
        });
        assert_eq!(status, 1, "a report that cannot be written is fatal");

        let error = text.find("fdu: ").expect("the error on stderr");
        assert!(
            text[..error].ends_with(ERASE_LINE),
            "the erase is the last thing before the error:\n{text:?}"
        );
        assert_eq!(&text[error..], "fdu: output failed\n");
    }

    /// A watch start keeps its line up while the first answer is built (fdu-wku3): the
    /// build is drawn as `Summarizing` under the line the start drew, and the line is
    /// erased once the answer exists, before anything is written.
    #[cfg(feature = "watch")]
    #[test]
    fn a_watch_start_draws_through_its_first_answer_and_stops_before_writing_it() {
        let root = wide_tree();
        let command = parse(&[
            "fdu",
            "--watch",
            "--view",
            "files",
            "--cache",
            "off",
            "--color",
            "never",
            "--progress",
            "always",
            root.path().to_str().expect("Unicode"),
        ]);
        let request = command.request(root.path(), SystemTime::now()).expect("request");
        let delivery = Delivery {
            cache: CachePolicy::Off,
            stale_ok: false,
            cache_path: None,
            cache_dir: None,
            workers: fdu_core::query::Workers::default(),
            batch_size: fdu_core::ScanConfig::default().batch_size,
            order: fdu_core::ScanOrder::default(),
            watch: command.watch_delivery().expect("watch delivery"),
            accept_partial: false,
        };
        let render = report_format::RenderOptions { color: false, bar_size: 0 };

        // Whether the ticker thread is scheduled during the build is up to the OS, as
        // with the one-shot orderings above: a few runs, and a runner whose frames never
        // reach the build in five is a result worth failing on.
        let mut texts = Vec::new();
        for _ in 0..5 {
            let err = SharedBuffer::default();
            let plan = command.progress_plan(&interactive_terminal(), &request);
            assert!(plan.draw, "an interactive watch start draws");
            let (session, answer) = Cli::start_watch(
                &request,
                &delivery,
                report_format::Format::Text,
                render,
                Some((plan, drawing_io(&err))),
            )
            .expect("a watch start");
            drop(session);
            let text = err.text();
            assert!(!answer.sections.is_empty(), "the start built the first answer");
            if drawn_frames(&text) >= 1 {
                assert!(
                    text.ends_with(ERASE_LINE),
                    "the line is erased once the answer exists:\n{text:?}"
                );
            }
            if text.contains("Summarizing") {
                return;
            }
            texts.push(text);
        }
        panic!("no frame showed the first answer's build in five runs: {texts:?}");
    }

    /// The same run, the same tree, and the same drawing resources, but no person at
    /// the terminal: only diagnostics reach stderr, whatever `--progress` says.
    #[test]
    fn a_non_interactive_run_draws_no_progress_frame() {
        let root = wide_tree();
        for terminal in [
            TerminalFacts::default(),
            TerminalFacts { stderr_is_terminal: false, ..interactive_terminal() },
            TerminalFacts { term: Some(OsString::from("dumb")), ..interactive_terminal() },
            TerminalFacts { ci: Some(OsString::from("true")), ..interactive_terminal() },
        ] {
            for mode in ["auto", "always"] {
                let err = SharedBuffer::default();
                let mut out = Vec::new();
                let args = [
                    "fdu",
                    "--cache",
                    "off",
                    "--color",
                    "never",
                    "--progress",
                    mode,
                    root.path().to_str().expect("Unicode"),
                ]
                .map(OsString::from);
                let status = run_with_io(
                    &args,
                    &mut out,
                    &mut err.clone(),
                    false,
                    &terminal,
                    drawing_io(&err),
                );
                assert_eq!(status, 0);
                assert!(!String::from_utf8(out).expect("UTF-8").contains("perf:"));
                assert!(err.text().lines().last().is_some_and(|line| line.starts_with("perf:")));
                assert!(
                    !err.text().contains(ERASE_LINE),
                    "{terminal:?} --progress {mode}:\n{:?}",
                    err.text()
                );
            }
        }
    }

    /// A stdout whose reader has left, as `head` leaves once it has seen enough.
    ///
    /// It also records what stderr held the first time the report tried to write, so a
    /// test can check the line was already erased by then: after a stdout write fails,
    /// the ticker's drop erases the line anyway, so stderr's final bytes alone cannot
    /// tell a stop before the report from a stop after it.
    struct ClosedPipe {
        stderr: SharedBuffer,
        stderr_at_first_write: Option<String>,
    }

    impl Write for ClosedPipe {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            self.stderr_at_first_write.get_or_insert_with(|| self.stderr.text());
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "reader closed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// The broken-pipe rule with the indicator active: a drawing run whose stdout
    /// closes early still ends quietly with status 0, and the line is erased with
    /// nothing written after it, so a person who piped into `head` gets a clean prompt.
    #[test]
    fn a_drawing_run_whose_stdout_closes_early_ends_quietly_with_a_clean_line() {
        let root = wide_tree();
        let args = [
            "fdu",
            "--cache",
            "off",
            "--color",
            "never",
            "--progress",
            "always",
            root.path().to_str().expect("Unicode"),
        ]
        .map(OsString::from);
        let mut at_first_write = None;
        let (status, _, text) = run_until_drawn(|err| {
            let mut stdout = ClosedPipe { stderr: err.clone(), stderr_at_first_write: None };
            let status = run_with_io(
                &args,
                &mut stdout,
                &mut err.clone(),
                false,
                &interactive_terminal(),
                drawing_io(err),
            );
            at_first_write = stdout.stderr_at_first_write;
            (status, Vec::new())
        });
        assert_eq!(status, 0, "a consumer that has seen enough is not a failure");
        assert_eq!(
            at_first_write.as_deref(),
            Some(text.as_str()),
            "the line was erased, and nothing more drawn, before the report touched stdout"
        );
        assert!(text.starts_with(&format!("{ERASE_LINE}⠋ ")), "{text:?}");
        assert!(text.ends_with(ERASE_LINE), "the erase is the last thing on stderr:\n{text:?}");
        let after_last_frame = text.rsplit_once(ERASE_LINE).map(|(_, rest)| rest);
        assert_eq!(after_last_frame, Some(""), "nothing follows the erase:\n{text:?}");
        assert!(!text.contains("fdu:"), "a closed pipe is reported to nobody:\n{text:?}");
    }
}
