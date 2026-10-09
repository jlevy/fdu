//! Serializing a [`Report`] to text, JSON, JSONL, and YAML.
//!
//! Formats are serializations, not features: every view renders in every format, so a
//! caller picks the shape of the answer and the shape of the bytes independently.
//!
//! # Output design system
//!
//! Keep measured results and explanatory diagnostics separate. Renderers return only
//! result data; frontends route the categorized messages from [`diagnostic_lines`] and
//! [`report_warnings`] to their diagnostic stream. Machine formats must remain parseable
//! and ANSI-free.
//!
//! Human rows use bright bold cyan names, ordinary foreground file counts, and gray
//! parenthetical detail. Ignored amounts embedded in a row are always gray parentheses;
//! file counts belong directly after the name, outside parentheses. Secondary breakdowns
//! such as nonblank/blank counts use the same gray parenthetical role. Human directory
//! names have a gray slash except `.` and `..`. Directly or ancestrally gitignored
//! directories use regular, nonbold cyan; merely containing ignored files does not
//! change a directory name, and file-name styling is unchanged. Sizes >= 1 GiB are bold even in gray
//! details; zero sizes and exact shares below 1% are gray. Pad cells before applying ANSI styles.
//! Colored bars use green solid non-gitignored and shaded gitignored usage, with dim green
//! light-shade cells for unused width. Plain bars keep their original glyphs. Human tree bar width
//! is caller-selectable, including zero to remove the bar and its gutter; machine
//! formats and non-tree views ignore it. Human integer quantities share one grouping
//! policy through [`human_count`] and [`human_count_u128`]. Every human byte quantity,
//! including cache rows and lifecycle totals, uses [`styled_bytes`] for its units and
//! zero/large-value emphasis. Machine fields retain exact integer bytes.
//!
//! A grouped row's first line ends at its file count; each further measure (lines, words,
//! flags, coverage reasons) is a continuation line of its own, its size, share, and label
//! cells blank and its text two columns right of the file count. See the layout rules
//! beside `render_text_metrics`.
//!
//! Tree columns are bar, root percentage, size, then indented name. One remainder
//! row per tree uses those same columns and quantity styles for unlisted root branches.
//! Its `… and` prefix is gray; the recursive hidden file count uses normal foreground.
//! That usage is already included in directory totals.
//! Unknown coverage must show unknown size and no fabricated bar or percentage.
//! Keep rerun flags out of rows: collect applicable remedies once per report in
//! `report_epilogue`.
//!
//! The category, ordering, and debugging contract lives beside that collector; CLI
//! stream/color handling lives in `write_report_diagnostics`. The contributor guide is
//! `docs/project/architecture/fdu-output-design.md`. Changes must keep the shared golden
//! corpus, Python parity, and terminal stream/color assertions consistent.
//!
//! # Why these are hand-written
//!
//! `serde` plus a JSON crate plus a YAML crate would be three dependency additions —
//! and the maintained-YAML question is genuinely unsettled, since `serde_yaml` is
//! unmaintained. The schema here is small, closed, and fully known at compile time, the
//! crate already hand-writes its JSON, and hand-writing keeps the machine formats
//! provably free of a serializer's own opinions about key order and number formatting.
//! Key order is fixed by the code, which is what makes the goldens byte-stable.

mod report_epilogue;

pub use report_epilogue::{
    DiagnosticLines, diagnostic_lines, diagnostics, report_notes, report_tips, report_warnings,
};

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use anstyle::{AnsiColor, Style as AnsiStyle};

use crate::classify::human_language_name;
use crate::content::{CoverageReason, METRICS};
use crate::control::ControlCoverage;
use crate::emit::{Event, IoFmt, JsonSink, Scalar, Shape, Sink, YamlSink};
use crate::engine_contract::{Coverage, EntryKind, Freshness, IssueKind, Source};
use crate::query::{
    CodeOverview, CodeTally, FileRow, IgnoredEntries, IgnoredTally, MONTH_SECONDS, MetricGroup,
    MetricRow, MetricSummary, Report, ReportSource, Section, ShareMetric, SizeMetric, SummaryRow,
    TierState, TreeNode, TypeRow, ViewSpec, YEAR_SECONDS, format_rfc3339, format_rfc3339_nanos,
    pages, with_rfc3339_nanos,
};

/// The all-caps label naming which view a block of text output belongs to.
///
/// Bold cyan is what `cli.rs` already gives a section heading in `--help`, so a report
/// and the help that describes it use one visual language for the same idea.
/// View headers share the CLI's one header style; see `cli::STYLE_HEADING`.
pub const STYLE_HEADING: AnsiStyle = AnsiColor::Cyan.on_default().bold();

/// Directory names in a tree, so structure reads at a glance.
pub const STYLE_NAME: AnsiStyle = AnsiColor::BrightCyan.on_default().bold();

/// A directory whose own path is gitignored, directly or by an ignored ancestor.
const STYLE_IGNORED_NAME: AnsiStyle = AnsiColor::Cyan.on_default();

const STYLE_BAR: AnsiStyle = AnsiColor::Green.on_default();

/// Category labels keep ordinary cyan; bold bright cyan identifies names.
pub const STYLE_CATEGORY: AnsiStyle = AnsiColor::Cyan.on_default();

/// Secondary information: parenthetical detail, omissions, notes, tips, and telemetry.
/// Keep this gray and non-bold except for the shared >= 1 GiB size emphasis.
pub const STYLE_DETAIL: AnsiStyle = AnsiColor::BrightBlack.on_default();

/// Established label width for non-language metric summaries.
const TEXT_METRIC_LABEL_WIDTH: usize = 18;
/// Floor for the extensions view's label column.
const TEXT_TYPE_LABEL_WIDTH: usize = 12;

/// Machine-output schema identity.
///
/// Any change to a field's name, type, or meaning bumps this, and a golden test fails if
/// the schema moves without it — the versioning is the promise, not the intention.
///
/// `fdu.report/11` gives tree nodes their age (`mtime_ns`, `complete`, `age_ns`, and
/// `modified_at`), list rows `modified_at`, and the envelope `age_reference_at`.
pub const REPORT_SCHEMA: &str = "fdu.report/11";
/// All reports now use one shape-versioned schema regardless of requested analyzers.
pub const CONTENT_REPORT_SCHEMA: &str = REPORT_SCHEMA;
/// Machine-output schema identity for cache status.
///
/// Its own identity because cache status is its own document: a fact about the cache
/// directory rather than about a tree, which is why it is not a `Report` section. It
/// carries the same promise as [`REPORT_SCHEMA`] and versions independently, so a change
/// to the report shape never invalidates a cache-status consumer, or the reverse.
///
/// `fdu.cache/3` adds the identity of every tier a store holds: a current snapshot's
/// `identity`, and a `content` object for the sidecar beside any snapshot, in place of
/// `content_bytes`.
pub const CACHE_SCHEMA: &str = "fdu.cache/3";

/// How a report is serialized.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Format {
    /// Human-readable text.
    #[default]
    Text,
    /// The bounded directory hierarchy for a list.
    Tree,
    /// Matching paths, one safely escaped path per line.
    Paths,
    /// Flat size, signed modification age, and path columns.
    Long,
    /// One JSON document.
    Json,
    /// One JSON document per line, one line per section.
    Jsonl,
    /// YAML.
    Yaml,
}

/// Maximum tree bar width accepted by the human renderer.
/// Bounds decoration allocation without changing measured report data.
pub const MAX_BAR_SIZE: usize = 4096;

/// Presentation choices for a human report; machine formats ignore both fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderOptions {
    /// Whether to emit terminal color and emphasis.
    pub color: bool,
    /// Width of tree usage bars in cells; zero removes the bar and its gutter.
    pub bar_size: usize,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self { color: false, bar_size: 10 }
    }
}

/// Start one document in a multi-document stream for `format`.
pub const fn document_start(format: Format) -> &'static str {
    match format {
        Format::Yaml => "---\n",
        Format::Text
        | Format::Tree
        | Format::Paths
        | Format::Long
        | Format::Json
        | Format::Jsonl => "",
    }
}

impl Format {
    /// Stable spelling used by request adapters and diagnostics.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Tree => "tree",
            Self::Paths => "paths",
            Self::Long => "long",
            Self::Json => "json",
            Self::Jsonl => "jsonl",
            Self::Yaml => "yaml",
        }
    }

    /// Whether this format is a structured serialization.
    pub const fn is_machine(self) -> bool {
        matches!(self, Self::Json | Self::Jsonl | Self::Yaml)
    }

    /// Parse a `--format` value.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "text" => Some(Self::Text),
            "tree" => Some(Self::Tree),
            "paths" => Some(Self::Paths),
            "long" => Some(Self::Long),
            "json" => Some(Self::Json),
            "jsonl" => Some(Self::Jsonl),
            "yaml" => Some(Self::Yaml),
            _ => None,
        }
    }

    /// Every accepted spelling, for help text and error messages.
    pub const ALL: &'static [&'static str] =
        &["text", "tree", "paths", "long", "json", "jsonl", "yaml"];
}

/// Render a report in the requested format.
///
/// `color` applies to the text form only: machine output is never colourized, because a
/// consumer parsing JSON should never have to strip escape sequences first.
///
/// # Errors
///
/// Returns an invalid-request error when Tree/Paths/Long cannot represent the stored
/// projection. Request the desired format on the query before reading: a detached,
/// folded tree does not retain the complete flat inventory.
pub fn render(report: &Report, format: Format, color: bool) -> crate::Result<String> {
    render_with_options(report, format, RenderOptions { color, ..RenderOptions::default() })
}

/// Render with explicit human presentation choices.
///
/// Machine and flat formats retain their existing bytes regardless of `bar_size`.
///
/// # Errors
///
/// Returns an invalid-request error for an incompatible format or a human tree bar wider
/// than [`MAX_BAR_SIZE`].
pub fn render_with_options(
    report: &Report,
    format: Format,
    options: RenderOptions,
) -> crate::Result<String> {
    let format = checked_format(report, format)?;
    checked_bar_size(report, format, options)?;
    Ok(match format {
        Format::Text | Format::Tree => render_text(report, options),
        Format::Paths | Format::Long => render_flat(report, format),
        Format::Json => render_report_machine(report, true, JsonSink::pretty()),
        Format::Jsonl => render_report_jsonl(report),
        Format::Yaml => render_report_machine(report, true, YamlSink::new()),
    })
}

/// One path for a line-oriented listing: control characters become escapes so a row stays
/// one row, and everything else, the separator included, is written as it is.
///
/// Only control characters. This once escaped `\` as well, and on Windows the separator
/// *is* `\`, so `--format paths` printed `c\\target`, a path that does not exist, and the
/// golden that covered it matched the doubled separator instead of failing on it. The
/// price of not escaping it is that a name holding a literal backslash followed by a
/// letter is ambiguous with an escape; the listing is lossy by contract, and a consumer
/// that needs byte identity reads JSON's `path_raw`.
fn flat_path(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .flat_map(|c| if c.is_control() { c.escape_default().collect::<Vec<_>>() } else { vec![c] })
        .collect()
}

/// A compact signed age, the one formatter for the tree's age column and `--long`.
/// Exact nanoseconds remain available in machine output.
///
/// Each unit holds until the next one's length: seconds under a minute, minutes under an
/// hour, hours under a day, days under a display month of 30.44 days, months under a
/// display year of 365.25 days, and years beyond. So no cell reads `0mo` or `0y`, and an
/// old tree reads `12y` rather than `4,382d`. The amount is floored toward zero and the
/// sign kept, so a time ahead of the reference reads `-5m`, and less than a second ahead
/// `-0s`. Below a century a cell is at most four characters and a sign. Months and years
/// are display units only: the age grammar refuses them ([`MONTH_SECONDS`]).
fn human_age(age: Option<i128>) -> String {
    const MINUTE: u128 = 60;
    const HOUR: u128 = 60 * MINUTE;
    const DAY: u128 = 24 * HOUR;
    let Some(age) = age else { return "unknown".to_string() };
    let seconds = age.unsigned_abs() / 1_000_000_000;
    let (month, year) = (u128::from(MONTH_SECONDS), u128::from(YEAR_SECONDS));
    let (amount, unit) = if seconds >= year {
        (seconds / year, "y")
    } else if seconds >= month {
        (seconds / month, "mo")
    } else if seconds >= DAY {
        (seconds / DAY, "d")
    } else if seconds >= HOUR {
        (seconds / HOUR, "h")
    } else if seconds >= MINUTE {
        (seconds / MINUTE, "m")
    } else {
        (seconds, "s")
    };
    format!("{}{}{unit}", if age < 0 { "-" } else { "" }, human_count_u128(amount))
}

/// Notes, the report's warnings, and tips excluded from flat stdout, for a frontend's
/// diagnostic stream.
pub fn flat_diagnostics(report: &Report) -> Vec<String> {
    report_epilogue::with_warnings(report, flat_diagnostic_lines(report))
}

/// Categorized flat-output diagnostics; paths and long rows stay alone on stdout.
///
/// The row bound is already a display-limit note: a flat report has one section.
///
/// A cache-only answer is stated by [`report_warnings`], on every format, so it is not
/// repeated here as a note.
pub fn flat_diagnostic_lines(report: &Report) -> DiagnosticLines {
    let DiagnosticLines { mut notes, tips } = diagnostic_lines(report);
    if !report.status.complete || report.provenance.freshness != Freshness::Fresh {
        notes.push(format!(
            "note: result {}, {}",
            freshness_label(report.provenance.freshness),
            if report.status.complete { "complete" } else { "incomplete" }
        ));
    }
    if let Some(depth) = report.scope.max_depth {
        notes.push(format!(
            "note: scanned to depth {} only; totals cover that scope",
            human_count_u128(depth as u128)
        ));
    }
    DiagnosticLines { notes, tips }
}

/// A row's path as text prints it: after its root's label when the report has several, as
/// `find docs src` prints `docs/guide.md`, and as it is when the report has one root.
///
/// Text alone does this. Machine formats keep the path relative to its root and carry the
/// root's position beside it, so an absolute label never makes a relative path absolute.
fn display_path<'a>(report: &Report, row: &'a FileRow) -> std::borrow::Cow<'a, Path> {
    match report.roots.as_deref().and_then(|roots| roots.get(row.root)) {
        Some(root) => std::borrow::Cow::Owned(root.label.join(&row.path)),
        None => std::borrow::Cow::Borrowed(&row.path),
    }
}

fn render_flat(report: &Report, format: Format) -> String {
    let mut out = String::new();
    for section in &report.sections {
        if let Section::Files { rows, .. } = section {
            for row in rows {
                let path = display_path(report, row);
                if format == Format::Long {
                    let _ = writeln!(
                        out,
                        "{:>10} {:>8} {}",
                        human_bytes(pick(report.size, row.bytes, row.allocated)),
                        human_age(row.age_ns),
                        flat_path(&path)
                    );
                } else {
                    let _ = writeln!(out, "{}", flat_path(&path));
                }
            }
        }
    }
    out
}

fn checked_format(report: &Report, format: Format) -> crate::Result<Format> {
    let format = if format == Format::Text { report.format } else { format };
    let valid = match format {
        Format::Paths | Format::Long => {
            report.sections.len() == 1 && matches!(report.sections[0], Section::Files { .. })
        }
        Format::Tree => {
            report.sections.len() == 1 && matches!(report.sections[0], Section::Tree { .. })
        }
        Format::Text | Format::Json | Format::Jsonl | Format::Yaml => true,
    };
    if !valid {
        return Err(crate::Error::InvalidRequest(crate::query::Rejection::new(format.label(),
            "incompatible with this report projection; request the desired format when building the query (a folded tree cannot become a complete flat list)").on("format")));
    }
    Ok(format)
}

fn checked_bar_size(report: &Report, format: Format, options: RenderOptions) -> crate::Result<()> {
    if matches!(format, Format::Text | Format::Tree)
        && report.sections.iter().any(|section| matches!(section, Section::Tree { .. }))
        && options.bar_size > MAX_BAR_SIZE
    {
        return Err(crate::Error::InvalidRequest(
            crate::query::Rejection::new(
                options.bar_size.to_string(),
                format!("at most {MAX_BAR_SIZE} cells"),
            )
            .on("bar_size"),
        ));
    }
    Ok(())
}

/// Write a report directly to an output stream.
///
/// Machine formats retain only serializer depth while walking the report. Text remains a
/// presentation renderer and is written after it is formatted.
pub fn write(
    report: &Report,
    format: Format,
    color: bool,
    out: &mut dyn io::Write,
) -> io::Result<()> {
    write_with_options(report, format, RenderOptions { color, ..RenderOptions::default() }, out)
}

/// Write with explicit human presentation choices.
///
/// Machine and flat formats retain their existing bytes regardless of `bar_size`.
///
/// # Errors
///
/// Returns an I/O error when writing fails, the format is incompatible with the report,
/// or a human tree bar exceeds [`MAX_BAR_SIZE`].
pub fn write_with_options(
    report: &Report,
    format: Format,
    options: RenderOptions,
    out: &mut dyn io::Write,
) -> io::Result<()> {
    let format = checked_format(report, format)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    checked_bar_size(report, format, options)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    match format {
        Format::Text | Format::Tree => out.write_all(render_text(report, options).as_bytes()),
        Format::Paths | Format::Long => out.write_all(render_flat(report, format).as_bytes()),
        Format::Json => write_report_machine(report, true, JsonSink::pretty_to(out)),
        Format::Jsonl => write_report_jsonl(report, out),
        Format::Yaml => write_report_machine(report, true, YamlSink::to(out)),
    }
}

fn render_report_machine(
    report: &Report,
    with_sections: bool,
    mut sink: impl Sink<Output = String>,
) -> String {
    emit_report(&mut sink, report, with_sections);
    sink.finish()
}

fn write_report_machine<'a>(
    report: &Report,
    with_sections: bool,
    mut sink: impl Sink<Output = IoFmt<'a>>,
) -> io::Result<()> {
    emit_report(&mut sink, report, with_sections);
    sink.finish().finish()
}

fn render_report_jsonl(report: &Report) -> String {
    let mut sink = JsonSink::line();
    emit_report(&mut sink, report, false);
    let mut out = sink.finish();
    out.push('\n');
    for section in &report.sections {
        let mut sink = JsonSink::line();
        emit_section(&mut sink, section, report.roots.is_some());
        out.push_str(&sink.finish());
        out.push('\n');
    }
    out
}

fn write_report_jsonl(report: &Report, out: &mut dyn io::Write) -> io::Result<()> {
    let mut sink = JsonSink::line_to(out);
    emit_report(&mut sink, report, false);
    sink.finish().finish()?;
    out.write_all(b"\n")?;
    for section in &report.sections {
        let mut sink = JsonSink::line_to(out);
        emit_section(&mut sink, section, report.roots.is_some());
        sink.finish().finish()?;
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn emit_field<S: Sink>(sink: &mut S, field: Field, condition: bool, emit: impl FnOnce(&mut S)) {
    let present = match field.presence {
        Presence::Always | Presence::Nullable => true,
        Presence::WhenAnalyzer(_) => {
            panic!("an analyzer-owned field must use emit_analyzer_field")
        }
        Presence::WhenLossy | Presence::WhenSet => condition,
    };
    if present {
        sink.event(Event::Key(field.name));
        emit(sink);
    }
}

fn emit_analyzer_field<S: Sink>(
    sink: &mut S,
    requested: crate::content::AnalysisSet,
    field: Field,
    emit: impl FnOnce(&mut S),
) {
    let Presence::WhenAnalyzer(owner) = field.presence else {
        panic!("an analyzer field must declare its owning unit");
    };
    if requested.contains(owner) {
        sink.event(Event::Key(field.name));
        emit(sink);
    }
}

fn emit_scalar(sink: &mut impl Sink, value: Scalar<'_>) {
    sink.event(Event::Scalar(value));
}

fn emit_report(sink: &mut impl Sink, report: &Report, with_sections: bool) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_field(sink, REPORT_FIELDS.schema, true, |sink| {
        emit_scalar(sink, Scalar::Str(REPORT_SCHEMA));
    });
    let generator = generator();
    emit_field(sink, REPORT_FIELDS.generator, true, |sink| {
        emit_scalar(sink, Scalar::Str(&generator));
    });
    // One root is named here; several are listed in `roots`, and this is null.
    let root = report.root.as_deref().map(Path::to_string_lossy);
    emit_field(sink, REPORT_FIELDS.root, true, |sink| match &root {
        Some(root) => emit_scalar(sink, Scalar::Str(root)),
        None => emit_scalar(sink, Scalar::Null),
    });
    if let Some(root) = &report.root {
        emit_raw_identity(sink, REPORT_FIELDS.root_raw.name, root);
    }
    emit_field(sink, REPORT_FIELDS.roots, report.roots.is_some(), |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for root in report.roots.iter().flatten() {
            sink.event(Event::BeginMap(Shape::Block));
            emit_str_field(sink, "label", &root.label.to_string_lossy());
            emit_raw_identity(sink, "label_raw", &root.label);
            emit_path_fields(sink, &root.path);
            sink.event(Event::EndMap);
        }
        sink.event(Event::EndSeq);
    });
    emit_field(sink, REPORT_FIELDS.age_reference_ns, true, |sink| match report.age_reference_ns {
        Some(value) => emit_scalar(sink, Scalar::I64(value)),
        None => emit_scalar(sink, Scalar::Null),
    });
    emit_field(sink, REPORT_FIELDS.age_reference_at, true, |sink| {
        emit_instant(sink, report.age_reference_ns);
    });
    emit_field(sink, REPORT_FIELDS.request, true, |sink| emit_request(sink, report));
    emit_field(sink, REPORT_FIELDS.status, true, |sink| emit_status(sink, report));
    emit_field(sink, REPORT_FIELDS.provenance, true, |sink| emit_provenance(sink, report));
    let several = report.roots.is_some();
    emit_field(sink, REPORT_FIELDS.ignore_rules, true, |sink| {
        emit_ignore_rules(sink, &report.ignore_rules, several);
    });
    emit_field(sink, REPORT_FIELDS.analysis, true, |sink| {
        emit_analysis(sink, report.analysis.as_ref());
    });
    emit_field(sink, REPORT_FIELDS.reports, with_sections, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for section in &report.sections {
            emit_section(sink, section, several);
        }
        sink.event(Event::EndSeq);
    });
    sink.event(Event::EndMap);
}

fn emit_request(sink: &mut impl Sink, report: &Report) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_field(sink, Field::always("scope"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Block));
        emit_field(sink, Field::nullable("max_depth"), true, |sink| {
            emit_optional_usize(sink, report.scope.max_depth);
        });
        emit_bool_field(sink, "follow_symlinks", report.scope.follow_symlinks);
        emit_bool_field(sink, "one_filesystem", report.scope.one_filesystem);
        emit_bool_field(sink, "exclude_special", report.scope.exclude_special);
        emit_bool_field(sink, "read_controls", report.scope.observes_controls());
        emit_str_field(sink, "population", report.scope.population.label());
        sink.event(Event::EndMap);
    });
    emit_field(sink, Field::always("analyze"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Inline));
        for label in analysis_set_labels(report.requested_analysis) {
            emit_scalar(sink, Scalar::Str(label));
        }
        sink.event(Event::EndSeq);
    });
    emit_str_field(sink, "size", report.size.label());
    emit_field(sink, Field::nullable("sort_metric"), true, |sink| match report.sort_metric {
        Some(name) => emit_scalar(sink, Scalar::Str(name)),
        None => emit_scalar(sink, Scalar::Null),
    });
    emit_field(sink, Field::always("views"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Inline));
        for view in &report.requested_views {
            emit_scalar(sink, Scalar::Str(view.label()));
        }
        sink.event(Event::EndSeq);
    });
    emit_field(sink, Field::always("omitted_views"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Inline));
        for view in &report.omitted_views {
            emit_scalar(sink, Scalar::Str(view.label()));
        }
        sink.event(Event::EndSeq);
    });
    sink.event(Event::EndMap);
}

fn emit_status(sink: &mut impl Sink, report: &Report) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_bool_field(sink, "complete", report.status.complete);
    emit_field(sink, Field::always("coverage"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Inline));
        match report.status.coverage {
            Coverage::Complete => emit_str_field(sink, "kind", "complete"),
            Coverage::Partial(reason) => {
                emit_str_field(sink, "kind", "partial");
                emit_str_field(sink, "reason", structural_coverage_label(reason));
            }
        }
        sink.event(Event::EndMap);
    });
    emit_field(sink, Field::always("errors"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for detail in &report.status.errors {
            let error = &detail.issue;
            sink.event(Event::BeginMap(Shape::Block));
            emit_root_field(sink, detail.root, report.roots.is_some());
            emit_field(sink, Field::when_set("path"), error.path.is_some(), |sink| {
                let path = error.path.as_ref().expect("present error path");
                let display = path.to_string_lossy();
                emit_scalar(sink, Scalar::Str(&display));
            });
            if let Some(path) = &error.path {
                emit_raw_identity(sink, "path_raw", path);
            }
            emit_str_field(sink, "kind", issue_kind_label(error.kind));
            emit_str_field(sink, "message", &error.message);
            emit_field(sink, Field::when_set("os_error"), error.os_error.is_some(), |sink| {
                emit_scalar(sink, Scalar::I64(i64::from(error.os_error.expect("present errno"))));
            });
            sink.event(Event::EndMap);
        }
        sink.event(Event::EndSeq);
    });
    emit_u64_field(sink, "errors_omitted", report.status.errors_omitted);
    sink.event(Event::EndMap);
}

fn emit_provenance(sink: &mut impl Sink, report: &Report) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_str_field(sink, "source", source_label(report.provenance.source));
    emit_str_field(sink, "freshness", freshness_label(report.provenance.freshness));
    emit_field(sink, Field::nullable("scan_started_at"), true, |sink| {
        if let Some(at) = report.provenance.scan_started_at {
            let value = format_rfc3339(at);
            emit_scalar(sink, Scalar::Str(&value));
        } else {
            emit_scalar(sink, Scalar::Null);
        }
    });
    let generated_at = format_rfc3339(report.provenance.generated_at);
    emit_str_field(sink, "generated_at", &generated_at);
    emit_field(sink, Field::always("tiers"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Block));
        emit_field(sink, Field::always("entries"), true, |sink| {
            emit_tier_state(sink, report.provenance.tiers.entries);
        });
        emit_field(sink, Field::nullable("content"), true, |sink| {
            if let Some(content) = report.provenance.tiers.content {
                emit_tier_state(sink, content);
            } else {
                emit_scalar(sink, Scalar::Null);
            }
        });
        sink.event(Event::EndMap);
    });
    sink.event(Event::EndMap);
}

fn emit_tier_state(sink: &mut impl Sink, tier: TierState) {
    sink.event(Event::BeginMap(Shape::Inline));
    emit_str_field(sink, "source", tier_source_label(tier.source));
    emit_str_field(sink, "freshness", freshness_label(tier.freshness));
    emit_field(sink, Field::nullable("observed_at_ns"), true, |sink| match tier.observed_at_ns {
        Some(value) => emit_scalar(sink, Scalar::I64(value)),
        None => emit_scalar(sink, Scalar::Null),
    });
    sink.event(Event::EndMap);
}

/// The position of a row's, an error's, or a refusal's root in the envelope's `roots`,
/// written first and only for a report over several roots, so one root's documents keep
/// their shape.
fn emit_root_field(sink: &mut impl Sink, root: usize, several: bool) {
    emit_field(sink, Field::when_set("root"), several, |sink| {
        emit_scalar(sink, Scalar::U64(root as u64));
    });
}

fn emit_ignore_rules(sink: &mut impl Sink, rules: &ControlCoverage, several: bool) {
    let ControlCoverage::Observed(observed) = rules else {
        emit_scalar(sink, Scalar::Null);
        return;
    };
    sink.event(Event::BeginMap(Shape::Block));
    emit_field(sink, Field::always("limits"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Inline));
        emit_field(sink, Field::nullable("budget"), true, |sink| {
            emit_optional_usize(sink, observed.limits.budget);
        });
        emit_field(sink, Field::nullable("line_limit"), true, |sink| {
            emit_optional_usize(sink, observed.limits.line_limit);
        });
        sink.event(Event::EndMap);
    });
    emit_u64_field(sink, "applied", observed.applied);
    emit_u64_field(sink, "rules", observed.rules);
    emit_u64_field(sink, "refused", observed.refused);
    emit_field(sink, Field::always("refusals"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for refusal in &observed.refusals {
            sink.event(Event::BeginMap(Shape::Block));
            emit_root_field(sink, refusal.root, several);
            emit_path_fields(sink, &refusal.path);
            emit_str_field(sink, "reason", refusal.reason.label());
            sink.event(Event::EndMap);
        }
        sink.event(Event::EndSeq);
    });
    sink.event(Event::EndMap);
}

fn emit_analysis(sink: &mut impl Sink, analysis: Option<&crate::query::ContentReportMetadata>) {
    let Some(analysis) = analysis else {
        emit_scalar(sink, Scalar::Null);
        return;
    };
    sink.event(Event::BeginMap(Shape::Block));
    emit_field(sink, Field::always("analyze"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Inline));
        for label in analysis_set_labels(analysis.profile) {
            emit_scalar(sink, Scalar::Str(label));
        }
        sink.event(Event::EndSeq);
    });
    emit_u64_field(sink, "type_rules_fingerprint", analysis.provenance.type_rules_fingerprint);
    emit_u64_field(sink, "options_fingerprint", analysis.provenance.options_fingerprint.0);
    emit_field(sink, Field::always("analyzers"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for (id, version) in &analysis.provenance.analyzers {
            sink.event(Event::BeginMap(Shape::Inline));
            emit_str_field(sink, "id", id.0);
            emit_u64_field(sink, "version", u64::from(version.0));
            sink.event(Event::EndMap);
        }
        sink.event(Event::EndSeq);
    });
    sink.event(Event::EndMap);
}

fn emit_optional_usize(sink: &mut impl Sink, value: Option<usize>) {
    match value {
        Some(value) => emit_scalar(sink, Scalar::U64(value as u64)),
        None => emit_scalar(sink, Scalar::Null),
    }
}

fn emit_str_field(sink: &mut impl Sink, name: &'static str, value: &str) {
    emit_field(sink, Field::always(name), true, |sink| {
        emit_scalar(sink, Scalar::Str(value));
    });
}

fn emit_u64_field(sink: &mut impl Sink, name: &'static str, value: u64) {
    emit_field(sink, Field::always(name), true, |sink| {
        emit_scalar(sink, Scalar::U64(value));
    });
}

fn emit_bool_field(sink: &mut impl Sink, name: &'static str, value: bool) {
    emit_field(sink, Field::always(name), true, |sink| {
        emit_scalar(sink, Scalar::Bool(value));
    });
}

fn emit_i64_field(sink: &mut impl Sink, name: &'static str, value: i64) {
    emit_field(sink, Field::always(name), true, |sink| {
        emit_scalar(sink, Scalar::I64(value));
    });
}

fn emit_raw_identity(sink: &mut impl Sink, name: &'static str, path: &Path) {
    let raw = raw_os_identity(path.as_os_str());
    emit_field(sink, Field::when_lossy(name), raw.is_some(), |sink| {
        let (encoding, hex) = raw.expect("lossy field predicate checked the raw identity");
        sink.event(Event::BeginMap(Shape::Inline));
        emit_str_field(sink, "encoding", encoding);
        emit_str_field(sink, "hex", &hex);
        sink.event(Event::EndMap);
    });
}

fn emit_path_fields(sink: &mut impl Sink, path: &Path) {
    let lossy = path.to_string_lossy();
    emit_str_field(sink, "path", &lossy);
    emit_raw_identity(sink, "path_raw", path);
}

fn emit_section(sink: &mut impl Sink, section: &Section, several: bool) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_str_field(sink, "view", section.view().label());
    match section {
        Section::Code(overview) => {
            emit_field(sink, Field::always("code"), true, |sink| {
                emit_code_overview(sink, overview);
            });
        }
        Section::Tree { root, omissions, limits, roots, .. } => {
            sink.event(Event::Key("limits"));
            sink.event(Event::BeginMap(Shape::Inline));
            emit_bound_value(sink, "depth", limits.depth);
            emit_str_field(sink, "min_share", &limits.min_share.label());
            emit_bound_value(sink, "breadth", limits.breadth);
            emit_bound_value(sink, "rows", limits.rows);
            sink.event(Event::EndMap);
            emit_field(sink, Field::always("tree"), true, |sink| match root {
                Some(root) => emit_tree(sink, root),
                None => emit_scalar(sink, Scalar::Null),
            });
            // Several roots have no single root node: a total row and a tree per root,
            // each tree carrying its own remainder as one root's section does.
            if let Some(roots) = roots {
                emit_field(sink, Field::when_set("total"), true, |sink| match &roots.total {
                    Some(total) => emit_tree_total(sink, total),
                    None => emit_scalar(sink, Scalar::Null),
                });
                emit_field(sink, Field::when_set("trees"), true, |sink| {
                    sink.event(Event::BeginSeq(Shape::Block));
                    for tree in &roots.trees {
                        sink.event(Event::BeginMap(Shape::Block));
                        emit_u64_field(sink, "root", tree.root as u64);
                        emit_field(sink, Field::always("tree"), true, |sink| {
                            emit_tree(sink, &tree.tree);
                        });
                        emit_field(sink, Field::nullable("remainder"), true, |sink| {
                            emit_tree_remainder(
                                sink,
                                crate::query::TreeRemainder::from_tree(Some(&tree.tree), &[]),
                            );
                        });
                        sink.event(Event::EndMap);
                    }
                    sink.event(Event::EndSeq);
                });
            }
            emit_tree_omissions(sink, omissions);
            sink.event(Event::Key("remainder"));
            emit_tree_remainder(
                sink,
                crate::query::TreeRemainder::from_tree(root.as_deref(), omissions),
            );
        }
        Section::Extensions { rows, total, share_omitted } => {
            emit_bound_field(sink, rows.len(), *total);
            emit_u64_field(sink, "share_omitted", *share_omitted as u64);
            emit_field(sink, Field::always("extensions"), true, |sink| {
                sink.event(Event::BeginSeq(Shape::Block));
                for row in rows {
                    sink.event(Event::BeginMap(Shape::Block));
                    emit_str_field(sink, "extension", &row.extension);
                    emit_u64_field(sink, "files", row.files);
                    emit_u64_field(sink, "bytes", row.bytes);
                    emit_u64_field(sink, "allocated", row.allocated);
                    emit_field(sink, Field::nullable("ignored"), true, |sink| {
                        emit_ignored(sink, row.ignored, false);
                    });
                    sink.event(Event::EndMap);
                }
                sink.event(Event::EndSeq);
            });
        }
        Section::Metrics { summary, .. } => {
            emit_field(sink, Field::always("metrics"), true, |sink| {
                emit_metric_summary(sink, summary);
            });
        }
        Section::Files { rows, total, .. } => {
            emit_bound_field(sink, rows.len(), *total);
            emit_field(sink, Field::always("files"), true, |sink| {
                sink.event(Event::BeginSeq(Shape::Block));
                for row in rows {
                    emit_file_row(sink, row, several);
                }
                sink.event(Event::EndSeq);
            });
        }
        Section::Summary(row) => {
            emit_field(sink, Field::always("summary"), true, |sink| {
                emit_summary_row(sink, row);
            });
        }
    }
    sink.event(Event::EndMap);
}

fn emit_bound_value(sink: &mut impl Sink, field: &'static str, bound: crate::query::Bound) {
    sink.event(Event::Key(field));
    match bound {
        crate::query::Bound::All => emit_scalar(sink, Scalar::Null),
        crate::query::Bound::Limit(value) => emit_scalar(sink, Scalar::U64(value as u64)),
    }
}

fn emit_bound_field(sink: &mut impl Sink, shown: usize, total: usize) {
    emit_field(sink, Field::nullable("bound"), true, |sink| {
        if shown >= total {
            emit_scalar(sink, Scalar::Null);
        } else {
            sink.event(Event::BeginMap(Shape::Inline));
            emit_u64_field(sink, "shown", shown as u64);
            emit_u64_field(sink, "total", total as u64);
            sink.event(Event::EndMap);
        }
    });
}

fn emit_file_row(sink: &mut impl Sink, row: &FileRow, several: bool) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_root_field(sink, row.root, several);
    emit_path_fields(sink, &row.path);
    emit_str_field(sink, "kind", kind_label(row.kind));
    emit_u64_field(sink, "bytes", row.bytes);
    emit_u64_field(sink, "allocated", row.allocated);
    emit_i64_field(sink, "mtime_ns", row.mtime_ns);
    for (name, value) in [("files", row.files), ("dirs", row.dirs)] {
        emit_field(sink, Field::nullable(name), true, |sink| match value {
            Some(value) => emit_scalar(sink, Scalar::U64(value)),
            None => emit_scalar(sink, Scalar::Null),
        });
    }
    emit_field(sink, Field::nullable("complete"), true, |sink| match row.complete {
        Some(value) => emit_scalar(sink, Scalar::Bool(value)),
        None => emit_scalar(sink, Scalar::Null),
    });
    emit_field(sink, Field::nullable("age_ns"), true, |sink| match row.age_ns {
        Some(value) => emit_scalar(sink, Scalar::I128(value)),
        None => emit_scalar(sink, Scalar::Null),
    });
    emit_field(sink, Field::nullable("modified_at"), true, |sink| {
        emit_instant(sink, (row.complete != Some(false)).then_some(row.mtime_ns));
    });

    emit_field(sink, Field::nullable("ignored"), true, |sink| match row.ignored {
        Some(value) => emit_scalar(sink, Scalar::Bool(value)),
        None => emit_scalar(sink, Scalar::Null),
    });
    emit_field(sink, Field::nullable("sort_value"), true, |sink| match row.sort_value {
        Some(value) => emit_scalar(sink, Scalar::U64(value)),
        None => emit_scalar(sink, Scalar::Null),
    });
    emit_field(sink, Field::nullable("classification"), true, |sink| match &row.classification {
        Some(classification) => {
            sink.event(Event::BeginMap(Shape::Block));
            emit_str_field(sink, "file_type", classification.file_type.as_str());
            emit_str_field(sink, "family", classification.family.as_str());
            emit_str_field(sink, "source", classification.source.as_str());
            emit_str_field(sink, "confidence", classification.confidence.as_str());
            emit_field(sink, Field::always("flags"), true, |sink| {
                sink.event(Event::BeginMap(Shape::Inline));
                emit_bool_field(sink, "generated", classification.flags.generated);
                emit_bool_field(sink, "vendored", classification.flags.vendored);
                emit_bool_field(sink, "documentation", classification.flags.documentation);
                sink.event(Event::EndMap);
            });
            sink.event(Event::EndMap);
        }
        None => emit_scalar(sink, Scalar::Null),
    });
    sink.event(Event::EndMap);
}

/// An instant as RFC 3339 UTC to the nanosecond, or null.
///
/// The readable twin of an exact `*_ns` field, never a replacement for it: a row's
/// `modified_at` is null exactly when its time is a lower bound or absent, because a
/// lower-bound maximum rendered as an instant would read as the time it was modified.
fn emit_instant(sink: &mut impl Sink, nanos: Option<i64>) {
    match nanos {
        Some(nanos) => with_rfc3339_nanos(nanos, |text| emit_scalar(sink, Scalar::Str(text))),
        None => emit_scalar(sink, Scalar::Null),
    }
}

fn emit_summary_row(sink: &mut impl Sink, row: &SummaryRow) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_u64_field(sink, "files", row.files);
    emit_u64_field(sink, "dirs", row.dirs);
    emit_u64_field(sink, "bytes", row.bytes);
    emit_u64_field(sink, "allocated", row.allocated);
    emit_field(sink, Field::nullable("ignored"), true, |sink| {
        emit_ignored(sink, row.ignored, true);
    });
    emit_field(sink, Field::nullable("newest_mtime_ns"), true, |sink| match row.newest_mtime_ns {
        Some(value) => emit_scalar(sink, Scalar::I64(value)),
        None => emit_scalar(sink, Scalar::Null),
    });
    sink.event(Event::EndMap);
}

fn emit_ignored(sink: &mut impl Sink, ignored: Option<IgnoredTally>, with_dirs: bool) {
    let Some(ignored) = ignored else {
        emit_scalar(sink, Scalar::Null);
        return;
    };
    sink.event(Event::BeginMap(Shape::Inline));
    emit_u64_field(sink, "files", ignored.files);
    if with_dirs {
        emit_u64_field(sink, "dirs", ignored.dirs);
    }
    emit_u64_field(sink, "bytes", ignored.bytes);
    emit_u64_field(sink, "allocated", ignored.allocated);
    sink.event(Event::EndMap);
}

fn emit_metric_summary(sink: &mut impl Sink, summary: &MetricSummary) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_str_field(sink, "group", metric_group_label(summary.group));
    emit_str_field(sink, "share_metric", summary.share_metric.as_str());
    emit_bound_field(sink, summary.rows.len(), summary.total_rows);
    emit_u64_field(sink, "share_omitted", summary.share_omitted as u64);
    emit_field(sink, Field::always("total"), true, |sink| {
        emit_metric_row(sink, &summary.total, summary.words_per_page);
    });
    emit_field(sink, Field::always("rows"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for row in &summary.rows {
            emit_metric_row(sink, row, summary.words_per_page);
        }
        sink.event(Event::EndSeq);
    });
    sink.event(Event::EndMap);
}

fn emit_code_tally(sink: &mut impl Sink, tally: &CodeTally) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_u64_field(sink, "source_files", tally.source_files);
    emit_u64_field(sink, "analyzed_files", tally.analyzed_files);
    emit_u64_field(sink, "code_lines", tally.metrics.code_lines);
    emit_u64_field(sink, "comment_lines", tally.metrics.comment_lines);
    emit_u64_field(sink, "blank_lines", tally.metrics.code_blank_lines);
    emit_u64_field(sink, "missing_records", tally.missing_records);
    emit_field(sink, Field::always("coverage"), true, |sink| {
        emit_coverage_map(sink, &tally.coverage);
    });
    sink.event(Event::EndMap);
}

fn emit_optional_code_tally(sink: &mut impl Sink, tally: Option<&CodeTally>) {
    match tally {
        Some(tally) => emit_code_tally(sink, tally),
        None => emit_scalar(sink, Scalar::Null),
    }
}

fn emit_code_overview(sink: &mut impl Sink, overview: &CodeOverview) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_str_field(sink, "population", overview.population.label());
    emit_str_field(sink, "share_metric", overview.share_metric.as_str());
    emit_bound_field(sink, overview.languages.len(), overview.total_languages);
    emit_u64_field(sink, "share_omitted", overview.share_omitted as u64);
    emit_u64_field(sink, "analyzed_languages", overview.analyzed_languages);
    emit_u64_field(sink, "unclassified_files", overview.unclassified_files);
    emit_field(sink, Field::always("selected"), true, |sink| {
        emit_code_tally(sink, &overview.selected);
    });
    emit_field(sink, Field::nullable("non_ignored"), true, |sink| {
        emit_optional_code_tally(sink, overview.non_ignored.as_ref());
    });
    emit_field(sink, Field::nullable("ignored"), true, |sink| {
        emit_optional_code_tally(sink, overview.ignored.as_ref());
    });
    emit_field(sink, Field::always("unknown"), true, |sink| {
        emit_code_tally(sink, &overview.unknown);
    });
    emit_field(sink, Field::always("languages"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for row in &overview.languages {
            sink.event(Event::BeginMap(Shape::Block));
            emit_str_field(sink, "language", &row.language);
            emit_field(sink, Field::always("share"), true, |sink| {
                sink.event(Event::BeginMap(Shape::Inline));
                emit_u64_field(sink, "numerator", row.share.numerator);
                emit_u64_field(sink, "denominator", row.share.denominator);
                sink.event(Event::EndMap);
            });
            emit_field(sink, Field::always("selected"), true, |sink| {
                emit_code_tally(sink, &row.selected);
            });
            emit_field(sink, Field::nullable("non_ignored"), true, |sink| {
                emit_optional_code_tally(sink, row.non_ignored.as_ref());
            });
            emit_field(sink, Field::nullable("ignored"), true, |sink| {
                emit_optional_code_tally(sink, row.ignored.as_ref());
            });
            emit_field(sink, Field::always("unknown"), true, |sink| {
                emit_code_tally(sink, &row.unknown);
            });
            sink.event(Event::EndMap);
        }
        sink.event(Event::EndSeq);
    });
    sink.event(Event::EndMap);
}

fn emit_metric_row(sink: &mut impl Sink, row: &MetricRow, words_per_page: u64) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_str_field(sink, "id", &row.id);
    emit_str_field(sink, "family", row.family.as_str());
    emit_u64_field(sink, "files", row.files);
    emit_u64_field(sink, "bytes", row.bytes);
    emit_u64_field(sink, "allocated", row.allocated);
    emit_field(sink, Field::always("share"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Inline));
        emit_u64_field(sink, "numerator", row.share.numerator);
        emit_u64_field(sink, "denominator", row.share.denominator);
        sink.event(Event::EndMap);
    });
    emit_field(sink, Field::always("metrics"), true, |sink| {
        emit_metric_values(sink, row);
    });
    emit_field(sink, Field::always("coverage"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Block));
        emit_analyzer_field(
            sink,
            row.analysis,
            Field::when_analyzer("lines", crate::content::AnalysisSet::LINES_ONLY),
            |sink| emit_coverage_map(sink, &row.lines_coverage),
        );
        emit_analyzer_field(
            sink,
            row.analysis,
            Field::when_analyzer("code", crate::content::AnalysisSet::CODE_ONLY),
            |sink| emit_coverage_map(sink, row.code_coverage.as_ref().expect("code requested")),
        );
        emit_analyzer_field(
            sink,
            row.analysis,
            Field::when_analyzer("words", crate::content::AnalysisSet::WORDS_ONLY),
            |sink| emit_coverage_map(sink, row.words_coverage.as_ref().expect("words requested")),
        );
        sink.event(Event::EndMap);
    });
    emit_field(sink, Field::always("detection"), true, |sink| {
        emit_detection(sink, row);
    });
    emit_analyzer_field(
        sink,
        row.analysis,
        Field::when_analyzer("pages", crate::content::AnalysisSet::WORDS_ONLY),
        |sink| {
            let page = pages(row, words_per_page).expect("words request has page inputs");
            sink.event(Event::BeginMap(Shape::Inline));
            emit_u64_field(sink, "words", page.words);
            emit_u64_field(sink, "words_per_page", page.words_per_page);
            sink.event(Event::EndMap);
        },
    );
    sink.event(Event::EndMap);
}

fn emit_metric_values(sink: &mut impl Sink, row: &MetricRow) {
    sink.event(Event::BeginMap(Shape::Inline));
    for metric in METRICS {
        emit_analyzer_field(
            sink,
            row.analysis,
            Field::when_analyzer(metric.name, metric.owner),
            |sink| {
                emit_scalar(
                    sink,
                    Scalar::U64(row.metric_value(metric).expect("metric owner requested")),
                );
            },
        );
    }
    sink.event(Event::EndMap);
}

fn emit_coverage_map(
    sink: &mut impl Sink,
    coverage: &std::collections::BTreeMap<CoverageReason, u64>,
) {
    sink.event(Event::BeginMap(Shape::Inline));
    for (reason, count) in coverage {
        emit_u64_field(sink, coverage_label(*reason), *count);
    }
    sink.event(Event::EndMap);
}

fn emit_detection(sink: &mut impl Sink, row: &MetricRow) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_field(sink, Field::always("sources"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Inline));
        for (source, count) in &row.detection_sources {
            emit_u64_field(sink, source.as_str(), *count);
        }
        sink.event(Event::EndMap);
    });
    emit_field(sink, Field::always("confidence"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Inline));
        for (level, count) in &row.detection_confidence {
            emit_u64_field(sink, level.as_str(), *count);
        }
        sink.event(Event::EndMap);
    });
    emit_field(sink, Field::always("flags"), true, |sink| {
        sink.event(Event::BeginMap(Shape::Inline));
        emit_u64_field(sink, "generated", row.generated_files);
        emit_u64_field(sink, "vendored", row.vendored_files);
        emit_u64_field(sink, "documentation", row.documentation_files);
        sink.event(Event::EndMap);
    });
    sink.event(Event::EndMap);
}

fn emit_tree(sink: &mut impl Sink, root: &TreeNode) {
    enum Step<'a> {
        Node(&'a TreeNode),
        Children(std::slice::Iter<'a, TreeNode>),
        EndMap,
        EndSeq,
    }
    let mut stack = vec![Step::Node(root)];
    while let Some(step) = stack.pop() {
        match step {
            Step::Node(node) => {
                sink.event(Event::BeginMap(Shape::Block));
                emit_str_field(sink, "name", &node.name);
                emit_path_fields(sink, &node.path);
                emit_str_field(sink, "kind", kind_label(node.kind));
                emit_field(sink, Field::nullable("entry_ignored"), true, |sink| {
                    emit_scalar(sink, node.entry_ignored.map_or(Scalar::Null, Scalar::Bool));
                });
                emit_u64_field(sink, "bytes", node.bytes);
                emit_u64_field(sink, "allocated", node.allocated);
                emit_u64_field(sink, "files", node.files);
                emit_u64_field(sink, "dirs", node.dirs);
                emit_field(sink, Field::nullable("ignored"), true, |sink| {
                    emit_ignored(sink, node.ignored, true);
                });
                emit_field(sink, Field::nullable("newest_mtime_ns"), true, |sink| {
                    match node.newest_mtime_ns {
                        Some(value) => emit_scalar(sink, Scalar::I64(value)),
                        None => emit_scalar(sink, Scalar::Null),
                    }
                });
                emit_field(sink, Field::nullable("mtime_ns"), true, |sink| {
                    emit_scalar(sink, node.mtime_ns.map_or(Scalar::Null, Scalar::I64));
                });
                emit_field(sink, Field::nullable("complete"), true, |sink| {
                    emit_scalar(sink, node.complete.map_or(Scalar::Null, Scalar::Bool));
                });
                emit_field(sink, Field::nullable("age_ns"), true, |sink| {
                    emit_scalar(sink, node.age_ns.map_or(Scalar::Null, Scalar::I128));
                });
                emit_field(sink, Field::nullable("modified_at"), true, |sink| {
                    emit_instant(sink, node.mtime_ns.filter(|_| node.complete != Some(false)));
                });
                emit_field(sink, Field::always("truncated"), true, |sink| {
                    emit_scalar(sink, Scalar::Bool(node.truncated));
                });
                emit_tree_omissions(sink, &node.omissions);
                sink.event(Event::Key("children"));
                sink.event(Event::BeginSeq(Shape::Block));
                stack.push(Step::EndMap);
                stack.push(Step::EndSeq);
                stack.push(Step::Children(node.children.iter()));
            }
            Step::Children(mut children) => {
                if let Some(child) = children.next() {
                    stack.push(Step::Children(children));
                    stack.push(Step::Node(child));
                }
            }
            Step::EndMap => sink.event(Event::EndMap),
            Step::EndSeq => sink.event(Event::EndSeq),
        }
    }
}

/// What the displayed rows do not represent, or null when they represent everything.
fn emit_tree_remainder(sink: &mut impl Sink, remainder: Option<crate::query::TreeRemainder>) {
    let Some(remainder) = remainder else {
        emit_scalar(sink, Scalar::Null);
        return;
    };
    sink.event(Event::BeginMap(Shape::Block));
    for (key, value) in
        [("files", remainder.files), ("bytes", remainder.bytes), ("allocated", remainder.allocated)]
    {
        sink.event(Event::Key(key));
        emit_scalar(sink, value.map_or(Scalar::Null, Scalar::U64));
    }
    sink.event(Event::Key("reasons"));
    sink.event(Event::BeginSeq(Shape::Inline));
    for reason in remainder.reasons {
        emit_scalar(sink, Scalar::Str(reason.label()));
    }
    sink.event(Event::EndSeq);
    sink.event(Event::EndMap);
}

/// The total row of a tree over several roots: a tree node's values, in a tree node's
/// order, without the identity of an entry, since no entry is the total.
fn emit_tree_total(sink: &mut impl Sink, total: &crate::query::TreeTotal) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_u64_field(sink, "bytes", total.bytes);
    emit_u64_field(sink, "allocated", total.allocated);
    emit_u64_field(sink, "files", total.files);
    emit_u64_field(sink, "dirs", total.dirs);
    emit_field(sink, Field::nullable("ignored"), true, |sink| {
        emit_ignored(sink, total.ignored, true);
    });
    emit_field(sink, Field::nullable("newest_mtime_ns"), true, |sink| {
        emit_scalar(sink, total.newest_mtime_ns.map_or(Scalar::Null, Scalar::I64));
    });
    emit_field(sink, Field::nullable("mtime_ns"), true, |sink| {
        emit_scalar(sink, total.mtime_ns.map_or(Scalar::Null, Scalar::I64));
    });
    emit_bool_field(sink, "complete", total.complete);
    emit_field(sink, Field::nullable("age_ns"), true, |sink| {
        emit_scalar(sink, total.age_ns.map_or(Scalar::Null, Scalar::I128));
    });
    emit_field(sink, Field::nullable("modified_at"), true, |sink| {
        emit_instant(sink, total.mtime_ns.filter(|_| total.complete));
    });
    sink.event(Event::EndMap);
}

fn emit_tree_omissions(sink: &mut impl Sink, omissions: &[crate::query::TreeOmission]) {
    sink.event(Event::Key("omissions"));
    sink.event(Event::BeginSeq(Shape::Block));
    for omission in omissions {
        sink.event(Event::BeginMap(Shape::Inline));
        emit_str_field(sink, "reason", omission.reason.label());
        emit_u64_field(sink, "entries", omission.entries as u64);
        sink.event(Event::Key("files"));
        emit_scalar(sink, omission.files.map_or(Scalar::Null, Scalar::U64));
        sink.event(Event::Key("bytes"));
        match omission.bytes {
            Some(value) => emit_scalar(sink, Scalar::U64(value)),
            None => emit_scalar(sink, Scalar::Null),
        }
        sink.event(Event::Key("allocated"));
        match omission.allocated {
            Some(value) => emit_scalar(sink, Scalar::U64(value)),
            None => emit_scalar(sink, Scalar::Null),
        }
        sink.event(Event::EndMap);
    }
    sink.event(Event::EndSeq);
}

/// The report envelope's ordered field and presence contract.
struct ReportFields {
    schema: Field,
    generator: Field,
    root: Field,
    root_raw: Field,
    roots: Field,
    request: Field,
    status: Field,
    provenance: Field,
    ignore_rules: Field,
    analysis: Field,
    reports: Field,
    age_reference_ns: Field,
    age_reference_at: Field,
}

const REPORT_FIELDS: ReportFields = ReportFields {
    schema: Field::always("schema"),
    generator: Field::always("generator"),
    // Null for a report over several roots, which `roots` names instead.
    root: Field::nullable("root"),
    root_raw: Field::when_lossy("root_raw"),
    roots: Field::when_set("roots"),
    request: Field::always("request"),
    status: Field::always("status"),
    provenance: Field::always("provenance"),
    ignore_rules: Field::always("ignore_rules"),
    analysis: Field::nullable("analysis"),
    reports: Field::when_set("reports"),
    age_reference_ns: Field::nullable("age_reference_ns"),
    age_reference_at: Field::nullable("age_reference_at"),
};

/// Why a field is present in a wire document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Presence {
    Always,
    Nullable,
    WhenLossy,
    WhenAnalyzer(crate::content::AnalysisSet),
    WhenSet,
}

/// One declared field in a machine-output schema.
#[derive(Clone, Copy, Debug)]
struct Field {
    name: &'static str,
    presence: Presence,
}

impl Field {
    const fn always(name: &'static str) -> Self {
        Self { name, presence: Presence::Always }
    }

    const fn nullable(name: &'static str) -> Self {
        Self { name, presence: Presence::Nullable }
    }

    const fn when_lossy(name: &'static str) -> Self {
        Self { name, presence: Presence::WhenLossy }
    }

    const fn when_analyzer(name: &'static str, analysis: crate::content::AnalysisSet) -> Self {
        Self { name, presence: Presence::WhenAnalyzer(analysis) }
    }

    const fn when_set(name: &'static str) -> Self {
        Self { name, presence: Presence::WhenSet }
    }
}

/// Wrap text in a style when colour is on.
pub fn paint(text: &str, style: AnsiStyle, color: bool) -> String {
    if color { format!("{style}{text}{style:#}") } else { text.to_string() }
}

/// Escape controls before styling so an entry cannot move a cursor or add a row.
pub fn escaped_human(text: &str) -> String {
    text.chars()
        .flat_map(|c| if c.is_control() { c.escape_default().collect::<Vec<_>>() } else { vec![c] })
        .collect()
}

/// The shared human byte color role: zero is gray; large values are bold, including gray details.
/// The threshold uses exact bytes, not the rounded display unit.
pub fn byte_style(bytes: u64, secondary: bool) -> AnsiStyle {
    let style = if secondary || bytes == 0 { STYLE_DETAIL } else { AnsiStyle::new() };
    if bytes >= 1 << 30 { style.bold() } else { style }
}

/// Render a human byte quantity after padding with the shared size and emphasis rules.
/// Plain and structured numbers never depend on styling.
pub fn styled_bytes(bytes: u64, width: usize, color: bool, secondary: bool) -> String {
    let style = byte_style(bytes, secondary);
    let text = format!("{:>width$}", human_bytes(bytes));
    if bytes > 0 && bytes < 1 << 30 && !secondary { text } else { paint(&text, style, color) }
}

/// Style a root share using its exact ratio, before display rounding.
fn percentage_cell(part: u64, whole: u64, decimals: usize, width: usize, color: bool) -> String {
    let text = format!("{:>width$}", human_percentage(part, whole, decimals));
    if whole > 0 && u128::from(part) * 100 < u128::from(whole) {
        detail(&text, color)
    } else {
        text
    }
}

/// Human directory markers are presentation only, never part of structured paths.
fn human_name(name: &str, kind: EntryKind, ignored: Option<bool>, color: bool) -> String {
    // Own classification matters: a directory merely containing ignored files keeps
    // the primary style, even if every selected descendant happens to be ignored.
    let style = if kind == EntryKind::Dir && ignored == Some(true) {
        STYLE_IGNORED_NAME
    } else {
        STYLE_NAME
    };
    // A root's label can already end in a separator: `/`, or a drive root such as `C:\`,
    // which would otherwise print as `C:\/`. No entry's own name can.
    let slash = kind == EntryKind::Dir
        && !matches!(name, "." | "..")
        && !name.ends_with(['/', std::path::MAIN_SEPARATOR]);
    format!(
        "{}{}",
        paint(&escaped_human(name), style, color),
        if slash { detail("/", color) } else { String::new() }
    )
}

fn detail(text: &str, color: bool) -> String {
    paint(text, STYLE_DETAIL, color)
}

// ---- text ----

/// Render the human-facing form.
///
/// A multi-view report introduces each section with an all-caps header naming its view,
/// blocks separated by a blank line. Machine formats already carry a `view` field on
/// every report, so text was the only format that lost the labelling: several tables of
/// similar-looking rows arrived concatenated, and the reader had to work out which view
/// each block came from by remembering the order they were requested in.
///
/// A single-view report is left bare, which keeps `fdu --view files` a listing of
/// paths and nothing else. Paths escape control characters for display; use a structured
/// format when arbitrary native filenames must be consumed without loss.
/// One block needs no label to be
/// unambiguous, so the header appears precisely when it disambiguates something.
fn render_text(report: &Report, options: RenderOptions) -> String {
    let color = options.color;
    let mut out = String::new();
    let headed = report.sections.len() > 1;
    for (index, section) in report.sections.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        // A single-view report has no header; its row bound is a display-limit note in
        // the epilogue rather than a line of its own above the result.
        if headed {
            let _ = writeln!(
                out,
                "{}{}",
                paint(view_header(section.view()), STYLE_HEADING, color),
                paint(&bound_note(section), STYLE_DETAIL, color),
            );
        }
        match section {
            Section::Code(overview) => render_text_code(&mut out, overview, color),
            Section::Tree { root, omissions, roots, .. } => {
                let tree =
                    TextTree { size: report.size, selected: report.ignored_entries, options };
                match roots {
                    Some(roots) => tree.render_roots(&mut out, roots, omissions),
                    None => tree.render_root(&mut out, root.as_deref(), omissions),
                }
            }
            Section::Extensions { rows, .. } => {
                render_text_types(&mut out, rows, report.size, report.ignored_entries, color);
            }
            Section::Metrics { view, summary } => {
                render_text_metrics(&mut out, *view, summary, report.size, color);
            }
            // `files` stays one path per line: it is the enumeration, and a bare list is
            // what pipes into xargs. The bounded presets are summaries, and a summary
            // that ranks by something must show that something — "the twenty largest"
            // with no sizes does not answer the question it is named for, and leaves the
            // ranking unverifiable.
            // The ranking measure is named by an epilogue note.
            Section::Files { rows, .. } if report.sort_metric.is_some() => {
                render_text_metric_files(&mut out, report, rows, color);
            }
            Section::Files { view, rows, .. } => match view {
                ViewSpec::Largest => {
                    let size = Some(report.size);
                    render_text_ranked_files(&mut out, report, rows, color, size, |row| {
                        human_bytes(pick(report.size, row.bytes, row.allocated))
                    });
                }
                ViewSpec::Recent => {
                    render_text_ranked_files(&mut out, report, rows, color, None, |row| {
                        format_rfc3339_nanos(row.mtime_ns)
                    });
                }
                _ => {
                    for row in rows {
                        let path = display_path(report, row);
                        let _ = writeln!(out, "{}", escaped_human(&path.to_string_lossy()));
                    }
                }
            },
            Section::Summary(row) => {
                render_text_summary(&mut out, row, report.size, report.ignored_entries, color);
            }
        }
    }
    out
}

// ---- the layout rules -----------------------------------------------------------------
//
// One contract every grouped row follows, so the four grouped views line up as one table
// rather than four:
//
//   size    right-aligned, width 10
//   share   right-aligned, width 6
//   label   left-aligned, padded to the section's widest label
//   files   the file count, after a single space, which ends the row's first line
//
// Every further measure a row carries is a continuation line of its own below it, in a
// fixed order: lines with their breakdown, words with pages, generated and vendored
// counts, then each coverage reason other than analyzed. A continuation leaves the size,
// share, and label cells blank and starts `TEXT_CONTINUATION_INDENT` columns right of the
// file count, so it reads as a breakdown of that tally, and every continuation in a
// section starts at one column. A row is then as wide as its widest measure rather than
// the sum of them: joined on one line, the Linux kernel's DOCUMENTS rows reached 154
// columns. A row with nothing past its file count stays one line.
//
// The documentation-file count is machine output only (`documentation_files`). It
// counts files under a doc, docs, or documentation directory or named like a README,
// CHANGELOG, or CONTRIBUTING file, which in DOCUMENTS is nearly every file, and a line
// under a document row would read as a measure of the documents themselves.
//
// The rule that is easy to get wrong: **a column's width is measured on visible text**.
// `paint` wraps its argument in escape sequences, and a width specifier counts those
// toward the field, so `{:<12}` on a painted label is already full before a single visible
// character lands and the padding silently collapses. `label_cell` is the only sanctioned
// way to lay out a styled label: it measures the plain text and appends the padding
// outside the paint. Never hand a painted string to `{:<N}` or `{:>N}`.

/// The size column: right-aligned in a fixed width, never styled.
const TEXT_SIZE_WIDTH: usize = 10;
/// The share column: right-aligned in a fixed width, never styled.
const TEXT_SHARE_WIDTH: usize = 6;
/// How far a grouped row's continuation lines start right of its file count.
const TEXT_CONTINUATION_INDENT: usize = 2;

/// A styled label padded to `width`, measured on the visible text.
fn label_cell(label: &str, width: usize, style: AnsiStyle, color: bool) -> String {
    let padding = " ".repeat(width.saturating_sub(display_width(label)));
    format!("{}{padding}", paint(label, style, color))
}

/// The widest visible label in a set of rows, floored at `minimum`.
fn label_width<'a>(labels: impl Iterator<Item = &'a str>, minimum: usize) -> usize {
    minimum.max(labels.map(|label| display_width(&escaped_human(label))).max().unwrap_or_default())
}

/// Terminal columns occupied by an already escaped, unstyled label.
///
/// Common zero-width combining marks and East Asian wide characters need different
/// treatment from Unicode scalar counts when columns are padded for a terminal.
fn display_width(text: &str) -> usize {
    text.chars()
        .map(|c| {
            let point = c as u32;
            if (0x0300..=0x036f).contains(&point)
                || (0x1ab0..=0x1aff).contains(&point)
                || (0x1dc0..=0x1dff).contains(&point)
                || (0x20d0..=0x20ff).contains(&point)
                || (0xfe20..=0xfe2f).contains(&point)
            {
                0
            } else if (0x1100..=0x115f).contains(&point)
                || (0x2e80..=0xa4cf).contains(&point)
                || (0xac00..=0xd7a3).contains(&point)
                || (0xf900..=0xfaff).contains(&point)
                || (0xfe10..=0xfe19).contains(&point)
                || (0xfe30..=0xfe6f).contains(&point)
                || (0xff01..=0xff60).contains(&point)
                || (0xffe0..=0xffe6).contains(&point)
                || (0x1f300..=0x1faff).contains(&point)
                || (0x20000..=0x3fffd).contains(&point)
            {
                2
            } else {
                1
            }
        })
        .sum()
}

fn human_percentage(part: u64, whole: u64, decimals: usize) -> String {
    if whole == 0 {
        return "—".to_string();
    }
    let scale = match decimals {
        0 => 100_u128,
        1 => 1_000_u128,
        _ => panic!("human percentage supports whole and tenths only"),
    };
    // Test the less-than label exactly; float conversion can round a value just
    // below 1% (or 0.1%) onto the boundary when the byte totals exceed 2^53.
    if part > 0 && u128::from(part) * scale < u128::from(whole) {
        return if decimals == 0 {
            "<1%".to_string()
        } else {
            format!("<0.{}1%", "0".repeat(decimals - 1))
        };
    }
    format!("{:.*}%", decimals, ratio(part, whole) * 100.0)
}

/// One grouped section: TYPES, FAMILIES, LANGUAGES, or DOCUMENTS.
///
/// Each row's first line is its size, share, label, and file count; every further
/// measure it carries is a continuation line of its own, as the layout rules above say.
fn render_text_metrics(
    out: &mut String,
    view: ViewSpec,
    summary: &MetricSummary,
    size: SizeMetric,
    color: bool,
) {
    // Languages pad one past the longest name; the other groupings share a floor so
    // separate sections still line up with one another.
    let width = if view == ViewSpec::Languages {
        label_width(summary.rows.iter().map(|row| human_metric_label(view, &row.id)), 0)
            .saturating_add(1)
    } else {
        label_width(
            summary.rows.iter().map(|row| human_metric_label(view, &row.id)),
            TEXT_METRIC_LABEL_WIDTH,
        )
    };
    // The file count starts one space past the size, share, and label cells, whose
    // visible widths are fixed for the section, so every continuation shares one column.
    let continuation = " "
        .repeat(TEXT_SIZE_WIDTH + 2 + TEXT_SHARE_WIDTH + 2 + width + 1 + TEXT_CONTINUATION_INDENT);
    for row in &summary.rows {
        let selected = pick(size, row.bytes, row.allocated);
        let percentage =
            percentage_cell(row.share.numerator, row.share.denominator, 1, TEXT_SHARE_WIDTH, color);
        let mut measures = Vec::new();
        if let Some(physical_lines) = row.metrics.physical_lines.filter(|lines| *lines > 0) {
            let code_fully_analyzed = row.code_coverage.as_ref().is_some_and(|coverage| {
                coverage.len() == 1 && coverage.get(&CoverageReason::Analyzed) == Some(&row.files)
            });
            let breakdown =
                if let (true, Some(code_lines), Some(comment_lines), Some(code_blank_lines)) = (
                    code_fully_analyzed,
                    row.metrics.code_lines,
                    row.metrics.comment_lines,
                    row.metrics.code_blank_lines,
                ) {
                    format!(
                        "({} code, {} comment, {} blank)",
                        human_count(code_lines),
                        human_count(comment_lines),
                        human_count(code_blank_lines)
                    )
                } else {
                    format!(
                        "({} nonblank, {} blank)",
                        human_count(row.metrics.nonblank_lines.expect("lines requested")),
                        human_count(row.metrics.blank_lines.expect("lines requested"))
                    )
                };
            measures.push(format!(
                "{} lines {}",
                human_count(physical_lines),
                detail(&breakdown, color)
            ));
        }
        if let Some(page) = pages(row, summary.words_per_page).filter(|page| page.words > 0) {
            let page_tenths = page.words.saturating_mul(10) / page.words_per_page;
            measures.push(format!(
                "{} words {}",
                human_count(page.words),
                detail(
                    &format!("({}.{:01} pages)", human_count(page_tenths / 10), page_tenths % 10),
                    color,
                )
            ));
        }
        // `documentation_files` stays out of human rows; see the layout rules above.
        for (count, flag) in [(row.generated_files, "generated"), (row.vendored_files, "vendored")]
        {
            if count > 0 {
                measures.push(format!("{} {flag}", human_count(count)));
            }
        }
        let coverage = match view {
            ViewSpec::Languages => row.code_coverage.as_ref().unwrap_or(&row.lines_coverage),
            ViewSpec::Documents => row.words_coverage.as_ref().unwrap_or(&row.lines_coverage),
            _ => &row.lines_coverage,
        };
        for (reason, count) in coverage {
            if *reason != CoverageReason::Analyzed {
                measures.push(format!("{} {}", human_count(*count), human_coverage_label(*reason)));
            }
        }
        let _ = writeln!(
            out,
            "{}  {}  {} {} {}",
            styled_bytes(selected, TEXT_SIZE_WIDTH, color, false),
            percentage,
            label_cell(
                &escaped_human(human_metric_label(view, &row.id)),
                width,
                STYLE_CATEGORY,
                color
            ),
            human_count(row.files),
            plural(row.files, "file", "files"),
        );
        for measure in measures {
            let _ = writeln!(out, "{continuation}{measure}");
        }
    }
}

/// Keep measured zero distinct from missing measurements in every numeric column.
/// Language display bounds trim rows, never the TOTAL row or its global denominator.
fn render_text_code(out: &mut String, overview: &CodeOverview, color: bool) {
    let selected = &overview.selected;
    let code_width = human_count(selected.metrics.code_lines).len().max("Code lines".len());
    let comment_width = human_count(selected.metrics.comment_lines).len().max("Comments".len());
    let blank_width = human_count(selected.metrics.code_blank_lines).len().max("Blank".len());
    let analyzed_width =
        format!("{}/{}", human_count(selected.analyzed_files), human_count(selected.source_files))
            .len()
            .max("Analyzed files".len());
    let language_width = label_width(
        overview.languages.iter().map(|row| human_language_name(&row.language)),
        "Language".len(),
    );
    let _ = writeln!(
        out,
        "{}  {}  {}  {}  {}  {}",
        paint(&format!("{:>code_width$}", "Code lines"), STYLE_CATEGORY, color),
        paint(&format!("{:>6}", "Share"), STYLE_CATEGORY, color),
        paint(&format!("{:>comment_width$}", "Comments"), STYLE_CATEGORY, color),
        paint(&format!("{:>blank_width$}", "Blank"), STYLE_CATEGORY, color),
        paint(&format!("{:>analyzed_width$}", "Analyzed files"), STYLE_CATEGORY, color),
        paint("Language", STYLE_CATEGORY, color),
    );
    for row in &overview.languages {
        let measured = row.selected.analyzed_files > 0;
        // The row's value already includes every population, so the parenthetical names
        // only the gitignored share (and any unknown share), never its complement.
        let mut annotation = String::new();
        if let (true, Some(ignored)) = (measured, &row.ignored) {
            let _ = write!(annotation, "{} gitignored", human_count(ignored.metrics.code_lines));
        }
        if measured && row.unknown.source_files > 0 {
            let _ = write!(
                annotation,
                "{}{} unknown",
                if annotation.is_empty() { "" } else { ", " },
                human_count(row.unknown.metrics.code_lines)
            );
        }
        let annotation = if annotation.is_empty() {
            String::new()
        } else {
            format!(" {}", detail(&format!("({annotation})"), color))
        };
        let analyzed = format!(
            "{}/{}",
            human_count(row.selected.analyzed_files),
            human_count(row.selected.source_files)
        );
        let _ = writeln!(
            out,
            "{}  {}  {}  {}  {:>analyzed_width$}  {}{}",
            code_cell(row.selected.metrics.code_lines, measured, code_width, color),
            if measured {
                percentage_cell(row.share.numerator, row.share.denominator, 1, 6, color)
            } else {
                detail(&format!("{:>6}", "—"), color)
            },
            code_cell(row.selected.metrics.comment_lines, measured, comment_width, color),
            code_cell(row.selected.metrics.code_blank_lines, measured, blank_width, color),
            analyzed,
            label_cell(
                human_language_name(&row.language),
                if annotation.is_empty() { 0 } else { language_width },
                STYLE_NAME,
                color
            ),
            annotation
        );
    }
    let mut total_detail = String::new();
    if let (true, Some(ignored)) = (selected.analyzed_files > 0, &overview.ignored) {
        let _ = write!(total_detail, "{} gitignored", human_count(ignored.metrics.code_lines));
    }
    if selected.analyzed_files > 0 && overview.unknown.source_files > 0 {
        let _ = write!(
            total_detail,
            "{}{} unknown",
            if total_detail.is_empty() { "" } else { ", " },
            human_count(overview.unknown.metrics.code_lines)
        );
    }
    let total_detail = if total_detail.is_empty() {
        String::new()
    } else {
        format!(" {}", detail(&format!("({total_detail})"), color))
    };
    let bold = AnsiStyle::new().bold();
    let total_share = if selected.analyzed_files > 0 {
        percentage_cell(selected.metrics.code_lines, selected.metrics.code_lines, 1, 6, false)
    } else {
        format!("{:>6}", "—")
    };
    let total_analyzed = format!(
        "{:>analyzed_width$}",
        format!("{}/{}", human_count(selected.analyzed_files), human_count(selected.source_files))
    );
    let _ = writeln!(
        out,
        "{}  {}  {}  {}  {}  {}{}",
        code_total_cell(
            selected.metrics.code_lines,
            selected.analyzed_files > 0,
            code_width,
            color
        ),
        paint(&total_share, bold, color),
        code_total_cell(
            selected.metrics.comment_lines,
            selected.analyzed_files > 0,
            comment_width,
            color
        ),
        code_total_cell(
            selected.metrics.code_blank_lines,
            selected.analyzed_files > 0,
            blank_width,
            color
        ),
        paint(&total_analyzed, bold, color),
        paint(
            &format!(
                "{:<width$}",
                "TOTAL",
                width = if total_detail.is_empty() { 0 } else { language_width }
            ),
            bold,
            color
        ),
        total_detail
    );
    // Coverage facts (analyzed languages, unclassified and unanalyzed files) are notes in
    // the epilogue, not lines below the table.
}

fn code_cell(value: u64, measured: bool, width: usize, color: bool) -> String {
    if measured {
        format!("{:>width$}", human_count(value))
    } else {
        detail(&format!("{:>width$}", "—"), color)
    }
}

fn code_total_cell(value: u64, measured: bool, width: usize, color: bool) -> String {
    let text = if measured { human_count(value) } else { "—".to_owned() };
    paint(&format!("{text:>width$}"), AnsiStyle::new().bold(), color)
}

/// The denominator of a percentage column that is not the byte column beside it.
///
/// Byte shares need no annotation because the adjacent size column already names their
/// numerator. Code and document reports deliberately rank by a content metric while
/// retaining bytes in the first column, so leaving the percentage unexplained makes two
/// unlike quantities look as though they must agree; the epilogue names it in a note.
pub(super) fn share_metric_label(metric: ShareMetric) -> Option<&'static str> {
    match metric {
        ShareMetric::CodeLines => Some("code lines"),
        ShareMetric::DocumentWords => Some("document words"),
        ShareMetric::RawWords => Some("raw words"),
        ShareMetric::ApparentBytes | ShareMetric::AllocatedBytes => None,
    }
}

fn human_metric_label(view: ViewSpec, id: &str) -> &str {
    if view == ViewSpec::Languages { human_language_name(id) } else { id }
}

pub(super) fn human_coverage_label(reason: CoverageReason) -> &'static str {
    match reason {
        CoverageReason::Analyzed => "analyzed",
        CoverageReason::Binary => "binary",
        CoverageReason::InvalidUtf8 => "invalid UTF-8",
        CoverageReason::UnsupportedEncoding => "unsupported encoding",
        CoverageReason::Unsupported => "unsupported",
        CoverageReason::IoError => "I/O error",
        CoverageReason::ChangedDuringRead => "changed during read",
        CoverageReason::TextOnly => "counted as text",
    }
}

/// The gitignored subset a text row ends with, as ` (128 B gitignored)`, or nothing.
/// This amount is already included in the row total, not additional usage.
///
/// One placement for every row that carries a share: after the row's own detail, so the
/// fixed size, bar, and percentage columns keep their alignment. Nothing is appended when
/// no file is ignored, when the index observed no control state, or when the selection
/// admitted only ignored entries, where the share would repeat the row's size. A share of
/// ignored directories alone holds no bytes, and `(0 B gitignored)` would say nothing a
/// reader can act on; the machine formats still count them. Text cannot tell "nothing
/// ignored" from "no rules read"; the performance line says whether any rule was read,
/// and machine formats carry a zero share and `null` respectively.
fn ignored_suffix(
    ignored: Option<IgnoredTally>,
    size: SizeMetric,
    selected: IgnoredEntries,
    color: bool,
) -> String {
    let shown = match selected {
        IgnoredEntries::Include | IgnoredEntries::Exclude => {
            ignored.filter(|share| share.files > 0)
        }
        IgnoredEntries::Only => None,
    };
    shown.map_or_else(String::new, |share| {
        let bytes = pick(size, share.bytes, share.allocated);
        if bytes < 1 << 30 || !color {
            format!(" {}", detail(&format!("({} gitignored)", human_bytes(bytes)), color))
        } else {
            format!(
                " {}{}{}",
                detail("(", color),
                styled_bytes(bytes, 0, color, true),
                detail(" gitignored)", color)
            )
        }
    })
}

/// How a tree section's rows are laid out: fixed bar, percentage, size, and age columns,
/// then the name indented by depth.
///
/// Iterative for the same reason the expansion is: a deep tree must render, not panic.
struct TextTree {
    /// The size each row shows and its share is of.
    size: SizeMetric,
    /// Which entries the rows count, which decides whether an ignored share is shown.
    selected: IgnoredEntries,
    /// Color and bar width.
    options: RenderOptions,
}

/// Every row's age cell, in the order the rows print, and the column's width.
///
/// The age column is as wide as the section's widest cell, so a first pass formats every
/// row's cell and the rows reuse them rather than formatting each age twice (review C8 on
/// #191). A section with no rows has no column.
struct AgeCells {
    cells: std::vec::IntoIter<(String, Option<AnsiStyle>)>,
    width: usize,
}

impl AgeCells {
    /// The cells of the given rows, in order: each a total row's cell or a whole tree's.
    fn of<'a>(rows: impl IntoIterator<Item = AgeRows<'a>>) -> Self {
        let mut cells = Vec::new();
        for rows in rows {
            match rows {
                AgeRows::Cell(cell) => cells.push(cell),
                AgeRows::Tree(root) => {
                    let mut stack = vec![root];
                    while let Some(node) = stack.pop() {
                        cells.push(tree_age_cell(node));
                        stack.extend(node.children.iter().rev());
                    }
                }
            }
        }
        let width = cells.iter().map(|(age, _)| display_width(age)).max().unwrap_or(0);
        Self { cells: cells.into_iter(), width }
    }

    /// The next row's cell.
    fn next(&mut self) -> (String, Option<AnsiStyle>) {
        self.cells.next().expect("one age cell per row, in row order")
    }
}

/// Rows whose age cells [`AgeCells::of`] formats: one cell, or a tree's rows in pre-order.
enum AgeRows<'a> {
    Cell((String, Option<AnsiStyle>)),
    Tree(&'a TreeNode),
}

impl TextTree {
    /// The tree of a report over one root: its rows, then one remainder line at the
    /// highest displayed level for what the bounds hid, or for the root itself when the
    /// row limit is zero.
    fn render_root(
        &self,
        out: &mut String,
        root: Option<&TreeNode>,
        omissions: &[crate::query::TreeOmission],
    ) {
        let hidden = crate::query::TreeRemainder::from_tree(root, omissions);
        let mut ages = AgeCells::of(root.map(AgeRows::Tree));
        let age_width = ages.width;
        let grand = root.map(|root| pick(self.size, root.bytes, root.allocated));
        if let (Some(root), Some(grand)) = (root, grand) {
            self.render_rows(out, root, 0, grand, &mut ages);
        }
        // One annotation at the highest displayed level, even when several independent
        // bounds hide descendants at different depths. Reasons belong in the epilogue.
        if let Some(hidden) = hidden {
            let depth = usize::from(root.is_some());
            self.render_remainder(out, &hidden, grand, depth, age_width);
        }
    }

    /// The tree of a report over several roots: the total row, then each root's tree one
    /// level in, each followed by its own remainder, then one line for the root rows the
    /// row limit cut. Every share is of the total.
    ///
    /// With no total, under a zero row limit, the only line is the remainder for every
    /// root, at the top level and with no denominator, as one root's is.
    fn render_roots(
        &self,
        out: &mut String,
        roots: &crate::query::RootTrees,
        omissions: &[crate::query::TreeOmission],
    ) {
        // One column over the whole section: the total row and every root's tree.
        let total_age =
            roots.total.map(|total| age_cell(Some(total.complete), total.mtime_ns, total.age_ns));
        let mut ages = AgeCells::of(
            total_age
                .into_iter()
                .map(AgeRows::Cell)
                .chain(roots.trees.iter().map(|tree| AgeRows::Tree(&tree.tree))),
        );
        let age_width = ages.width;
        let grand = roots.total.map(|total| pick(self.size, total.bytes, total.allocated));
        if let (Some(total), Some(grand)) = (roots.total, grand) {
            let name = paint("(total)", STYLE_NAME, self.options.color);
            let line = TextRow {
                bytes: grand,
                ignored: total.ignored,
                age: ages.next(),
                name,
                files: Some(total.files),
            };
            self.render_row(out, &line, grand, 0, age_width);
        }
        let depth = usize::from(roots.total.is_some());
        for tree in &roots.trees {
            let grand = grand.expect("a root tree is shown only below the total row");
            self.render_rows(out, &tree.tree, depth, grand, &mut ages);
            if let Some(hidden) = crate::query::TreeRemainder::from_tree(Some(&tree.tree), &[]) {
                self.render_remainder(out, &hidden, Some(grand), depth + 1, age_width);
            }
        }
        if let Some(hidden) = crate::query::TreeRemainder::from_tree(None, omissions) {
            self.render_remainder(out, &hidden, grand, depth, age_width);
        }
    }

    /// A tree's rows in pre-order, its root at `depth`, each sharing `grand`, each taking
    /// its age cell from `ages` in turn.
    fn render_rows(
        &self,
        out: &mut String,
        root: &TreeNode,
        depth: usize,
        grand: u64,
        ages: &mut AgeCells,
    ) {
        let color = self.options.color;
        let mut stack = vec![(root, depth)];
        while let Some((node, depth)) = stack.pop() {
            let line = TextRow {
                bytes: pick(self.size, node.bytes, node.allocated),
                ignored: node.ignored,
                age: ages.next(),
                name: human_name(&node.name, node.kind, node.entry_ignored, color),
                files: (node.kind != EntryKind::File).then_some(node.files),
            };
            self.render_row(out, &line, grand, depth, ages.width);
            stack.extend(node.children.iter().rev().map(|child| (child, depth + 1)));
        }
    }

    /// One row: bar, share of `grand`, size, age, then the name at `depth`.
    fn render_row(
        &self,
        out: &mut String,
        row: &TextRow,
        grand: u64,
        depth: usize,
        age_width: usize,
    ) {
        let RenderOptions { color, bar_size } = self.options;
        let size = self.size;
        let indent = "  ".repeat(depth);
        let count = row.files.map_or_else(String::new, |files| {
            format!(" {} {}", human_count(files), plural(files, "file", "files"))
        });
        let bar_prefix = if bar_size == 0 {
            String::new()
        } else {
            format!(
                "{}  ",
                usage_bar(
                    row.bytes,
                    grand,
                    row.ignored.map(|value| pick(size, value.bytes, value.allocated)),
                    color,
                    bar_size,
                )
            )
        };
        let (age, style) = &row.age;
        let _ = writeln!(
            out,
            "{bar_prefix}{}  {}  {}  {indent}{}{}{}",
            percentage_cell(row.bytes, grand, 0, 5, color),
            styled_bytes(row.bytes, 10, color, false),
            right_cell(age, age_width, *style, color),
            row.name,
            count,
            ignored_suffix(row.ignored, size, self.selected, color),
        );
    }

    fn render_remainder(
        &self,
        out: &mut String,
        remainder: &crate::query::TreeRemainder,
        grand: Option<u64>,
        depth: usize,
        age_width: usize,
    ) {
        render_tree_remainder(out, remainder, grand, depth, age_width, self.size, self.options);
    }
}

/// What one tree line shows, whichever row it is: an entry's or the total's.
struct TextRow {
    /// The row's size in the section's metric.
    bytes: u64,
    /// Its ignored part, for the bar and the suffix.
    ignored: Option<IgnoredTally>,
    /// Its age cell and style.
    age: (String, Option<AnsiStyle>),
    /// The name, already styled.
    name: String,
    /// The file count a directory or total shows; none for a file.
    files: Option<u64>,
}

/// A tree row's age cell and its style: the row's age, or the gray word `unknown` when
/// its subtree was not listed in full or the reference cannot be represented, or a gray
/// dash when the row counts no entry at all, as a missing percentage is.
///
/// Completeness comes first. A row whose subtree was not listed in full is `unknown` even
/// when it has counted nothing yet, such as the root of `--scan-depth 0` or an opened
/// root before its first listing: the dash says the subtree holds nothing to count, and
/// nobody has looked.
fn tree_age_cell(node: &TreeNode) -> (String, Option<AnsiStyle>) {
    age_cell(node.complete, node.mtime_ns, node.age_ns)
}

/// [`tree_age_cell`] from its three inputs, which a total row has without being a node.
fn age_cell(
    complete: Option<bool>,
    mtime_ns: Option<i64>,
    age_ns: Option<i128>,
) -> (String, Option<AnsiStyle>) {
    if complete == Some(false) {
        (human_age(None), Some(STYLE_DETAIL))
    } else if mtime_ns.is_none() {
        ("—".to_string(), Some(STYLE_DETAIL))
    } else if age_ns.is_none() {
        (human_age(None), Some(STYLE_DETAIL))
    } else {
        (human_age(age_ns), None)
    }
}

/// A cell right-aligned to `width`, measured on the visible text and padded outside the
/// paint, as [`label_cell`] pads a left-aligned one.
fn right_cell(text: &str, width: usize, style: Option<AnsiStyle>, color: bool) -> String {
    let padding = " ".repeat(width.saturating_sub(display_width(text)));
    match style {
        Some(style) => format!("{padding}{}", paint(text, style, color)),
        None => format!("{padding}{text}"),
    }
}

/// Human projection of the same remainder serialized in machine formats.
///
/// The age cell is blank: the remainder stands for rows the bounds hid, and the folded
/// one-shot tier counts most of their files without keeping a time, so an age here would
/// exist on one route and not another. The parent row's age already covers everything
/// beneath it. The blank keeps the name column aligned with the rows above, and a section
/// with no rows has no age column to keep.
fn render_tree_remainder(
    out: &mut String,
    remainder: &crate::query::TreeRemainder,
    grand: Option<u64>,
    depth: usize,
    age_width: usize,
    size: SizeMetric,
    options: RenderOptions,
) {
    let RenderOptions { color, bar_size } = options;
    let bytes = match size {
        SizeMetric::Apparent => remainder.bytes,
        SizeMetric::Allocated => remainder.allocated,
    };
    let measure = bytes.map_or_else(
        || detail(&format!("{:>10}", "unknown"), color),
        |bytes| styled_bytes(bytes, 10, color, false),
    );
    // With no visible root, a known remainder represents the whole selected root.
    // A missing measurement cannot honestly produce either a bar or a percentage.
    let (usage_bar, percentage) = match bytes.zip(grand.or(bytes)) {
        Some((bytes, grand)) => (
            usage_bar(
                bytes,
                grand,
                remainder.ignored.map(|value| pick(size, value.bytes, value.allocated)),
                color,
                bar_size,
            ),
            percentage_cell(bytes, grand, 0, 5, color),
        ),
        None => (" ".repeat(bar_size), detail(&format!("{:>5}", "—"), color)),
    };
    let files = remainder.files.map_or_else(
        || format!("more files {}", detail("(count unknown)", color)),
        |files| format!("{} more {}", human_count(files), plural(files, "file", "files")),
    );
    let indent = "  ".repeat(depth);
    let note = format!("{} {files}", detail(&format!("{indent}… and"), color));
    let bar_prefix = if bar_size == 0 { String::new() } else { format!("{usage_bar}  ") };
    let age = if age_width == 0 { String::new() } else { format!("{}  ", " ".repeat(age_width)) };
    let _ = writeln!(out, "{bar_prefix}{percentage}  {measure}  {age}{note}");
}

/// Render a types section as aligned rows.
fn render_text_types(
    out: &mut String,
    rows: &[TypeRow],
    size: SizeMetric,
    selected: IgnoredEntries,
    color: bool,
) {
    let width = label_width(rows.iter().map(|row| row.extension.as_str()), TEXT_TYPE_LABEL_WIDTH);
    for row in rows {
        let _ = writeln!(
            out,
            "{}  {} {} {}{}",
            styled_bytes(pick(size, row.bytes, row.allocated), TEXT_SIZE_WIDTH, color, false),
            label_cell(&escaped_human(&row.extension), width, STYLE_CATEGORY, color),
            human_count(row.files),
            plural(row.files, "file", "files"),
            ignored_suffix(row.ignored, size, selected, color),
        );
    }
}

/// Counts row omissions after share filtering, so a share threshold alone never
/// produces a row-limit remedy.
fn bounded_rows(section: &Section) -> Option<(usize, usize)> {
    let (shown, total) = match section {
        Section::Code(overview) => (overview.languages.len(), overview.total_languages),
        Section::Files { rows, total, .. } => (rows.len(), *total),
        Section::Extensions { rows, total, .. } => (rows.len(), *total),
        Section::Metrics { summary, .. } => (summary.rows.len(), summary.total_rows),
        // Trees carry their own omission records; summary cannot be bounded.
        Section::Tree { .. } | Section::Summary(_) => return None,
    };
    (shown < total).then_some((shown, total))
}

/// Factual row count near a bounded section. Actionable guidance is emitted once at
/// the end of the report by the shared diagnostic collector.
fn bound_note(section: &Section) -> String {
    let Some((shown, total)) = bounded_rows(section) else {
        return String::new();
    };
    format!("  ({} of {})", human_count(shown as u64), human_count(total as u64))
}

/// A bounded flat listing, showing the measure it was ranked by.
///
/// The measure comes first at a fixed width so the paths line up under it, matching the
/// grouped views' size-then-label shape rather than inventing a third layout.
fn render_text_ranked_files(
    out: &mut String,
    report: &Report,
    rows: &[FileRow],
    color: bool,
    size: Option<SizeMetric>,
    measure: impl Fn(&FileRow) -> String,
) {
    let width = rows.iter().map(|row| display_width(&measure(row))).max().unwrap_or_default();
    for row in rows {
        let value = size.map_or_else(
            || format!("{:>width$}", measure(row)),
            |size| styled_bytes(pick(size, row.bytes, row.allocated), width, color, false),
        );
        let _ = writeln!(
            out,
            "{}  {}",
            value,
            human_name(&display_path(report, row).to_string_lossy(), row.kind, row.ignored, color)
        );
    }
}

fn render_text_metric_files(out: &mut String, report: &Report, rows: &[FileRow], color: bool) {
    let width = rows
        .iter()
        .map(|row| row.sort_value.map_or(1, |value| human_count(value).len()))
        .max()
        .unwrap_or_default();
    for row in rows {
        let value = row.sort_value.map_or_else(|| "—".to_string(), human_count);
        let classification =
            row.classification.as_ref().map_or_else(String::new, |classification| {
                let mut parts =
                    vec![classification.file_type.as_str(), classification.source.as_str()];
                if classification.flags.generated {
                    parts.push("generated");
                }
                if classification.flags.vendored {
                    parts.push("vendored");
                }
                if classification.flags.documentation {
                    parts.push("documentation");
                }
                format!(" {}", detail(&format!("({})", parts.join(", ")), color))
            });
        let _ = writeln!(
            out,
            "{:>width$}  {}{}",
            value,
            human_name(&display_path(report, row).to_string_lossy(), row.kind, row.ignored, color),
            classification
        );
    }
}

fn render_text_summary(
    out: &mut String,
    row: &SummaryRow,
    size: SizeMetric,
    selected: IgnoredEntries,
    color: bool,
) {
    let _ = writeln!(
        out,
        "{}  {} {}, {} {}{}",
        styled_bytes(pick(size, row.bytes, row.allocated), 10, color, false),
        human_count(row.files),
        plural(row.files, "file", "files"),
        human_count(row.dirs),
        plural(row.dirs, "directory", "directories"),
        ignored_suffix(row.ignored, size, selected, color),
    );
}

/// Quote a YAML scalar whenever a bare word would be ambiguous.
///
/// Always quoting would be simpler and uglier; quoting only what needs it keeps the
/// output readable, which is the reason to offer YAML at all.
#[cfg(test)]
fn yaml_scalar(value: &str) -> String {
    let mut out = String::new();
    crate::emit::write_yaml_scalar(&mut out, value);
    out
}

/// Quote and escape a string as a JSON scalar.
#[cfg(test)]
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    crate::emit::write_json_string(&mut out, text);
    out
}

/// This binary's identity, for the `generator` field.
fn generator() -> String {
    format!("fdu {}", env!("CARGO_PKG_VERSION"))
}

/// All-caps header naming a view in multi-view text output.
///
/// Deliberately not `view.label().to_uppercase()`: the wire label is a schema promise
/// machine consumers match on, and deriving the human header from it would let a
/// presentation change reach into the schema, or freeze the schema for a presentation
/// reason. They spell the same word today because the same word is right in both places,
/// and a test holds them in step rather than a shared expression.
pub(super) fn view_header(view: ViewSpec) -> &'static str {
    match view {
        ViewSpec::List => "LIST",
        ViewSpec::Tree => "TREE",
        ViewSpec::Types => "TYPES",
        ViewSpec::Extensions => "EXTENSIONS",
        ViewSpec::Families => "FAMILIES",
        ViewSpec::Languages => "LANGUAGES",
        ViewSpec::Code => "CODE",
        ViewSpec::Documents => "DOCUMENTS",
        ViewSpec::Files => "FILES",
        ViewSpec::Largest => "LARGEST",
        ViewSpec::Recent => "RECENT",
        ViewSpec::Summary => "SUMMARY",
    }
}

fn metric_group_label(group: MetricGroup) -> &'static str {
    match group {
        MetricGroup::Type => "type",
        MetricGroup::Family => "family",
    }
}

fn coverage_label(reason: CoverageReason) -> &'static str {
    match reason {
        CoverageReason::Analyzed => "analyzed",
        CoverageReason::Binary => "binary",
        CoverageReason::InvalidUtf8 => "invalid_utf8",
        CoverageReason::UnsupportedEncoding => "unsupported_encoding",
        CoverageReason::Unsupported => "unsupported",
        CoverageReason::IoError => "io_error",
        CoverageReason::ChangedDuringRead => "changed_during_read",
        CoverageReason::TextOnly => "text_only",
    }
}

/// The requested analyzer set, in the vocabulary `--analyze` accepts.
///
/// A list rather than one label because the set is what was requested; the neighbouring
/// `analyzers` array reports what actually ran, with each dialect's version.
fn analysis_set_labels(profile: crate::content::AnalysisSet) -> Vec<&'static str> {
    profile.labels()
}

/// Stable wire label for a cache tier.
fn source_label(source: ReportSource) -> &'static str {
    match source {
        ReportSource::ColdScan => "cold_scan",
        ReportSource::WarmRevalidate => "warm_revalidate",
        ReportSource::CacheOnly => "cache_only",
    }
}

fn tier_source_label(source: Source) -> &'static str {
    match source {
        Source::Scanned => "scanned",
        Source::Revalidated => "revalidated",
        Source::JournalScoped => "journal_scoped",
        Source::Cached => "cached",
    }
}

fn structural_coverage_label(reason: crate::engine_contract::CoverageReason) -> &'static str {
    match reason {
        crate::engine_contract::CoverageReason::Building => "building",
        crate::engine_contract::CoverageReason::Budget => "budget",
        crate::engine_contract::CoverageReason::Cancelled => "cancelled",
        crate::engine_contract::CoverageReason::Inaccessible => "inaccessible",
        crate::engine_contract::CoverageReason::Failed => "failed",
    }
}

fn issue_kind_label(kind: IssueKind) -> &'static str {
    match kind {
        IssueKind::Permission => "permission",
        IssueKind::Disappeared => "disappeared",
        IssueKind::InvalidMetadata => "invalid_metadata",
        IssueKind::ResourceBudget => "resource_budget",
        IssueKind::ObservationGap => "observation_gap",
        IssueKind::ProviderFailure => "provider_failure",
    }
}

/// Stable wire label for freshness.
fn freshness_label(freshness: Freshness) -> &'static str {
    match freshness {
        Freshness::Fresh => "fresh",
        Freshness::Reconciling => "reconciling",
        Freshness::Stale => "stale",
        Freshness::Partial => "partial",
    }
}

/// Stable wire label for an entry kind.
fn kind_label(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::File => "file",
        EntryKind::Dir => "dir",
        EntryKind::Symlink => "symlink",
        EntryKind::Other => "other",
    }
}

/// The byte count for the metric a report answers in.
fn pick(size: SizeMetric, apparent: u64, allocated: u64) -> u64 {
    match size {
        SizeMetric::Apparent => apparent,
        SizeMetric::Allocated => allocated,
    }
}

/// Pick the singular or plural noun for a count.
fn plural<'a>(count: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

/// A bounded share for the human size bar and percentage.
fn ratio(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let share = part as f64 / whole as f64;
    share.clamp(0.0, 1.0)
}

// Rounding a bounded share to a requested bar width is presentation arithmetic:
// clamp the ratio and the resulting cell count before constructing the glyphs.
/// Nearest whole-cell share, with half cells rounded up and no floating-point loss.
fn bar_cells(part: u64, whole: u64, width: usize) -> usize {
    if whole == 0 {
        return 0;
    }
    let numerator = u128::from(part.min(whole)) * width as u128;
    usize::try_from((numerator + u128::from(whole) / 2) / u128::from(whole))
        .expect("a bounded share fits the supplied width")
}

/// Split a colored bar into measured populations; plain bars retain their glyphs.
/// Round the total against the root, then apportion its visible cells by the row's
/// population ratio. Independent root-relative rounding can erase a majority ignored
/// population in a one-cell bar. Shading distinguishes populations within green;
/// unknown classification uses medium shading. Numeric columns remain authoritative.
fn usage_bar(bytes: u64, total: u64, ignored: Option<u64>, color: bool, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let filled = bar_cells(bytes, total, width);
    if !color {
        return format!("{}{}", "█".repeat(filled), "░".repeat(width - filled));
    }
    if ignored.is_none() {
        return format!(
            "{}{}",
            paint(&"▒".repeat(filled), STYLE_BAR, true),
            paint(&"░".repeat(width - filled), STYLE_BAR.dimmed(), true)
        );
    }
    let ignored = bar_cells(ignored.unwrap_or(0), bytes, filled);
    format!(
        "{}{}{}",
        paint(&"█".repeat(filled - ignored), STYLE_BAR, true),
        paint(&"▓".repeat(ignored), STYLE_BAR, true),
        paint(&"░".repeat(width - filled), STYLE_BAR.dimmed(), true)
    )
}

/// Render a count with thousands separators, the way every fdu report does.
///
/// Lived in the command line, and `report_format` called *into* it -- so the library
/// depended on its own front end, which is the inverse of the rule that the CLI invents
/// nothing. A crate boundary rejects that outright, which is how it was found.
pub fn human_count(value: u64) -> String {
    human_count_u128(u128::from(value))
}

/// Group a full-width count using the same human policy as [`human_count`].
///
/// Keep the grouping rule here so a future locale or no-grouping choice changes both
/// widths together without losing precision in rates wider than u64.
pub fn human_count_u128(value: u128) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, byte) in digits.bytes().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(char::from(byte));
    }
    grouped
}

/// Render a byte count at human scale, the way every fdu report does.
///
/// Public because a caller formatting fdu's numbers should not reimplement its unit
/// rules: two spellings of one quantity inside a single tool is how a report and the
/// line summarising it come to disagree.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    // Integer arithmetic to the unit, then one bounded division for the tenths digit:
    // a byte count can exceed f64's exact-integer range, and a size that renders wrong
    // at the top of the scale is worse than one that renders plainly.
    let mut whole = bytes;
    let mut remainder = 0u64;
    let mut unit = 0;
    while whole >= 1024 && unit + 1 < UNITS.len() {
        remainder = whole % 1024;
        whole /= 1024;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", human_count(bytes))
    } else if whole < 10 {
        let tenths = (remainder * 10) / 1024;
        format!("{}.{tenths} {}", human_count(whole), UNITS[unit])
    } else {
        format!("{} {}", human_count(whole), UNITS[unit])
    }
}

/// Whether a path renders losslessly as UTF-8.
///
/// Non-UTF-8 names exist and a report must not pretend otherwise; the CLI layer adds the
/// raw-bytes companion field, and this is the predicate that decides when.
pub fn is_lossy(path: &Path) -> bool {
    path.to_str().is_none()
}

/// Streaming-output schema identity.
///
/// Distinct from the one-shot schema on purpose: a stream is a sequence of tagged
/// records over time, not one document, and a consumer should not have to discover which
/// it is holding.
pub const STREAM_SCHEMA: &str = "fdu.stream/2";

/// The rule drawn above a watch repaint, carrying the instant it was rendered.
///
/// The time is what makes the rule worth a line rather than a bare separator: a watch
/// reader wants to know when the tree last moved, and it is the one fact that
/// distinguishes one repaint from another whose numbers happen to match. RFC 3339 in UTC
/// is the spelling every other timestamp this tool prints uses.
///
/// Lives here rather than in the command line because it is presentation, and a caller
/// repainting fdu's views should draw fdu's separator rather than invent one that will
/// drift from it.
pub fn watch_rule(at: std::time::SystemTime) -> String {
    format!("──── {} ────", format_rfc3339(at))
}

/// The watch repaint rule for an integer nanosecond timestamp.
///
/// This is distinct from [`watch_rule`] because an integer can carry finer precision than
/// the platform's [`std::time::SystemTime`]. In particular, Windows would otherwise
/// truncate the final two digits while converting through 100-nanosecond FILETIME ticks.
pub fn watch_rule_nanos(at_nanos: i64) -> String {
    format!("──── {} ────", format_rfc3339_nanos(at_nanos))
}

/// Render one streamed change as a tagged record.
#[cfg(feature = "watch")]
pub fn render_change(change: &crate::Change, format: Format) -> String {
    let kind = match change.kind {
        crate::ChangeKind::Upsert => "upsert",
        crate::ChangeKind::Remove => "remove",
        crate::ChangeKind::Invalidate => "invalidate",
    };

    if !format.is_machine() {
        // Path first, so the stream stays greppable and cuts the same way a one-shot
        // listing does; the operation follows on the same line.
        return format!("{}\t{kind}", change.path.display());
    }

    match format {
        Format::Json | Format::Jsonl => render_change_machine(change, kind, JsonSink::line()),
        Format::Yaml => {
            format!(
                "{}{}",
                document_start(format),
                render_change_machine(change, kind, YamlSink::new())
            )
        }
        Format::Text | Format::Tree | Format::Paths | Format::Long => {
            unreachable!("text returned above")
        }
    }
}

#[cfg(feature = "watch")]
fn render_change_machine(
    change: &crate::Change,
    kind: &str,
    mut sink: impl Sink<Output = String>,
) -> String {
    sink.event(Event::BeginMap(Shape::Block));
    emit_str_field(&mut sink, "schema", STREAM_SCHEMA);
    emit_str_field(&mut sink, "record", "change");
    emit_str_field(&mut sink, "op", kind);
    emit_path_fields(&mut sink, &change.path);
    emit_u64_field(&mut sink, "clock", change.clock);
    emit_field(&mut sink, Field::when_set("kind"), change.entry_kind.is_some(), |sink| {
        emit_scalar(sink, Scalar::Str(kind_label(change.entry_kind.expect("presence checked"))));
    });
    emit_field(&mut sink, Field::when_set("bytes"), change.bytes.is_some(), |sink| {
        emit_scalar(sink, Scalar::U64(change.bytes.expect("presence checked")));
    });
    emit_field(&mut sink, Field::when_set("allocated"), change.allocated.is_some(), |sink| {
        emit_scalar(sink, Scalar::U64(change.allocated.expect("presence checked")));
    });
    emit_field(&mut sink, Field::when_set("mtime_ns"), change.mtime_ns.is_some(), |sink| {
        emit_scalar(sink, Scalar::I64(change.mtime_ns.expect("presence checked")));
    });
    emit_field(&mut sink, Field::when_set("ignored"), change.ignored.is_some(), |sink| {
        emit_scalar(sink, Scalar::Bool(change.ignored.expect("presence checked")));
    });
    sink.event(Event::EndMap);
    sink.finish()
}

/// Lossless identity for a path that does not render as UTF-8.
///
/// `to_string_lossy` replaces undecodable bytes with U+FFFD, so a consumer reading only
/// `root` cannot tell two different names apart. Machine output therefore carries the
/// native bytes alongside, and only when they are actually needed.
fn raw_os_identity(value: &std::ffi::OsStr) -> Option<(&'static str, String)> {
    if value.to_str().is_some() {
        return None;
    }

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        Some(("unix-bytes", hex_bytes(value.as_bytes().iter().copied())))
    }

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;

        Some(("windows-wtf16le", hex_bytes(value.encode_wide().flat_map(u16::to_le_bytes))))
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// Hex-encode bytes for the raw identity field.
#[cfg(any(unix, windows))]
fn hex_bytes(bytes: impl IntoIterator<Item = u8>) -> String {
    let mut out = String::new();
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Render cache status in any format.
///
/// A separate entry point rather than a `Report` section: cache status is a fact about
/// the cache directory, not about a tree, and folding it into the report schema would
/// make every consumer parse a variant that is empty on every normal run.
///
/// `scope` is the request the statuses answer. It decides only which command the text
/// names for reclaiming stale snapshots: a root's snapshot is cleared by its path, while a
/// stale file found by listing the directory may name no root this build can read.
///
/// Every machine format carries [`CACHE_SCHEMA`], the way every machine report carries its
/// own: the first field of the JSON document, an envelope line of its own ahead of the
/// rows in JSON Lines, and the first line of the YAML.
pub fn render_cache_status(
    statuses: &[crate::CacheStatus],
    scope: crate::CacheScope,
    format: Format,
) -> String {
    render_cache_status_with_options(statuses, scope, format, RenderOptions::default())
}

/// Render cache status with the same human color roles as report rows.
/// Machine formats ignore the presentation options; cache text has no usage bar.
pub fn render_cache_status_with_options(
    statuses: &[crate::CacheStatus],
    scope: crate::CacheScope,
    format: Format,
    options: RenderOptions,
) -> String {
    match format {
        Format::Jsonl => {
            let mut sink = JsonSink::line();
            sink.event(Event::BeginMap(Shape::Inline));
            emit_str_field(&mut sink, "schema", CACHE_SCHEMA);
            sink.event(Event::EndMap);
            let mut out = sink.finish();
            for status in statuses {
                out.push('\n');
                let row = cache_row(status);
                let mut sink = JsonSink::line();
                emit_cache_field(&mut sink, &row);
                out.push_str(&sink.finish());
            }
            out
        }
        Format::Yaml => render_cache_machine(statuses, YamlSink::new()),
        // The human layout lives here beside every other human layout. It used to live in
        // the CLI, which meant the only way to print cache status the way fdu prints it
        // was to be the CLI: the Python API returned CacheStatus values nothing could
        // render, so the parity shim printed repr() and nine sessions differed (fdu-1kw3).
        Format::Text | Format::Tree | Format::Paths | Format::Long => {
            render_cache_status_text(statuses, scope, options.color)
        }
        Format::Json => render_cache_machine(statuses, JsonSink::pretty()),
    }
}

fn render_cache_machine(
    statuses: &[crate::CacheStatus],
    mut sink: impl Sink<Output = String>,
) -> String {
    sink.event(Event::BeginMap(Shape::Block));
    emit_str_field(&mut sink, "schema", CACHE_SCHEMA);
    emit_field(&mut sink, Field::always("caches"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for status in statuses {
            let row = cache_row(status);
            emit_cache_field(sink, &row);
        }
        sink.event(Event::EndSeq);
    });
    sink.event(Event::EndMap);
    sink.finish()
}

fn emit_cache_field(sink: &mut impl Sink, field: &CacheField) {
    match field {
        CacheField::Null => emit_scalar(sink, Scalar::Null),
        CacheField::Bool(value) => emit_scalar(sink, Scalar::Bool(*value)),
        CacheField::Count(value) => emit_scalar(sink, Scalar::U64(*value)),
        CacheField::Text(value) => emit_scalar(sink, Scalar::Str(value)),
        CacheField::List(values) => {
            sink.event(Event::BeginSeq(Shape::Block));
            for value in values {
                emit_cache_field(sink, value);
            }
            sink.event(Event::EndSeq);
        }
        CacheField::Map(fields) => {
            sink.event(Event::BeginMap(Shape::Block));
            for (name, value) in fields {
                sink.event(Event::Key(name));
                emit_cache_field(sink, value);
            }
            sink.event(Event::EndMap);
        }
    }
}

/// One value in a cache-status row.
///
/// The row is built once as fields and serialized by JSON and YAML alike, so the two
/// formats cannot disagree about which keys a row has or how an identity nests.
enum CacheField {
    Null,
    Bool(bool),
    Count(u64),
    Text(String),
    List(Vec<CacheField>),
    Map(Vec<(&'static str, CacheField)>),
}

impl CacheField {
    fn count(value: Option<u64>) -> Self {
        value.map_or(Self::Null, Self::Count)
    }
}

/// One cache-status row as fields: what every row carries, what its state adds, and the
/// content sidecar beside it.
fn cache_row(status: &crate::CacheStatus) -> CacheField {
    use crate::CacheState;

    let mut fields = vec![("path", CacheField::Text(status.path.to_string_lossy().into_owned()))];
    if let Some((encoding, hex)) = raw_os_identity(status.path.as_os_str()) {
        fields.push((
            "path_raw",
            CacheField::Map(vec![
                ("encoding", CacheField::Text(encoding.to_string())),
                ("hex", CacheField::Text(hex)),
            ]),
        ));
    }
    fields.extend([
        ("bytes", CacheField::Count(status.bytes)),
        ("state", CacheField::Text(status.state.label().to_string())),
    ]);
    match &status.state {
        CacheState::Current(info) => {
            fields.push(("root", CacheField::Text(info.root.to_string_lossy().into_owned())));
            if let Some((encoding, hex)) = raw_os_identity(info.root.as_os_str()) {
                fields.push((
                    "root_raw",
                    CacheField::Map(vec![
                        ("encoding", CacheField::Text(encoding.to_string())),
                        ("hex", CacheField::Text(hex)),
                    ]),
                ));
            }
            fields.push(("entries", CacheField::Count(info.entries)));
            fields.push(("identity", snapshot_identity_field(info.identity)));
        }
        CacheState::Stale(reason) => fields.extend(stale_fields(*reason)),
        CacheState::Leftover(kind) => {
            fields.push(("leftover_kind", CacheField::Text(kind.label().to_string())));
        }
        CacheState::Unrecognized | CacheState::Absent => {}
    }
    // Every row carries it, whatever the state, so a consumer reads one shape rather than
    // discovering which keys this row happens to have.
    fields.push(("content", status.content.as_ref().map_or(CacheField::Null, content_field)));
    CacheField::Map(fields)
}

/// Why a store is stale, and the format version when that is the reason.
fn stale_fields(reason: crate::StaleReason) -> [(&'static str, CacheField); 2] {
    [
        ("stale_reason", CacheField::Text(reason.label().to_string())),
        ("format_version", CacheField::count(reason.format_version().map(u64::from))),
    ]
}

/// The content sidecar beside a snapshot: its size and state, and a current one's identity
/// and record count.
fn content_field(content: &crate::ContentStatus) -> CacheField {
    use crate::ContentState;

    let mut fields = vec![
        ("bytes", CacheField::Count(content.bytes)),
        ("state", CacheField::Text(content.state.label().to_string())),
    ];
    match &content.state {
        ContentState::Current(info) => {
            fields.push(("records", CacheField::Count(info.records)));
            fields.push(("identity", content_identity_field(&info.identity)));
        }
        ContentState::Stale(reason) => fields.extend(stale_fields(*reason)),
    }
    CacheField::Map(fields)
}

/// A snapshot's tier identities: its entry tier, and its `.gitignore` control tier as the
/// report's `ignore_rules` names it, `null` when no rule was read.
fn snapshot_identity_field(identity: crate::SnapshotIdentity) -> CacheField {
    let ignore_rules = match identity.controls {
        crate::ControlTierIdentity::NotObserved => CacheField::Null,
        crate::ControlTierIdentity::Observed { limits } => {
            let limit = |limit: Option<usize>| {
                CacheField::count(limit.map(|limit| u64::try_from(limit).unwrap_or(u64::MAX)))
            };
            CacheField::Map(vec![(
                "limits",
                CacheField::Map(vec![
                    ("budget", limit(limits.budget)),
                    ("line_limit", limit(limits.line_limit)),
                ]),
            )])
        }
    };
    CacheField::Map(vec![
        ("entries", entry_identity_field(identity.entries)),
        ("ignore_rules", ignore_rules),
    ])
}

/// An entry tier's identity: the engine that built it, the scope fields, and the type-rules
/// and reducer-set fingerprints.
fn entry_identity_field(identity: crate::EntryTierIdentity) -> CacheField {
    let scope = identity.scope;
    let depth = scope.max_depth.map(|depth| u64::try_from(depth).unwrap_or(u64::MAX));
    CacheField::Map(vec![
        ("engine", CacheField::Count(identity.engine)),
        ("max_depth", CacheField::count(depth)),
        ("follow_symlinks", CacheField::Bool(scope.follow_symlinks)),
        ("one_filesystem", CacheField::Bool(scope.one_filesystem)),
        ("hidden_fingerprint", CacheField::Count(scope.hidden_fingerprint)),
        ("exclude_special", CacheField::Bool(scope.exclude_special)),
        ("population", CacheField::Text(scope.population.label().to_string())),
        ("control_fingerprint", CacheField::Count(scope.control_fingerprint)),
        ("type_rules_fingerprint", CacheField::Count(identity.type_rules_fingerprint)),
        ("reducers_fingerprint", CacheField::Count(identity.reducers_fingerprint)),
    ])
}

/// A content tier's identity: the entry tier it was analyzed over, which holds its type
/// rules, then the analyzer set, options, and analyzers under the names a report's
/// `analysis` object gives them.
fn content_identity_field(identity: &crate::ContentTierIdentity) -> CacheField {
    let analyze = analysis_set_labels(identity.analysis)
        .into_iter()
        .map(|label| CacheField::Text(label.to_string()))
        .collect();
    let analyzers = identity
        .provenance
        .analyzers
        .iter()
        .map(|(id, version)| {
            CacheField::Map(vec![
                ("id", CacheField::Text(id.0.to_string())),
                ("version", CacheField::Count(u64::from(version.0))),
            ])
        })
        .collect();
    CacheField::Map(vec![
        ("entries", entry_identity_field(identity.entries)),
        ("analyze", CacheField::List(analyze)),
        ("options_fingerprint", CacheField::Count(identity.provenance.options_fingerprint.0)),
        ("analyzers", CacheField::List(analyzers)),
    ])
}

/// The human cache-status layout: one line per file, then what can be done about the
/// files this build cannot use.
fn render_cache_status_text(
    statuses: &[crate::CacheStatus],
    scope: crate::CacheScope,
    color: bool,
) -> String {
    use crate::{CacheScope, CacheState, LeftoverKind, StaleReason};

    let mut lines = Vec::new();
    let mut current = 0_usize;
    let (mut stale, mut stale_bytes) = (0_usize, 0_u64);
    let (mut leftover, mut leftover_bytes, mut staging) = (0_usize, 0_u64, 0_usize);
    let (mut unrecognized, mut unrecognized_bytes) = (0_usize, 0_u64);
    for status in statuses {
        let content_bytes = status.content_bytes().unwrap_or(0);
        match &status.state {
            CacheState::Current(info) => {
                current += 1;
                // A sidecar this build cannot serve is named, so the bytes are not read as a
                // usable content cache.
                let stale_content = status
                    .content
                    .as_ref()
                    .is_some_and(|content| matches!(content.state, crate::ContentState::Stale(_)));
                lines.push(format!(
                    "{}  {} entries, {} metadata, {} {}content  {}",
                    status.path.display(),
                    human_count(info.entries),
                    styled_bytes(status.bytes, 0, color, false),
                    styled_bytes(content_bytes, 0, color, false),
                    if stale_content { "stale " } else { "" },
                    info.root.display()
                ));
            }
            CacheState::Stale(reason) => {
                stale += 1;
                stale_bytes =
                    stale_bytes.saturating_add(status.bytes).saturating_add(content_bytes);
                let why = match reason {
                    StaleReason::OlderFormat { version } => {
                        format!("older snapshot format {version}")
                    }
                    StaleReason::NewerFormat { version } => {
                        format!("newer snapshot format {version}")
                    }
                    StaleReason::OtherEngine => "written by another fdu version".to_string(),
                    StaleReason::Unreadable => "unreadable by this build".to_string(),
                };
                lines.push(format!(
                    "{}  stale {}, {} metadata, {} content",
                    status.path.display(),
                    detail(&format!("({why})"), color),
                    styled_bytes(status.bytes, 0, color, false),
                    styled_bytes(content_bytes, 0, color, false)
                ));
            }
            CacheState::Leftover(kind) => {
                leftover += 1;
                leftover_bytes = leftover_bytes.saturating_add(status.bytes);
                let what = match kind {
                    LeftoverKind::StagingTemporary => {
                        staging += 1;
                        "staging temporary"
                    }
                    LeftoverKind::OrphanedContent => "orphaned content sidecar",
                };
                lines.push(format!(
                    "{}  leftover {}, {}",
                    status.path.display(),
                    detail(&format!("({what})"), color),
                    styled_bytes(status.bytes, 0, color, false)
                ));
            }
            CacheState::Unrecognized => {
                unrecognized += 1;
                unrecognized_bytes = unrecognized_bytes.saturating_add(status.bytes);
                lines.push(format!(
                    "{}  unrecognized, {}",
                    status.path.display(),
                    styled_bytes(status.bytes, 0, color, false)
                ));
            }
            // Root scope synthesises a status for the path a snapshot *would* occupy, so a
            // tree that has never been cached yields one absent entry. Absence is not a
            // file to describe.
            CacheState::Absent => {}
        }
    }
    if lines.is_empty() {
        return "No cached snapshots.".to_string();
    }

    if stale > 0 {
        let (subject, object) = if stale == 1 {
            ("1 stale snapshot".to_string(), "it")
        } else {
            (format!("{} stale snapshots", human_count_u128(stale as u128)), "them")
        };
        let remedy = match scope {
            CacheScope::Root => format!("fdu --cache-clear PATH removes {object}"),
            CacheScope::All if current == 0 => format!("fdu --cache-clear=all removes {object}"),
            CacheScope::All => {
                format!("fdu --cache-clear=all removes {object}, along with every current snapshot")
            }
        };
        lines.push(format!(
            "{subject} {} cannot be served by this build; {remedy}.",
            cache_size_detail(stale_bytes, color)
        ));
    }
    if leftover > 0 {
        // Named as fdu's own, because they are: calling them foreign would tell the user
        // to leave fdu's debris alone. `=all` is the scope that reclaims them; a root's
        // clear reaches only the one path that root's snapshot occupies.
        let (subject, predicate, object) = if leftover == 1 {
            ("1 leftover file".to_string(), "is", "it")
        } else {
            (format!("{} leftover files", human_count_u128(leftover as u128)), "are", "them")
        };
        // A staging file is reclaimed only once it is too old to belong to a running
        // writer, and a status knows no file's age, so the promise names the exception
        // rather than counting files the clear will then decline and explain.
        let caveat = if staging > 0 {
            ", though a staging file waits until it is too old to be a running writer's"
        } else {
            ""
        };
        lines.push(format!(
            "{subject} {} {predicate} fdu's own, left by an interrupted \
             write; fdu --cache-clear=all reclaims {object}{caveat}.",
            cache_size_detail(leftover_bytes, color)
        ));
    }
    if unrecognized > 0 {
        let (subject, predicate, object) = if unrecognized == 1 {
            ("1 unrecognized file".to_string(), "is not an fdu snapshot", "it")
        } else {
            (
                format!("{} unrecognized files", human_count_u128(unrecognized as u128)),
                "are not fdu snapshots",
                "them",
            )
        };
        lines.push(format!(
            "{subject} {} {predicate}, so fdu leaves {object} in place.",
            cache_size_detail(unrecognized_bytes, color)
        ));
    }
    lines.join("\n")
}

fn cache_size_detail(bytes: u64, color: bool) -> String {
    format!("{}{}{}", detail("(", color), styled_bytes(bytes, 0, color, true), detail(")", color))
}

#[cfg(test)]
mod tests {
    fn render(report: &Report, format: Format, color: bool) -> String {
        super::render(report, format, color).expect("compatible report format")
    }

    use super::*;
    use crate::Index;
    use crate::engine_contract::{Attrs, Observation, Op, ScanScope};
    use crate::query::{Bound, Query, Request, Selection, ShareThreshold};
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    struct Provenance {
        scan_started_at: Option<SystemTime>,
        generated_at: SystemTime,
        source: ReportSource,
        complete: bool,
        errors: Vec<String>,
    }

    fn report(index: &Index, request: &Request, provenance: &Provenance) -> crate::Result<Report> {
        let mut report = crate::query::report(index, request, provenance.generated_at)?;
        report.provenance.scan_started_at = provenance.scan_started_at;
        report.provenance.source = provenance.source;
        report.status.complete = provenance.complete;
        report.status.errors = provenance
            .errors
            .iter()
            .cloned()
            .map(|message| crate::Issue::provider_failure(None, message))
            .map(crate::query::StatusIssue::of_one_root)
            .collect();
        Ok(report)
    }

    fn attrs(size: u64, mtime_ns: i64) -> Attrs {
        Attrs {
            size,
            allocated: size.div_ceil(512) * 512,
            mtime_ns,
            ctime_ns: mtime_ns,
            inode: 7,
            dev: 1,
        }
    }

    fn cache_file(name: &str, bytes: u64, state: crate::CacheState) -> crate::CacheStatus {
        // A stale snapshot here keeps the sidecar an older format wrote beside it.
        let content =
            matches!(state, crate::CacheState::Stale(_)).then_some(crate::ContentStatus {
                bytes: 5,
                state: crate::ContentState::Stale(crate::StaleReason::OlderFormat { version: 4 }),
            });
        crate::CacheStatus { path: PathBuf::from(name), bytes, content, state }
    }

    /// A snapshot identity with a small value in every field, so a rendering is readable.
    fn small_snapshot_identity() -> crate::SnapshotIdentity {
        crate::SnapshotIdentity {
            entries: crate::EntryTierIdentity {
                engine: 1,
                scope: crate::EntryScope {
                    max_depth: None,
                    follow_symlinks: false,
                    one_filesystem: true,
                    hidden_fingerprint: 2,
                    exclude_special: false,
                    population: IgnoredEntries::Include,
                    control_fingerprint: 0,
                },
                type_rules_fingerprint: 3,
                reducers_fingerprint: 4,
            },
            controls: crate::ControlTierIdentity::Observed {
                limits: crate::control::ControlLimits { budget: Some(10), line_limit: None },
            },
        }
    }

    /// Stale, leftover, and unrecognized files are shown, sized, and followed by what
    /// reclaims them.
    ///
    /// A unit test beside the goldens because a golden cannot produce a newer format or
    /// every reason at once, and because the remedy depends on the scope and on whether
    /// clearing would also take current snapshots.
    #[test]
    fn cache_status_shows_stale_and_unrecognized_files_with_their_remedy() {
        use crate::{CacheScope, CacheState, LeftoverKind, StaleReason};

        let stale = [
            cache_file("a.fdu", 10, CacheState::Stale(StaleReason::OlderFormat { version: 2 })),
            cache_file("b.fdu", 20, CacheState::Stale(StaleReason::NewerFormat { version: 99 })),
            cache_file("c.fdu", 30, CacheState::Stale(StaleReason::OtherEngine)),
            cache_file("d.fdu", 40, CacheState::Stale(StaleReason::Unreadable)),
            cache_file("notes.txt", 14, CacheState::Unrecognized),
        ];
        assert_eq!(
            render_cache_status(&stale, CacheScope::All, Format::Text),
            "a.fdu  stale (older snapshot format 2), 10 B metadata, 5 B content\n\
             b.fdu  stale (newer snapshot format 99), 20 B metadata, 5 B content\n\
             c.fdu  stale (written by another fdu version), 30 B metadata, 5 B content\n\
             d.fdu  stale (unreadable by this build), 40 B metadata, 5 B content\n\
             notes.txt  unrecognized, 14 B\n\
             4 stale snapshots (120 B) cannot be served by this build; \
             fdu --cache-clear=all removes them.\n\
             1 unrecognized file (14 B) is not an fdu snapshot, so fdu leaves it in place."
        );
        let colored = render_cache_status_with_options(
            &stale,
            CacheScope::All,
            Format::Text,
            RenderOptions { color: true, ..RenderOptions::default() },
        );
        assert!(colored.contains(&detail("(older snapshot format 2)", true)), "{colored:?}");
        assert!(
            colored.contains(&format!("4 stale snapshots {} cannot", cache_size_detail(120, true))),
            "{colored:?}"
        );
        assert_eq!(
            strip_ansi(&colored),
            render_cache_status(&stale, CacheScope::All, Format::Text)
        );

        // fdu's own debris is named as fdu's, so a reader is not told to leave it alone.
        let leftovers = [
            cache_file(
                ".g.fdu.tmp.1.2.3",
                60,
                CacheState::Leftover(LeftoverKind::StagingTemporary),
            ),
            cache_file("h.analysis.bin", 70, CacheState::Leftover(LeftoverKind::OrphanedContent)),
        ];
        assert_eq!(
            render_cache_status(&leftovers, CacheScope::All, Format::Text),
            ".g.fdu.tmp.1.2.3  leftover (staging temporary), 60 B\n\
             h.analysis.bin  leftover (orphaned content sidecar), 70 B\n\
             2 leftover files (130 B) are fdu's own, left by an interrupted write; \
             fdu --cache-clear=all reclaims them, though a staging file waits until it is \
             too old to be a running writer's."
        );
        assert!(render_cache_status(&leftovers[..1], CacheScope::Root, Format::Text).ends_with(
            "1 leftover file (60 B) is fdu's own, left by an interrupted write; \
                 fdu --cache-clear=all reclaims it, though a staging file waits until it \
                 is too old to be a running writer's."
        ));
        // With no staging file listed, nothing is held back and the promise is plain: a
        // status that named an exception with no file it could apply to would be noise.
        assert!(render_cache_status(&leftovers[1..], CacheScope::All, Format::Text).ends_with(
            "1 leftover file (70 B) is fdu's own, left by an interrupted write; \
                 fdu --cache-clear=all reclaims it."
        ));

        let root = [cache_file("a.fdu", 10, CacheState::Stale(StaleReason::OtherEngine))];
        assert!(render_cache_status(&root, CacheScope::Root, Format::Text).ends_with(
            "1 stale snapshot (15 B) cannot be served by this build; \
                 fdu --cache-clear PATH removes it."
        ));

        let current = cache_file(
            "e.fdu",
            50,
            CacheState::Current(crate::SnapshotInfo {
                root: PathBuf::from("/tree"),
                identity: small_snapshot_identity(),
                entries: 3,
            }),
        );
        let mixed = [stale[0].clone(), current, stale[4].clone(), stale[4].clone()];
        assert!(render_cache_status(&mixed, CacheScope::All, Format::Text).ends_with(
            "e.fdu  3 entries, 50 B metadata, 0 B content  /tree\n\
             notes.txt  unrecognized, 14 B\n\
             notes.txt  unrecognized, 14 B\n\
             1 stale snapshot (15 B) cannot be served by this build; \
             fdu --cache-clear=all removes it, along with every current snapshot.\n\
             2 unrecognized files (28 B) are not fdu snapshots, so fdu leaves them in place."
        ));

        let absent = [cache_file("f.fdu", 0, CacheState::Absent)];
        assert_eq!(
            render_cache_status(&absent, CacheScope::Root, Format::Text),
            "No cached snapshots."
        );
        // Every row carries the same keys whatever its state, and the envelope line
        // carries the schema even when nothing follows it.
        assert_eq!(
            render_cache_status(
                &[stale[0].clone(), absent[0].clone(), stale[4].clone(), leftovers[0].clone()],
                CacheScope::All,
                Format::Jsonl
            ),
            "{\"schema\": \"fdu.cache/3\"}\n\
             {\"path\": \"a.fdu\", \"bytes\": 10, \"state\": \"stale\", \"stale_reason\": \"older_format\", \"format_version\": 2, \"content\": {\"bytes\": 5, \"state\": \"stale\", \"stale_reason\": \"older_format\", \"format_version\": 4}}\n\
             {\"path\": \"f.fdu\", \"bytes\": 0, \"state\": \"absent\", \"content\": null}\n\
             {\"path\": \"notes.txt\", \"bytes\": 14, \"state\": \"unrecognized\", \"content\": null}\n\
             {\"path\": \".g.fdu.tmp.1.2.3\", \"bytes\": 60, \"state\": \"leftover\", \"leftover_kind\": \"staging_temporary\", \"content\": null}"
        );
        assert_eq!(
            render_cache_status(&[], CacheScope::All, Format::Jsonl),
            "{\"schema\": \"fdu.cache/3\"}"
        );
        assert_eq!(
            render_cache_status(&[], CacheScope::All, Format::Json),
            "{\n  \"schema\": \"fdu.cache/3\",\n  \"caches\": []\n}\n"
        );
        // An empty sequence in both formats: a bare `caches:` is YAML null, and a reader
        // of one schema should not have to tell null from a list it can iterate.
        assert_eq!(
            render_cache_status(&[], CacheScope::All, Format::Yaml),
            "schema: fdu.cache/3\ncaches: []\n"
        );
        assert!(
            render_cache_status(&stale[2..3], CacheScope::All, Format::Json)
                .starts_with("{\n  \"schema\": \"fdu.cache/3\",\n  \"caches\": [\n    {")
        );
        assert!(
            render_cache_status(&stale[2..3], CacheScope::All, Format::Yaml)
                .starts_with("schema: fdu.cache/3\ncaches:\n  -\n    path: c.fdu")
        );
        assert!(
            render_cache_status(&stale[2..3], CacheScope::All, Format::Yaml).ends_with(
                "state: stale\n    stale_reason: other_engine\n    format_version: null\n    \
                 content:\n      bytes: 5\n      state: stale\n      stale_reason: older_format\n      \
                 format_version: 4\n"
            )
        );
        assert!(render_cache_status(&leftovers[1..], CacheScope::All, Format::Yaml).ends_with(
            "state: leftover\n    leftover_kind: orphaned_content\n    content: null\n"
        ));
    }

    /// A current snapshot carries the identity of every tier it holds, and the sidecar
    /// beside it its own, nested the same way in JSON and YAML: the entry tier, then the
    /// control tier as the report's `ignore_rules` names it, and for content its entry tier,
    /// which alone holds the type rules, then the analyzer set, options, and analyzers under
    /// the names a report's `analysis` object gives them.
    #[test]
    fn cache_status_carries_every_stored_tier_identity() {
        use crate::{CacheScope, CacheState, ContentInfo, ContentState, ContentStatus};

        let snapshot = small_snapshot_identity();
        let content = crate::ContentTierIdentity {
            entries: snapshot.entries,
            analysis: crate::content::AnalysisSet::NONE.with_lines(),
            provenance: crate::AnalyzerProvenance {
                options_fingerprint: crate::content::OptionsFingerprint(5),
                analyzers: vec![(
                    crate::content::CONTENT_BASIC,
                    crate::content::AnalyzerVersion(1),
                )],
            },
        };
        let status = crate::CacheStatus {
            path: PathBuf::from("e.fdu"),
            bytes: 50,
            content: Some(ContentStatus {
                bytes: 9,
                state: ContentState::Current(ContentInfo { identity: content, records: 2 }),
            }),
            state: CacheState::Current(crate::SnapshotInfo {
                root: PathBuf::from("/tree"),
                identity: snapshot,
                entries: 3,
            }),
        };
        let entries = "{\"engine\": 1, \"max_depth\": null, \"follow_symlinks\": false, \
                       \"one_filesystem\": true, \"hidden_fingerprint\": 2, \"exclude_special\": false, \
                       \"population\": \"include\", \"control_fingerprint\": 0, \
                       \"type_rules_fingerprint\": 3, \"reducers_fingerprint\": 4}";
        assert_eq!(
            render_cache_status(std::slice::from_ref(&status), CacheScope::Root, Format::Jsonl),
            format!(
                "{{\"schema\": \"fdu.cache/3\"}}\n\
                 {{\"path\": \"e.fdu\", \"bytes\": 50, \"state\": \"current\", \"root\": \"/tree\", \
                 \"entries\": 3, \"identity\": {{\"entries\": {entries}, \"ignore_rules\": \
                 {{\"limits\": {{\"budget\": 10, \"line_limit\": null}}}}}}, \"content\": {{\"bytes\": 9, \
                 \"state\": \"current\", \"records\": 2, \"identity\": {{\"entries\": {entries}, \
                 \"analyze\": [\"lines\"], \"options_fingerprint\": 5, \
                 \"analyzers\": [{{\"id\": \"content-basic-v1\", \"version\": 1}}]}}}}}}"
            )
        );
        let entries = "\n          engine: 1\n          max_depth: null\n          \
                       follow_symlinks: false\n          one_filesystem: true\n          \
                       hidden_fingerprint: 2\n          exclude_special: false\n          \
                       population: include\n          control_fingerprint: 0\n          \
                       type_rules_fingerprint: 3\n          reducers_fingerprint: 4";
        assert_eq!(
            render_cache_status(std::slice::from_ref(&status), CacheScope::Root, Format::Yaml),
            format!(
                "schema: fdu.cache/3\ncaches:\n  -\n    path: e.fdu\n    bytes: 50\n    state: current\n    \
                 root: /tree\n    entries: 3\n    identity:\n      entries:{}\n      \
                 ignore_rules:\n        limits:\n          budget: 10\n          line_limit: null\n    \
                 content:\n      bytes: 9\n      state: current\n      records: 2\n      identity:\n        \
                 entries:{entries}\n        analyze:\n          - lines\n        \
                 options_fingerprint: 5\n        analyzers:\n          -\n            id: content-basic-v1\n            \
                 version: 1\n",
                entries.replace("\n  ", "\n")
            )
        );
        let mut large = status.clone();
        large.bytes = 60_696_111;
        large.content.as_mut().expect("fixture has a content sidecar").bytes = 120_783_062;
        assert_eq!(
            render_cache_status(std::slice::from_ref(&large), CacheScope::Root, Format::Text),
            "e.fdu  3 entries, 57 MiB metadata, 115 MiB content  /tree"
        );
        let colored = render_cache_status_with_options(
            std::slice::from_ref(&large),
            CacheScope::Root,
            Format::Text,
            RenderOptions { color: true, ..RenderOptions::default() },
        );
        assert_eq!(
            strip_ansi(&colored),
            render_cache_status(&[large], CacheScope::Root, Format::Text)
        );
        let zero = render_cache_status_with_options(
            std::slice::from_ref(&status),
            CacheScope::Root,
            Format::Text,
            RenderOptions { color: true, ..RenderOptions::default() },
        );
        assert!(!zero.contains("\x1b["), "positive sub-GiB sizes stay unstyled: {zero:?}");
        let mut zero_and_gib = status.clone();
        zero_and_gib.bytes = 0;
        zero_and_gib.content.as_mut().expect("fixture has a content sidecar").bytes = 1 << 30;
        let colored = render_cache_status_with_options(
            std::slice::from_ref(&zero_and_gib),
            CacheScope::Root,
            Format::Text,
            RenderOptions { color: true, ..RenderOptions::default() },
        );
        assert!(colored.contains(&paint("0 B", STYLE_DETAIL, true)), "{colored:?}");
        assert!(colored.contains(&paint("1.0 GiB", AnsiStyle::new().bold(), true)), "{colored:?}");
        assert_eq!(strip_ansi(&colored), "e.fdu  3 entries, 0 B metadata, 1.0 GiB content  /tree");
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            assert_eq!(
                render_cache_status_with_options(
                    std::slice::from_ref(&status),
                    CacheScope::Root,
                    format,
                    RenderOptions { color: true, ..RenderOptions::default() },
                ),
                render_cache_status(std::slice::from_ref(&status), CacheScope::Root, format)
            );
        }
        // A sidecar this build cannot serve is named in text too.
        let stale_content = crate::CacheStatus {
            content: Some(ContentStatus {
                bytes: 9,
                state: ContentState::Stale(crate::StaleReason::OtherEngine),
            }),
            ..status
        };
        assert_eq!(
            render_cache_status(&[stale_content], CacheScope::Root, Format::Text),
            "e.fdu  3 entries, 50 B metadata, 9 B stale content  /tree"
        );
    }

    /// The cache schema is a promise, like the report schema beside it.
    ///
    /// Fails loudly when the string moves, so the field rename this constant was added
    /// for — `recognized` to `state` — cannot happen again without a version to key on.
    #[test]
    fn the_cache_schema_constant_is_the_versioning_promise() {
        assert_eq!(CACHE_SCHEMA, "fdu.cache/3");
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            let rendered = render_cache_status(&[], crate::CacheScope::All, format);
            assert!(rendered.contains(CACHE_SCHEMA), "{format:?} carries no schema: {rendered}");
        }
    }

    #[cfg(all(unix, feature = "watch"))]
    #[test]
    fn cache_and_change_rows_preserve_non_unicode_raw_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(OsString::from_vec(vec![b'n', 0x80]));
        let status = crate::CacheStatus {
            path: path.clone(),
            bytes: 3,
            content: None,
            state: crate::CacheState::Unrecognized,
        };
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            let rendered =
                render_cache_status(std::slice::from_ref(&status), crate::CacheScope::All, format);
            assert!(rendered.contains("path_raw"), "{format:?}: {rendered}");
            assert!(rendered.contains("6e80"), "{format:?}: {rendered}");
        }

        let change = crate::Change {
            path,
            kind: crate::ChangeKind::Remove,
            entry_kind: None,
            bytes: None,
            allocated: None,
            mtime_ns: None,
            ignored: None,
            clock: 1,
        };
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            let rendered = render_change(&change, format);
            assert!(rendered.contains("path_raw"), "{format:?}: {rendered}");
            assert!(rendered.contains("6e80"), "{format:?}: {rendered}");
        }
    }

    /// A bound states itself, and the count it states is the count it dropped.
    ///
    /// The second half is why this is a unit test: a golden fixture is too small for a
    /// wrong total to look wrong, and the defect being guarded against — twenty rows of
    /// 192,871 presented as everything — only shows at a scale goldens do not reach.
    #[test]
    fn a_bound_states_itself_and_states_it_accurately() {
        let report = fixture(&[ViewSpec::Files]);
        let Section::Files { rows, total, .. } = &report.sections[0] else {
            panic!("expected a files section");
        };
        let full = rows.len();
        assert_eq!(*total, full, "an unbounded view drops nothing");
        assert!(!render(&report, Format::Text, false).contains("--limit all"), "and says nothing");
        assert!(render(&report, Format::Json, false).contains("\"bound\": null"));

        // Now bound it to one row and check the report agrees with reality.
        let mut query = Query { views: vec![ViewSpec::Files], ..Query::default() };
        query.selection.limit = Some(Bound::Limit(1));
        let bounded = fixture_for(&query);
        let Section::Files { rows, total, .. } = &bounded.sections[0] else {
            panic!("expected a files section");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(*total, full, "the total is what there was, not what was kept");

        let text = render(&bounded, Format::Text, false);
        assert!(!text.contains("of {full}"), "a single view states its bound in a note: {text}");
        assert!(!text.contains("--limit"), "actionable guidance stays out of the body: {text}");
        assert!(
            report_notes(&bounded)
                .contains(&format!("note: display limits: 1 of {full} rows shown")),
            "the epilogue states the bound: {:?}",
            report_notes(&bounded)
        );
        assert_eq!(
            report_tips(&bounded),
            vec!["tip: show more: limit=all"],
            "the shared epilogue names the row remedy once"
        );
        let json = render(&bounded, Format::Json, false);
        assert!(json.contains(&format!("\"shown\": 1, \"total\": {full}")), "{json:.200}");
        let yaml = render(&bounded, Format::Yaml, false);
        assert!(yaml.contains("shown: 1"), "{yaml:.200}");
    }

    /// Colour must not move anything.
    ///
    /// The golden suite structurally cannot check this: it runs under `NO_COLOR=1` and
    /// only ever sees the uncoloured form, which is exactly how the extensions view
    /// shipped misaligned — `{:<12}` counted the escape sequences toward the field width,
    /// so the padding collapsed the moment colour was on and every golden still passed.
    fn strip_ansi(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut chars = text.chars();
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

    #[test]
    fn colour_never_changes_the_layout_of_any_view() {
        for view in [
            ViewSpec::Tree,
            ViewSpec::Extensions,
            ViewSpec::Types,
            ViewSpec::Families,
            ViewSpec::Languages,
            ViewSpec::Files,
            ViewSpec::Summary,
        ] {
            let report = fixture(&[view]);
            let plain = render(&report, Format::Text, false);
            let coloured = render(&report, Format::Text, true);
            // Files is a bare listing meant for piping; this summary has no ignored
            // contribution to annotate. The other fixtures have a styled label.
            let styles_a_label = !matches!(view, ViewSpec::Files | ViewSpec::Summary);
            assert_eq!(
                plain != coloured,
                styles_a_label,
                "{view:?} disagrees with whether it styles a label"
            );
            assert_eq!(
                strip_ansi(&coloured).replace('·', "░").replace('▒', "█"),
                plain,
                "{view:?} lays out differently once colour is on"
            );
        }
    }

    /// The rule the helper exists to enforce, stated directly.
    #[test]
    fn a_label_cell_is_measured_on_visible_text() {
        let plain = label_cell("md", 6, STYLE_CATEGORY, false);
        let coloured = label_cell("md", 6, STYLE_CATEGORY, true);
        assert_eq!(plain, "md    ", "four spaces of padding");
        assert!(coloured.starts_with('\u{1b}'), "the label is styled");
        assert!(coloured.ends_with("    "), "and padded by the same four: {coloured:?}");
        // A label at or past the width gets no padding rather than a negative one.
        assert_eq!(label_cell("verylonglabel", 4, STYLE_CATEGORY, false), "verylonglabel");
    }

    /// Every view, so a matrix test cannot silently skip one that was added later.
    const ALL_TEST_VIEWS: [ViewSpec; 11] = [
        ViewSpec::List,
        ViewSpec::Tree,
        ViewSpec::Types,
        ViewSpec::Extensions,
        ViewSpec::Families,
        ViewSpec::Languages,
        ViewSpec::Documents,
        ViewSpec::Files,
        ViewSpec::Largest,
        ViewSpec::Recent,
        ViewSpec::Summary,
    ];

    /// Check a walk against its declaration without parsing or trusting a writer.
    struct SchemaCheck<S> {
        inner: S,
        expected: Vec<&'static str>,
        next: usize,
        depth: usize,
    }

    impl<S: Sink> SchemaCheck<S> {
        fn report(inner: S, lossy: bool, sections: bool) -> Self {
            Self::report_over(inner, lossy, sections, false)
        }

        /// The envelope's order for a report over one root, or over several (`several`),
        /// whose `root` is null and whose `roots` follows it.
        fn report_over(inner: S, lossy: bool, sections: bool, several: bool) -> Self {
            let fields = &REPORT_FIELDS;
            let ordered = [
                fields.schema,
                fields.generator,
                fields.root,
                fields.root_raw,
                fields.roots,
                fields.age_reference_ns,
                fields.age_reference_at,
                fields.request,
                fields.status,
                fields.provenance,
                fields.ignore_rules,
                fields.analysis,
                fields.reports,
            ];
            let expected = ordered
                .into_iter()
                .filter_map(|field| {
                    let present = match field.presence {
                        Presence::Always | Presence::Nullable => true,
                        // A lossy root is named in `root_raw` only when there is one root.
                        Presence::WhenLossy => lossy && !several,
                        Presence::WhenSet if field.name == fields.roots.name => several,
                        Presence::WhenSet => sections,
                        Presence::WhenAnalyzer(_) => panic!("envelope has no analyzer-owned field"),
                    };
                    present.then_some(field.name)
                })
                .collect();
            Self { inner, expected, next: 0, depth: 0 }
        }
    }

    impl<S: Sink> Sink for SchemaCheck<S> {
        type Output = S::Output;

        fn event(&mut self, event: Event<'_>) {
            match event {
                Event::BeginMap(_) | Event::BeginSeq(_) => self.depth += 1,
                Event::EndMap | Event::EndSeq => self.depth -= 1,
                Event::Key(name) if self.depth == 1 => {
                    assert_eq!(
                        Some(&name),
                        self.expected.get(self.next),
                        "wire field order/presence"
                    );
                    self.next += 1;
                }
                Event::Key(_) | Event::Scalar(_) => {}
            }
            self.inner.event(event);
        }

        fn finish(self) -> Self::Output {
            assert_eq!(self.depth, 0, "unclosed collection");
            assert_eq!(self.next, self.expected.len(), "required field missing");
            self.inner.finish()
        }
    }

    #[test]
    fn report_walk_obeys_declared_field_order_and_presence_for_every_writer() {
        for view in [ViewSpec::Summary, ViewSpec::Documents] {
            let report = fixture(&[view]);
            for sections in [false, true] {
                let mut json = SchemaCheck::report(JsonSink::pretty(), false, sections);
                emit_report(&mut json, &report, sections);
                assert!(!json.finish().is_empty());
                let mut line = SchemaCheck::report(JsonSink::line(), false, sections);
                emit_report(&mut line, &report, sections);
                assert!(!line.finish().is_empty());
                let mut yaml = SchemaCheck::report(YamlSink::new(), false, sections);
                emit_report(&mut yaml, &report, sections);
                assert!(!yaml.finish().is_empty());
            }
        }
    }

    #[test]
    #[should_panic(expected = "required field missing")]
    fn schema_check_rejects_a_missing_required_field() {
        let mut check = SchemaCheck::report(JsonSink::line(), false, true);
        check.event(Event::BeginMap(Shape::Inline));
        check.event(Event::EndMap);
        check.finish();
    }

    #[test]
    fn list_formats_preserve_the_default_tree_and_expose_flat_subtree_metrics() {
        let legacy = fixture(&[ViewSpec::Tree]);
        let list = fixture(&[ViewSpec::List]);
        assert_eq!(
            super::render(&legacy, Format::Text, false).expect("tree"),
            super::render(&list, Format::Tree, false).expect("list tree")
        );
        assert!(
            super::render(&list, Format::Paths, false).is_err(),
            "folding cannot silently become an inventory"
        );
        let mut rejected = Vec::new();
        assert_eq!(
            write(&list, Format::Paths, false, &mut rejected)
                .expect_err("folded projection")
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(rejected.is_empty(), "validate before writing any bytes");
        let flat = fixture_for(&Query {
            views: vec![ViewSpec::List],
            format: Format::Paths,
            selection: Selection {
                kinds: vec![EntryKind::Dir],
                size: SizeMetric::Apparent,
                ..Selection::default()
            },
            ..Query::default()
        });
        assert_eq!(super::render(&flat, Format::Paths, false).expect("paths"), "src\n");
        assert!(super::render(&flat, Format::Long, false).expect("long").contains("100 B"));
        assert!(super::render(&flat, Format::Tree, false).is_err());
        let Section::Files { rows, .. } = &flat.sections[0] else { panic!("flat list") };
        assert_eq!((rows[0].files, rows[0].dirs, rows[0].mtime_ns), (Some(1), Some(0), 10));
        assert_eq!(rows[0].complete, Some(true));
        assert!(
            super::render(&flat, Format::Json, false).expect("json").contains("\"complete\": true")
        );
        assert_eq!(
            rows[0].age_ns,
            flat.age_reference_ns.map(|reference| i128::from(reference) - 10)
        );
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            let wire = super::render(&flat, format, false).expect("serialization");
            assert!(wire.contains("age_reference_ns"));
            assert!(wire.contains("age_ns"));
        }
        assert_eq!(human_age(Some(-1)), "-0s");
        assert_eq!(human_age(Some(30 * 86400 * 1_000_000_000)), "30d");
        assert_eq!(human_age(None), "unknown");
        assert_eq!(flat_path(Path::new("a\nb\tc")), "a\\nb\\tc");
        // A backslash is the Windows separator; escaping it would print a path that
        // does not exist, so it is written as it is on every platform.
        assert_eq!(flat_path(Path::new("d/a\\b")), "d/a\\b");
        let mut stale = flat.clone();
        stale.provenance.source = ReportSource::CacheOnly;
        stale.provenance.freshness = Freshness::Stale;
        stale.scope.max_depth = Some(2);
        let notes = flat_diagnostics(&stale).join("\n");
        assert!(notes.contains("warn: stale answer"), "{notes}");
        assert!(!notes.contains("note: cache-only"), "the warning states it once: {notes}");
        assert!(notes.contains("note: result stale, complete"), "{notes}");
        assert!(notes.contains("scanned to depth 2 only"), "{notes}");
        assert_eq!(super::render(&stale, Format::Paths, false).expect("paths"), "src\n");
    }

    /// Only an answer nothing verified carries the stale warning, it names the surface's
    /// own option for a fresh answer, and it sits between the notes and the tips, where a
    /// quiet frontend's filter keeps it (fdu-mdop).
    #[test]
    fn only_a_cache_only_answer_warns_that_it_is_stale_in_the_surface_vocabulary() {
        let tree = Query { views: vec![ViewSpec::Tree], ..Query::default() };
        let mut verified = fixture_for(&tree);
        for source in [ReportSource::ColdScan, ReportSource::WarmRevalidate] {
            verified.provenance.source = source;
            assert!(report_warnings(&verified).is_empty(), "{source:?} is verified");
            assert!(!diagnostics(&verified).iter().any(|line| line.starts_with("warn:")));
        }
        for (axes, option) in [
            (&crate::query::AxisNames::FLAGS, "--stale-ok"),
            (&crate::query::AxisNames::FIELDS, "stale_ok"),
        ] {
            let mut stale = fixture_for(&Query {
                axes,
                selection: Selection { depth: Some(Bound::Limit(0)), ..Selection::default() },
                ..tree.clone()
            });
            stale.provenance.source = ReportSource::CacheOnly;
            stale.provenance.freshness = Freshness::Stale;
            let warning = format!(
                "warn: stale answer: served from the snapshot without filesystem verification; \
                 drop {option} for a fresh answer"
            );
            assert_eq!(report_warnings(&stale), std::slice::from_ref(&warning));
            assert!(!report_notes(&stale).contains(&warning), "a note would be quieted");
            let lines = diagnostics(&stale);
            let at = lines.iter().position(|line| *line == warning).expect("warning");
            assert!(lines[..at].iter().all(|line| line.starts_with("note:")), "{lines:?}");
            assert!(lines[at + 1..].iter().all(|line| line.starts_with("tip:")), "{lines:?}");
            assert!(lines.first().is_some_and(|line| line.starts_with("note:")), "{lines:?}");
            assert!(lines.last().is_some_and(|line| line.starts_with("tip:")), "{lines:?}");
            let rendered = super::render(&stale, Format::Text, false).expect("text");
            assert!(!rendered.contains("warn:"), "stdout carries only the answer: {rendered}");
        }
    }

    #[test]
    fn signed_ages_preserve_exact_endpoints_in_streaming_machine_formats() {
        let mut report = fixture_for(&Query {
            views: vec![ViewSpec::List],
            format: Format::Json,
            ..Query::default()
        });
        for (reference, modified) in [(i64::MAX, i64::MIN), (i64::MIN, i64::MAX)] {
            let age = i128::from(reference) - i128::from(modified);
            report.age_reference_ns = Some(reference);
            let Section::Files { rows, .. } = &mut report.sections[0] else { panic!("flat rows") };
            rows[0].mtime_ns = modified;
            rows[0].age_ns = Some(age);
            for format in [Format::Json, Format::Jsonl, Format::Yaml] {
                let rendered = super::render(&report, format, false).expect("machine format");
                assert!(rendered.contains(&age.to_string()), "{format:?} lost exact signed age");
                let mut streamed = Vec::new();
                write(&report, format, false, &mut streamed).expect("streaming writer");
                assert_eq!(streamed, rendered.as_bytes());
            }
        }
    }

    fn fixture(views: &[ViewSpec]) -> Report {
        fixture_for(&Query { views: views.to_vec(), ..Query::default() })
    }

    fn fixture_for(query: &Query) -> Report {
        let mut index = Index::new_with_scope("/root", ScanScope::default());
        // A documents view is an answer about analyzers, so the index it is rendered from
        // holds one: the request model refuses that view over an index with no content tier,
        // whichever door the request came through. `words` and not `all`, because the code
        // analyzer would move the languages view's share off bytes.
        if query.views.contains(&ViewSpec::Documents) {
            index.prepare_content_analysis(crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_words(),
                ..crate::content::AnalysisRequest::default()
            });
        }
        index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("src"),
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("src/main.rs"),
                    kind: EntryKind::File,
                    attrs: attrs(100, 10),
                },
                Op::Upsert {
                    path: PathBuf::from("notes.md"),
                    kind: EntryKind::File,
                    attrs: attrs(20, 20),
                },
            ]))
            .expect("apply");
        report(
            &index,
            &crate::test_support::read_of(&index, query.clone()),
            &Provenance {
                scan_started_at: Some(UNIX_EPOCH + Duration::from_secs(1_786_386_151)),
                generated_at: UNIX_EPOCH + Duration::from_secs(1_786_386_152),
                source: ReportSource::ColdScan,
                complete: true,
                errors: Vec::new(),
            },
        )
        .expect("report")
    }

    /// Whether a rendered line is a view header rather than a data row.
    ///
    /// Blank lines are excluded explicitly: `all` is vacuously true on an empty line, so
    /// the separator between blocks would otherwise count as a header.
    fn is_view_header_line(line: &str) -> bool {
        !line.is_empty() && line.chars().all(|c| c.is_ascii_uppercase())
    }

    /// A structural check that output is well-formed JSON.
    ///
    /// Hand-written serializers earn their keep only if something proves they balance, so
    /// this walks the text tracking nesting depth and string state.
    fn is_valid_json(text: &str) -> bool {
        let (mut depth, mut in_string, mut escaped) = (0i32, false, false);
        for ch in text.chars() {
            if in_string {
                match ch {
                    _ if escaped => escaped = false,
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }
            match ch {
                '"' => in_string = true,
                '{' | '[' => depth += 1,
                '}' | ']' => {
                    depth -= 1;
                    if depth < 0 {
                        return false;
                    }
                }
                _ => {}
            }
        }
        depth == 0 && !in_string
    }

    /// Remove insignificant JSON whitespace without touching string contents.
    ///
    /// Exact layout is intentionally allowed to change when the structural sink changes;
    /// field names, values, and ordering remain part of the wire promise.
    fn compact_json(text: &str) -> String {
        let mut compact = String::with_capacity(text.len());
        let (mut in_string, mut escaped) = (false, false);
        for ch in text.chars() {
            if in_string {
                compact.push(ch);
                match ch {
                    _ if escaped => escaped = false,
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }
            } else if ch == '"' {
                in_string = true;
                compact.push(ch);
            } else if !ch.is_ascii_whitespace() {
                compact.push(ch);
            }
        }
        compact
    }

    /// Formats are serializations, not features: no view may lack one.
    ///
    /// Driven from `ALL_TEST_VIEWS` rather than a hand-written list, because a list is
    /// exactly what goes stale — `largest` and `recent` would not have been in it.
    #[test]
    fn every_view_renders_in_every_format() {
        for view in ALL_TEST_VIEWS {
            let report = fixture(&[view]);
            for format in [Format::Text, Format::Json, Format::Jsonl, Format::Yaml] {
                let rendered = render(&report, format, false);
                assert!(!rendered.trim().is_empty(), "{view:?} in {format:?} rendered nothing");
                if format != Format::Text {
                    assert!(
                        rendered.contains("\"schema\"") || rendered.contains("schema:"),
                        "{view:?} as {format:?} carries no schema: {rendered:.120}"
                    );
                }
            }
        }
    }

    #[test]
    fn text_tree_restores_compact_bars_and_keeps_structural_indent_in_the_name_column() {
        // Apparent, so both rows print sizes of one width and the alignment below is about
        // the layout rather than about which sizes happen to round to the same block.
        let apparent = Query {
            views: vec![ViewSpec::Tree],
            selection: crate::query::Selection {
                size: crate::query::SizeMetric::Apparent,
                ..crate::query::Selection::default()
            },
            ..Query::default()
        };
        let text = render(&fixture_for(&apparent), Format::Text, false);
        // Every time in the fixture is a few nanoseconds after its epoch reference.
        assert_eq!(
            text,
            concat!(
                "██████████   100%       120 B  -0s  . 2 files\n",
                "████████░░    83%       100 B  -0s    src/ 1 file\n",
                "████████░░    83%       100 B  -0s      main.rs\n",
                "██░░░░░░░░    17%        20 B  -0s    notes.md\n",
            )
        );

        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0].find("120 B"), lines[1].find("100 B"));
        assert_eq!(lines[0].find('█'), lines[1].find('█'));
    }

    /// The age column sits between size and name, right-aligned to the section's widest
    /// cell: an age, the gray word `unknown` for a subtree not listed in full, a gray dash
    /// for a row that counts nothing, and a blank on the remainder row, which keeps the
    /// names aligned. Color changes none of the layout.
    #[test]
    fn the_tree_age_column_aligns_every_row_and_leaves_the_remainder_blank() {
        const SECOND: i64 = 1_000_000_000;
        let day = 86_400 * SECOND;
        // `--scan-depth 2`: `env/lib` was retained and never listed.
        let mut index = Index::new_with_scope(
            "/root",
            ScanScope { max_depth: Some(2), ..ScanScope::default() },
        );
        index
            .apply(&Observation::new(vec![
                Op::Upsert { path: "old".into(), kind: EntryKind::Dir, attrs: attrs(0, SECOND) },
                Op::Upsert {
                    path: "old/a.bin".into(),
                    kind: EntryKind::File,
                    attrs: attrs(1_000, SECOND),
                },
                Op::Upsert { path: "env".into(), kind: EntryKind::Dir, attrs: attrs(0, SECOND) },
                Op::Upsert {
                    path: "env/lib".into(),
                    kind: EntryKind::Dir,
                    attrs: attrs(0, SECOND),
                },
                Op::Upsert {
                    path: "env/x.bin".into(),
                    kind: EntryKind::File,
                    attrs: attrs(500, SECOND),
                },
                Op::Upsert {
                    path: "new.txt".into(),
                    kind: EntryKind::File,
                    attrs: attrs(300, 40 * day - 30 * SECOND),
                },
                Op::Upsert { path: "tiny".into(), kind: EntryKind::File, attrs: attrs(1, 2 * day) },
            ]))
            .expect("apply");
        index.set_initial_freshness(true);
        let query = Query {
            views: vec![ViewSpec::Tree],
            selection: Selection { size: SizeMetric::Apparent, ..Selection::default() },
            ..Query::default()
        };
        let mut request = crate::test_support::read_of(&index, query);
        request.now = UNIX_EPOCH + Duration::from_secs(40 * 86_400);
        let provenance = Provenance {
            scan_started_at: None,
            generated_at: request.now,
            source: ReportSource::ColdScan,
            complete: true,
            errors: Vec::new(),
        };
        let answer = report(&index, &request, &provenance).expect("report");
        let text = render(&answer, Format::Text, false);
        assert_eq!(
            text,
            concat!(
                "██████████   100%     1.7 KiB  unknown  . 4 files\n",
                "██████░░░░    56%     1,000 B      1mo    old/ 1 file\n",
                "██████░░░░    56%     1,000 B      1mo      a.bin\n",
                "███░░░░░░░    28%       500 B  unknown    env/ 1 file\n",
                "███░░░░░░░    28%       500 B      1mo      x.bin\n",
                "░░░░░░░░░░     0%         0 B  unknown      lib/ 0 files\n",
                "██░░░░░░░░    17%       300 B      30s    new.txt\n",
                "░░░░░░░░░░    <1%         1 B             … and 1 more file\n",
            ),
            "{text}"
        );
        assert_eq!(strip_ansi(&render(&answer, Format::Text, true)), text);

        // Machine formats carry the exact activity, completeness, and signed age, and an
        // RFC 3339 instant that is null wherever the time is a lower bound.
        let json = compact_json(&render(&answer, Format::Json, false));
        for expected in [
            "\"age_reference_ns\": 3456000000000000, \
             \"age_reference_at\": \"1970-02-10T00:00:00.000000000Z\"",
            "\"name\": \".\", \"path\": \"\", \"kind\": \"dir\"",
            "\"newest_mtime_ns\": 3455970000000000, \"mtime_ns\": 3455970000000000, \
             \"complete\": false, \"age_ns\": null, \"modified_at\": null, \"truncated\": true",
            "\"name\": \"old\", \"path\": \"old\", \"kind\": \"dir\"",
            "\"newest_mtime_ns\": 1000000000, \"mtime_ns\": 1000000000, \"complete\": true, \
             \"age_ns\": 3455999000000000, \"modified_at\": \"1970-01-01T00:00:01.000000000Z\"",
            "\"newest_mtime_ns\": 3455970000000000, \"mtime_ns\": 3455970000000000, \
             \"complete\": null, \"age_ns\": 30000000000, \
             \"modified_at\": \"1970-02-09T23:59:30.000000000Z\"",
        ] {
            assert!(json.contains(&compact_json(expected)), "missing {expected}\nin {json}");
        }
        for format in [Format::Jsonl, Format::Yaml] {
            let wire = render(&answer, format, false);
            for field in ["age_reference_at", "mtime_ns", "complete", "age_ns", "modified_at"] {
                assert!(wire.contains(field), "{format:?} lacks {field}: {wire}");
            }
        }

        // An empty root counts nothing, so its cell is a dash, not an age.
        let mut empty = Index::new_with_scope("/root", ScanScope::default());
        empty.set_initial_freshness(true);
        let lone = Query { views: vec![ViewSpec::Tree], ..Query::default() };
        let answer = report(&empty, &crate::test_support::read_of(&empty, lone), &provenance)
            .expect("report");
        assert_eq!(
            render(&answer, Format::Text, false),
            "░░░░░░░░░░      —         0 B  —  . 0 files\n"
        );
        let colored = render(&answer, Format::Text, true);
        assert!(colored.contains(&paint("—", STYLE_DETAIL, true)), "{colored:?}");

        // A root nobody listed (`--scan-depth 0`) has counted nothing either, but its age
        // is unknown: the dash would claim the subtree holds nothing.
        let mut unlisted = Index::new_with_scope(
            "/root",
            ScanScope { max_depth: Some(0), ..ScanScope::default() },
        );
        unlisted.set_initial_freshness(true);
        let lone = Query { views: vec![ViewSpec::Tree], ..Query::default() };
        let answer = report(&unlisted, &crate::test_support::read_of(&unlisted, lone), &provenance)
            .expect("report");
        let Section::Tree { root: Some(root), .. } = &answer.sections[0] else {
            panic!("expected a tree with a root row")
        };
        assert_eq!((root.mtime_ns, root.complete), (None, Some(false)), "{root:?}");
        let text = render(&answer, Format::Text, false);
        assert!(text.contains("  unknown  . 0 files\n"), "{text}");
        let colored = render(&answer, Format::Text, true);
        assert!(colored.contains(&paint("unknown", STYLE_DETAIL, true)), "{colored:?}");
    }

    #[test]
    fn language_text_uses_human_names_and_aligns_suffixes_with_color() {
        let mut index = Index::new_with_scope("/root", ScanScope::default());
        index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("main.cpp"),
                    kind: EntryKind::File,
                    attrs: attrs(100, 10),
                },
                Op::Upsert {
                    path: PathBuf::from("main.js"),
                    kind: EntryKind::File,
                    attrs: attrs(100, 20),
                },
            ]))
            .expect("apply");
        let report = report(
            &index,
            &crate::test_support::read_of(
                &index,
                Query { views: vec![ViewSpec::Languages], ..Query::default() },
            ),
            &Provenance {
                scan_started_at: None,
                generated_at: UNIX_EPOCH,
                source: ReportSource::ColdScan,
                complete: true,
                errors: Vec::new(),
            },
        )
        .expect("report");

        let plain = render(&report, Format::Text, false);
        assert!(plain.contains("C++"), "{plain}");
        assert!(plain.contains("JavaScript"), "{plain}");
        let plain_suffixes = plain
            .lines()
            .map(|line| line.find("1 file").expect("file count suffix"))
            .collect::<Vec<_>>();
        assert_eq!(plain_suffixes[0], plain_suffixes[1], "{plain}");

        let colored = render(&report, Format::Text, true);
        let colored_suffixes = colored
            .lines()
            .map(|line| line.find("1 file").expect("colored file count suffix"))
            .collect::<Vec<_>>();
        assert_eq!(colored_suffixes[0], colored_suffixes[1], "{colored:?}");

        let json = render(&report, Format::Json, false);
        assert!(json.contains("\"id\": \"cpp\""), "{json}");
        assert!(json.contains("\"id\": \"javascript\""), "{json}");
        assert!(!json.contains("\"id\": \"C++\""), "{json}");
        assert!(!json.contains("\"id\": \"JavaScript\""), "{json}");
    }

    #[test]
    fn text_labels_a_percentage_that_is_not_a_byte_share() {
        let mut languages = fixture(&[ViewSpec::Languages]);
        if let Section::Metrics { summary, .. } = &mut languages.sections[0] {
            summary.share_metric = ShareMetric::CodeLines;
        } else {
            panic!("languages should be a metric section");
        }
        let text = render(&languages, Format::Text, false);
        assert!(!text.contains("code lines"), "the result carries no annotation: {text}");
        let shares = |report: &Report| {
            report_notes(report)
                .into_iter()
                .filter(|note| note.starts_with("note: percentages are shares of"))
                .collect::<Vec<_>>()
        };
        assert_eq!(shares(&languages), ["note: percentages are shares of code lines"]);

        let Section::Metrics { summary, .. } = &mut languages.sections[0] else {
            unreachable!("languages should stay a metric section");
        };
        summary.share_metric = ShareMetric::AllocatedBytes;
        assert!(shares(&languages).is_empty(), "byte shares need no note");

        let mut both = fixture(&[ViewSpec::Languages, ViewSpec::Documents]);
        for (section, metric) in
            both.sections.iter_mut().zip([ShareMetric::CodeLines, ShareMetric::DocumentWords])
        {
            let Section::Metrics { summary, .. } = section else {
                panic!("both views should be metric sections");
            };
            summary.share_metric = metric;
        }
        assert_eq!(
            shares(&both),
            ["note: percentages are shares of code lines (LANGUAGES), document words (DOCUMENTS)"],
            "one note names each view's denominator"
        );

        // Beside a byte-share view, one non-byte denominator still names its section, so
        // the tree's percentages are not read as code lines.
        let mut beside_tree = fixture(&[ViewSpec::Tree, ViewSpec::Languages]);
        let Section::Metrics { summary, .. } = &mut beside_tree.sections[1] else {
            panic!("languages should be a metric section");
        };
        summary.share_metric = ShareMetric::CodeLines;
        assert_eq!(
            shares(&beside_tree),
            ["note: percentages are shares of code lines (LANGUAGES)"]
        );

        // Sections sharing a denominator are listed under it once, in first-seen order,
        // even when another denominator's section sits between them.
        let mut interleaved =
            fixture(&[ViewSpec::Languages, ViewSpec::Documents, ViewSpec::Families]);
        for (section, metric) in interleaved.sections.iter_mut().zip([
            ShareMetric::CodeLines,
            ShareMetric::DocumentWords,
            ShareMetric::CodeLines,
        ]) {
            let Section::Metrics { summary, .. } = section else {
                panic!("languages, documents, and families should be metric sections");
            };
            summary.share_metric = metric;
        }
        assert_eq!(
            shares(&interleaved),
            ["note: percentages are shares of code lines (LANGUAGES, FAMILIES), document words \
                 (DOCUMENTS)"],
            "a shared denominator is named once"
        );
    }

    #[test]
    fn share_floor_omissions_name_their_section_beside_other_views() {
        let mut report = fixture(&[ViewSpec::Types, ViewSpec::Families]);
        for section in &mut report.sections {
            let Section::Metrics { summary, .. } = section else {
                panic!("types and families should be metric sections");
            };
            summary.share_omitted = 2;
        }
        let limits = report_notes(&report)
            .into_iter()
            .filter(|note| note.starts_with("note: display limits:"))
            .collect::<Vec<_>>();
        assert_eq!(
            limits,
            ["note: display limits: 2 rows below min share in TYPES, 2 rows below min share in \
              FAMILIES"],
            "equal counts in different views stay distinct"
        );
    }

    #[test]
    fn json_output_is_well_formed_for_every_view() {
        for view in [
            ViewSpec::Tree,
            ViewSpec::Extensions,
            ViewSpec::Types,
            ViewSpec::Families,
            ViewSpec::Languages,
            ViewSpec::Documents,
            ViewSpec::Files,
            ViewSpec::Summary,
        ] {
            let json = render(&fixture(&[view]), Format::Json, false);
            assert!(is_valid_json(&json), "unbalanced JSON for {view:?}:\n{json}");
        }
        let all = render(
            &fixture(&[
                ViewSpec::Tree,
                ViewSpec::Extensions,
                ViewSpec::Types,
                ViewSpec::Files,
                ViewSpec::Summary,
            ]),
            Format::Json,
            false,
        );
        assert!(is_valid_json(&all), "unbalanced JSON for a multi-view report:\n{all}");
    }

    #[test]
    fn streaming_machine_writers_match_string_rendering() {
        struct Fails;
        impl std::io::Write for Fails {
            fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("closed"))
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let report = fixture(&[
            ViewSpec::Tree,
            ViewSpec::Extensions,
            ViewSpec::Types,
            ViewSpec::Files,
            ViewSpec::Summary,
        ]);
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            let expected = render(&report, format, false);
            let mut streamed = Vec::new();
            write(&report, format, false, &mut streamed).expect("stream report");
            assert_eq!(streamed, expected.as_bytes(), "{format:?} bytes differ");
        }

        let error = write(&report, Format::Json, false, &mut Fails).expect_err("writer fails");
        assert_eq!(error.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn streaming_tree_walk_handles_many_siblings_without_collecting_output() {
        #[derive(Default)]
        struct Count(u64);
        impl std::io::Write for Count {
            fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
                self.0 = self.0.saturating_add(buffer.len() as u64);
                Ok(buffer.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let mut report = fixture(&[ViewSpec::Tree]);
        let Section::Tree { root: Some(root), .. } = &mut report.sections[0] else {
            panic!("tree fixture must contain a tree");
        };
        let template = root.children[0].clone();
        root.children = (0..10_000)
            .map(|index| {
                let mut child = template.clone();
                child.name = format!("child-{index}");
                child.path = PathBuf::from(&child.name);
                child
            })
            .collect();

        let mut output = Count::default();
        write(&report, Format::Json, false, &mut output).expect("stream wide report");
        assert!(output.0 > 1_000_000, "wide fixture must exercise substantial output");
    }

    #[test]
    fn nested_json_separates_siblings_without_a_trailing_comma() {
        // The original fixture had no directory with two children, so a balanced-but-
        // invalid `[{a}{b},]` passed the structural check. Sibling separators need a
        // case that actually has siblings, at more than one level.
        let mut index = Index::new_with_scope("/root", ScanScope::default());
        index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a"),
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b"),
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("c"),
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a/inner"),
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a/other"),
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                },
            ]))
            .expect("apply");
        let report = report(
            &index,
            &crate::test_support::read_of(
                &index,
                Query {
                    selection: Selection {
                        depth: Some(Bound::All),
                        min_share: Some(crate::query::ShareThreshold::parse("0%").expect("share")),
                        ..Selection::default()
                    },
                    views: vec![ViewSpec::Tree],
                    ..Query::default()
                },
            ),
            &Provenance {
                scan_started_at: None,
                generated_at: UNIX_EPOCH,
                source: ReportSource::ColdScan,
                complete: true,
                errors: Vec::new(),
            },
        )
        .expect("report");

        let json = render(&report, Format::Json, false);
        assert!(is_valid_json(&json), "{json}");
        assert!(!json.contains("}{"), "siblings must be separated:\n{json}");
        assert!(!json.contains(",]"), "no trailing comma before a close:\n{json}");
        assert!(!json.contains("[,"), "no leading comma after an open:\n{json}");
        // Three top-level siblings and two nested ones must all be present.
        for name in ["\"a\"", "\"b\"", "\"c\"", "\"inner\"", "\"other\""] {
            assert!(json.contains(name), "missing {name} in:\n{json}");
        }
    }

    #[test]
    fn jsonl_emits_one_document_per_line() {
        let rendered =
            render(&fixture(&[ViewSpec::Extensions, ViewSpec::Summary]), Format::Jsonl, false);
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 3, "one envelope plus one line per section");
        for line in &lines {
            assert!(is_valid_json(line), "line is not a JSON document: {line}");
        }
        assert!(lines[0].contains("\"schema\""), "the envelope carries provenance");
    }

    #[test]
    fn machine_output_carries_the_schema_and_provenance() {
        let json = render(&fixture(&[ViewSpec::Summary]), Format::Json, false);
        assert!(json.contains("\"schema\": \"fdu.report/11\""));
        assert!(json.contains("\"request\": {"));
        assert!(json.contains("\"status\": {"));
        assert!(json.contains("\"provenance\": {"));
        assert!(json.contains("\"source\": \"cold_scan\""));
        assert!(json.contains("\"complete\": true"));
        // Timestamps render in the same grammar the CLI accepts back as a watermark.
        assert!(json.contains("\"scan_started_at\": \"2026-08-10T18:22:31.000000000Z\""), "{json}");
        assert!(json.contains("\"generated_at\": \"2026-08-10T18:22:32.000000000Z\""), "{json}");
    }

    #[test]
    fn the_schema_constant_is_the_versioning_promise() {
        // Fails loudly when the schema string moves, so a field rename cannot ship
        // without a deliberate version bump and a golden update.
        assert_eq!(REPORT_SCHEMA, "fdu.report/11");
        assert_eq!(CONTENT_REPORT_SCHEMA, REPORT_SCHEMA);
    }

    /// Every format says whether ignore rules were read and which files were refused, and
    /// text names the directories and the knob as the requesting surface spells it.
    #[test]
    fn every_format_states_the_ignore_rules_a_report_could_apply() {
        let unobserved = crate::test_support::not_observing_controls();
        let mut blind = Index::new_with_scope("/root", unobserved);
        blind
            .apply(&Observation::new(vec![Op::Upsert {
                path: PathBuf::from("a.txt"),
                kind: EntryKind::File,
                attrs: attrs(1, 1),
            }]))
            .expect("apply");
        let provenance = Provenance {
            scan_started_at: None,
            generated_at: UNIX_EPOCH,
            source: ReportSource::ColdScan,
            complete: true,
            errors: Vec::new(),
        };
        let query = Query { views: vec![ViewSpec::Summary], ..Query::default() };
        let blind_report =
            report(&blind, &crate::test_support::read_of(&blind, query.clone()), &provenance)
                .expect("report");
        assert!(render(&blind_report, Format::Json, false).contains("\"ignore_rules\": null"));
        assert!(render(&blind_report, Format::Yaml, false).contains("\nignore_rules: null\n"));
        assert!(blind_report.notes.is_empty());

        let mut observed =
            Index::new_with_scope("/root", crate::test_support::observing_controls());
        let mut long_line = vec![b'x'; crate::control::DEFAULT_CONTROL_LINE_LIMIT + 1];
        long_line.push(b'\n');
        observed
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("vendor"),
                    kind: EntryKind::Dir,
                    attrs: attrs(0, 1),
                },
                Op::ControlUpsert {
                    path: PathBuf::from(".gitignore"),
                    source: b"*.log\n".to_vec(),
                },
                Op::ControlUpsert { path: PathBuf::from("vendor/.gitignore"), source: long_line },
            ]))
            .expect("apply");
        // The platform spells the refused path, so Windows writes a backslash.
        let refused = Path::new("vendor").join(".gitignore");
        let refused = refused.to_string_lossy();
        let json = render(
            &report(
                &observed,
                &crate::test_support::read_of(&observed, query.clone()),
                &provenance,
            )
            .expect("report"),
            Format::Json,
            false,
        );
        let expected = format!(
            "\"ignore_rules\": {{\"limits\": {{\"budget\": 4194304, \"line_limit\": 16384}}, \
             \"applied\": 1, \"rules\": 1, \"refused\": 1, \"refusals\": [{{\"path\": {}, \"reason\": \
             \"line_limit\"}}]}}",
            quote(&refused)
        );
        assert!(compact_json(&json).contains(&compact_json(&expected)), "{json}");
        assert!(json.contains("\"complete\": true"), "a refusal is not an operational partial");
        let yaml = render(
            &report(
                &observed,
                &crate::test_support::read_of(&observed, query.clone()),
                &provenance,
            )
            .expect("report"),
            Format::Yaml,
            false,
        );
        let expected = format!(
            "ignore_rules:\n  limits: {{budget: 4194304, line_limit: 16384}}\n  applied: 1\n  rules: 1\n  \
             refused: 1\n  refusals:\n    -\n      path: {}\n      reason: line_limit\n",
            yaml_scalar(&refused)
        );
        assert!(yaml.contains(&expected), "{yaml}");

        let flags = Query { axes: &crate::query::AxisNames::FLAGS, ..query.clone() };
        let candidate =
            report(&observed, &crate::test_support::read_of(&observed, flags), &provenance)
                .expect("report");
        let text = render(&candidate, Format::Text, false);
        assert!(
            !text.contains("note:") && !text.contains("tip:"),
            "stdout holds only formatted data"
        );
        let lines = diagnostics(&candidate);
        assert!(
            lines.iter().any(|line| line.starts_with("note: ignore classification incomplete:")
                && line.contains("vendor"))
        );
        assert!(lines.last().expect("remedy").contains("--gitignore-line-limit"));
        let fields =
            report(&observed, &crate::test_support::read_of(&observed, query), &provenance)
                .expect("report");
        assert!(report_tips(&fields).iter().any(|tip| tip.contains("control_line_limit")));
        assert!(!diagnostics(&fields).iter().any(|line| line.contains("--gitignore-line-limit")));
    }

    /// Every row that carries an ignored share says so in every format: text appends it
    /// only when something is ignored and the selection is not ignored entries alone, and
    /// machine formats write a zero share when nothing is and `null` when no rule was read.
    #[test]
    fn every_format_carries_each_rows_ignored_share() {
        let build = |scope: ScanScope| {
            let mut index = Index::new_with_scope("/root", scope);
            let mut ops = vec![
                Op::Upsert {
                    path: PathBuf::from("dist"),
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("dist/a.gz"),
                    kind: EntryKind::File,
                    attrs: attrs(128, 10),
                },
                Op::Upsert {
                    path: PathBuf::from("src"),
                    kind: EntryKind::Dir,
                    attrs: Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("src/b.rs"),
                    kind: EntryKind::File,
                    attrs: attrs(36, 20),
                },
            ];
            if scope.observes_controls() {
                ops.insert(
                    0,
                    Op::ControlUpsert {
                        path: PathBuf::from(".gitignore"),
                        source: b"dist/\n".to_vec(),
                    },
                );
            }
            index.apply(&Observation::new(ops)).expect("apply");
            index
        };
        let provenance = Provenance {
            scan_started_at: None,
            generated_at: UNIX_EPOCH,
            source: ReportSource::ColdScan,
            complete: true,
            errors: Vec::new(),
        };
        let views = vec![ViewSpec::Summary, ViewSpec::Tree, ViewSpec::Extensions, ViewSpec::Files];
        let query = |ignored| Query {
            views: views.clone(),
            selection: Selection {
                ignored,
                size: SizeMetric::Apparent,
                limit: Some(Bound::All),
                ..Selection::default()
            },
            ..Query::default()
        };

        let observed = build(crate::test_support::observing_controls());
        let text = render(
            &report(
                &observed,
                &crate::test_support::read_of(&observed, query(IgnoredEntries::Include)),
                &provenance,
            )
            .expect("report"),
            Format::Text,
            false,
        );
        assert_eq!(
            text,
            concat!(
                "SUMMARY\n",
                "     164 B  2 files, 2 directories (128 B gitignored)\n",
                "\n",
                "TREE\n",
                "██████████   100%       164 B  -0s  . 2 files (128 B gitignored)\n",
                "████████░░    78%       128 B  -0s    dist/ 1 file (128 B gitignored)\n",
                "████████░░    78%       128 B  -0s      a.gz (128 B gitignored)\n",
                "██░░░░░░░░    22%        36 B  -0s    src/ 1 file\n",
                "██░░░░░░░░    22%        36 B  -0s      b.rs\n",
                "\n",
                "EXTENSIONS\n",
                "     128 B  .gz          1 file (128 B gitignored)\n",
                "      36 B  .rs          1 file\n",
                "\n",
                "FILES\n",
                "dist\n",
                "dist/a.gz\n",
                "src\n",
                "src/b.rs\n",
            )
            .replace("dist/a.gz", &format!("dist{}a.gz", std::path::MAIN_SEPARATOR))
            .replace("src/b.rs", &format!("src{}b.rs", std::path::MAIN_SEPARATOR))
        );
        // A share of ignored directories alone holds no bytes, so text says nothing of it.
        let dirs_only = IgnoredTally { files: 0, dirs: 1, bytes: 0, allocated: 0 };
        assert_eq!(
            ignored_suffix(Some(dirs_only), SizeMetric::Apparent, IgnoredEntries::Include, false),
            ""
        );
        let large_ignored = IgnoredTally { files: 1, dirs: 0, bytes: 1 << 30, allocated: 1 << 30 };
        let suffix = ignored_suffix(
            Some(large_ignored),
            SizeMetric::Apparent,
            IgnoredEntries::Include,
            true,
        );
        assert!(suffix.contains(&paint("1.0 GiB", STYLE_DETAIL.bold(), true)), "{suffix:?}");
        assert_eq!(strip_ansi(&suffix), " (1.0 GiB gitignored)");
        let only = render(
            &report(
                &observed,
                &crate::test_support::read_of(&observed, query(IgnoredEntries::Only)),
                &provenance,
            )
            .expect("report"),
            Format::Text,
            false,
        );
        assert!(only.contains("     128 B  1 file, 1 directory\n"), "{only}");
        assert!(!only.contains("ignored"), "every row is ignored, so none repeats it: {only}");

        let json = render(
            &report(
                &observed,
                &crate::test_support::read_of(&observed, query(IgnoredEntries::Include)),
                &provenance,
            )
            .expect("report"),
            Format::Json,
            false,
        );
        assert!(is_valid_json(&json), "{json}");
        let compact = compact_json(&json);
        for expected in [
            "\"summary\": {\"files\": 2, \"dirs\": 2, \"bytes\": 164, \"allocated\": 1024, \
             \"ignored\": {\"files\": 1, \"dirs\": 1, \"bytes\": 128, \"allocated\": 512}, ",
            "\"name\": \"src\", \"path\": \"src\", \"kind\": \"dir\", \"entry_ignored\": false, \"bytes\": 36, \
             \"allocated\": 512, \"files\": 1, \"dirs\": 0, \"ignored\": {\"files\": 0, \
             \"dirs\": 0, \"bytes\": 0, \"allocated\": 0}, ",
            "{\"extension\": \".gz\", \"files\": 1, \"bytes\": 128, \"allocated\": 512, \
             \"ignored\": {\"files\": 1, \"bytes\": 128, \"allocated\": 512}}",
            "\"kind\": \"dir\", \"bytes\": 128, \"allocated\": 512, \"mtime_ns\": 10, \"files\": 1, \"dirs\": 0, \"complete\": true, \"age_ns\": -10, \
             \"modified_at\": \"1970-01-01T00:00:00.000000010Z\", \"ignored\": true, \"sort_value\": null, \"classification\": null}",
        ] {
            assert!(compact.contains(&compact_json(expected)), "missing {expected}\nin {json}");
        }
        let yaml = render(
            &report(
                &observed,
                &crate::test_support::read_of(&observed, query(IgnoredEntries::Include)),
                &provenance,
            )
            .expect("report"),
            Format::Yaml,
            false,
        );
        assert!(
            yaml.contains(
                "      allocated: 1024\n      ignored: {files: 1, dirs: 1, bytes: 128, \
                 allocated: 512}\n      newest_mtime_ns: 20\n"
            ),
            "{yaml}"
        );
        assert!(yaml.contains("        ignored: true\n"), "{yaml}");

        let blind = build(crate::test_support::not_observing_controls());
        let blind_report = report(
            &blind,
            &crate::test_support::read_of(&blind, query(IgnoredEntries::Include)),
            &provenance,
        )
        .expect("report");
        assert!(!render(&blind_report, Format::Text, false).contains("ignored"));
        let json = render(&blind_report, Format::Json, false);
        assert!(!json.contains("\"ignored\": {"), "never a zero share for an unread rule: {json}");
        assert!(json.contains("\"ignored\": null"), "{json}");
        assert!(render(&blind_report, Format::Yaml, false).contains("ignored: null\n"));
    }

    #[test]
    fn every_report_uses_one_schema_and_states_nullable_analysis() {
        let metadata = render(&fixture(&[ViewSpec::Tree]), Format::Json, false);
        assert!(metadata.contains("\"schema\": \"fdu.report/11\""));
        assert!(metadata.contains("\"analysis\": null"));

        let metrics = render(&fixture(&[ViewSpec::Types]), Format::Json, false);
        assert!(metrics.contains("\"schema\": \"fdu.report/11\""));
        assert!(metrics.contains("\"analysis\": null"));
        assert!(metrics.contains("\"share\": {\"numerator\":"));
    }

    /// The change stream carries the same promise the report does.
    ///
    /// A constant assertion alone would not: it pins the version string while leaving the
    /// record's shape free to change underneath it, which is the failure the promise
    /// exists to prevent. This pins the whole record, so adding, renaming, or reordering
    /// a field fails here and forces a deliberate version bump.
    ///
    /// `ignored` was added to `fdu.stream/2` before the first release, so no consumer has
    /// ever read the earlier draft shape it extends.
    #[cfg(feature = "watch")]
    #[test]
    fn a_stream_record_is_pinned_field_by_field() {
        use crate::{Change, ChangeKind};

        assert_eq!(STREAM_SCHEMA, "fdu.stream/2");

        let upsert = Change {
            path: ["src", "main.rs"].iter().collect(),
            kind: ChangeKind::Upsert,
            entry_kind: Some(EntryKind::File),
            bytes: Some(2_048),
            allocated: Some(4_096),
            mtime_ns: Some(1_700_000_000_000_000_000),
            ignored: Some(false),
            clock: 7,
        };
        // Path separators differ by platform, so the expectation is built the same way
        // the renderer builds it rather than hardcoding a slash.
        let path = upsert.path.to_string_lossy().replace('\\', "\\\\");
        assert_eq!(
            render_change(&upsert, Format::Json),
            format!(
                "{{\"schema\": \"fdu.stream/2\", \"record\": \"change\", \"op\": \"upsert\", \
                 \"path\": \"{path}\", \"clock\": 7, \"kind\": \"file\", \"bytes\": 2048, \
                 \"allocated\": 4096, \"mtime_ns\": 1700000000000000000, \"ignored\": false}}"
            )
        );

        // A run that read no ignore rules classifies nothing, and the field is absent
        // rather than false: the same distinction every report row draws.
        let unclassified = Change { ignored: None, ..upsert.clone() };
        assert_eq!(
            render_change(&unclassified, Format::Json),
            format!(
                "{{\"schema\": \"fdu.stream/2\", \"record\": \"change\", \"op\": \"upsert\", \
                 \"path\": \"{path}\", \"clock\": 7, \"kind\": \"file\", \"bytes\": 2048, \
                 \"allocated\": 4096, \"mtime_ns\": 1700000000000000000}}"
            )
        );

        // A removal has no metadata to report, and the optional fields must be absent
        // rather than null: a consumer distinguishes "gone" from "unknown" by their
        // absence.
        let removed = Change {
            path: PathBuf::from("gone.txt"),
            kind: ChangeKind::Remove,
            entry_kind: None,
            bytes: None,
            allocated: None,
            mtime_ns: None,
            ignored: None,
            clock: 8,
        };
        assert_eq!(
            render_change(&removed, Format::Json),
            "{\"schema\": \"fdu.stream/2\", \"record\": \"change\", \"op\": \"remove\", \
             \"path\": \"gone.txt\", \"clock\": 8}"
        );

        // A removal an ignore-rule edit caused is the one that carries a classification:
        // the entry is still on disk, and the new bit is why it left the selection.
        let reclassified =
            Change { path: PathBuf::from("debug.log"), ignored: Some(true), ..removed.clone() };
        assert_eq!(
            render_change(&reclassified, Format::Json),
            "{\"schema\": \"fdu.stream/2\", \"record\": \"change\", \"op\": \"remove\", \
             \"path\": \"debug.log\", \"clock\": 8, \"ignored\": true}"
        );

        // An invalidation says the consumer's view may have gaps. It is the one record
        // that must never be dropped, so its shape is pinned too.
        let invalidated = Change {
            path: PathBuf::from("subtree"),
            kind: ChangeKind::Invalidate,
            entry_kind: None,
            bytes: None,
            allocated: None,
            mtime_ns: None,
            ignored: None,
            clock: 9,
        };
        assert_eq!(
            render_change(&invalidated, Format::Json),
            "{\"schema\": \"fdu.stream/2\", \"record\": \"change\", \"op\": \"invalidate\", \
             \"path\": \"subtree\", \"clock\": 9}"
        );

        // Text is the greppable form: path first, operation second, tab-separated.
        assert_eq!(render_change(&removed, Format::Text), "gone.txt\tremove");
    }

    #[test]
    fn a_files_view_prints_one_path_per_line_and_nothing_else() {
        // The property that makes `fdu --view files | xargs` work. It is why the view
        // header is conditional: a lone files view is a path listing, not a table that
        // needs labelling, so nothing is prepended to it.
        let text = render(&fixture(&[ViewSpec::Files]), Format::Text, false);
        for line in text.lines() {
            assert!(!line.contains(' '), "text files output must be bare paths, got {line:?}");
        }
        let expected: PathBuf = ["src", "main.rs"].iter().collect();
        let expected = expected.display().to_string();
        assert!(text.lines().any(|line| line == expected), "{text}");
    }

    #[test]
    fn several_views_are_labelled_and_a_lone_view_is_left_bare() {
        // Concatenated blocks of similar-looking rows were the problem: a reader had to
        // recover which view produced which table from the order they were requested in.
        let text = render(
            &fixture(&[ViewSpec::Tree, ViewSpec::Types, ViewSpec::Summary]),
            Format::Text,
            false,
        );
        let headers: Vec<&str> = text.lines().filter(|line| is_view_header_line(line)).collect();
        assert_eq!(headers, ["TREE", "TYPES", "SUMMARY"], "{text}");

        // Each header sits directly above the rows it labels, and one blank line
        // separates the blocks.
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if headers.contains(line) {
                assert!(
                    lines.get(index + 1).is_some_and(|next| !next.is_empty()),
                    "header {line} must sit directly above its rows:\n{text}"
                );
                if index > 0 {
                    assert!(
                        lines[index - 1].is_empty(),
                        "a blank line must precede header {line}:\n{text}"
                    );
                }
            }
        }

        // The same views alone keep the pre-header layout exactly.
        for view in [ViewSpec::Tree, ViewSpec::Types, ViewSpec::Summary] {
            let lone = render(&fixture(&[view]), Format::Text, false);
            assert!(
                !lone.lines().any(is_view_header_line),
                "{view:?} alone must not be labelled:\n{lone}"
            );
        }
    }

    #[test]
    fn view_headers_are_colorized_only_when_color_is_on() {
        let views = [ViewSpec::Tree, ViewSpec::Summary];
        let plain = render(&fixture(&views), Format::Text, false);
        assert!(plain.starts_with("TREE\n"), "{plain}");
        assert!(!plain.contains('\u{1b}'), "uncolored text carries no escapes: {plain:?}");

        let colored = render(&fixture(&views), Format::Text, true);
        assert!(colored.contains(&paint("TREE", STYLE_HEADING, true)), "{colored:?}");
        assert!(colored.contains(&paint("SUMMARY", STYLE_HEADING, true)), "{colored:?}");
    }

    #[test]
    fn human_rows_style_only_their_semantic_spans() {
        let ignored = IgnoredTally { files: 1, dirs: 0, bytes: 43, allocated: 43 };
        let mut tree = fixture(&[ViewSpec::Tree]);
        let Section::Tree { root: Some(root), .. } = &mut tree.sections[0] else { panic!("tree") };
        root.name = "a(b)\n界".into();
        root.files = 3_508;
        root.ignored = Some(ignored);
        let plain = render(&tree, Format::Text, false);
        let colored = render(&tree, Format::Text, true);
        assert!(
            colored.contains(&format!(
                "{} 3,508 files {}",
                human_name("a(b)\n界", EntryKind::Dir, None, true),
                detail("(43 B gitignored)", true)
            )),
            "{colored:?}"
        );
        assert_eq!(strip_ansi(&colored).replace('·', "░").replace('▒', "█"), plain);
        assert!(!colored.contains("a(b)\n界"));

        let mut summary = fixture(&[ViewSpec::Summary]);
        let Section::Summary(row) = &mut summary.sections[0] else { panic!("summary") };
        row.files = 3_508;
        row.ignored = Some(ignored);
        let colored = render(&summary, Format::Text, true);
        assert!(
            colored.contains(&format!(
                "3,508 files, 1 directory {}",
                detail("(43 B gitignored)", true)
            )),
            "{colored:?}"
        );
        assert_eq!(strip_ansi(&colored), render(&summary, Format::Text, false));

        let mut types = fixture(&[ViewSpec::Extensions]);
        let Section::Extensions { rows, .. } = &mut types.sections[0] else { panic!("extensions") };
        rows[0].extension = ".(txt)".into();
        rows[0].files = 3_508;
        rows[0].ignored = Some(ignored);
        let colored = render(&types, Format::Text, true);
        assert!(colored.contains(&paint(".(txt)", STYLE_CATEGORY, true)), "{colored:?}");
        assert!(
            colored.contains(&format!("3,508 files {}", detail("(43 B gitignored)", true))),
            "{colored:?}"
        );
        assert_eq!(strip_ansi(&colored), render(&types, Format::Text, false));
    }

    /// Where a non-language grouped row's continuation lines start: past the size, share,
    /// and floored label cells and the space before the file count, then two more.
    const TEXT_METRIC_INDENT: usize =
        TEXT_SIZE_WIDTH + 2 + TEXT_SHARE_WIDTH + 2 + TEXT_METRIC_LABEL_WIDTH + 1 + 2;

    #[test]
    fn metric_breakdowns_and_ranked_paths_keep_span_boundaries() {
        let mut metrics = fixture(&[ViewSpec::Types]);
        let Section::Metrics { summary, .. } = &mut metrics.sections[0] else { panic!("metrics") };
        let row = &mut summary.rows[0];
        row.files = 1_234;
        row.metrics.physical_lines = Some(477_298);
        row.metrics.nonblank_lines = Some(439_949);
        row.metrics.blank_lines = Some(37_349);
        let colored = render(&metrics, Format::Text, true);
        assert!(
            colored.contains(&format!(
                "1,234 files\n{}477,298 lines {}\n",
                " ".repeat(TEXT_METRIC_INDENT),
                detail("(439,949 nonblank, 37,349 blank)", true)
            )),
            "{colored:?}"
        );
        assert_eq!(strip_ansi(&colored), render(&metrics, Format::Text, false));

        let mut largest = fixture(&[ViewSpec::Largest]);
        let Section::Files { rows, .. } = &mut largest.sections[0] else { panic!("files") };
        rows[0].path = PathBuf::from("a(b)\n界.rs");
        let colored = render(&largest, Format::Text, true);
        assert!(colored.contains(&paint("a(b)\\n界.rs", STYLE_NAME, true)), "{colored:?}");
        assert_eq!(strip_ansi(&colored), render(&largest, Format::Text, false));
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            assert!(!render(&largest, format, true).contains('\u{1b}'));
        }
    }

    /// A grouped row's first line is its tabulated cells and file count; each further
    /// measure is a line of its own, two columns right of the file count, with the size,
    /// share, and label cells left blank. The widest row then no longer sets the report's
    /// width, which on the Linux kernel's DOCUMENTS was 154 columns (fdu-r7lw).
    #[test]
    fn grouped_rows_stack_each_further_measure_under_the_file_count() {
        for view in [ViewSpec::Types, ViewSpec::Families, ViewSpec::Languages, ViewSpec::Documents]
        {
            let mut metrics = fixture(&[view]);
            let Section::Metrics { summary, .. } = &mut metrics.sections[0] else {
                panic!("{view:?} should be a metric section")
            };
            let row = &mut summary.rows[0];
            row.files = 4_064;
            row.metrics.physical_lines = Some(834_425);
            row.metrics.nonblank_lines = Some(633_499);
            row.metrics.blank_lines = Some(200_926);
            row.metrics.document_words = Some(4_390_955);
            row.generated_files = 2;
            row.vendored_files = 3;
            row.documentation_files = 4_063;
            row.code_coverage = None;
            row.words_coverage = None;
            row.lines_coverage = std::collections::BTreeMap::from([
                (CoverageReason::Analyzed, 3_997),
                (CoverageReason::Binary, 60),
                (CoverageReason::Unsupported, 7),
            ]);
            let plain = render(&metrics, Format::Text, false);
            let lines = plain.lines().collect::<Vec<_>>();
            // In columns, not bytes: an unmeasured share is the three-byte `—`.
            let at = lines[0].find("4,064 files").expect("the file count ends the first line");
            let column = display_width(&lines[0][..at]);
            assert!(lines[0].ends_with("4,064 files"), "{plain}");
            let indent = " ".repeat(column + 2);
            let stacked = [
                "834,425 lines (633,499 nonblank, 200,926 blank)",
                "4,390,955 words (17,563.8 pages)",
                "2 generated",
                "3 vendored",
                "60 binary",
                "7 unsupported",
            ]
            .map(|measure| format!("{indent}{measure}"));
            assert_eq!(lines[1..=stacked.len()], stacked, "{view:?}:\n{plain}");
            assert!(plain.lines().all(|line| !line.ends_with(' ')), "{plain:?}");
            // The documentation-file count is machine output only.
            assert!(!plain.contains("documentation"), "{view:?}:\n{plain}");

            let colored = render(&metrics, Format::Text, true);
            assert_eq!(strip_ansi(&colored), plain, "color must not move a continuation");
            for (measure, breakdown) in [
                ("834,425 lines", "(633,499 nonblank, 200,926 blank)"),
                ("4,390,955 words", "(17,563.8 pages)"),
            ] {
                let expected = format!("\n{indent}{measure} {}\n", detail(breakdown, true));
                assert!(colored.contains(&expected), "{view:?}: {colored:?}");
            }
        }

        // The non-language views share a label floor, so their continuations sit at one
        // column across sections; a row with nothing shown past its file count, its
        // documentation-file count included, stays one line.
        let mut types = fixture(&[ViewSpec::Types]);
        let Section::Metrics { summary, .. } = &mut types.sections[0] else { panic!("types") };
        assert_eq!(summary.rows.len(), 2, "a measured row and a bare one");
        summary.rows[0].files = 12;
        summary.rows[0].generated_files = 12;
        summary.rows[1].files = 1;
        summary.rows[1].documentation_files = 1;
        let plain = render(&types, Format::Text, false);
        let lines = plain.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 3, "{plain}");
        assert_eq!(lines[1], format!("{}12 generated", " ".repeat(TEXT_METRIC_INDENT)));
        assert!(lines[2].ends_with(" 1 file"), "{plain}");
        assert_eq!(lines[2].find("1 file"), lines[0].find("12 files"), "{plain}");
        let json = render(&types, Format::Json, false);
        assert!(json.contains("\"documentation\": 1}"), "machine output keeps it: {json}");
    }

    #[test]
    fn code_population_details_keep_known_and_unknown_contributions_visible() {
        use crate::query::{CodeLanguageRow, MetricShare};
        let tally = |lines| CodeTally {
            source_files: 1,
            analyzed_files: 1,
            metrics: crate::content::CodeMetrics { code_lines: lines, ..Default::default() },
            ..Default::default()
        };
        let mut overview = CodeOverview {
            population: IgnoredEntries::Include,
            selected: CodeTally { source_files: 3, analyzed_files: 3, ..tally(110) },
            non_ignored: Some(tally(80)),
            ignored: Some(tally(20)),
            unknown: tally(10),
            unclassified_files: 0,
            analyzed_languages: 1,
            total_languages: 1,
            share_omitted: 0,
            languages: vec![CodeLanguageRow {
                language: "rust".into(),
                selected: CodeTally { source_files: 3, analyzed_files: 3, ..tally(110) },
                non_ignored: Some(tally(80)),
                ignored: Some(tally(20)),
                unknown: tally(10),
                share: MetricShare { numerator: 110, denominator: 110 },
            }],
            share_metric: ShareMetric::CodeLines,
        };
        let mut report = fixture(&[ViewSpec::Summary]);
        report.sections = vec![Section::Code(Box::new(overview.clone()))];
        let colored = render(&report, Format::Text, true);
        assert!(colored.contains(&paint("TOTAL   ", AnsiStyle::new().bold(), true)), "{colored:?}");
        let colored_total = colored.lines().find(|line| line.contains("TOTAL")).expect("total row");
        assert_eq!(colored_total.matches("\x1b[1m").count(), 6, "all primary TOTAL cells are bold");
        assert!(colored.contains(&detail("(20 gitignored, 10 unknown)", true)), "{colored:?}");
        assert!(!colored.contains("non-gitignored"), "the complement is never repeated");
        let plain = strip_ansi(&colored);
        assert!(
            plain.lines().next().expect("header").contains("Analyzed files  Language"),
            "{plain}"
        );
        let total = plain.lines().find(|line| line.contains("TOTAL")).expect("total row");
        assert_eq!(
            total.split_whitespace().take(6).collect::<Vec<_>>(),
            ["110", "100.0%", "0", "0", "3/3", "TOTAL"]
        );
        assert_eq!(
            plain.matches("       110").count(),
            2,
            "row and TOTAL retain the same measured total"
        );
        assert_eq!(strip_ansi(&colored), render(&report, Format::Text, false));
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            assert!(!render(&report, format, true).contains('\u{1b}'));
        }
        overview.population = IgnoredEntries::Only;
        overview.non_ignored = None;
        overview.ignored = None;
        overview.unknown = CodeTally::default();
        overview.languages[0].non_ignored = None;
        overview.languages[0].ignored = None;
        overview.languages[0].unknown = CodeTally::default();
        report.sections = vec![Section::Code(Box::new(overview))];
        let text = render(&report, Format::Text, false);
        assert!(!text.contains("gitignored") && !text.contains(" unknown"), "{text}");
    }

    #[test]
    fn code_table_keeps_global_total_when_rows_are_bounded_and_marks_unmeasured_values() {
        use crate::query::{CodeLanguageRow, MetricShare};
        let mut overview = CodeOverview {
            population: IgnoredEntries::Include,
            selected: CodeTally {
                source_files: 2,
                analyzed_files: 2,
                metrics: crate::content::CodeMetrics { code_lines: 12, ..Default::default() },
                ..Default::default()
            },
            non_ignored: None,
            ignored: None,
            unknown: CodeTally::default(),
            unclassified_files: 0,
            analyzed_languages: 1,
            total_languages: 1,
            share_omitted: 0,
            languages: Vec::new(),
            share_metric: ShareMetric::CodeLines,
        };
        let mut report = fixture(&[ViewSpec::Summary]);
        report.sections = vec![Section::Code(Box::new(overview.clone()))];
        let bounded = render(&report, Format::Text, false);
        let total = bounded.lines().find(|line| line.contains("TOTAL")).expect("total row");
        assert_eq!(
            total.split_whitespace().take(6).collect::<Vec<_>>(),
            ["12", "100.0%", "0", "0", "2/2", "TOTAL"]
        );
        assert!(!bounded.contains("Rust"), "{bounded}");

        overview.selected.analyzed_files = 0;
        overview.selected.metrics = crate::content::CodeMetrics::default();
        overview.selected.coverage.insert(CoverageReason::Unsupported, 2);
        overview.analyzed_languages = 0;
        overview.non_ignored = Some(CodeTally::default());
        overview.ignored = Some(CodeTally::default());
        overview.languages = vec![CodeLanguageRow {
            language: "rust".into(),
            selected: CodeTally { source_files: 2, ..Default::default() },
            non_ignored: Some(CodeTally::default()),
            ignored: Some(CodeTally::default()),
            unknown: CodeTally::default(),
            share: MetricShare { numerator: 0, denominator: 0 },
        }];
        report.sections = vec![Section::Code(Box::new(overview))];
        let unmeasured = render(&report, Format::Text, false);
        let language = unmeasured.lines().find(|line| line.contains("Rust")).expect("language row");
        let total = unmeasured.lines().find(|line| line.contains("TOTAL")).expect("total row");
        assert_eq!(
            language.split_whitespace().take(6).collect::<Vec<_>>(),
            ["—", "—", "—", "—", "0/2", "Rust"]
        );
        assert_eq!(
            total.split_whitespace().take(6).collect::<Vec<_>>(),
            ["—", "—", "—", "—", "0/2", "TOTAL"]
        );
        assert!(!unmeasured.contains("gitignored"), "{unmeasured}");
        assert!(!unmeasured.contains("unsupported"), "coverage is a note, not a row: {unmeasured}");
        assert!(
            report_notes(&report).contains(&"note: not analyzed: 2 unsupported".to_owned()),
            "{:?}",
            report_notes(&report)
        );
        assert!(unmeasured.lines().all(|line| line.trim_end() == line), "{unmeasured:?}");
    }

    #[test]
    fn percentage_and_unicode_width_keep_small_values_visible() {
        assert_eq!(human_percentage(1, 10_000, 1), "<0.1%");
        assert_eq!(human_percentage(1, 10_000, 0), "<1%");
        assert_eq!(human_percentage(0, 10_000, 1), "0.0%");
        assert_eq!(human_percentage(0, 0, 1), "—");
        assert_eq!(label_cell("界", 4, STYLE_CATEGORY, false), "界  ");
        assert_eq!(label_cell("e\u{301}", 4, STYLE_CATEGORY, false), "e\u{301}   ");
    }

    #[test]
    fn no_machine_format_gains_a_text_header() {
        // Machine formats already name their view in a field. Text is a presentation
        // layer over the same report and must not leak into the versioned schemas.
        let views = [ViewSpec::Tree, ViewSpec::Types, ViewSpec::Files, ViewSpec::Summary];
        for format in [Format::Json, Format::Jsonl, Format::Yaml] {
            let rendered = render(&fixture(&views), format, false);
            for header in ["TREE", "TYPES", "FILES", "SUMMARY"] {
                assert!(!rendered.contains(header), "{format:?} leaked {header}:\n{rendered}");
            }
        }
    }

    #[test]
    fn every_view_has_a_header_that_matches_its_wire_label() {
        // The two spellings are written out separately so a schema change and a
        // presentation change stay independent; this is what keeps them from drifting
        // apart by accident while they are meant to agree.
        for view in [
            ViewSpec::Tree,
            ViewSpec::Extensions,
            ViewSpec::Types,
            ViewSpec::Families,
            ViewSpec::Languages,
            ViewSpec::Documents,
            ViewSpec::Files,
            ViewSpec::Summary,
        ] {
            let header = view_header(view);
            assert_eq!(header, view.label().to_uppercase(), "{view:?}");
            assert!(
                !header.is_empty() && header.chars().all(|c| c.is_ascii_uppercase()),
                "{view:?}"
            );
        }
    }

    #[test]
    fn yaml_quotes_only_what_would_be_ambiguous() {
        assert_eq!(yaml_scalar("cold_scan"), "cold_scan");
        assert_eq!(yaml_scalar("src/main.rs"), "src/main.rs");
        // Bare words YAML would read as another type have to be quoted.
        assert_eq!(yaml_scalar("true"), "\"true\"");
        assert_eq!(yaml_scalar("null"), "\"null\"");
        assert_eq!(yaml_scalar("12345"), "\"12345\"");
        assert_eq!(yaml_scalar(""), "\"\"");
        assert_eq!(yaml_scalar("has space"), "\"has space\"");
        assert_eq!(yaml_scalar("-leading-dash"), "\"-leading-dash\"");
    }

    #[test]
    fn json_strings_escape_control_characters_and_quotes() {
        assert_eq!(quote("a\"b"), "\"a\\\"b\"");
        assert_eq!(quote("a\\b"), "\"a\\\\b\"");
        assert_eq!(quote("a\nb"), "\"a\\nb\"");
        assert_eq!(quote("a\u{1}b"), "\"a\\u0001b\"");
    }

    #[test]
    fn format_values_parse_and_reject_by_name() {
        assert_eq!(Format::parse("json"), Some(Format::Json));
        assert_eq!(Format::parse("  YAML "), Some(Format::Yaml));
        assert_eq!(Format::parse("xml"), None);
        assert_eq!(Format::ALL.len(), 7);
    }

    #[test]
    fn human_bytes_reads_at_scale() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(999), "999 B");
        assert_eq!(human_bytes(1000), "1,000 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(1024 * 1024 * 20), "20 MiB");
    }

    #[test]
    fn human_counts_share_one_full_width_grouping_policy() {
        assert_eq!(human_count(999), "999");
        assert_eq!(human_count(1000), "1,000");
        assert_eq!(
            human_count_u128(u128::MAX),
            "340,282,366,920,938,463,463,374,607,431,768,211,455"
        );
        let year = i128::from(YEAR_SECONDS) * 1_000_000_000;
        assert_eq!(human_age(Some(1_000 * year)), "1,000y");
    }

    /// Every unit of the age ladder at both of its edges, the sign, and the floor: a unit
    /// holds until the next one's length, so no cell reads `0mo` or `0y`.
    #[test]
    fn human_ages_climb_one_ladder_and_switch_units_at_their_own_length() {
        const SECOND: i128 = 1_000_000_000;
        let month = i128::from(MONTH_SECONDS) * SECOND;
        let year = i128::from(YEAR_SECONDS) * SECOND;
        for (age, shown) in [
            (0, "0s"),
            (SECOND - 1, "0s"),
            (59 * SECOND, "59s"),
            (60 * SECOND - 1, "59s"),
            (60 * SECOND, "1m"),
            (3_600 * SECOND - 1, "59m"),
            (3_600 * SECOND, "1h"),
            (86_400 * SECOND - 1, "23h"),
            (86_400 * SECOND, "1d"),
            (30 * 86_400 * SECOND, "30d"),
            (month - 1, "30d"),
            (month, "1mo"),
            (2 * month, "2mo"),
            (year - 1, "11mo"),
            (year, "1y"),
            (26 * year + month, "26y"),
            (100 * year - 1, "99y"),
            (-1, "-0s"),
            (-SECOND, "-1s"),
            (-5 * 60 * SECOND - 1, "-5m"),
            (-2 * month, "-2mo"),
        ] {
            assert_eq!(human_age(Some(age)), shown, "{age} ns");
        }
        assert_eq!(human_age(None), "unknown");
        // Below a century, at most four characters and a sign.
        let widest =
            [month * 11, year * 99, 59 * SECOND, 3_599 * SECOND].map(|age| human_age(Some(-age)));
        assert!(widest.iter().all(|cell| cell.len() <= 5), "{widest:?}");
    }

    #[test]
    fn bars_are_fixed_at_ten_cells_and_saturate() {
        assert_eq!(usage_bar(0, 100, Some(0), false, 10), "░░░░░░░░░░");
        assert_eq!(usage_bar(50, 100, Some(0), false, 10), "█████░░░░░");
        assert_eq!(usage_bar(200, 100, Some(0), false, 10), "██████████");
        assert!((ratio(5, 0) - 0.0).abs() < f64::EPSILON);
        assert_eq!(bar_cells(u64::MAX / 2, u64::MAX, 1), 0);
        assert_eq!(bar_cells(u64::MAX / 2 + 1, u64::MAX, 1), 1);
    }

    #[test]
    fn human_styles_respect_exact_thresholds_and_directory_identity() {
        assert_eq!(styled_bytes(0, 0, true, false), "\x1b[90m0 B\x1b[0m");
        assert_eq!(styled_bytes(0, 0, false, false), "0 B");
        let gib = 1 << 30;
        assert_eq!(styled_bytes(gib - 1, 10, true, false), format!("{:>10}", human_bytes(gib - 1)));
        assert_eq!(
            styled_bytes(gib, 10, true, false),
            paint("   1.0 GiB", AnsiStyle::new().bold(), true)
        );
        assert_eq!(styled_bytes(gib, 0, true, true), paint("1.0 GiB", STYLE_DETAIL.bold(), true));
        assert_eq!(styled_bytes(gib, 0, false, true), "1.0 GiB");
        assert_eq!(percentage_cell(99, 10_000, 0, 5, true), detail("  <1%", true));
        assert_eq!(percentage_cell(100, 10_000, 0, 5, true), "   1%");
        assert_eq!(human_percentage(u64::MAX / 100, u64::MAX, 0), "<1%");
        assert_eq!(human_percentage(u64::MAX / 100 + 1, u64::MAX, 0), "1%");
        assert_eq!(human_percentage(u64::MAX / 1_000, u64::MAX, 1), "<0.1%");
        for name in [".", ".."] {
            assert_eq!(human_name(name, EntryKind::Dir, None, false), name);
        }
        assert_eq!(
            human_name("build", EntryKind::Dir, None, true),
            format!("{}{}", paint("build", STYLE_NAME, true), detail("/", true))
        );
        assert_eq!(human_name("build", EntryKind::File, None, false), "build");
        assert_eq!(human_name("build", EntryKind::Dir, None, false), "build/");
        // A root's label that ends in a separator gains no second one: `/`, and a drive
        // root, which ends in the platform's separator (`C:\` on Windows).
        assert_eq!(human_name("/", EntryKind::Dir, None, false), "/");
        let drive = format!("C:{}", std::path::MAIN_SEPARATOR);
        assert_eq!(human_name(&drive, EntryKind::Dir, None, false), drive);
    }

    #[test]
    fn only_own_ignored_directories_lose_bold_name_styling() {
        for ignored in [None, Some(false), Some(true)] {
            let style = if ignored == Some(true) { STYLE_IGNORED_NAME } else { STYLE_NAME };
            assert_eq!(
                human_name("node_modules", EntryKind::Dir, ignored, true),
                format!("{}{}", paint("node_modules", style, true), detail("/", true))
            );
            assert_eq!(
                human_name("file.rs", EntryKind::File, ignored, true),
                paint("file.rs", STYLE_NAME, true)
            );
            assert_eq!(human_name("node_modules", EntryKind::Dir, ignored, false), "node_modules/");
        }
    }

    #[test]
    fn colored_bars_partition_selected_usage_and_keep_ten_cells() {
        let split = usage_bar(60, 100, Some(20), true, 10);
        assert_eq!(
            split,
            format!(
                "{}{}{}",
                paint("████", STYLE_BAR, true),
                paint("▓▓", STYLE_BAR, true),
                paint("░░░░", STYLE_BAR.dimmed(), true)
            )
        );
        assert_eq!(strip_ansi(&split).chars().count(), 10);
        assert_eq!(usage_bar(60, 100, Some(20), false, 10), "██████░░░░");
        assert_eq!(strip_ansi(&usage_bar(60, 60, Some(60), true, 10)), "▓▓▓▓▓▓▓▓▓▓");
        assert_eq!(
            usage_bar(60, 60, Some(60), true, 10),
            format!(
                "{}{}{}",
                paint("", STYLE_BAR, true),
                paint("▓▓▓▓▓▓▓▓▓▓", STYLE_BAR, true),
                paint("", STYLE_BAR.dimmed(), true)
            )
        );
        assert_eq!(strip_ansi(&usage_bar(0, 0, None, true, 10)), "░░░░░░░░░░");
        let mostly_ignored = usage_bar(130, 2_120, Some(96), true, 10);
        assert_eq!(strip_ansi(&mostly_ignored), "▓░░░░░░░░░");
        assert!(mostly_ignored.contains("\x1b[32m▓\x1b[0m"));
        assert_eq!(strip_ansi(&usage_bar(130, 2_120, Some(96), true, 20)), "▓░░░░░░░░░░░░░░░░░░░");
        assert_eq!(strip_ansi(&usage_bar(130, 2_120, Some(96), true, 100)).matches('▓').count(), 4);
    }

    #[test]
    fn render_options_resize_or_remove_tree_bars_without_changing_data() {
        let report = fixture_for(&Query {
            views: vec![ViewSpec::Tree],
            selection: Selection {
                size: SizeMetric::Apparent,
                depth: Some(Bound::Limit(0)),
                ..Selection::default()
            },
            ..Query::default()
        });
        for width in [0, 10, 20] {
            let options = RenderOptions { color: false, bar_size: width };
            let text = render_with_options(&report, Format::Text, options).expect("tree");
            let lines = text.lines().collect::<Vec<_>>();
            assert_eq!(lines.len(), 2, "{width}: {text:?}");
            for line in &lines {
                if width == 0 {
                    assert!(line.starts_with(" 100%"), "{line:?}");
                } else {
                    assert!(line.starts_with(&"█".repeat(width)), "{line:?}");
                    assert!(line["█".repeat(width).len()..].starts_with("   100%"));
                }
            }
            assert!(lines[0].ends_with("120 B  -0s  . 2 files"), "{text:?}");
            assert!(lines[1].ends_with("120 B         … and 2 more files"), "{text:?}");
            let mut streamed = Vec::new();
            write_with_options(&report, Format::Text, options, &mut streamed).expect("stream");
            assert_eq!(streamed, text.as_bytes());
        }
        assert_eq!(
            render_with_options(&report, Format::Json, RenderOptions { color: true, bar_size: 20 })
                .expect("machine"),
            render(&report, Format::Json, false)
        );
        let excessive = RenderOptions { color: false, bar_size: MAX_BAR_SIZE + 1 };
        assert!(
            render_with_options(&report, Format::Text, excessive)
                .expect_err("tree width must be bounded")
                .to_string()
                .contains("bar_size")
        );
        let mut output = Vec::new();
        assert_eq!(
            write_with_options(&report, Format::Text, excessive, &mut output)
                .expect_err("streaming tree width must be bounded")
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(output.is_empty());
        let summary = fixture(&[ViewSpec::Summary]);
        assert_eq!(
            render_with_options(&summary, Format::Text, excessive).expect("no tree bar"),
            render(&summary, Format::Text, false)
        );
    }

    const DEEP_RENDER_CHILD_ENV: &str = "FDU_DEEP_RENDER_CHILD";
    const DEEP_RENDER_DEPTH: usize = 1_024;
    const DEEP_REPORT_STACK_BYTES: usize = 128 * 1_024;
    const DEEP_RENDER_STACK_BYTES: usize = 64 * 1_024;

    // ---- renderer tests that lived in the command line -------------------------------
    //
    // They test expansion and the three renderers, not argument handling, and they build
    // an index by hand -- which is why moving the CLI into its own crate surfaced them:
    // the fixture helpers they need are `pub(crate)` here and unreachable from there.

    #[test]
    fn deep_rendering_is_stack_safe() {
        if std::env::var_os(DEEP_RENDER_CHILD_ENV).is_some() {
            run_deep_render_child();
            return;
        }

        let started = Instant::now();
        let mut child = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", DEEP_RENDER_TEST_PATH, "--nocapture"])
            .env(DEEP_RENDER_CHILD_ENV, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn deep-render child");
        let stdout = drain_pipe(child.stdout.take().expect("piped stdout"));
        let stderr = drain_pipe(child.stderr.take().expect("piped stderr"));
        let status = loop {
            if let Some(status) = child.try_wait().expect("poll deep-render child") {
                break Some(status);
            }
            if started.elapsed() >= DEEP_RENDER_CHILD_TIMEOUT {
                break None;
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        let Some(status) = status else {
            // `kill` is SIGKILL on Unix, which the child can neither catch nor ignore: the
            // hang this bounds also ignored SIGTERM. Its result can be ignored because the
            // child is not yet reaped, so even one that exited a moment ago is still there
            // to signal, and `wait` returns once it is gone.
            let _ = child.kill();
            let reaped = child.wait().expect("reap the killed deep-render child");
            panic!(
                "deep-render child still running after {DEEP_RENDER_CHILD_TIMEOUT:?}; killed at \
                 {:?} ({reaped}). The last `deep-render phase:` line is where it stalled.\n\
                 stdout:\n{}\nstderr:\n{}",
                started.elapsed(),
                stdout.join().expect("stdout reader"),
                stderr.join().expect("stderr reader")
            );
        };
        let stdout = stdout.join().expect("stdout reader");
        let stderr = stderr.join().expect("stderr reader");
        assert!(
            status.success(),
            "deep renderer failed in child process\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );

        // The exit code alone cannot tell "the deep render survived" from "the filter
        // matched nothing": libtest runs zero tests and exits 0 for a name that does not
        // exist, so a moved test would keep reporting success having stopped running --
        // which is what happened when this test moved out of `cli::tests` (fdu-rdom).
        assert!(
            stdout.contains("1 passed"),
            "the child must actually run the deep render, not filter it away\nstdout:\n{stdout}"
        );
    }

    /// The child re-invocation filters on this, so it has to track the module the test
    /// lives in. Named once, beside the test, rather than spelled in the argument list
    /// where a move leaves it silently stale.
    const DEEP_RENDER_TEST_PATH: &str = "report_format::tests::deep_rendering_is_stack_safe";

    /// How long the parent lets the deep-render child run before killing it.
    ///
    /// Without a bound a stuck child blocks the suite indefinitely: on 2026-09-28 one spun
    /// for more than 85 minutes (fdu-xsg1). Measured on Linux x86-64 debug with four
    /// cores, the child takes about 21 s alone and 28 s under the full parallel suite,
    /// nearly all of it building the 1,024-level fixture one batch at a time. Five minutes
    /// is ten times the loaded figure, room for a runner several times slower, and still
    /// turns a hang into a failure well before anyone would notice a stalled gate.
    const DEEP_RENDER_CHILD_TIMEOUT: Duration = Duration::from_secs(300);

    /// Read a child's pipe to its end on a helper thread.
    ///
    /// One reader per pipe means a child that fills one pipe's buffer never blocks while
    /// the parent waits on the other pipe or on its exit, and what the child wrote before
    /// a hang is still there to report once it is killed.
    fn drain_pipe(mut pipe: impl io::Read + Send + 'static) -> std::thread::JoinHandle<String> {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            // A read error ends the capture early; what arrived before it is still reported.
            let _ = pipe.read_to_end(&mut bytes);
            String::from_utf8_lossy(&bytes).into_owned()
        })
    }

    fn run_deep_render_child() {
        // A deep tree must build and render without depth-recursive stack growth.
        // Windows reserves 20 KiB of a spawned thread's stack for overflow handling;
        // a 64 KiB reservation leaves too little dependable room for report setup in
        // debug builds. Keep construction bounded at 128 KiB, then test rendering and
        // release separately on the original 64 KiB stack.
        eprintln!("deep-render phase: fixture");
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

        let report = std::thread::Builder::new()
            .name("deep-report".to_string())
            .stack_size(DEEP_REPORT_STACK_BYTES)
            .spawn(move || {
                let query = Query {
                    selection: Selection {
                        depth: Some(Bound::All),
                        breadth: Some(Bound::All),
                        limit: Some(Bound::All),
                        min_share: Some(ShareThreshold::parse("0%").expect("zero share is valid")),
                        ..Selection::default()
                    },
                    views: vec![ViewSpec::Tree],
                    ..Query::default()
                };
                let provenance = Provenance {
                    scan_started_at: None,
                    generated_at: SystemTime::UNIX_EPOCH,
                    source: ReportSource::ColdScan,
                    complete: true,
                    errors: Vec::new(),
                };
                eprintln!("deep-render phase: request");
                let request = crate::test_support::read_of(&index, query);
                eprintln!("deep-render phase: report");
                let report = report(&index, &request, &provenance).expect("report");
                eprintln!("deep-render phase: verify tree");
                let Section::Tree { root: Some(root), omissions, .. } = &report.sections[0] else {
                    panic!("expected a tree section with a root")
                };
                assert!(omissions.is_empty(), "the root must not be omitted");
                let mut nodes = 0;
                let mut pending = vec![root.as_ref()];
                while let Some(node) = pending.pop() {
                    nodes += 1;
                    assert!(node.omissions.is_empty(), "no branch may be omitted: {:?}", node.path);
                    pending.extend(node.children.iter());
                }
                assert_eq!(nodes, DEEP_RENDER_DEPTH + 1, "the test must reach every directory");
                report
            })
            .expect("spawn deep-report thread")
            .join()
            .expect("deep-report thread");

        std::thread::Builder::new()
            .name("deep-render".to_string())
            .stack_size(DEEP_RENDER_STACK_BYTES)
            .spawn(move || {
                for format in [Format::Text, Format::Json, Format::Jsonl, Format::Yaml] {
                    eprintln!("deep-render phase: render {format:?}");
                    let rendered = render(&report, format, false);
                    assert!(!rendered.is_empty(), "{format:?} rendered nothing for a deep tree");
                    if format != Format::Text {
                        eprintln!("deep-render phase: stream {format:?}");
                        let mut streamed = Vec::new();
                        write(&report, format, false, &mut streamed).expect("stream deep report");
                        assert_eq!(streamed, rendered.as_bytes(), "{format:?} bytes differ");
                    }
                }
                eprintln!("deep-render phase: drop");
                drop(report);
                eprintln!("deep-render phase: complete");
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

        // Built directly rather than through the command line's argument struct: what is
        // under test is that the renderer preserves a non-UTF-8 name's raw identity, and
        // routing that through argument parsing tied a renderer test to a front end.
        let query = crate::query::Query {
            views: vec![ViewSpec::Files],
            selection: Selection { limit: Some(crate::query::Bound::All), ..Selection::default() },
            ..crate::query::Query::default()
        };
        let provenance = Provenance {
            scan_started_at: None,
            generated_at: std::time::UNIX_EPOCH,
            source: ReportSource::ColdScan,
            complete: true,
            errors: Vec::new(),
        };
        let files_report =
            report(&index, &crate::test_support::read_of(&index, query.clone()), &provenance)
                .expect("report");
        let rendered = render(&files_report, Format::Json, false);
        let mut checked = SchemaCheck::report(JsonSink::pretty(), true, true);
        emit_report(&mut checked, &files_report, true);
        assert_eq!(checked.finish(), rendered);

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
                 \"kind\": \"file\", \"bytes\": 1, \"allocated\": 1, \"mtime_ns\": 0, \
                 \"files\": null, \"dirs\": null, \"complete\": null, \"age_ns\": 0, \
                 \"modified_at\": \"1970-01-01T00:00:00.000000000Z\", \"ignored\": false, \"sort_value\": null, \
                 \"classification\": {{\"file_type\": \"unknown\", \"family\": \"unknown\", \"source\": \"unknown\", \"confidence\": \"heuristic\", \
                 \"flags\": {{\"generated\": false, \"vendored\": false, \"documentation\": false}}}}}}"
            );
            assert!(
                compact_json(&rendered).contains(&compact_json(&row)),
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
        let tree_query = crate::query::Query {
            views: vec![ViewSpec::Tree],
            selection: Selection {
                depth: Some(crate::query::Bound::All),
                limit: Some(crate::query::Bound::All),
                ..Selection::default()
            },
            ..crate::query::Query::default()
        };
        let tree =
            report(&dirs, &crate::test_support::read_of(&dirs, tree_query.clone()), &provenance)
                .expect("report");
        let tree_rendered = render(&tree, Format::Json, false);
        assert!(
            compact_json(&tree_rendered).contains(&compact_json(&format!(
                ", \"path_raw\": {{\"encoding\": \"{encoding}\", \"hex\": \"{first_hex}\"}}, \"kind\":"
            ))),
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

    /// A report over two roots, `docs` and `src`, read two minutes after the epoch.
    fn several_roots_fixture(query: &Query) -> Report {
        let index = |root: &str, files: &[(&str, u64, i64)]| {
            let mut index = Index::new_with_scope(root, ScanScope::default());
            let mut ops = vec![Op::Upsert {
                path: PathBuf::from("sub"),
                kind: EntryKind::Dir,
                attrs: Attrs { mtime_ns: 1_000_000_000, ..Attrs::default() },
            }];
            ops.extend(files.iter().map(|(path, size, seconds)| Op::Upsert {
                path: PathBuf::from(path),
                kind: EntryKind::File,
                attrs: attrs(*size, seconds * 1_000_000_000),
            }));
            index.apply(&Observation::new(ops)).expect("apply");
            index
        };
        let docs = index("/abs/docs", &[("guide.md", 3_000, 30), ("sub/a.md", 1_000, 60)]);
        let src = index("/abs/src", &[("main.rs", 5_000, 90), ("sub/lib.rs", 1, 100)]);
        let roots = crate::query::Roots::named(vec![
            crate::query::NamedRoot { label: "docs".into(), path: "/abs/docs".into() },
            crate::query::NamedRoot { label: "src".into(), path: "/abs/src".into() },
        ]);
        let mut request = crate::test_support::read_of(&docs, query.clone());
        request.now = UNIX_EPOCH + Duration::from_secs(120);
        crate::query::report_roots(&[&docs, &src], &roots, &request, UNIX_EPOCH)
            .expect("several roots")
    }

    /// The tree over several roots: a total row, then each root's tree one level in under
    /// its label, its own remainder below it, every share of the total, and one age column
    /// wide enough for every row.
    #[test]
    fn a_tree_over_several_roots_renders_a_total_row_and_labelled_roots() {
        let query = Query {
            views: vec![ViewSpec::Tree],
            selection: Selection {
                size: SizeMetric::Apparent,
                min_share: Some(ShareThreshold::parse("10%").expect("share")),
                ..Selection::default()
            },
            ..Query::default()
        };
        let report = several_roots_fixture(&query);
        let text =
            render_with_options(&report, Format::Text, RenderOptions { color: false, bar_size: 0 })
                .expect("text");
        let expected = [
            " 100%     8.7 KiB  20s  (total) 4 files",
            "  56%     4.8 KiB  20s    src/ 2 files",
            "  56%     4.8 KiB  30s      main.rs",
            "  <1%         1 B           … and 1 more file",
            "  44%     3.9 KiB   1m    docs/ 2 files",
            "  33%     2.9 KiB   1m      guide.md",
            "  11%     1,000 B   1m      sub/ 1 file",
            "  11%     1,000 B   1m        a.md",
        ];
        assert_eq!(text.lines().collect::<Vec<_>>(), expected, "{text}");
    }

    /// Machine output over several roots: `root` is null and `roots` names each, the tree
    /// is a total and one tree per root, and every row, error, and refusal carries its root.
    #[test]
    fn machine_output_over_several_roots_names_each_root() {
        let tree =
            several_roots_fixture(&Query { views: vec![ViewSpec::Tree], ..Query::default() });
        for sections in [false, true] {
            let mut checked = SchemaCheck::report_over(JsonSink::pretty(), false, sections, true);
            emit_report(&mut checked, &tree, sections);
            checked.finish();
        }
        let json = super::render(&tree, Format::Json, false).expect("json");
        let compact: String = json.split_whitespace().collect();
        for expected in [
            r#""root":null,"roots":[{"label":"docs","path":"/abs/docs"},{"label":"src","path":"/abs/src"}]"#,
            r#""tree":null,"total":{"bytes":9001,"allocated":9728,"files":4,"dirs":2,"#,
            r#""mtime_ns":100000000000,"complete":true,"age_ns":20000000000,"#,
            r#""modified_at":"1970-01-01T00:01:40"#,
            r#""trees":[{"root":1,"tree":{"name":"src","path":"","kind":"dir","#,
        ] {
            assert!(compact.contains(expected), "{expected} in {json}");
        }

        let files = several_roots_fixture(&Query {
            views: vec![ViewSpec::Files],
            selection: Selection {
                sort: Some(crate::query::SortKey::Name),
                ..Selection::default()
            },
            ..Query::default()
        });
        let json = super::render(&files, Format::Jsonl, false).expect("json lines");
        let compact: String = json.split_whitespace().collect();
        assert!(compact.contains(r#""files":[{"root":0,"path":"guide.md","#), "{json}");
        assert!(compact.contains(r#"{"root":1,"path":"main.rs","#), "{json}");
        let paths = super::render(&files, Format::Paths, false).expect("paths");
        // Joined by component, as every text path is, so with `\` on Windows.
        let expected: String = [
            &["docs", "guide.md"][..],
            &["docs", "sub"],
            &["docs", "sub", "a.md"],
            &["src", "main.rs"],
            &["src", "sub"],
            &["src", "sub", "lib.rs"],
        ]
        .iter()
        .map(|parts| format!("{}\n", parts.iter().collect::<PathBuf>().display()))
        .collect();
        assert_eq!(paths, expected);
        let one = fixture(&[ViewSpec::Files]);
        let json = super::render(&one, Format::Json, false).expect("json");
        assert!(!json.contains("\"roots\"") && !json.contains("\"root\": 0"), "{json}");
    }

    /// The display-limit note says what a share is of: the total, over several roots.
    #[test]
    fn display_limits_over_several_roots_are_of_the_total() {
        let report = several_roots_fixture(&Query {
            views: vec![ViewSpec::Tree],
            selection: Selection {
                min_share: Some(ShareThreshold::parse("50%").expect("share")),
                ..Selection::default()
            },
            ..Query::default()
        });
        let notes = report_notes(&report);
        assert!(notes.iter().any(|note| note.contains("below 50% of total")), "{notes:?}");
        assert!(!notes.iter().any(|note| note.contains("of root")), "{notes:?}");
    }
}
