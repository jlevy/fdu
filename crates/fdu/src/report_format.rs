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

use crate::query::{
    FileRow, Report, ReportSource, Section, SummaryRow, TreeNode, TypeRow, ViewSpec, format_rfc3339,
};
use crate::types::{EntryKind, Freshness};

/// Machine-output schema identity.
///
/// Any change to a field's name, type, or meaning bumps this, and a golden test fails if
/// the schema moves without it — the versioning is the promise, not the intention.
pub const REPORT_SCHEMA: &str = "fdu.report/1";

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
pub fn render(report: &Report, format: Format) -> String {
    match format {
        Format::Text => render_text(report),
        Format::Json => render_json(report),
        Format::Jsonl => render_jsonl(report),
        Format::Yaml => render_yaml(report),
    }
}

// ---- text ----

/// Render the human-facing form.
fn render_text(report: &Report) -> String {
    let mut out = String::new();
    for (index, section) in report.sections.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        match section {
            Section::Tree(root) => render_text_tree(&mut out, root),
            Section::Types(rows) => render_text_types(&mut out, rows),
            // A flat listing prints one path per line and nothing else, so it pipes
            // straight into xargs and diffs cleanly against another run.
            Section::Files(rows) => {
                for row in rows {
                    let _ = writeln!(out, "{}", row.path.display());
                }
            }
            Section::Summary(row) => render_text_summary(&mut out, row),
        }
    }
    out
}

/// Render a tree section as an indented outline.
fn render_text_tree(out: &mut String, root: &TreeNode) {
    fn walk(out: &mut String, node: &TreeNode, depth: usize) {
        let indent = "  ".repeat(depth);
        let _ = writeln!(
            out,
            "{indent}{:>10}  {} ({} {})",
            human_bytes(node.bytes),
            node.name,
            node.files,
            plural(node.files, "file", "files"),
        );
        for child in &node.children {
            walk(out, child, depth + 1);
        }
        if node.truncated {
            let _ = writeln!(out, "{indent}  …");
        }
    }
    walk(out, root, 0);
}

/// Render a types section as aligned rows.
fn render_text_types(out: &mut String, rows: &[TypeRow]) {
    for row in rows {
        let _ = writeln!(
            out,
            "{:>10}  {:<12} {} {}",
            human_bytes(row.bytes),
            row.extension,
            row.files,
            plural(row.files, "file", "files"),
        );
    }
}

/// Render a summary section as one line.
fn render_text_summary(out: &mut String, row: &SummaryRow) {
    let _ = writeln!(
        out,
        "{:>10}  {} {}, {} {}",
        human_bytes(row.bytes),
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
    let _ = write!(out, "  \"schema\": {}", quote(REPORT_SCHEMA));
    let _ = write!(out, ",\n  \"generator\": {}", quote(&generator()));
    let _ = write!(out, ",\n  \"root\": {}", quote(&report.root.to_string_lossy()));
    let _ = write!(
        out,
        ",\n  \"scan_started_at\": {}",
        report.scan_started_at.map_or_else(|| "null".to_string(), |at| quote(&format_rfc3339(at)))
    );
    let _ = write!(out, ",\n  \"generated_at\": {}", quote(&format_rfc3339(report.generated_at)));
    let _ = write!(out, ",\n  \"source\": {}", quote(source_label(report.source)));
    let _ = write!(out, ",\n  \"freshness\": {}", quote(freshness_label(report.freshness)));
    let _ = write!(out, ",\n  \"complete\": {}", report.complete);
}

/// One section as a JSON object.
fn section_json(section: &Section, _indent: usize) -> String {
    let mut out = String::new();
    let _ = write!(out, "{{\n  \"view\": {},\n  ", quote(view_label(section.view())));
    match section {
        Section::Tree(root) => {
            let _ = write!(out, "\"tree\": {}", indent(&tree_json(root), 2).trim_start());
        }
        Section::Types(rows) => {
            let _ = write!(out, "\"types\": [");
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
        Section::Files(rows) => {
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

/// One file row as a JSON object.
fn file_json(row: &FileRow) -> String {
    format!(
        "{{\"path\": {}, \"kind\": {}, \"bytes\": {}, \"allocated\": {}, \"mtime_ns\": {}}}",
        quote(&row.path.to_string_lossy()),
        quote(kind_label(row.kind)),
        row.bytes,
        row.allocated,
        row.mtime_ns
    )
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
fn tree_json(node: &TreeNode) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "{{\"name\": {}, \"path\": {}, \"kind\": {}, \"bytes\": {}, \"allocated\": {}, \"files\": {}, \"dirs\": {}, \"newest_mtime_ns\": {}, \"truncated\": {}",
        quote(&node.name),
        quote(&node.path.to_string_lossy()),
        quote(kind_label(node.kind)),
        node.bytes,
        node.allocated,
        node.files,
        node.dirs,
        node.newest_mtime_ns.map_or_else(|| "null".to_string(), |value| value.to_string()),
        node.truncated
    );
    if node.children.is_empty() {
        out.push_str(", \"children\": []}");
    } else {
        out.push_str(", \"children\": [");
        for (index, child) in node.children.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            let _ = write!(out, "\n{}", indent(&tree_json(child), 2));
        }
        out.push_str("\n]}");
    }
    out
}

// ---- yaml ----

/// Render the report as YAML.
fn render_yaml(report: &Report) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "schema: {}", yaml_scalar(REPORT_SCHEMA));
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
    out.push_str("reports:\n");

    for section in &report.sections {
        let _ = writeln!(out, "  - view: {}", yaml_scalar(view_label(section.view())));
        match section {
            Section::Tree(root) => {
                out.push_str("    tree:\n");
                yaml_tree(&mut out, root, 6);
            }
            Section::Types(rows) => {
                if rows.is_empty() {
                    out.push_str("    types: []\n");
                } else {
                    out.push_str("    types:\n");
                    for row in rows {
                        let _ = writeln!(out, "      - extension: {}", yaml_scalar(&row.extension));
                        let _ = writeln!(out, "        files: {}", row.files);
                        let _ = writeln!(out, "        bytes: {}", row.bytes);
                        let _ = writeln!(out, "        allocated: {}", row.allocated);
                    }
                }
            }
            Section::Files(rows) => {
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

/// Render a tree node as YAML at a given indent.
fn yaml_tree(out: &mut String, node: &TreeNode, pad: usize) {
    let indent = " ".repeat(pad);
    let _ = writeln!(out, "{indent}name: {}", yaml_scalar(&node.name));
    let _ = writeln!(out, "{indent}path: {}", yaml_scalar(&node.path.to_string_lossy()));
    let _ = writeln!(out, "{indent}kind: {}", yaml_scalar(kind_label(node.kind)));
    let _ = writeln!(out, "{indent}bytes: {}", node.bytes);
    let _ = writeln!(out, "{indent}allocated: {}", node.allocated);
    let _ = writeln!(out, "{indent}files: {}", node.files);
    let _ = writeln!(out, "{indent}dirs: {}", node.dirs);
    let _ = writeln!(out, "{indent}newest_mtime_ns: {}", yaml_option(node.newest_mtime_ns));
    let _ = writeln!(out, "{indent}truncated: {}", node.truncated);
    if node.children.is_empty() {
        let _ = writeln!(out, "{indent}children: []");
    } else {
        let _ = writeln!(out, "{indent}children:");
        for child in &node.children {
            let _ = writeln!(out, "{indent}  - name: {}", yaml_scalar(&child.name));
            let mut nested = String::new();
            yaml_tree(&mut nested, child, 0);
            // The name line is already written as the sequence marker, so skip its repeat.
            for line in nested.lines().skip(1) {
                let _ = writeln!(out, "{indent}    {line}");
            }
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
fn collapse(block: &str) -> String {
    block.lines().map(str::trim).collect::<Vec<_>>().join(" ").replace("{ ", "{").replace(" }", "}")
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

/// Stable wire label for a view.
fn view_label(view: ViewSpec) -> &'static str {
    match view {
        ViewSpec::Tree => "tree",
        ViewSpec::Types => "types",
        ViewSpec::Files => "files",
        ViewSpec::Summary => "summary",
    }
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

/// Pick the singular or plural noun for a count.
fn plural<'a>(count: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

/// Render a byte count at human scale.
fn human_bytes(bytes: u64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;
    use crate::query::{Provenance, Query, Selection, report};
    use crate::types::{Attrs, Observation, Op, ScanScope};
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
            &Query { selection: Selection::default(), views: views.to_vec() },
            &Provenance {
                scan_started_at: Some(UNIX_EPOCH + Duration::from_secs(1_786_386_151)),
                generated_at: UNIX_EPOCH + Duration::from_secs(1_786_386_152),
                source: ReportSource::ColdScan,
                complete: true,
            },
        )
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
        for view in [ViewSpec::Tree, ViewSpec::Types, ViewSpec::Files, ViewSpec::Summary] {
            let report = fixture(&[view]);
            for format in [Format::Text, Format::Json, Format::Jsonl, Format::Yaml] {
                let rendered = render(&report, format);
                assert!(!rendered.trim().is_empty(), "{view:?} in {format:?} rendered nothing");
            }
        }
    }

    #[test]
    fn json_output_is_well_formed_for_every_view() {
        for view in [ViewSpec::Tree, ViewSpec::Types, ViewSpec::Files, ViewSpec::Summary] {
            let json = render(&fixture(&[view]), Format::Json);
            assert!(is_valid_json(&json), "unbalanced JSON for {view:?}:\n{json}");
        }
        let all = render(
            &fixture(&[ViewSpec::Tree, ViewSpec::Types, ViewSpec::Files, ViewSpec::Summary]),
            Format::Json,
        );
        assert!(is_valid_json(&all), "unbalanced JSON for a multi-view report:\n{all}");
    }

    #[test]
    fn jsonl_emits_one_document_per_line() {
        let rendered = render(&fixture(&[ViewSpec::Types, ViewSpec::Summary]), Format::Jsonl);
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 3, "one envelope plus one line per section");
        for line in &lines {
            assert!(is_valid_json(line), "line is not a JSON document: {line}");
        }
        assert!(lines[0].contains("\"schema\""), "the envelope carries provenance");
    }

    #[test]
    fn machine_output_carries_the_schema_and_provenance() {
        let json = render(&fixture(&[ViewSpec::Summary]), Format::Json);
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
    }

    #[test]
    fn a_files_view_prints_one_path_per_line_and_nothing_else() {
        // The property that makes `fdu --view files | xargs` work.
        let text = render(&fixture(&[ViewSpec::Files]), Format::Text);
        for line in text.lines() {
            assert!(!line.contains(' '), "text files output must be bare paths, got {line:?}");
        }
        assert!(text.lines().any(|line| line == "src/main.rs"), "{text}");
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
}
