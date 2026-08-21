//! Serializing a [`Report`] to text, JSON, JSONL, and YAML.
//!
//! Formats are serializations, not features: every view renders in every format, so a
//! caller picks the shape of the answer and the shape of the bytes independently.
//!
//! # Why these are hand-written
//!
//! `serde` plus a JSON crate plus a YAML crate would be three dependency additions —
//! and the maintained-YAML question is genuinely unsettled, since `serde_yaml` is
//! unmaintained. The schema here is small, closed, and fully known at compile time, the
//! crate already hand-writes its JSON, and hand-writing keeps the machine formats
//! provably free of a serializer's own opinions about key order and number formatting.
//! Key order is fixed by the code, which is what makes the goldens byte-stable.

use std::fmt::Write as _;
use std::path::Path;

use clap::builder::styling::{AnsiColor, Style as AnsiStyle};

use crate::classify::{DetectionConfidence, DetectionSource, human_language_name};
use crate::content::{CoverageReason, MetricValues};
use crate::engine_contract::{EntryKind, Freshness};
use crate::query::{
    FileRow, MetricGroup, MetricRow, MetricSummary, Report, ReportSource, Section, SizeMetric,
    SummaryRow, TreeNode, TypeRow, ViewSpec, document_words, format_rfc3339,
};

/// The all-caps label naming which view a block of text output belongs to.
///
/// Bold cyan is what `cli.rs` already gives a section heading in `--help`, so a report
/// and the help that describes it use one visual language for the same idea.
/// View headers share the CLI's one header style; see `cli::STYLE_HEADING`.
const STYLE_VIEW_HEADER: AnsiStyle = AnsiColor::Cyan.on_default().bold();

/// Directory names in a tree, so structure reads at a glance.
const STYLE_DIRECTORY: AnsiStyle = AnsiColor::Cyan.on_default();

/// Relative-size bars in a tree.
const STYLE_BAR: AnsiStyle = AnsiColor::Green.on_default();

/// Extensions in a type breakdown.
const STYLE_TYPE: AnsiStyle = AnsiColor::Green.on_default();

/// Established label width for non-language metric summaries.
const TEXT_METRIC_LABEL_WIDTH: usize = 18;
/// Floor for the extensions view's label column.
const TEXT_TYPE_LABEL_WIDTH: usize = 12;

/// Machine-output schema identity.
///
/// Any change to a field's name, type, or meaning bumps this, and a golden test fails if
/// the schema moves without it — the versioning is the promise, not the intention.
pub const REPORT_SCHEMA: &str = "fdu.report/1";
/// Machine schema used when a generic metric-summary section is present.
pub const CONTENT_REPORT_SCHEMA: &str = "fdu.report/3";

/// How a report is serialized.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Format {
    /// Human-readable text.
    #[default]
    Text,
    /// One JSON document.
    Json,
    /// One JSON document per line, one line per section.
    Jsonl,
    /// YAML.
    Yaml,
}

impl Format {
    /// Parse a `--format` value.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            "jsonl" => Some(Self::Jsonl),
            "yaml" => Some(Self::Yaml),
            _ => None,
        }
    }

    /// Every accepted spelling, for help text and error messages.
    pub const ALL: &'static [&'static str] = &["text", "json", "jsonl", "yaml"];
}

/// Render a report in the requested format.
///
/// `color` applies to the text form only: machine output is never colourized, because a
/// consumer parsing JSON should never have to strip escape sequences first.
pub fn render(report: &Report, format: Format, color: bool) -> String {
    match format {
        Format::Text => render_text(report, color),
        Format::Json => render_json(report),
        Format::Jsonl => render_jsonl(report),
        Format::Yaml => render_yaml(report),
    }
}

/// Wrap text in a style when colour is on.
fn paint(text: &str, style: AnsiStyle, color: bool) -> String {
    if color { format!("{style}{text}{style:#}") } else { text.to_string() }
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
/// A single-view report is left bare, which is what keeps `fdu --view files` a listing
/// of paths and nothing else — the property behind `fdu --view files | xargs` — and
/// keeps the default one-view report exactly as it was. One block needs no label to be
/// unambiguous, so the header appears precisely when it disambiguates something.
fn render_text(report: &Report, color: bool) -> String {
    let mut out = String::new();
    let headed = report.sections.len() > 1;
    for (index, section) in report.sections.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        if headed {
            let _ =
                writeln!(out, "{}", paint(view_header(section.view()), STYLE_VIEW_HEADER, color));
        }
        match section {
            Section::Tree(root) => render_text_tree(&mut out, root, report.size, color),
            Section::Extensions(rows) => render_text_types(&mut out, rows, report.size, color),
            Section::Metrics { view, summary } => {
                render_text_metrics(&mut out, *view, summary, report.size, color);
            }
            // `files` stays one path per line: it is the enumeration, and a bare list is
            // what pipes into xargs. The bounded presets are summaries, and a summary
            // that ranks by something must show that something — "the twenty largest"
            // with no sizes does not answer the question it is named for, and leaves the
            // ranking unverifiable.
            Section::Files { view, rows } => match view {
                ViewSpec::Largest => render_text_ranked_files(&mut out, rows, report.size, |row| {
                    human_bytes(pick(report.size, row.bytes, row.allocated))
                }),
                ViewSpec::Recent => render_text_ranked_files(&mut out, rows, report.size, |row| {
                    format_rfc3339(system_time_from_nanos(row.mtime_ns))
                }),
                _ => {
                    for row in rows {
                        let _ = writeln!(out, "{}", row.path.display());
                    }
                }
            },
            Section::Summary(row) => render_text_summary(&mut out, row, report.size),
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
//   detail  free-form, after a single space
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

/// A styled label padded to `width`, measured on the visible text.
fn label_cell(label: &str, width: usize, style: AnsiStyle, color: bool) -> String {
    let padding = " ".repeat(width.saturating_sub(label.chars().count()));
    format!("{}{padding}", paint(label, style, color))
}

/// The widest visible label in a set of rows, floored at `minimum`.
fn label_width<'a>(labels: impl Iterator<Item = &'a str>, minimum: usize) -> usize {
    minimum.max(labels.map(|label| label.chars().count()).max().unwrap_or_default())
}

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
    for row in &summary.rows {
        let selected = pick(size, row.bytes, row.allocated);
        let percentage = if row.share.denominator == 0 {
            "—".to_string()
        } else {
            format!("{:.1}%", ratio(row.share.numerator, row.share.denominator) * 100.0)
        };
        let mut suffix = format!("{} {}", row.files, plural(row.files, "file", "files"));
        if row.metrics.physical_lines > 0 {
            if row.metrics.code_lines > 0 || row.metrics.comment_lines > 0 {
                let _ = write!(
                    suffix,
                    ", {} lines ({} code, {} comment, {} blank)",
                    row.metrics.physical_lines,
                    row.metrics.code_lines,
                    row.metrics.comment_lines,
                    row.metrics.code_blank_lines
                );
            } else {
                let _ = write!(
                    suffix,
                    ", {} lines ({} nonblank, {} blank)",
                    row.metrics.physical_lines, row.metrics.nonblank_lines, row.metrics.blank_lines
                );
            }
        }
        let words = document_words(row);
        if words > 0 {
            let page_tenths = words.saturating_mul(10) / summary.words_per_page;
            let _ = write!(
                suffix,
                ", {} words ({}.{:01} pages)",
                words,
                page_tenths / 10,
                page_tenths % 10
            );
        }
        if row.generated_files > 0 {
            let _ = write!(suffix, ", {} generated", row.generated_files);
        }
        if row.vendored_files > 0 {
            let _ = write!(suffix, ", {} vendored", row.vendored_files);
        }
        if row.documentation_files > 0 {
            let _ = write!(suffix, ", {} documentation", row.documentation_files);
        }
        for (reason, count) in &row.coverage {
            if *reason != CoverageReason::Analyzed {
                let _ = write!(suffix, ", {count} {}", human_coverage_label(*reason));
            }
        }
        let _ = writeln!(
            out,
            "{:>TEXT_SIZE_WIDTH$}  {:>TEXT_SHARE_WIDTH$}  {} {suffix}",
            human_bytes(selected),
            percentage,
            label_cell(human_metric_label(view, &row.id), width, STYLE_TYPE, color),
        );
    }
}

fn human_metric_label(view: ViewSpec, id: &str) -> &str {
    if view == ViewSpec::Languages { human_language_name(id) } else { id }
}

fn human_coverage_label(reason: CoverageReason) -> &'static str {
    match reason {
        CoverageReason::Analyzed => "analyzed",
        CoverageReason::Binary => "binary",
        CoverageReason::InvalidUtf8 => "invalid UTF-8",
        CoverageReason::Unsupported => "unsupported",
        CoverageReason::IoError => "I/O error",
        CoverageReason::ChangedDuringRead => "changed during read",
    }
}

/// Render a tree section with fixed size, bar, and percentage columns.
///
/// Iterative for the same reason the expansion is: a deep tree must render, not panic.
fn render_text_tree(out: &mut String, root: &TreeNode, size: SizeMetric, color: bool) {
    enum Row<'a> {
        Node(&'a TreeNode, usize),
        Truncation(usize),
    }

    let grand = pick(size, root.bytes, root.allocated);
    // Children are pushed in reverse so they pop back in their sorted order. A
    // truncation row is pushed first so it appears after the retained children.
    let mut stack = vec![Row::Node(root, 0)];
    while let Some(row) = stack.pop() {
        match row {
            Row::Node(node, depth) => {
                let bytes = pick(size, node.bytes, node.allocated);
                let share = ratio(bytes, grand);
                let indent = "  ".repeat(depth);
                let _ = writeln!(
                    out,
                    "{:>10}  {}  {:>4.0}%  {indent}{} ({} {})",
                    human_bytes(bytes),
                    bar(share, color),
                    share * 100.0,
                    paint(&node.name, STYLE_DIRECTORY, color),
                    node.files,
                    plural(node.files, "file", "files"),
                );
                // Reaching the requested depth is visible from the outline itself and
                // marking every boundary directory overwhelms a real tree with dots.
                // Retained children plus truncation means the sibling list hit its
                // limit; that omission gets one marker after the rows that were kept.
                if node.truncated && !node.children.is_empty() {
                    stack.push(Row::Truncation(depth + 1));
                }
                for child in node.children.iter().rev() {
                    stack.push(Row::Node(child, depth + 1));
                }
            }
            Row::Truncation(depth) => {
                let indent = "  ".repeat(depth);
                let _ = writeln!(out, "{:>10}  {:10}  {:>5}  {indent}…", "", "", "");
            }
        }
    }
}

/// Render a types section as aligned rows.
fn render_text_types(out: &mut String, rows: &[TypeRow], size: SizeMetric, color: bool) {
    let width = label_width(rows.iter().map(|row| row.extension.as_str()), TEXT_TYPE_LABEL_WIDTH);
    for row in rows {
        let _ = writeln!(
            out,
            "{:>TEXT_SIZE_WIDTH$}  {} {} {}",
            human_bytes(pick(size, row.bytes, row.allocated)),
            label_cell(&row.extension, width, STYLE_TYPE, color),
            row.files,
            plural(row.files, "file", "files"),
        );
    }
}

/// Render a summary section as one line.
/// A nanosecond stamp as a `SystemTime`, saturating rather than panicking.
///
/// Timestamps before the epoch are legal on disk — an archive can restore one — and a
/// listing must render them rather than abort, so the conversion is total.
fn system_time_from_nanos(nanos: i64) -> std::time::SystemTime {
    const NANOS_PER_SEC: i64 = 1_000_000_000;
    let (secs, subsec) = (nanos.div_euclid(NANOS_PER_SEC), nanos.rem_euclid(NANOS_PER_SEC));
    let duration =
        std::time::Duration::new(secs.unsigned_abs(), u32::try_from(subsec).unwrap_or(0));
    if secs < 0 {
        std::time::UNIX_EPOCH.checked_sub(duration).unwrap_or(std::time::UNIX_EPOCH)
    } else {
        std::time::UNIX_EPOCH.checked_add(duration).unwrap_or(std::time::UNIX_EPOCH)
    }
}

/// A bounded flat listing, showing the measure it was ranked by.
///
/// The measure comes first at a fixed width so the paths line up under it, matching the
/// grouped views' size-then-label shape rather than inventing a third layout.
fn render_text_ranked_files(
    out: &mut String,
    rows: &[FileRow],
    _size: SizeMetric,
    measure: impl Fn(&FileRow) -> String,
) {
    let width = rows.iter().map(|row| measure(row).chars().count()).max().unwrap_or_default();
    for row in rows {
        let _ = writeln!(out, "{:>width$}  {}", measure(row), row.path.display());
    }
}

fn render_text_summary(out: &mut String, row: &SummaryRow, size: SizeMetric) {
    let _ = writeln!(
        out,
        "{:>10}  {} {}, {} {}",
        human_bytes(pick(size, row.bytes, row.allocated)),
        row.files,
        plural(row.files, "file", "files"),
        row.dirs,
        plural(row.dirs, "directory", "directories"),
    );
}

// ---- json ----

/// Render one JSON document carrying every section.
fn render_json(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    write_envelope_json(&mut out, report);
    out.push_str(",\n  \"reports\": [\n");
    for (index, section) in report.sections.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        let body = section_json(section, 4);
        out.push_str(&indent(&body, 4));
    }
    out.push_str("\n  ]\n}\n");
    out
}

/// Render one JSON document per line: the envelope, then one line per section.
///
/// The streaming shape, so a consumer can process sections as they arrive rather than
/// buffering a whole document.
fn render_jsonl(report: &Report) -> String {
    let mut out = String::new();
    let mut envelope = String::new();
    envelope.push('{');
    let mut fields = String::new();
    write_envelope_json(&mut fields, report);
    envelope.push_str(&collapse(&fields));
    envelope.push_str("}\n");
    out.push_str(&envelope);

    for section in &report.sections {
        out.push_str(&collapse(&section_json(section, 0)));
        out.push('\n');
    }
    out
}

/// The provenance fields every machine format carries.
fn write_envelope_json(out: &mut String, report: &Report) {
    let _ = write!(out, "  \"schema\": {}", quote(report_schema(report)));
    let _ = write!(out, ",\n  \"generator\": {}", quote(&generator()));
    let _ = write!(out, ",\n  \"root\": {}", quote(&report.root.to_string_lossy()));
    if let Some(raw) = raw_identity_object(&report.root) {
        let _ = write!(out, ",\n  \"root_raw\": {raw}");
    }
    let _ = write!(
        out,
        ",\n  \"scan_started_at\": {}",
        report.scan_started_at.map_or_else(|| "null".to_string(), |at| quote(&format_rfc3339(at)))
    );
    let _ = write!(out, ",\n  \"generated_at\": {}", quote(&format_rfc3339(report.generated_at)));
    let _ = write!(out, ",\n  \"source\": {}", quote(source_label(report.source)));
    let _ = write!(out, ",\n  \"freshness\": {}", quote(freshness_label(report.freshness)));
    let _ = write!(out, ",\n  \"complete\": {}", report.complete);
    let _ = write!(out, ",\n  \"errors\": [");
    for (index, error) in report.errors.iter().enumerate() {
        let _ = write!(out, "{}\n    {}", if index > 0 { "," } else { "" }, quote(error));
    }
    out.push_str(if report.errors.is_empty() { "]" } else { "\n  ]" });
    if report_schema(report) == CONTENT_REPORT_SCHEMA {
        let _ = write!(out, ",\n  \"analysis\": {}", analysis_json(report.analysis.as_ref()));
    }
}

fn analysis_json(analysis: Option<&crate::query::ContentReportMetadata>) -> String {
    let Some(analysis) = analysis else { return "null".to_string() };
    let mut analyzers = String::from("[");
    for (index, (id, version)) in analysis.provenance.analyzers.iter().enumerate() {
        let _ = write!(
            analyzers,
            "{}{{\"id\": {}, \"version\": {}}}",
            if index > 0 { ", " } else { "" },
            quote(id.0),
            version.0
        );
    }
    analyzers.push(']');
    let mut requested = String::from("[");
    for (index, label) in analysis_set_labels(analysis.profile).into_iter().enumerate() {
        let _ = write!(requested, "{}{}", if index > 0 { ", " } else { "" }, quote(label));
    }
    requested.push(']');
    format!(
        "{{\"analyze\": {requested}, \"type_rules_fingerprint\": {}, \"options_fingerprint\": {}, \"analyzers\": {analyzers}}}",
        analysis.provenance.type_rules_fingerprint, analysis.provenance.options_fingerprint.0,
    )
}

/// One section as a JSON object.
fn section_json(section: &Section, _indent: usize) -> String {
    let mut out = String::new();
    let _ = write!(out, "{{\n  \"view\": {},\n  ", quote(view_label(section.view())));
    match section {
        Section::Tree(root) => {
            let _ = write!(out, "\"tree\": {}", indent(&tree_json(root), 2).trim_start());
        }
        Section::Extensions(rows) => {
            let _ = write!(out, "\"extensions\": [");
            for (index, row) in rows.iter().enumerate() {
                let _ = write!(
                    out,
                    "{}\n    {{\"extension\": {}, \"files\": {}, \"bytes\": {}, \"allocated\": {}}}",
                    if index > 0 { "," } else { "" },
                    quote(&row.extension),
                    row.files,
                    row.bytes,
                    row.allocated
                );
            }
            out.push_str(if rows.is_empty() { "]" } else { "\n  ]" });
        }
        Section::Metrics { summary, .. } => {
            let _ = write!(out, "\"metrics\": {}", metric_summary_json(summary));
        }
        Section::Files { rows, .. } => {
            let _ = write!(out, "\"files\": [");
            for (index, row) in rows.iter().enumerate() {
                let _ = write!(out, "{}\n    {}", if index > 0 { "," } else { "" }, file_json(row));
            }
            out.push_str(if rows.is_empty() { "]" } else { "\n  ]" });
        }
        Section::Summary(row) => {
            let _ = write!(out, "\"summary\": {}", summary_json(row));
        }
    }
    out.push_str("\n}");
    out
}

fn metric_summary_json(summary: &MetricSummary) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "{{\"group\": {}, \"share_metric\": {}, \"words_per_page\": {}, \"total\": {}, \"rows\": [",
        quote(metric_group_label(summary.group)),
        quote(summary.share_metric.as_str()),
        summary.words_per_page,
        metric_row_json(&summary.total, summary.words_per_page)
    );
    for (index, row) in summary.rows.iter().enumerate() {
        let _ = write!(
            out,
            "{}\n  {}",
            if index > 0 { "," } else { "" },
            metric_row_json(row, summary.words_per_page)
        );
    }
    out.push_str(if summary.rows.is_empty() { "]}" } else { "\n]}" });
    out
}

fn metric_row_json(row: &MetricRow, words_per_page: u64) -> String {
    format!(
        "{{\"id\": {}, \"family\": {}, \"files\": {}, \"bytes\": {}, \"allocated\": {}, \"analyzed_files\": {}, \"share\": {{\"numerator\": {}, \"denominator\": {}}}, \"metrics\": {}, \"coverage\": {}, \"detection\": {}, \"pages\": {{\"words\": {}, \"words_per_page\": {}}}}}",
        quote(&row.id),
        quote(row.family.as_str()),
        row.files,
        row.bytes,
        row.allocated,
        row.analyzed_files,
        row.share.numerator,
        row.share.denominator,
        metric_values_json(row.metrics, document_words(row)),
        coverage_json(&row.coverage),
        detection_json(row),
        document_words(row),
        words_per_page,
    )
}

fn detection_json(row: &MetricRow) -> String {
    let mut sources = String::from("{");
    for (index, (source, count)) in row.detection_sources.iter().enumerate() {
        let _ = write!(
            sources,
            "{}{}: {}",
            if index > 0 { ", " } else { "" },
            quote(source.as_str()),
            count
        );
    }
    sources.push('}');
    let mut confidence = String::from("{");
    for (index, (level, count)) in row.detection_confidence.iter().enumerate() {
        let _ = write!(
            confidence,
            "{}{}: {}",
            if index > 0 { ", " } else { "" },
            quote(level.as_str()),
            count
        );
    }
    confidence.push('}');
    format!(
        "{{\"sources\": {sources}, \"confidence\": {confidence}, \"flags\": {{\"generated\": {}, \"vendored\": {}, \"documentation\": {}}}}}",
        row.generated_files, row.vendored_files, row.documentation_files
    )
}

fn metric_values_json(metrics: MetricValues, document_words: u64) -> String {
    format!(
        "{{\"physical_lines\": {}, \"blank_lines\": {}, \"nonblank_lines\": {}, \"code_lines\": {}, \"comment_lines\": {}, \"code_blank_lines\": {}, \"raw_words\": {}, \"logical_words\": {}, \"paragraphs\": {}, \"visible_words\": {}, \"visible_logical_words\": {}, \"document_words\": {}}}",
        metrics.physical_lines,
        metrics.blank_lines,
        metrics.nonblank_lines,
        metrics.code_lines,
        metrics.comment_lines,
        metrics.code_blank_lines,
        metrics.raw_words,
        metrics.logical_word_stats.logical_words(),
        metrics.paragraphs,
        metrics.visible_words,
        metrics.visible_logical_word_stats.logical_words(),
        document_words,
    )
}

fn coverage_json(coverage: &std::collections::BTreeMap<CoverageReason, u64>) -> String {
    let mut out = String::from("{");
    for (index, (reason, count)) in coverage.iter().enumerate() {
        let _ = write!(
            out,
            "{}{}: {}",
            if index > 0 { ", " } else { "" },
            quote(coverage_label(*reason)),
            count
        );
    }
    out.push('}');
    out
}

/// One file row as a JSON object.
fn file_json(row: &FileRow) -> String {
    format!(
        "{{\"path\": {}{}, \"kind\": {}, \"bytes\": {}, \"allocated\": {}, \"mtime_ns\": {}}}",
        quote(&row.path.to_string_lossy()),
        path_raw_field(&row.path),
        quote(kind_label(row.kind)),
        row.bytes,
        row.allocated,
        row.mtime_ns
    )
}

/// The `path_raw` field for a path that cannot survive `to_string_lossy`, or nothing.
///
/// Two names differing only in bytes that are not valid Unicode render as the same
/// string, so without this a machine consumer cannot tell them apart -- and this is the
/// output that drives tooling. Absent for the overwhelmingly common case, so a
/// well-behaved tree pays nothing for the guarantee.
fn path_raw_field(path: &Path) -> String {
    raw_identity_object(path).map_or_else(String::new, |raw| format!(", \"path_raw\": {raw}"))
}

/// A summary row as a JSON object.
fn summary_json(row: &SummaryRow) -> String {
    format!(
        "{{\"files\": {}, \"dirs\": {}, \"bytes\": {}, \"allocated\": {}, \"newest_mtime_ns\": {}}}",
        row.files,
        row.dirs,
        row.bytes,
        row.allocated,
        row.newest_mtime_ns.map_or_else(|| "null".to_string(), |value| value.to_string())
    )
}

/// A tree node as a nested JSON object.
///
/// Built with an explicit work stack rather than recursion, so nesting depth costs heap
/// rather than stack and a deep tree serializes instead of aborting.
fn tree_json(node: &TreeNode) -> String {
    /// One step of the serialization.
    enum Step<'a> {
        /// Open a node and queue its children.
        Open(&'a TreeNode),
        /// Emit literal text, for separators and closers.
        Text(&'static str),
    }

    let mut out = String::new();
    let mut stack: Vec<Step<'_>> = vec![Step::Open(node)];
    while let Some(step) = stack.pop() {
        match step {
            Step::Text(text) => out.push_str(text),
            Step::Open(node) => {
                let _ = write!(
                    out,
                    "{{\"name\": {}, \"path\": {}{}, \"kind\": {}, \"bytes\": {}, \"allocated\": {}, \"files\": {}, \"dirs\": {}, \"newest_mtime_ns\": {}, \"truncated\": {}",
                    quote(&node.name),
                    quote(&node.path.to_string_lossy()),
                    path_raw_field(&node.path),
                    quote(kind_label(node.kind)),
                    node.bytes,
                    node.allocated,
                    node.files,
                    node.dirs,
                    node.newest_mtime_ns
                        .map_or_else(|| "null".to_string(), |value| value.to_string()),
                    node.truncated
                );
                if node.children.is_empty() {
                    out.push_str(", \"children\": []}");
                    continue;
                }
                out.push_str(", \"children\": [");
                // The stack pops in reverse, so pushes run last-to-first: closer, then
                // each child followed by the separator that precedes it. Getting this
                // backwards emits `[{a}{b},]` — balanced, and not valid JSON.
                stack.push(Step::Text("]}"));
                for (index, child) in node.children.iter().enumerate().rev() {
                    stack.push(Step::Open(child));
                    if index > 0 {
                        stack.push(Step::Text(","));
                    }
                }
            }
        }
    }
    out
}

// ---- yaml ----

/// Render the report as YAML.
fn render_yaml(report: &Report) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "schema: {}", yaml_scalar(report_schema(report)));
    let _ = writeln!(out, "generator: {}", yaml_scalar(&generator()));
    let _ = writeln!(out, "root: {}", yaml_scalar(&report.root.to_string_lossy()));
    match report.scan_started_at {
        Some(at) => {
            let _ = writeln!(out, "scan_started_at: {}", yaml_scalar(&format_rfc3339(at)));
        }
        None => out.push_str("scan_started_at: null\n"),
    }
    let _ = writeln!(out, "generated_at: {}", yaml_scalar(&format_rfc3339(report.generated_at)));
    let _ = writeln!(out, "source: {}", yaml_scalar(source_label(report.source)));
    let _ = writeln!(out, "freshness: {}", yaml_scalar(freshness_label(report.freshness)));
    let _ = writeln!(out, "complete: {}", report.complete);
    if report.errors.is_empty() {
        out.push_str("errors: []\n");
    } else {
        out.push_str("errors:\n");
        for error in &report.errors {
            let _ = writeln!(out, "  - {}", yaml_scalar(error));
        }
    }
    if report_schema(report) == CONTENT_REPORT_SCHEMA {
        match report.analysis.as_ref() {
            None => out.push_str("analysis: null\n"),
            Some(analysis) => {
                out.push_str("analysis:\n");
                let requested = analysis_set_labels(analysis.profile);
                if requested.is_empty() {
                    out.push_str("  analyze: []\n");
                } else {
                    out.push_str("  analyze:\n");
                    for label in requested {
                        let _ = writeln!(out, "    - {label}");
                    }
                }
                let _ = writeln!(
                    out,
                    "  type_rules_fingerprint: {}",
                    analysis.provenance.type_rules_fingerprint
                );
                let _ = writeln!(
                    out,
                    "  options_fingerprint: {}",
                    analysis.provenance.options_fingerprint.0
                );
                if analysis.provenance.analyzers.is_empty() {
                    out.push_str("  analyzers: []\n");
                } else {
                    out.push_str("  analyzers:\n");
                    for (id, version) in &analysis.provenance.analyzers {
                        let _ = writeln!(out, "    - id: {}", id.0);
                        let _ = writeln!(out, "      version: {}", version.0);
                    }
                }
            }
        }
    }
    out.push_str("reports:\n");

    for section in &report.sections {
        let _ = writeln!(out, "  - view: {}", yaml_scalar(view_label(section.view())));
        match section {
            Section::Tree(root) => {
                out.push_str("    tree:\n");
                yaml_tree(&mut out, root, 6);
            }
            Section::Extensions(rows) => {
                if rows.is_empty() {
                    out.push_str("    extensions: []\n");
                } else {
                    out.push_str("    extensions:\n");
                    for row in rows {
                        let _ = writeln!(out, "      - extension: {}", yaml_scalar(&row.extension));
                        let _ = writeln!(out, "        files: {}", row.files);
                        let _ = writeln!(out, "        bytes: {}", row.bytes);
                        let _ = writeln!(out, "        allocated: {}", row.allocated);
                    }
                }
            }
            Section::Metrics { summary, .. } => yaml_metrics(&mut out, summary),
            Section::Files { rows, .. } => {
                if rows.is_empty() {
                    out.push_str("    files: []\n");
                } else {
                    out.push_str("    files:\n");
                    for row in rows {
                        let _ = writeln!(
                            out,
                            "      - path: {}",
                            yaml_scalar(&row.path.to_string_lossy())
                        );
                        let _ =
                            writeln!(out, "        kind: {}", yaml_scalar(kind_label(row.kind)));
                        let _ = writeln!(out, "        bytes: {}", row.bytes);
                        let _ = writeln!(out, "        allocated: {}", row.allocated);
                        let _ = writeln!(out, "        mtime_ns: {}", row.mtime_ns);
                    }
                }
            }
            Section::Summary(row) => {
                out.push_str("    summary:\n");
                let _ = writeln!(out, "      files: {}", row.files);
                let _ = writeln!(out, "      dirs: {}", row.dirs);
                let _ = writeln!(out, "      bytes: {}", row.bytes);
                let _ = writeln!(out, "      allocated: {}", row.allocated);
                let _ =
                    writeln!(out, "      newest_mtime_ns: {}", yaml_option(row.newest_mtime_ns));
            }
        }
    }
    out
}

fn yaml_metrics(out: &mut String, summary: &MetricSummary) {
    out.push_str("    metrics:\n");
    let _ = writeln!(out, "      group: {}", metric_group_label(summary.group));
    let _ = writeln!(out, "      share_metric: {}", summary.share_metric.as_str());
    let _ = writeln!(out, "      words_per_page: {}", summary.words_per_page);
    out.push_str("      total:\n");
    yaml_metric_row(out, &summary.total, summary.words_per_page, 8, false);
    if summary.rows.is_empty() {
        out.push_str("      rows: []\n");
    } else {
        out.push_str("      rows:\n");
        for row in &summary.rows {
            yaml_metric_row(out, row, summary.words_per_page, 8, true);
        }
    }
}

fn yaml_metric_row(out: &mut String, row: &MetricRow, words_per_page: u64, pad: usize, item: bool) {
    let indent = " ".repeat(pad);
    let lead = if item { format!("{indent}- ") } else { indent.clone() };
    let rest = if item { format!("{indent}  ") } else { indent };
    let _ = writeln!(out, "{lead}id: {}", yaml_scalar(&row.id));
    let _ = writeln!(out, "{rest}family: {}", row.family.as_str());
    let _ = writeln!(out, "{rest}files: {}", row.files);
    let _ = writeln!(out, "{rest}bytes: {}", row.bytes);
    let _ = writeln!(out, "{rest}allocated: {}", row.allocated);
    let _ = writeln!(out, "{rest}analyzed_files: {}", row.analyzed_files);
    let _ = writeln!(out, "{rest}share_numerator: {}", row.share.numerator);
    let _ = writeln!(out, "{rest}share_denominator: {}", row.share.denominator);
    let _ = writeln!(out, "{rest}physical_lines: {}", row.metrics.physical_lines);
    let _ = writeln!(out, "{rest}blank_lines: {}", row.metrics.blank_lines);
    let _ = writeln!(out, "{rest}nonblank_lines: {}", row.metrics.nonblank_lines);
    let _ = writeln!(out, "{rest}code_lines: {}", row.metrics.code_lines);
    let _ = writeln!(out, "{rest}comment_lines: {}", row.metrics.comment_lines);
    let _ = writeln!(out, "{rest}code_blank_lines: {}", row.metrics.code_blank_lines);
    let _ = writeln!(out, "{rest}raw_words: {}", row.metrics.raw_words);
    let _ =
        writeln!(out, "{rest}logical_words: {}", row.metrics.logical_word_stats.logical_words());
    let _ = writeln!(out, "{rest}paragraphs: {}", row.metrics.paragraphs);
    let _ = writeln!(out, "{rest}visible_words: {}", row.metrics.visible_words);
    let _ = writeln!(
        out,
        "{rest}visible_logical_words: {}",
        row.metrics.visible_logical_word_stats.logical_words()
    );
    let _ = writeln!(out, "{rest}document_words: {}", document_words(row));
    let _ = writeln!(out, "{rest}page_words: {}", document_words(row));
    let _ = writeln!(out, "{rest}words_per_page: {words_per_page}");
    if row.coverage.is_empty() {
        let _ = writeln!(out, "{rest}coverage: {{}}");
    } else {
        let _ = writeln!(out, "{rest}coverage:");
        for (reason, count) in &row.coverage {
            let _ = writeln!(out, "{rest}  {}: {count}", coverage_label(*reason));
        }
    }
    let _ = writeln!(out, "{rest}detection:");
    yaml_detection_map(out, "sources", &row.detection_sources, &rest, DetectionSource::as_str);
    yaml_detection_map(
        out,
        "confidence",
        &row.detection_confidence,
        &rest,
        DetectionConfidence::as_str,
    );
    let _ = writeln!(out, "{rest}  flags:");
    let _ = writeln!(out, "{rest}    generated: {}", row.generated_files);
    let _ = writeln!(out, "{rest}    vendored: {}", row.vendored_files);
    let _ = writeln!(out, "{rest}    documentation: {}", row.documentation_files);
}

fn yaml_detection_map<K: Ord + Copy>(
    out: &mut String,
    name: &str,
    values: &std::collections::BTreeMap<K, u64>,
    indent: &str,
    label: impl Fn(K) -> &'static str,
) {
    if values.is_empty() {
        let _ = writeln!(out, "{indent}  {name}: {{}}");
        return;
    }
    let _ = writeln!(out, "{indent}  {name}:");
    for (key, count) in values {
        let _ = writeln!(out, "{indent}    {}: {count}", label(*key));
    }
}

/// Render a tree node as YAML at a given indent.
///
/// Iterative, matching the other two renderers: nesting lives on the heap.
fn yaml_tree(out: &mut String, root: &TreeNode, pad: usize) {
    let mut stack: Vec<(&TreeNode, usize, bool)> = vec![(root, pad, false)];
    while let Some((node, indent_width, as_item)) = stack.pop() {
        let indent = " ".repeat(indent_width);
        let (lead, rest) = if as_item {
            (format!("{indent}- "), format!("{indent}  "))
        } else {
            (indent.clone(), indent.clone())
        };
        let _ = writeln!(out, "{lead}name: {}", yaml_scalar(&node.name));
        let _ = writeln!(out, "{rest}path: {}", yaml_scalar(&node.path.to_string_lossy()));
        let _ = writeln!(out, "{rest}kind: {}", yaml_scalar(kind_label(node.kind)));
        let _ = writeln!(out, "{rest}bytes: {}", node.bytes);
        let _ = writeln!(out, "{rest}allocated: {}", node.allocated);
        let _ = writeln!(out, "{rest}files: {}", node.files);
        let _ = writeln!(out, "{rest}dirs: {}", node.dirs);
        let _ = writeln!(out, "{rest}newest_mtime_ns: {}", yaml_option(node.newest_mtime_ns));
        let _ = writeln!(out, "{rest}truncated: {}", node.truncated);
        if node.children.is_empty() {
            let _ = writeln!(out, "{rest}children: []");
            continue;
        }
        let _ = writeln!(out, "{rest}children:");
        let child_indent = rest.len() + 2;
        for child in node.children.iter().rev() {
            stack.push((child, child_indent, true));
        }
    }
}

/// An optional integer as a YAML scalar.
fn yaml_option(value: Option<i64>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

/// Quote a YAML scalar whenever a bare word would be ambiguous.
///
/// Always quoting would be simpler and uglier; quoting only what needs it keeps the
/// output readable, which is the reason to offer YAML at all.
fn yaml_scalar(value: &str) -> String {
    let safe = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | '-' | '+'))
        && !value.starts_with('-')
        && value.parse::<f64>().is_err()
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "~"
        );
    if safe { value.to_string() } else { quote(value) }
}

// ---- shared helpers ----

/// Indent every line of a block.
fn indent(block: &str, pad: usize) -> String {
    let padding = " ".repeat(pad);
    block
        .lines()
        .map(|line| if line.is_empty() { line.to_string() } else { format!("{padding}{line}") })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Collapse a pretty-printed JSON fragment onto one line.
///
/// Joining with a space and then tidying is what keeps the one-line form readable, but
/// the tidying has to cover both bracket kinds: `[ {` in a JSONL record is the kind of
/// cosmetic difference that makes two equivalent documents fail a byte-exact golden.
fn collapse(block: &str) -> String {
    let joined = block.lines().map(str::trim).collect::<Vec<_>>().join(" ");
    joined.replace("{ ", "{").replace(" }", "}").replace("[ ", "[").replace(" ]", "]")
}

/// Quote and escape a string as a JSON scalar.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// This binary's identity, for the `generator` field.
fn generator() -> String {
    format!("fdu {}", env!("CARGO_PKG_VERSION"))
}

/// All-caps header naming a view in multi-view text output.
///
/// Deliberately not `view_label(..).to_uppercase()`: the wire label is a schema promise
/// machine consumers match on, and deriving the human header from it would let a
/// presentation change reach into the schema, or freeze the schema for a presentation
/// reason. They spell the same word today because the same word is right in both places,
/// and a test holds them in step rather than a shared expression.
fn view_header(view: ViewSpec) -> &'static str {
    match view {
        ViewSpec::Tree => "TREE",
        ViewSpec::Types => "TYPES",
        ViewSpec::Extensions => "EXTENSIONS",
        ViewSpec::Families => "FAMILIES",
        ViewSpec::Languages => "LANGUAGES",
        ViewSpec::Documents => "DOCUMENTS",
        ViewSpec::Files => "FILES",
        ViewSpec::Largest => "LARGEST",
        ViewSpec::Recent => "RECENT",
        ViewSpec::Summary => "SUMMARY",
    }
}

/// Stable wire label for a view.
pub(crate) fn view_label(view: ViewSpec) -> &'static str {
    match view {
        ViewSpec::Tree => "tree",
        ViewSpec::Types => "types",
        ViewSpec::Extensions => "extensions",
        ViewSpec::Families => "families",
        ViewSpec::Languages => "languages",
        ViewSpec::Documents => "documents",
        ViewSpec::Files => "files",
        ViewSpec::Largest => "largest",
        ViewSpec::Recent => "recent",
        ViewSpec::Summary => "summary",
    }
}

fn report_schema(report: &Report) -> &'static str {
    if report.analysis.is_some()
        || report.sections.iter().any(|section| matches!(section, Section::Metrics { .. }))
    {
        CONTENT_REPORT_SCHEMA
    } else {
        REPORT_SCHEMA
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
        CoverageReason::Unsupported => "unsupported",
        CoverageReason::IoError => "io_error",
        CoverageReason::ChangedDuringRead => "changed_during_read",
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

// Rounding a fraction to one of eleven bar widths is exactly the case where float-cast
// lints have nothing to protect: the value is clamped to [0, 1] before the cast and the
// result is clamped to WIDTH after it.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
fn bar(share: f64, color: bool) -> String {
    const WIDTH: usize = 10;
    let filled = ((share.clamp(0.0, 1.0) * WIDTH as f64).round() as usize).min(WIDTH);
    let rendered = format!("{}{}", "█".repeat(filled), "░".repeat(WIDTH - filled));
    paint(&rendered, STYLE_BAR, color)
}

/// Render a byte count at human scale.
pub(crate) fn human_bytes(bytes: u64) -> String {
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
        format!("{bytes} B")
    } else if whole < 10 {
        let tenths = (remainder * 10) / 1024;
        format!("{whole}.{tenths} {}", UNITS[unit])
    } else {
        format!("{whole} {}", UNITS[unit])
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
pub const STREAM_SCHEMA: &str = "fdu.stream/1";

/// Render one streamed change as a tagged record.
#[cfg(feature = "watch")]
pub fn render_change(change: &crate::Change, format: Format) -> String {
    let kind = match change.kind {
        crate::ChangeKind::Upsert => "upsert",
        crate::ChangeKind::Remove => "remove",
        crate::ChangeKind::Invalidate => "invalidate",
    };

    if format == Format::Text {
        // Path first, so the stream stays greppable and cuts the same way a one-shot
        // listing does; the operation follows on the same line.
        return format!("{}\t{kind}", change.path.display());
    }

    let mut out = String::new();
    let _ = write!(
        out,
        "{{\"schema\": {}, \"record\": \"change\", \"op\": {}, \"path\": {}, \"clock\": {}",
        quote(STREAM_SCHEMA),
        quote(kind),
        quote(&change.path.to_string_lossy()),
        change.clock
    );
    if let Some(entry_kind) = change.entry_kind {
        let _ = write!(out, ", \"kind\": {}", quote(kind_label(entry_kind)));
    }
    if let Some(bytes) = change.bytes {
        let _ = write!(out, ", \"bytes\": {bytes}");
    }
    if let Some(allocated) = change.allocated {
        let _ = write!(out, ", \"allocated\": {allocated}");
    }
    if let Some(mtime) = change.mtime_ns {
        let _ = write!(out, ", \"mtime_ns\": {mtime}");
    }
    out.push('}');
    out
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

/// Render the raw-identity companion for a path, when one is needed.
fn raw_identity_object(path: &Path) -> Option<String> {
    let (encoding, hex) = raw_os_identity(path.as_os_str())?;
    Some(format!("{{\"encoding\": {}, \"hex\": {}}}", quote(encoding), quote(&hex)))
}

/// Render cache status in a machine format.
///
/// A separate entry point rather than a `Report` section: cache status is a fact about
/// the cache directory, not about a tree, and folding it into the report schema would
/// make every consumer parse a variant that is empty on every normal run.
pub fn render_cache_status(statuses: &[crate::CacheStatus], format: Format) -> String {
    let row = |status: &crate::CacheStatus| match &status.snapshot {
        Some(info) => format!(
            "{{\"path\": {}, \"bytes\": {}, \"content_bytes\": {}, \"recognized\": true, \"root\": {}, \"entries\": {}}}",
            quote(&status.path.to_string_lossy()),
            status.bytes,
            status.content_bytes.map_or_else(|| "null".to_string(), |bytes| bytes.to_string()),
            quote(&info.root.to_string_lossy()),
            info.entries
        ),
        None => format!(
            "{{\"path\": {}, \"bytes\": {}, \"recognized\": false}}",
            quote(&status.path.to_string_lossy()),
            status.bytes
        ),
    };

    match format {
        Format::Jsonl => statuses.iter().map(row).collect::<Vec<_>>().join("\n"),
        Format::Yaml => {
            let mut out = String::from("caches:");
            for status in statuses {
                let _ = write!(out, "\n  - path: {}", yaml_scalar(&status.path.to_string_lossy()));
                let _ = write!(out, "\n    bytes: {}", status.bytes);
                let _ = write!(
                    out,
                    "\n    content_bytes: {}",
                    status
                        .content_bytes
                        .map_or_else(|| "null".to_string(), |bytes| bytes.to_string())
                );
                match &status.snapshot {
                    Some(info) => {
                        let _ = write!(out, "\n    recognized: true");
                        let _ = write!(
                            out,
                            "\n    root: {}",
                            yaml_scalar(&info.root.to_string_lossy())
                        );
                        let _ = write!(out, "\n    entries: {}", info.entries);
                    }
                    None => {
                        let _ = write!(out, "\n    recognized: false");
                    }
                }
            }
            out
        }
        // Text is rendered by the CLI, which owns the human layout; JSON is the default
        // machine shape here.
        Format::Json | Format::Text => {
            let rows = statuses.iter().map(row).collect::<Vec<_>>().join(",\n    ");
            if statuses.is_empty() {
                "{\n  \"caches\": []\n}".to_string()
            } else {
                format!("{{\n  \"caches\": [\n    {rows}\n  ]\n}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;
    use crate::engine_contract::{Attrs, Observation, Op, ScanScope};
    use crate::query::{Bound, Provenance, Query, Selection, report};
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

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

    /// Colour must not move anything.
    ///
    /// The golden suite structurally cannot check this: it runs under `NO_COLOR=1` and
    /// only ever sees the uncoloured form, which is exactly how the extensions view
    /// shipped misaligned — `{:<12}` counted the escape sequences toward the field width,
    /// so the padding collapsed the moment colour was on and every golden still passed.
    #[test]
    fn colour_never_changes_the_layout_of_any_view() {
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
            // Two views carry no label to style: `files` is a bare listing of paths meant
            // for piping, and `summary` is one aggregate line. Everything that draws a
            // label draws it styled, and this catches a view that quietly stops.
            let styles_a_label = !matches!(view, ViewSpec::Files | ViewSpec::Summary);
            assert_eq!(
                plain != coloured,
                styles_a_label,
                "{view:?} disagrees with whether it styles a label"
            );
            assert_eq!(
                strip_ansi(&coloured),
                plain,
                "{view:?} lays out differently once colour is on"
            );
        }
    }

    /// The rule the helper exists to enforce, stated directly.
    #[test]
    fn a_label_cell_is_measured_on_visible_text() {
        let plain = label_cell("md", 6, STYLE_TYPE, false);
        let coloured = label_cell("md", 6, STYLE_TYPE, true);
        assert_eq!(plain, "md    ", "four spaces of padding");
        assert!(coloured.starts_with('\u{1b}'), "the label is styled");
        assert!(coloured.ends_with("    "), "and padded by the same four: {coloured:?}");
        // A label at or past the width gets no padding rather than a negative one.
        assert_eq!(label_cell("verylonglabel", 4, STYLE_TYPE, false), "verylonglabel");
    }

    fn fixture(views: &[ViewSpec]) -> Report {
        let mut index = Index::new_with_scope("/root", ScanScope::default());
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
            &Query { selection: Selection::default(), views: views.to_vec(), ..Query::default() },
            &Provenance {
                scan_started_at: Some(UNIX_EPOCH + Duration::from_secs(1_786_386_151)),
                generated_at: UNIX_EPOCH + Duration::from_secs(1_786_386_152),
                source: ReportSource::ColdScan,
                complete: true,
                errors: Vec::new(),
            },
        )
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

    #[test]
    fn every_view_renders_in_every_format() {
        // Formats are serializations, not features: no view may lack one.
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
            let report = fixture(&[view]);
            for format in [Format::Text, Format::Json, Format::Jsonl, Format::Yaml] {
                let rendered = render(&report, format, false);
                assert!(!rendered.trim().is_empty(), "{view:?} in {format:?} rendered nothing");
            }
        }
    }

    #[test]
    fn text_tree_restores_compact_bars_and_keeps_structural_indent_in_the_name_column() {
        let text = render(&fixture(&[ViewSpec::Tree]), Format::Text, false);
        assert_eq!(
            text,
            concat!(
                "     120 B  ██████████   100%  . (2 files)\n",
                "     100 B  ████████░░    83%    src (1 file)\n",
            )
        );

        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0].find("120 B"), lines[1].find("100 B"));
        assert_eq!(lines[0].find('█'), lines[1].find('█'));
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
            &Query { views: vec![ViewSpec::Languages], ..Query::default() },
            &Provenance {
                scan_started_at: None,
                generated_at: UNIX_EPOCH,
                source: ReportSource::ColdScan,
                complete: true,
                errors: Vec::new(),
            },
        );

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
            &Query {
                selection: Selection { depth: Bound::All, ..Selection::default() },
                views: vec![ViewSpec::Tree],
                ..Query::default()
            },
            &Provenance {
                scan_started_at: None,
                generated_at: UNIX_EPOCH,
                source: ReportSource::ColdScan,
                complete: true,
                errors: Vec::new(),
            },
        );

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
        assert!(json.contains("\"schema\": \"fdu.report/1\""));
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
        assert_eq!(REPORT_SCHEMA, "fdu.report/1");
        // Bumped to /3 when the analysis block's `profile` scalar became the `analyze`
        // list: the requested analyzer set stopped being expressible as one label.
        assert_eq!(CONTENT_REPORT_SCHEMA, "fdu.report/3");
    }

    #[test]
    fn metric_sections_upgrade_schema_while_metadata_sections_stay_on_v1() {
        let metadata = render(&fixture(&[ViewSpec::Tree]), Format::Json, false);
        assert!(metadata.contains("\"schema\": \"fdu.report/1\""));
        assert!(!metadata.contains("\"analysis\""));

        let metrics = render(&fixture(&[ViewSpec::Types]), Format::Json, false);
        assert!(metrics.contains("\"schema\": \"fdu.report/3\""));
        assert!(metrics.contains("\"analysis\": null"));
        assert!(metrics.contains("\"share\": {\"numerator\":"));
    }

    /// The change stream carries the same promise the report does.
    ///
    /// A constant assertion alone would not: it pins the version string while leaving the
    /// record's shape free to change underneath it, which is the failure the promise
    /// exists to prevent. This pins the whole record, so adding, renaming, or reordering
    /// a field fails here and forces a deliberate version bump.
    #[cfg(feature = "watch")]
    #[test]
    fn a_stream_record_is_pinned_field_by_field() {
        use crate::{Change, ChangeKind};

        assert_eq!(STREAM_SCHEMA, "fdu.stream/1");

        let upsert = Change {
            path: ["src", "main.rs"].iter().collect(),
            kind: ChangeKind::Upsert,
            entry_kind: Some(EntryKind::File),
            bytes: Some(2_048),
            allocated: Some(4_096),
            mtime_ns: Some(1_700_000_000_000_000_000),
            clock: 7,
        };
        // Path separators differ by platform, so the expectation is built the same way
        // the renderer builds it rather than hardcoding a slash.
        let path = upsert.path.to_string_lossy().replace('\\', "\\\\");
        assert_eq!(
            render_change(&upsert, Format::Json),
            format!(
                "{{\"schema\": \"fdu.stream/1\", \"record\": \"change\", \"op\": \"upsert\", \
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
            clock: 8,
        };
        assert_eq!(
            render_change(&removed, Format::Json),
            "{\"schema\": \"fdu.stream/1\", \"record\": \"change\", \"op\": \"remove\", \
             \"path\": \"gone.txt\", \"clock\": 8}"
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
            clock: 9,
        };
        assert_eq!(
            render_change(&invalidated, Format::Json),
            "{\"schema\": \"fdu.stream/1\", \"record\": \"change\", \"op\": \"invalidate\", \
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
        assert!(colored.contains(&paint("TREE", STYLE_VIEW_HEADER, true)), "{colored:?}");
        assert!(colored.contains(&paint("SUMMARY", STYLE_VIEW_HEADER, true)), "{colored:?}");
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
            assert_eq!(header, view_label(view).to_uppercase(), "{view:?}");
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
        assert_eq!(Format::ALL.len(), 4);
    }

    #[test]
    fn human_bytes_reads_at_scale() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(1024 * 1024 * 20), "20 MiB");
    }

    #[test]
    fn bars_are_fixed_at_ten_cells_and_saturate() {
        assert_eq!(bar(0.0, false), "░░░░░░░░░░");
        assert_eq!(bar(0.5, false), "█████░░░░░");
        assert_eq!(bar(2.0, false), "██████████");
        assert!((ratio(5, 0) - 0.0).abs() < f64::EPSILON);
    }
}
