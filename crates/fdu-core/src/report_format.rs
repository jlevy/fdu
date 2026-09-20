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
use std::io;
use std::path::Path;

use anstyle::{AnsiColor, Style as AnsiStyle};

use crate::classify::human_language_name;
use crate::content::{CoverageReason, METRICS};
use crate::control::ControlCoverage;
use crate::emit::{Event, IoFmt, JsonSink, Scalar, Shape, Sink, YamlSink};
use crate::engine_contract::{Coverage, EntryKind, Freshness, IssueKind, Source};
use crate::query::{
    FileRow, IgnoredEntries, IgnoredTally, MetricGroup, MetricRow, MetricSummary, Report,
    ReportSource, Section, ShareMetric, SizeMetric, SummaryRow, TierState, TreeNode, TypeRow,
    ViewSpec, format_rfc3339, format_rfc3339_nanos, pages,
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

/// Telemetry: what the tool did, as against what it found.
///
/// The same role the CLI's performance footer and display notes use, so a bound stated in
/// a header reads as reporting rather than as data — see the styling system in `cli.rs`.
const STYLE_TELEMETRY: AnsiStyle = AnsiColor::BrightBlack.on_default();

/// Established label width for non-language metric summaries.
const TEXT_METRIC_LABEL_WIDTH: usize = 18;
/// Floor for the extensions view's label column.
const TEXT_TYPE_LABEL_WIDTH: usize = 12;

/// Machine-output schema identity.
///
/// Any change to a field's name, type, or meaning bumps this, and a golden test fails if
/// the schema moves without it — the versioning is the promise, not the intention.
pub const REPORT_SCHEMA: &str = "fdu.report/7";
/// All reports now use one shape-versioned schema regardless of requested analyzers.
pub const CONTENT_REPORT_SCHEMA: &str = REPORT_SCHEMA;
/// Machine-output schema identity for cache status.
///
/// Its own identity because cache status is its own document: a fact about the cache
/// directory rather than about a tree, which is why it is not a `Report` section. It
/// carries the same promise as [`REPORT_SCHEMA`] and versions independently, so a change
/// to the report shape never invalidates a cache-status consumer, or the reverse.
///
/// `fdu.cache/2` adds the identity of every tier a store holds: a current snapshot's
/// `identity`, and a `content` object for the sidecar beside any snapshot, in place of
/// `content_bytes`.
pub const CACHE_SCHEMA: &str = "fdu.cache/2";

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

/// Start one document in a multi-document stream for `format`.
pub const fn document_start(format: Format) -> &'static str {
    match format {
        Format::Yaml => "---\n",
        Format::Text | Format::Json | Format::Jsonl => "",
    }
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
        Format::Json => render_report_machine(report, true, JsonSink::pretty()),
        Format::Jsonl => render_report_jsonl(report),
        Format::Yaml => render_report_machine(report, true, YamlSink::new()),
    }
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
    match format {
        Format::Text => out.write_all(render_text(report, color).as_bytes()),
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
        emit_section(&mut sink, section);
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
        emit_section(&mut sink, section);
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
    emit_field(sink, Field::always("schema"), true, |sink| {
        emit_scalar(sink, Scalar::Str(REPORT_SCHEMA));
    });
    let generator = generator();
    emit_field(sink, Field::always("generator"), true, |sink| {
        emit_scalar(sink, Scalar::Str(&generator));
    });
    let root = report.root.to_string_lossy();
    emit_field(sink, Field::always("root"), true, |sink| {
        emit_scalar(sink, Scalar::Str(&root));
    });
    emit_raw_identity(sink, "root_raw", &report.root);
    emit_field(sink, Field::always("request"), true, |sink| emit_request(sink, report));
    emit_field(sink, Field::always("status"), true, |sink| emit_status(sink, report));
    emit_field(sink, Field::always("provenance"), true, |sink| emit_provenance(sink, report));
    emit_field(sink, Field::always("ignore_rules"), true, |sink| {
        emit_ignore_rules(sink, &report.ignore_rules);
    });
    emit_field(sink, Field::nullable("analysis"), true, |sink| {
        emit_analysis(sink, report.analysis.as_ref());
    });
    emit_field(sink, Field::when_set("reports"), with_sections, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for section in &report.sections {
            emit_section(sink, section);
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
        for error in &report.status.errors {
            sink.event(Event::BeginMap(Shape::Block));
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

fn emit_ignore_rules(sink: &mut impl Sink, rules: &ControlCoverage) {
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
    emit_u64_field(sink, "refused", observed.refused);
    emit_field(sink, Field::always("refusals"), true, |sink| {
        sink.event(Event::BeginSeq(Shape::Block));
        for refusal in &observed.refusals {
            sink.event(Event::BeginMap(Shape::Block));
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

fn emit_section(sink: &mut impl Sink, section: &Section) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_str_field(sink, "view", section.view().label());
    match section {
        Section::Tree(root) => {
            emit_field(sink, Field::always("tree"), true, |sink| emit_tree(sink, root));
        }
        Section::Extensions { rows, total } => {
            emit_bound_field(sink, rows.len(), *total);
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
                    emit_file_row(sink, row);
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

fn emit_file_row(sink: &mut impl Sink, row: &FileRow) {
    sink.event(Event::BeginMap(Shape::Block));
    emit_path_fields(sink, &row.path);
    emit_str_field(sink, "kind", kind_label(row.kind));
    emit_u64_field(sink, "bytes", row.bytes);
    emit_u64_field(sink, "allocated", row.allocated);
    emit_i64_field(sink, "mtime_ns", row.mtime_ns);
    emit_field(sink, Field::nullable("ignored"), true, |sink| match row.ignored {
        Some(value) => emit_scalar(sink, Scalar::Bool(value)),
        None => emit_scalar(sink, Scalar::Null),
    });
    sink.event(Event::EndMap);
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
                emit_field(sink, Field::always("truncated"), true, |sink| {
                    emit_scalar(sink, Scalar::Bool(node.truncated));
                });
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
        let bound = bound_note(section);
        if headed {
            let _ = writeln!(
                out,
                "{}{}",
                paint(view_header(section.view()), STYLE_VIEW_HEADER, color),
                paint(&bound, STYLE_TELEMETRY, color),
            );
        } else if !bound.is_empty() {
            // A single-view report has no header, and that is precisely the shape
            // `fdu --view largest` produces — so the bound gets its own line rather than
            // riding on a header that is not there.
            let _ = writeln!(out, "{}", paint(bound.trim_start(), STYLE_TELEMETRY, color));
        }
        match section {
            Section::Tree(root) => {
                render_text_tree(&mut out, root, report.size, report.ignored_entries, color);
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
            Section::Files { view, rows, .. } => match view {
                ViewSpec::Largest => render_text_ranked_files(&mut out, rows, report.size, |row| {
                    human_bytes(pick(report.size, row.bytes, row.allocated))
                }),
                ViewSpec::Recent => render_text_ranked_files(&mut out, rows, report.size, |row| {
                    format_rfc3339_nanos(row.mtime_ns)
                }),
                _ => {
                    for row in rows {
                        let _ = writeln!(out, "{}", row.path.display());
                    }
                }
            },
            Section::Summary(row) => {
                render_text_summary(&mut out, row, report.size, report.ignored_entries);
            }
        }
    }
    // Remarks about the report, after the report and before the caller's own epilogue.
    // These used to print after the CLI's performance footer, which read as though
    // something followed the terminator; and living in the CLI meant only the CLI could
    // tell anyone a view had been dropped (fdu-x8u6).
    for note in &report.notes {
        let _ = writeln!(out, "{}", paint(note, STYLE_TELEMETRY, color));
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
    if let Some(note) = share_metric_note(summary.share_metric) {
        let _ = writeln!(out, "{}", paint(note, STYLE_TELEMETRY, color));
    }
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
        if let Some(physical_lines) = row.metrics.physical_lines.filter(|lines| *lines > 0) {
            if let (Some(code_lines), Some(comment_lines), Some(code_blank_lines)) =
                (row.metrics.code_lines, row.metrics.comment_lines, row.metrics.code_blank_lines)
            {
                let _ = write!(
                    suffix,
                    ", {} lines ({} code, {} comment, {} blank)",
                    physical_lines, code_lines, comment_lines, code_blank_lines
                );
            } else {
                let _ = write!(
                    suffix,
                    ", {} lines ({} nonblank, {} blank)",
                    physical_lines,
                    row.metrics.nonblank_lines.expect("lines requested"),
                    row.metrics.blank_lines.expect("lines requested")
                );
            }
        }
        if let Some(page) = pages(row, summary.words_per_page).filter(|page| page.words > 0) {
            let page_tenths = page.words.saturating_mul(10) / page.words_per_page;
            let _ = write!(
                suffix,
                ", {} words ({}.{:01} pages)",
                page.words,
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
        let coverage = match view {
            ViewSpec::Languages => row.code_coverage.as_ref().unwrap_or(&row.lines_coverage),
            ViewSpec::Documents => row.words_coverage.as_ref().unwrap_or(&row.lines_coverage),
            _ => &row.lines_coverage,
        };
        for (reason, count) in coverage {
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

/// Explain a percentage column whose denominator is not the byte column beside it.
///
/// Byte shares need no annotation because the adjacent size column already names their
/// numerator. Code and document reports deliberately rank by a content metric while
/// retaining bytes in the first column, so leaving the percentage unlabeled makes two
/// unlike quantities look as though they must agree.
fn share_metric_note(metric: ShareMetric) -> Option<&'static str> {
    match metric {
        ShareMetric::CodeLines => Some("Percentage column: code lines"),
        ShareMetric::DocumentWords => Some("Percentage column: document words"),
        ShareMetric::RawWords => Some("Percentage column: raw words"),
        ShareMetric::ApparentBytes | ShareMetric::AllocatedBytes => None,
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
        CoverageReason::UnsupportedEncoding => "unsupported encoding",
        CoverageReason::Unsupported => "unsupported",
        CoverageReason::IoError => "I/O error",
        CoverageReason::ChangedDuringRead => "changed during read",
    }
}

/// The ignored share a text row ends with, as ` (128 B ignored)`, or nothing.
///
/// One placement for every row that carries a share: after the row's own detail, so the
/// fixed size, bar, and percentage columns keep their alignment. Nothing is appended when
/// no file is ignored, when the index observed no control state, or when the selection
/// admitted only ignored entries, where the share would repeat the row's size. A share of
/// ignored directories alone holds no bytes, and `(0 B ignored)` would say nothing a
/// reader can act on; the machine formats still count them. Text cannot tell "nothing
/// ignored" from "no rules read"; the performance line says whether any rule was read,
/// and machine formats carry a zero share and `null` respectively.
fn ignored_suffix(
    ignored: Option<IgnoredTally>,
    size: SizeMetric,
    selected: IgnoredEntries,
) -> String {
    let shown = match selected {
        IgnoredEntries::Include | IgnoredEntries::Exclude => {
            ignored.filter(|share| share.files > 0)
        }
        IgnoredEntries::Only => None,
    };
    shown.map_or_else(String::new, |share| {
        format!(" ({} ignored)", human_bytes(pick(size, share.bytes, share.allocated)))
    })
}

/// Render a tree section with fixed size, bar, and percentage columns.
///
/// Iterative for the same reason the expansion is: a deep tree must render, not panic.
fn render_text_tree(
    out: &mut String,
    root: &TreeNode,
    size: SizeMetric,
    selected: IgnoredEntries,
    color: bool,
) {
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
                    "{:>10}  {}  {:>4.0}%  {indent}{} ({} {}){}",
                    human_bytes(bytes),
                    bar(share, color),
                    share * 100.0,
                    paint(&node.name, STYLE_DIRECTORY, color),
                    node.files,
                    plural(node.files, "file", "files"),
                    ignored_suffix(node.ignored, size, selected),
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
            "{:>TEXT_SIZE_WIDTH$}  {} {} {}{}",
            human_bytes(pick(size, row.bytes, row.allocated)),
            label_cell(&row.extension, width, STYLE_TYPE, color),
            row.files,
            plural(row.files, "file", "files"),
            ignored_suffix(row.ignored, size, selected),
        );
    }
}

/// What a section dropped, or nothing when it dropped nothing.
///
/// Lives in the header rather than after the rows because a footer is lost to `head`,
/// which is exactly where a reader most needs telling — `fdu --view largest | head -5`
/// would otherwise cut off the only notice that 192,851 rows are missing. In a `full`
/// report a header also keeps each bound attached to the section it describes.
///
/// The flag that lifts the bound is named here too: a truncation the caller cannot remove
/// is a limitation wearing a default's clothes.
fn bound_note(section: &Section) -> String {
    let (shown, total) = match section {
        Section::Files { rows, total, .. } => (rows.len(), *total),
        Section::Extensions { rows, total } => (rows.len(), *total),
        Section::Metrics { summary, .. } => (summary.rows.len(), summary.total_rows),
        // A tree marks its dropped children in place, at the depth they were dropped; a
        // summary is one row and cannot be bounded.
        Section::Tree(_) | Section::Summary(_) => (0, 0),
    };
    if shown >= total {
        return String::new();
    }
    format!(
        "  ({} of {}; --limit all for every one)",
        human_count(shown as u64),
        human_count(total as u64)
    )
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

fn render_text_summary(
    out: &mut String,
    row: &SummaryRow,
    size: SizeMetric,
    selected: IgnoredEntries,
) {
    let _ = writeln!(
        out,
        "{:>10}  {} {}, {} {}{}",
        human_bytes(pick(size, row.bytes, row.allocated)),
        row.files,
        plural(row.files, "file", "files"),
        row.dirs,
        plural(row.dirs, "directory", "directories"),
        ignored_suffix(row.ignored, size, selected),
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

/// Render a count with thousands separators, the way every fdu report does.
///
/// Lived in the command line, and `report_format` called *into* it -- so the library
/// depended on its own front end, which is the inverse of the rule that the CLI invents
/// nothing. A crate boundary rejects that outright, which is how it was found.
pub fn human_count(value: u64) -> String {
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

    if format == Format::Text {
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
        Format::Text => unreachable!("text returned above"),
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
        emit_scalar(sink, Scalar::U64(change.allocated.expect("presence checked")))
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
        Format::Text => render_cache_status_text(statuses, scope),
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
fn render_cache_status_text(statuses: &[crate::CacheStatus], scope: crate::CacheScope) -> String {
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
                    "{}  {} entries, {} metadata bytes, {content_bytes} {}content bytes  {}",
                    status.path.display(),
                    info.entries,
                    status.bytes,
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
                    "{}  stale ({why}), {} metadata bytes, {content_bytes} content bytes",
                    status.path.display(),
                    status.bytes
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
                    "{}  leftover ({what}), {} bytes",
                    status.path.display(),
                    status.bytes
                ));
            }
            CacheState::Unrecognized => {
                unrecognized += 1;
                unrecognized_bytes = unrecognized_bytes.saturating_add(status.bytes);
                lines.push(format!(
                    "{}  unrecognized, {} bytes",
                    status.path.display(),
                    status.bytes
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
            (format!("{stale} stale snapshots"), "them")
        };
        let remedy = match scope {
            CacheScope::Root => format!("fdu --cache-clear PATH removes {object}"),
            CacheScope::All if current == 0 => format!("fdu --cache-clear=all removes {object}"),
            CacheScope::All => {
                format!("fdu --cache-clear=all removes {object}, along with every current snapshot")
            }
        };
        lines.push(format!(
            "{subject} ({stale_bytes} bytes) cannot be served by this build; {remedy}."
        ));
    }
    if leftover > 0 {
        // Named as fdu's own, because they are: calling them foreign would tell the user
        // to leave fdu's debris alone. `=all` is the scope that reclaims them; a root's
        // clear reaches only the one path that root's snapshot occupies.
        let (subject, predicate, object) = if leftover == 1 {
            ("1 leftover file".to_string(), "is", "it")
        } else {
            (format!("{leftover} leftover files"), "are", "them")
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
            "{subject} ({leftover_bytes} bytes) {predicate} fdu's own, left by an interrupted \
             write; fdu --cache-clear=all reclaims {object}{caveat}."
        ));
    }
    if unrecognized > 0 {
        let (subject, predicate, object) = if unrecognized == 1 {
            ("1 unrecognized file".to_string(), "is not an fdu snapshot", "it")
        } else {
            (format!("{unrecognized} unrecognized files"), "are not fdu snapshots", "them")
        };
        lines.push(format!(
            "{subject} ({unrecognized_bytes} bytes) {predicate}, so fdu leaves {object} in place."
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;
    use crate::engine_contract::{Attrs, Observation, Op, ScanScope};
    use crate::query::{Bound, Query, Request, Selection};
    use std::ffi::OsStr;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
            "a.fdu  stale (older snapshot format 2), 10 metadata bytes, 5 content bytes\n\
             b.fdu  stale (newer snapshot format 99), 20 metadata bytes, 5 content bytes\n\
             c.fdu  stale (written by another fdu version), 30 metadata bytes, 5 content bytes\n\
             d.fdu  stale (unreadable by this build), 40 metadata bytes, 5 content bytes\n\
             notes.txt  unrecognized, 14 bytes\n\
             4 stale snapshots (120 bytes) cannot be served by this build; \
             fdu --cache-clear=all removes them.\n\
             1 unrecognized file (14 bytes) is not an fdu snapshot, so fdu leaves it in place."
        );

        // fdu's own debris is named as fdu's, so a reader is not told to leave it alone.
        let leftovers = [
            cache_file(
                ".g.fdu.tmp.1.2.3",
                60,
                CacheState::Leftover(LeftoverKind::StagingTemporary),
            ),
            cache_file("h.fdu.content", 70, CacheState::Leftover(LeftoverKind::OrphanedContent)),
        ];
        assert_eq!(
            render_cache_status(&leftovers, CacheScope::All, Format::Text),
            ".g.fdu.tmp.1.2.3  leftover (staging temporary), 60 bytes\n\
             h.fdu.content  leftover (orphaned content sidecar), 70 bytes\n\
             2 leftover files (130 bytes) are fdu's own, left by an interrupted write; \
             fdu --cache-clear=all reclaims them, though a staging file waits until it is \
             too old to be a running writer's."
        );
        assert!(render_cache_status(&leftovers[..1], CacheScope::Root, Format::Text).ends_with(
            "1 leftover file (60 bytes) is fdu's own, left by an interrupted write; \
                 fdu --cache-clear=all reclaims it, though a staging file waits until it \
                 is too old to be a running writer's."
        ));
        // With no staging file listed, nothing is held back and the promise is plain: a
        // status that named an exception with no file it could apply to would be noise.
        assert!(render_cache_status(&leftovers[1..], CacheScope::All, Format::Text).ends_with(
            "1 leftover file (70 bytes) is fdu's own, left by an interrupted write; \
                 fdu --cache-clear=all reclaims it."
        ));

        let root = [cache_file("a.fdu", 10, CacheState::Stale(StaleReason::OtherEngine))];
        assert!(render_cache_status(&root, CacheScope::Root, Format::Text).ends_with(
            "1 stale snapshot (15 bytes) cannot be served by this build; \
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
            "e.fdu  3 entries, 50 metadata bytes, 0 content bytes  /tree\n\
             notes.txt  unrecognized, 14 bytes\n\
             notes.txt  unrecognized, 14 bytes\n\
             1 stale snapshot (15 bytes) cannot be served by this build; \
             fdu --cache-clear=all removes it, along with every current snapshot.\n\
             2 unrecognized files (28 bytes) are not fdu snapshots, so fdu leaves them in place."
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
            "{\"schema\": \"fdu.cache/2\"}\n\
             {\"path\": \"a.fdu\", \"bytes\": 10, \"state\": \"stale\", \"stale_reason\": \"older_format\", \"format_version\": 2, \"content\": {\"bytes\": 5, \"state\": \"stale\", \"stale_reason\": \"older_format\", \"format_version\": 4}}\n\
             {\"path\": \"f.fdu\", \"bytes\": 0, \"state\": \"absent\", \"content\": null}\n\
             {\"path\": \"notes.txt\", \"bytes\": 14, \"state\": \"unrecognized\", \"content\": null}\n\
             {\"path\": \".g.fdu.tmp.1.2.3\", \"bytes\": 60, \"state\": \"leftover\", \"leftover_kind\": \"staging_temporary\", \"content\": null}"
        );
        assert_eq!(
            render_cache_status(&[], CacheScope::All, Format::Jsonl),
            "{\"schema\": \"fdu.cache/2\"}"
        );
        assert_eq!(
            render_cache_status(&[], CacheScope::All, Format::Json),
            "{\n  \"schema\": \"fdu.cache/2\",\n  \"caches\": []\n}\n"
        );
        // An empty sequence in both formats: a bare `caches:` is YAML null, and a reader
        // of one schema should not have to tell null from a list it can iterate.
        assert_eq!(
            render_cache_status(&[], CacheScope::All, Format::Yaml),
            "schema: fdu.cache/2\ncaches: []\n"
        );
        assert!(
            render_cache_status(&stale[2..3], CacheScope::All, Format::Json)
                .starts_with("{\n  \"schema\": \"fdu.cache/2\",\n  \"caches\": [\n    {")
        );
        assert!(
            render_cache_status(&stale[2..3], CacheScope::All, Format::Yaml)
                .starts_with("schema: fdu.cache/2\ncaches:\n  -\n    path: c.fdu")
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
                       \"type_rules_fingerprint\": 3, \"reducers_fingerprint\": 4}";
        assert_eq!(
            render_cache_status(std::slice::from_ref(&status), CacheScope::Root, Format::Jsonl),
            format!(
                "{{\"schema\": \"fdu.cache/2\"}}\n\
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
                       type_rules_fingerprint: 3\n          reducers_fingerprint: 4";
        assert_eq!(
            render_cache_status(std::slice::from_ref(&status), CacheScope::Root, Format::Yaml),
            format!(
                "schema: fdu.cache/2\ncaches:\n  -\n    path: e.fdu\n    bytes: 50\n    state: current\n    \
                 root: /tree\n    entries: 3\n    identity:\n      entries:{}\n      \
                 ignore_rules:\n        limits:\n          budget: 10\n          line_limit: null\n    \
                 content:\n      bytes: 9\n      state: current\n      records: 2\n      identity:\n        \
                 entries:{entries}\n        analyze:\n          - lines\n        \
                 options_fingerprint: 5\n        analyzers:\n          -\n            id: content-basic-v1\n            \
                 version: 1\n",
                entries.replace("\n  ", "\n")
            )
        );
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
            "e.fdu  3 entries, 50 metadata bytes, 9 stale content bytes  /tree"
        );
    }

    /// The cache schema is a promise, like the report schema beside it.
    ///
    /// Fails loudly when the string moves, so the field rename this constant was added
    /// for — `recognized` to `state` — cannot happen again without a version to key on.
    #[test]
    fn the_cache_schema_constant_is_the_versioning_promise() {
        assert_eq!(CACHE_SCHEMA, "fdu.cache/2");
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
        assert!(text.contains(&format!("(1 of {full}")), "the header states the bound: {text}");
        assert!(text.contains("--limit all"), "and names the flag that lifts it: {text}");
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

    /// Every view, so a matrix test cannot silently skip one that was added later.
    const ALL_TEST_VIEWS: [ViewSpec; 10] = [
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
        assert!(text.starts_with("Percentage column: code lines\n"), "{text}");

        let Section::Metrics { summary, .. } = &mut languages.sections[0] else {
            unreachable!("languages should stay a metric section");
        };
        summary.share_metric = ShareMetric::AllocatedBytes;
        let text = render(&languages, Format::Text, false);
        assert!(!text.contains("Percentage column:"), "{text}");

        let mut documents = fixture(&[ViewSpec::Documents]);
        let Section::Metrics { summary, .. } = &mut documents.sections[0] else {
            panic!("documents should be a metric section");
        };
        summary.share_metric = ShareMetric::DocumentWords;
        let text = render(&documents, Format::Text, false);
        assert!(text.starts_with("Percentage column: document words\n"), "{text}");
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

        struct Fails;
        impl std::io::Write for Fails {
            fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("closed"))
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let error = write(&report, Format::Json, false, &mut Fails).expect_err("writer fails");
        assert_eq!(error.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn streaming_tree_walk_handles_many_siblings_without_collecting_output() {
        let mut report = fixture(&[ViewSpec::Tree]);
        let Section::Tree(root) = &mut report.sections[0] else {
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
                    selection: Selection { depth: Some(Bound::All), ..Selection::default() },
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
        assert!(json.contains("\"schema\": \"fdu.report/7\""));
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
        assert_eq!(REPORT_SCHEMA, "fdu.report/7");
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
             \"applied\": 1, \"refused\": 1, \"refusals\": [{{\"path\": {}, \"reason\": \
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
            "ignore_rules:\n  limits: {{budget: 4194304, line_limit: 16384}}\n  applied: 1\n  \
             refused: 1\n  refusals:\n    -\n      path: {}\n      reason: line_limit\n",
            yaml_scalar(&refused)
        );
        assert!(yaml.contains(&expected), "{yaml}");

        let flags = Query { axes: &crate::query::AxisNames::FLAGS, ..query.clone() };
        let note = "note: 1 .gitignore file not applied (1 with a line over the 16 KiB line \
                    limit), so ignored shares under vendor are not exact; sizes are. To apply \
                    them, raise --gitignore-line-limit above 16 KiB, or set it to all";
        let text = render(
            &report(
                &observed,
                &crate::test_support::read_of(&observed, flags.clone()),
                &provenance,
            )
            .expect("report"),
            Format::Text,
            false,
        );
        assert!(text.ends_with(&format!("{note}\n")), "{text}");
        let fields =
            report(&observed, &crate::test_support::read_of(&observed, query.clone()), &provenance)
                .expect("report");
        assert_eq!(fields.notes, [note.replace("--gitignore-line-limit", "control_line_limit")]);
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
                "     164 B  2 files, 2 directories (128 B ignored)\n",
                "\n",
                "TREE\n",
                "     164 B  ██████████   100%  . (2 files) (128 B ignored)\n",
                "     128 B  ████████░░    78%    dist (1 file) (128 B ignored)\n",
                "      36 B  ██░░░░░░░░    22%    src (1 file)\n",
                "\n",
                "EXTENSIONS\n",
                "     128 B  .gz          1 file (128 B ignored)\n",
                "      36 B  .rs          1 file\n",
                "\n",
                "FILES\n",
                "dist\n",
                "dist/a.gz\n",
                "src\n",
                "src/b.rs\n",
            )
            .replace('/', std::path::MAIN_SEPARATOR_STR)
        );
        // A share of ignored directories alone holds no bytes, so text says nothing of it.
        let dirs_only = IgnoredTally { files: 0, dirs: 1, bytes: 0, allocated: 0 };
        assert_eq!(
            ignored_suffix(Some(dirs_only), SizeMetric::Apparent, IgnoredEntries::Include),
            ""
        );
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
            "\"name\": \"src\", \"path\": \"src\", \"kind\": \"dir\", \"bytes\": 36, \
             \"allocated\": 512, \"files\": 1, \"dirs\": 0, \"ignored\": {\"files\": 0, \
             \"dirs\": 0, \"bytes\": 0, \"allocated\": 0}, ",
            "{\"extension\": \".gz\", \"files\": 1, \"bytes\": 128, \"allocated\": 512, \
             \"ignored\": {\"files\": 1, \"bytes\": 128, \"allocated\": 512}}",
            "\"kind\": \"dir\", \"bytes\": 0, \"allocated\": 0, \"mtime_ns\": 0, \"ignored\": true}",
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
        assert!(metadata.contains("\"schema\": \"fdu.report/7\""));
        assert!(metadata.contains("\"analysis\": null"));

        let metrics = render(&fixture(&[ViewSpec::Types]), Format::Json, false);
        assert!(metrics.contains("\"schema\": \"fdu.report/7\""));
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

    const DEEP_RENDER_CHILD_ENV: &str = "FDU_DEEP_RENDER_CHILD";
    const DEEP_RENDER_DEPTH: usize = 1_024;
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

        let output = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", DEEP_RENDER_TEST_PATH, "--nocapture"])
            .env(DEEP_RENDER_CHILD_ENV, "1")
            .output()
            .expect("run deep-render child");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "deep renderer failed in child process\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
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
                    selection: Selection { depth: Some(Bound::All), ..Selection::default() },
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
                let report = report(
                    &index,
                    &crate::test_support::read_of(&index, query.clone()),
                    &provenance,
                )
                .expect("report");
                for format in [Format::Text, Format::Json, Format::Jsonl, Format::Yaml] {
                    let rendered = render(&report, format, false);
                    assert!(!rendered.is_empty(), "{format:?} rendered nothing for a deep tree");
                    if format != Format::Text {
                        let mut streamed = Vec::new();
                        write(&report, format, false, &mut streamed).expect("stream deep report");
                        assert_eq!(streamed, rendered.as_bytes(), "{format:?} bytes differ");
                    }
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

        // Built directly rather than through the command line's argument struct: what is
        // under test is that the renderer preserves a non-UTF-8 name's raw identity, and
        // routing that through argument parsing tied a renderer test to a front end.
        let query = crate::query::Query {
            views: vec![ViewSpec::Files],
            selection: Selection {
                depth: Some(crate::query::Bound::All),
                limit: Some(crate::query::Bound::All),
                ..Selection::default()
            },
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
                 \"ignored\": false}}"
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
}
