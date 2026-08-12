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
use std::time::{Duration, SystemTime};

use clap::builder::styling::{AnsiColor, Style as AnsiStyle, Styles};
use clap::{ArgAction, ColorChoice, CommandFactory, FromArgMatches, Parser, ValueEnum};

use crate::query::{
    Bound, Pattern, Provenance, Query, ReportSource, Selection, SizeMetric, SortKey, ViewSpec,
    parse_size, parse_when, system_time_to_nanos,
};
use crate::report_format;
use crate::{
    CachePolicy, EntryKind, OpenConfig, ScanConfig, default_cache_path, open_with_pending_save,
};

const SKILL_TEMPLATE: &str = include_str!("skills/SKILL.md");

const STYLE_HEADING: AnsiStyle = AnsiColor::Cyan.on_default().bold();
const STYLE_WARNING: AnsiStyle = AnsiColor::Yellow.on_default().bold();
const STYLE_ERROR: AnsiStyle = AnsiColor::Red.on_default().bold();
const STYLE_CAUSE: AnsiStyle = AnsiStyle::new().dimmed();
const CLI_STYLES: Styles = Styles::styled()
    .header(STYLE_HEADING)
    .usage(STYLE_HEADING)
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .error(STYLE_ERROR)
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Yellow.on_default());

const AFTER_HELP: &str = "Examples:\n  fdu\n  fdu --view types ~/Downloads\n  fdu --view files --sort size --limit 20 ~/src\n  fdu --view files --modified-since 2h --format jsonl .\n  fdu --view summary,types --format json .\n\nFive axes, and every option belongs to exactly one:\n  Scope      PATH, --scan-depth        what is scanned and cached\n  Selection  --include, --exclude, --min-size, --modified-since, --modified-before,\n             --kind, --depth, --limit, --sort, --reverse, --size\n  View       --view tree,types,files,summary\n  Format     --format text|json|jsonl|yaml, --color\n  Mode       --cache auto|refresh|read-only|only|off\n\nScope versus selection:\n  --scan-depth limits what is scanned and retained; one cache then serves every query.\n  --depth and --limit bound only the rendered view, and never cost a rescan.\n  --depth 0 reports totals for the root and nothing beneath it.\n  --depth and --limit accept `all` for no bound.\n\nValues:\n  SIZE   512, 10k, 10M, 1.5GiB (decimal and binary units, case-insensitive)\n  WHEN   now, an age (45s, 2h, 1h30m), RFC 3339 with an offset, or @epoch seconds\n  --modified-since is inclusive; --modified-before is exclusive\n  --include and --exclude are repeatable globs; --view and --kind are comma lists\n\nCache:\n  auto       read, revalidate, and write back when complete (default)\n  refresh    ignore any snapshot, scan cold, and rewrite it\n  read-only  read and revalidate, but never write\n  only       answer from the snapshot without touching the tree; labeled stale,\n             and fails when no usable snapshot exists rather than scanning\n  off        ignore the snapshot and leave nothing behind\n\nOutput and automation:\n  Results go to stdout; warnings and errors go to stderr.\n  Machine formats are schema-versioned and never colorized.\n  Every report carries schema, source, freshness, complete, errors, and both timestamps.\n  Feed a report's scan_started_at back as --modified-since to list what changed since.\n  The command never prompts, pages, or animates progress.\n\nColor:\n  --color overrides NO_COLOR and FORCE_COLOR. In auto mode, NO_COLOR disables color,\n  FORCE_COLOR enables it, and otherwise the destination must be a terminal.\n\nExit status:\n  0  Complete result, or a partial result accepted with --allow-partial\n  1  Fatal filesystem or cache error\n  2  Partial result, or command-line usage error";

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
    after_help = AFTER_HELP
)]
// A command line is a flat bag of independent switches. Folding these into enums to
// satisfy the lint would obscure the one thing this struct exists to mirror: the flags a
// user actually types.
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    // ---- scope: what the engine observes and retains ----
    /// Directory to summarize.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Limit scanning and retention to N entry levels.
    #[arg(long, value_name = "N")]
    pub scan_depth: Option<usize>,

    /// Stay on the filesystem the root lives on.
    #[arg(long, action = ArgAction::SetTrue)]
    pub one_filesystem: bool,

    // ---- selection: which retained entries this query considers ----
    /// Report only entries matching this glob; repeatable.
    #[arg(long, value_name = "GLOB")]
    pub include: Vec<String>,

    /// Exclude entries matching this glob; repeatable, and wins over --include.
    #[arg(long, value_name = "GLOB")]
    pub exclude: Vec<String>,

    /// Report only entries at least this large, as 512, 10M, or 1.5GiB.
    #[arg(long, value_name = "SIZE")]
    pub min_size: Option<String>,

    /// Report only entries modified at or after this time, as 2h or an RFC 3339 stamp.
    #[arg(long, value_name = "WHEN")]
    pub modified_since: Option<String>,

    /// Report only entries modified before this time.
    #[arg(long, value_name = "WHEN")]
    pub modified_before: Option<String>,

    /// Entry kinds to report: file, dir, symlink, other.
    #[arg(long, value_name = "LIST")]
    pub kind: Option<String>,

    /// Directory levels to show; does not limit scanning. Accepts `all`.
    #[arg(short, long, default_value = "2", value_name = "N")]
    pub depth: String,

    /// Entries to show per directory. Accepts `all`.
    #[arg(short = 'n', long, default_value = "10", value_name = "N")]
    pub limit: String,

    /// Order results: size, count, mtime, or name.
    #[arg(long, value_name = "KEY")]
    pub sort: Option<String>,

    /// Reverse the ordering.
    #[arg(long, action = ArgAction::SetTrue)]
    pub reverse: bool,

    /// Which size metric to report: allocated or apparent.
    #[arg(long, value_name = "METRIC", default_value = "allocated")]
    pub size: String,

    // ---- view: which roll-ups are reported ----
    /// Views to report: tree, types, files, summary.
    #[arg(long, value_name = "LIST", default_value = "tree")]
    pub view: String,

    // ---- format: how the report is serialized ----
    /// Output format: text, json, jsonl, or yaml.
    #[arg(long, value_name = "FORMAT", default_value = "text")]
    pub format: String,

    /// Colorize human output: auto, always, or never.
    #[arg(long, value_name = "WHEN", default_value = "auto", hide_possible_values = true)]
    pub color: ColorWhen,

    // ---- mode: how the cache is used ----
    /// Cache policy: auto, refresh, read-only, only, or off.
    #[arg(long, value_name = "POLICY", default_value = "auto")]
    pub cache: String,

    /// Accept incomplete totals when paths cannot be read.
    #[arg(long, action = ArgAction::SetTrue)]
    pub allow_partial: bool,

    /// Report cache contents instead of scanning: root (default) or all.
    #[arg(long, value_name = "SCOPE", num_args = 0..=1, require_equals = true, default_missing_value = "root")]
    pub cache_status: Option<String>,

    /// Remove cached snapshots instead of scanning: root (default) or all.
    #[arg(long, value_name = "SCOPE", num_args = 0..=1, require_equals = true, default_missing_value = "root")]
    pub cache_clear: Option<String>,

    /// Stream changes continuously instead of returning one report.
    #[cfg(feature = "watch")]
    #[arg(long, action = ArgAction::SetTrue)]
    pub watch: bool,

    /// How often aggregate views re-render while watching, as a duration.
    ///
    /// Throttles rendering only; change detection is event-driven and unaffected.
    #[cfg(feature = "watch")]
    #[arg(long, value_name = "DUR", default_value = "2s")]
    pub interval: String,

    /// Print a portable agent skill to stdout.
    #[arg(long, action = ArgAction::SetTrue)]
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
        let query = self.parse_query().map_err(|error| usage(&error))?;

        let policy = self.parse_cache_policy().map_err(|error| usage(&error))?;
        let config = OpenConfig {
            scan: ScanConfig {
                max_depth: self.scan_depth,
                one_filesystem: self.one_filesystem,
                ..ScanConfig::default()
            },
            cache_path: default_cache_path(&self.path),
            policy,
        };

        #[cfg(feature = "watch")]
        if self.watch && (self.scan_depth.is_some() || self.one_filesystem) {
            // Scope narrows what is observed, and a watcher cannot filter raw backend
            // events against that boundary yet. Selection flags stay legal with --watch
            // precisely because they filter the retained index instead, and the message
            // says so rather than only naming the conflict.
            return Err(usage(&anyhow::anyhow!(concat!(
                "--watch cannot be combined with --scan-depth or --one-filesystem: watching ",
                "requires full scope. ",
                "Selection flags such as --depth, --include, and --modified-since do work with ",
                "--watch, because they filter the index rather than narrowing the scan"
            ))));
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

        let scan_started_at = SystemTime::now();
        let (index, open_report, pending_save) = open_with_pending_save(&self.path, &config)?;

        let provenance = Provenance {
            scan_started_at: Some(scan_started_at),
            generated_at: SystemTime::now(),
            source: match open_report.path_taken {
                crate::OpenPath::ColdScan => ReportSource::ColdScan,
                crate::OpenPath::WarmRevalidate => ReportSource::WarmRevalidate,
                crate::OpenPath::CacheOnly => ReportSource::CacheOnly,
            },
            complete: open_report.is_complete(),
            errors: open_report.errors().iter().map(ToString::to_string).collect(),
        };
        let report = crate::query::report(&index, &query, &provenance);

        let color = ColorContext::from_environment(
            self.color,
            self.machine_format(),
            self.skill,
            stdout_is_terminal,
        )
        .enabled();
        // The write is already running; rendering is the other reader. Whether output
        // finishes first or the save does, both complete.
        let rendered = report_format::render(&report, format, color);
        let render_result = write!(out, "{rendered}");

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

        if format == report_format::Format::Text && !open_report.is_complete() {
            let color =
                ColorContext::from_environment(self.color, false, false, stderr_is_terminal)
                    .enabled();
            for error in open_report.errors() {
                let _ = writeln!(
                    diagnostic,
                    "{}",
                    paint(&format!("warning: {error}"), STYLE_WARNING, color)
                );
            }
        }

        Ok(if open_report.is_complete() { RunOutcome::Complete } else { RunOutcome::Partial })
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
        use crate::query::ViewSpec;
        use crate::session::{ChangeKind, Session};
        use crate::watch::WatchConfig;

        let interval = parse_duration(&self.interval).map_err(|error| usage(&error))?;

        let scan_started_at = SystemTime::now();
        let (index, open_report, pending_save) = open_with_pending_save(&self.path, config)?;
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

        let handle = crate::IndexHandle::new(index);
        let mut session = Session::new(handle, config.scan.clone(), query, WatchConfig::default())?;

        // The initial answer, identical to a one-shot run's.
        let provenance = Provenance {
            scan_started_at: Some(scan_started_at),
            generated_at: SystemTime::now(),
            source: match open_report.path_taken {
                crate::OpenPath::ColdScan => ReportSource::ColdScan,
                crate::OpenPath::WarmRevalidate => ReportSource::WarmRevalidate,
                crate::OpenPath::CacheOnly => ReportSource::CacheOnly,
            },
            complete: open_report.is_complete(),
            errors: open_report.errors().iter().map(ToString::to_string).collect(),
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
        session: &crate::session::Session,
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
    fn save_live(session: &crate::session::Session, config: &OpenConfig) -> anyhow::Result<bool> {
        let (Some(cache_path), true) = (config.cache_path.as_deref(), config.policy.writes())
        else {
            return Ok(false);
        };
        let index = session.index_snapshot()?;
        if index.freshness() != crate::Freshness::Fresh {
            // Only a trustworthy index is worth persisting; a partial one would be
            // served as fact on the next run. Reported as "not written" so the caller
            // keeps the change pending and tries again once the index settles.
            return Ok(false);
        }
        crate::snapshot::save(&index, cache_path)?;
        Ok(true)
    }

    /// Re-render the aggregate views of a live session.
    #[cfg(feature = "watch")]
    fn render_live(
        out: &mut dyn Write,
        session: &crate::session::Session,
        format: report_format::Format,
        color: bool,
    ) -> anyhow::Result<()> {
        let provenance = session.live_provenance(SystemTime::now());
        let report = session.report(&provenance)?;
        write!(out, "{}", report_format::render(&report, format, color))?;
        out.flush()?;
        Ok(())
    }

    /// Run the cache lifecycle flags and report what they found or removed.
    fn run_cache_lifecycle(&self, out: &mut dyn Write) -> anyhow::Result<RunOutcome> {
        let cache_dir = crate::default_cache_path(&self.path)
            .and_then(|path| path.parent().map(Path::to_path_buf));

        if let Some(scope) = &self.cache_clear {
            let scope = CacheScope::parse(scope, "--cache-clear").map_err(|e| usage(&e))?;
            match (scope, &cache_dir) {
                (CacheScope::All, Some(dir)) => {
                    // Echo the directory before acting, so a destructive flag always says
                    // where it is pointed.
                    writeln!(out, "Cache directory: {}", dir.display())?;
                    let removed = crate::clear_all_caches(dir)?;
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
                    let path = crate::default_cache_path(&self.path);
                    let removed = match &path {
                        Some(path) => crate::clear_cache(path)?,
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
                (CacheScope::All, Some(dir)) => crate::list_caches(dir)?,
                (CacheScope::All, None) => Vec::new(),
                (CacheScope::Root, _) => match crate::default_cache_path(&self.path) {
                    Some(path) => vec![crate::cache_status(&path)?],
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
        statuses: &[crate::CacheStatus],
    ) -> anyhow::Result<()> {
        let format = self.parse_format().map_err(|e| usage(&e))?;
        if format == report_format::Format::Text {
            if statuses.iter().all(|status| !status.is_recognized()) {
                writeln!(out, "No cached snapshots.")?;
                return Ok(());
            }
            for status in statuses {
                match &status.snapshot {
                    Some(info) => writeln!(
                        out,
                        "{}  {} entries, {} bytes  {}",
                        status.path.display(),
                        info.entries,
                        status.bytes,
                        info.root.display()
                    )?,
                    None => writeln!(out, "{}  unrecognized", status.path.display())?,
                }
            }
            return Ok(());
        }

        // Machine output gets the same facts under the schema envelope's conventions.
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
    fn parse_query(&self) -> anyhow::Result<Query> {
        let now = SystemTime::now();

        let mut selection = Selection {
            depth: parse_bound(&self.depth, "--depth")?,
            limit: parse_bound(&self.limit, "--limit")?,
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

        Ok(Query { selection, views: parse_list(&self.view, "--view", parse_view)? })
    }
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

/// Parse one `--view` token.
fn parse_view(token: &str, flag: &str) -> anyhow::Result<ViewSpec> {
    match token.to_ascii_lowercase().as_str() {
        "tree" => Ok(ViewSpec::Tree),
        "types" => Ok(ViewSpec::Types),
        "files" => Ok(ViewSpec::Files),
        "summary" => Ok(ViewSpec::Summary),
        _ => anyhow::bail!("invalid {flag} {token:?}: expected one of tree, types, files, summary"),
    }
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
    if color { write!(out, "{}", rendered.ansi()) } else { write!(out, "{rendered}") }
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
            let _ = writeln!(diagnostic, "{} {error}", paint("fdu:", STYLE_ERROR, color));
            for cause in error.chain().skip(1) {
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

// Rounding a fraction to one of eleven bar widths is exactly the case where float-cast
// lints have nothing to protect: the value is clamped to [0, 1] before the cast and the
// result is clamped to WIDTH after it.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
/// Format a byte count the way a person reads it.
/// Encode a string as a JSON string literal.
/// Write lossless identity metadata when an operating-system string cannot be represented
/// by the display-oriented JSON string beside it.
#[cfg(any(unix, windows))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

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

    const DEEP_RENDER_CHILD_ENV: &str = "FDU_DEEP_RENDER_CHILD";
    const DEEP_RENDER_DEPTH: usize = 1_024;
    const DEEP_RENDER_STACK_BYTES: usize = 64 * 1_024;

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
            path: PathBuf::from("."),
            scan_depth: None,
            one_filesystem: false,
            include: Vec::new(),
            exclude: Vec::new(),
            min_size: None,
            modified_since: None,
            modified_before: None,
            kind: None,
            depth: "2".to_string(),
            limit: "10".to_string(),
            sort: None,
            reverse: false,
            size: "allocated".to_string(),
            view: "tree".to_string(),
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
            skill: false,
        }
    }

    fn query_error(cli: &Cli) -> String {
        cli.parse_query().expect_err("expected a rejection").to_string()
    }

    // ---- the five axes translate into library types, and nothing else ----

    #[test]
    fn views_parse_as_an_ordered_comma_list() {
        let parsed = Cli { view: "types,tree,summary".to_string(), ..cli() }
            .parse_query()
            .expect("views parse");
        assert_eq!(parsed.views, vec![ViewSpec::Types, ViewSpec::Tree, ViewSpec::Summary]);
    }

    #[test]
    fn an_unknown_view_names_every_valid_value() {
        let message = query_error(&Cli { view: "bogus".to_string(), ..cli() });
        assert!(message.contains("tree, types, files, summary"), "{message}");
    }

    #[test]
    fn a_repeated_view_is_a_typo_not_a_no_op() {
        let message = query_error(&Cli { view: "tree,tree".to_string(), ..cli() });
        assert!(message.contains("appears more than once"), "{message}");
    }

    #[test]
    fn an_empty_list_entry_is_rejected() {
        let message = query_error(&Cli { view: "tree,,types".to_string(), ..cli() });
        assert!(message.contains("empty entry"), "{message}");
    }

    #[test]
    fn bounds_accept_all_as_well_as_a_number() {
        let parsed = Cli { depth: "all".to_string(), limit: "3".to_string(), ..cli() }
            .parse_query()
            .expect("bounds parse");
        assert_eq!(parsed.selection.depth, Bound::All);
        assert_eq!(parsed.selection.limit, Bound::Limit(3));
        // du's meaning of depth 0 survives the rename from --max-depth.
        let zero = Cli { depth: "0".to_string(), ..cli() }.parse_query().expect("parses");
        assert_eq!(zero.selection.depth, Bound::Limit(0));
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
        .parse_query()
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
        .parse_query()
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
        .parse_query()
        .expect("parses");
        assert_eq!(parsed.selection.kinds, vec![EntryKind::File, EntryKind::Dir]);
        assert_eq!(parsed.selection.sort, Some(SortKey::Mtime));
        assert_eq!(parsed.selection.size, SizeMetric::Apparent);
        assert!(parsed.selection.reverse);
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
            path: PathBuf::from("/nonexistent-root-that-should-not-be-scanned"),
            view: "bogus".to_string(),
            ..cli()
        });
        assert!(message.contains("expected one of"), "{message}");
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

    #[test]
    fn deep_rendering_is_stack_safe() {
        if std::env::var_os(DEEP_RENDER_CHILD_ENV).is_some() {
            run_deep_render_child();
            return;
        }

        let output = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", "cli::tests::deep_rendering_is_stack_safe", "--nocapture"])
            .env(DEEP_RENDER_CHILD_ENV, "1")
            .output()
            .expect("run deep-render child");

        assert!(
            output.status.success(),
            "deep renderer failed in child process\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn run_deep_render_child() {
        // A deep tree must render, not abort: expansion and all three renderers use
        // explicit stacks, and this proves it on a 64 KiB stack where recursion would die.
        let mut index = crate::Index::new("/fixture");
        let mut path = PathBuf::new();
        for depth in 0..DEEP_RENDER_DEPTH {
            path.push("d");
            index.apply_ok(&crate::Observation::new(vec![crate::Op::Upsert {
                path: path.clone(),
                kind: EntryKind::Dir,
                attrs: crate::Attrs {
                    mtime_ns: i64::try_from(depth).expect("fixture depth fits i64"),
                    ..Default::default()
                },
            }]));
        }
        index.set_initial_freshness(false);

        std::thread::Builder::new()
            .name("deep-render".to_string())
            .stack_size(DEEP_RENDER_STACK_BYTES)
            .spawn(move || {
                let query = Query {
                    selection: Selection { depth: Bound::All, ..Selection::default() },
                    views: vec![ViewSpec::Tree],
                };
                let provenance = Provenance {
                    scan_started_at: None,
                    generated_at: SystemTime::UNIX_EPOCH,
                    source: ReportSource::ColdScan,
                    complete: true,
                    errors: Vec::new(),
                };
                let report = crate::query::report(&index, &query, &provenance);
                for format in [
                    report_format::Format::Text,
                    report_format::Format::Json,
                    report_format::Format::Jsonl,
                    report_format::Format::Yaml,
                ] {
                    let rendered = report_format::render(&report, format, false);
                    assert!(!rendered.is_empty(), "{format:?} rendered nothing for a deep tree");
                }
            })
            .expect("spawn deep-render thread")
            .join()
            .expect("deep-render thread");
    }

    /// Two names that differ only in bytes `to_string_lossy` cannot represent must stay
    /// distinguishable in machine output.
    ///
    /// This coverage was lost when the CLI moved to the five axes: `raw_identity_json`
    /// survived the rewrite, its tests did not, and the merge from PR #6 is what surfaced
    /// the gap. Retargeted here to the report path rather than restored to the old
    /// `write_json`, because the guarantee belongs to the format, not to the flag that
    /// used to select it.
    fn assert_json_preserves_raw_identity(
        root: PathBuf,
        first: &OsStr,
        second: &OsStr,
        encoding: &str,
        root_hex: &str,
        first_hex: &str,
        second_hex: &str,
    ) {
        // The premise: lossy rendering collapses these two into the same string, so a
        // consumer with only `name` cannot tell them apart.
        assert_eq!(first.to_string_lossy(), second.to_string_lossy());

        let mut index = crate::Index::new(root);
        index.apply_ok(&crate::Observation::new(vec![
            crate::Op::Upsert {
                path: PathBuf::from(first),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1, allocated: 1, ..Default::default() },
            },
            crate::Op::Upsert {
                path: PathBuf::from(second),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1, allocated: 1, ..Default::default() },
            },
        ]));
        index.set_initial_freshness(false);

        let query = Cli { view: "files".to_string(), depth: "all".to_string(), ..cli() }
            .parse_query()
            .expect("query parses");
        let provenance = Provenance {
            scan_started_at: None,
            generated_at: std::time::UNIX_EPOCH,
            source: ReportSource::ColdScan,
            complete: true,
            errors: Vec::new(),
        };
        let report = crate::query::report(&index, &query, &provenance);
        let rendered = report_format::render(&report, report_format::Format::Json, false);

        let lossy = first.to_string_lossy();
        assert_eq!(
            rendered.matches(&format!("\"{lossy}\"")).count(),
            2,
            "both names render the same lossy text: {rendered}"
        );
        assert!(
            rendered.contains(&format!(
                "\"root_raw\": {{\"encoding\": \"{encoding}\", \"hex\": \"{root_hex}\"}}"
            )),
            "{rendered}"
        );

        // Pinned as the whole row rather than as a substring of it. A loose `contains`
        // check on the `path_raw` object alone passed while the row around it was
        // malformed -- the field was emitted with a duplicated separator and a newline
        // inside a one-line object, so the document did not parse at all. Asserting the
        // exact row is what makes the surrounding punctuation part of the contract.
        for hex in [first_hex, second_hex] {
            let row = format!(
                "{{\"path\": \"{lossy}\", \"path_raw\": {{\"encoding\": \"{encoding}\", \"hex\": \"{hex}\"}}, \
                 \"kind\": \"file\", \"bytes\": 1, \"allocated\": 1, \"mtime_ns\": 0}}"
            );
            assert!(
                rendered.contains(&row),
                "a name that is not valid Unicode must carry its raw bytes in a well-formed \
                 row.\nexpected: {row}\nrendered: {rendered}"
            );
        }

        // Cheap structural guard against the same class of mistake anywhere else in the
        // document: an empty element is the signature of a separator emitted twice.
        assert!(
            !rendered.contains(", ,") && !rendered.contains(",,"),
            "duplicated separator in machine output: {rendered}"
        );

        // The tree writer names entries too, and carried the identical defect. Pinning
        // only the files view would have left half the fix untested. A tree lists
        // directories, so the case has to be a directory whose own name is not valid
        // Unicode rather than the files above.
        let mut dirs = crate::Index::new(PathBuf::from("/tree-fixture"));
        dirs.apply_ok(&crate::Observation::new(vec![
            crate::Op::Upsert {
                path: PathBuf::from(first),
                kind: EntryKind::Dir,
                attrs: crate::Attrs { size: 0, allocated: 0, ..Default::default() },
            },
            crate::Op::Upsert {
                path: PathBuf::from(first).join("inside.txt"),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1, allocated: 1, ..Default::default() },
            },
        ]));
        dirs.set_initial_freshness(false);
        let tree_query = Cli { view: "tree".to_string(), depth: "all".to_string(), ..cli() }
            .parse_query()
            .expect("query parses");
        let tree = crate::query::report(&dirs, &tree_query, &provenance);
        let tree_rendered = report_format::render(&tree, report_format::Format::Json, false);
        assert!(
            tree_rendered.contains(&format!(
                ", \"path_raw\": {{\"encoding\": \"{encoding}\", \"hex\": \"{first_hex}\"}}, \"kind\":"
            )),
            "the tree view must carry raw identity in a well-formed node: {tree_rendered}"
        );
        assert!(
            !tree_rendered.contains(", ,") && !tree_rendered.contains(",,"),
            "duplicated separator in tree output: {tree_rendered}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn json_preserves_distinct_non_unicode_unix_names() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        assert_json_preserves_raw_identity(
            PathBuf::from(OsString::from_vec(vec![b'/', 0x80])),
            &OsString::from_vec(vec![b'n', 0x80]),
            &OsString::from_vec(vec![b'n', 0x81]),
            "unix-bytes",
            "2f80",
            "6e80",
            "6e81",
        );
    }

    #[cfg(windows)]
    #[test]
    fn json_preserves_distinct_non_unicode_windows_names() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        assert_json_preserves_raw_identity(
            PathBuf::from(OsString::from_wide(&[u16::from(b'R'), u16::from(b':'), 0xd800])),
            &OsString::from_wide(&[u16::from(b'n'), 0xd800]),
            &OsString::from_wide(&[u16::from(b'n'), 0xd801]),
            "windows-wtf16le",
            "52003a0000d8",
            "6e0000d8",
            "6e0001d8",
        );
    }
}
