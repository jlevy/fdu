//! Shared diagnostic design system; rendering data never writes diagnostics.
//!
//! A completed report ends with short, self-contained lines in category order:
//! `note:` explains coverage or interpretation; `warn:` identifies an operational
//! failure or an answer nothing verified; `tip:` gives an actionable existing option;
//! `perf:` summarizes run cost. No section headings or empty categories. The engine
//! supplies notes, tips, and the warnings that describe the answer itself as distinct
//! values; frontends insert operational warnings and human-report telemetry.
//!
//! A stale-ok answer is the engine's one warning: the snapshot answered without the
//! filesystem being consulted, and a reader who misses that takes old numbers for current
//! ones. It is a warning rather than a note so that quiet output keeps it, and it names the
//! surface's own option for a fresh answer (`--stale-ok` or `stale_ok`).
//!
//! The rendered result carries no explanation of its own: what a percentage measures,
//! what the code overview analyzed, what a display limit hid, and how to see more are all
//! lines here, after the result. Keep each as short as its facts allow and merge related
//! ones: one note for what totals include, one listing every display limit that hid
//! something, and one runnable tip that lifts them all.
//!
//! Collect remedies from actual omissions across all directories and views, then emit
//! each once in stable order. A display omission changes neither totals nor scan work.
//! Never sum omitted bytes across views: their populations can overlap.
//!
//! Preserve debugging context: state the affected path, cause, count, or configured
//! limit where available. Bound long detail lists and report how many were left out.
//! Suggest a detail flag only when that flag exists and exposes retained information.
//! Keep the factual limitation separate from its suggested remedy.
//!
//! Frontends route these lines to stderr after result stdout, including for structured
//! output. Notes, tips, and performance are gray; warnings yellow without bold; fatal
//! errors red and bold. Color follows the receiving stream's terminal/color settings.
//! Quiet frontends suppress notes, tips, and performance, while retaining warnings --
//! the engine's [`report_warnings`] among them -- and fatal errors. The collector remains a
//! pure description of report facts.
//! See `docs/project/architecture/fdu-output-design.md` for examples and test coverage.

use crate::content::CoverageReason;
use crate::query::{
    IgnoredEntries, Report, ReportSource, Section, SizeMetric, TreeOmissionReason, ViewSpec,
};

/// Human diagnostics retain categories rather than parsing rendered text.
#[derive(Clone, Debug, Default)]
pub struct DiagnosticLines {
    /// Facts and limitations, in display order.
    pub notes: Vec<String>,
    /// Actionable suggestions, each emitted at most once.
    pub tips: Vec<String>,
}

impl DiagnosticLines {
    /// Facts followed by suggestions when no operational warnings intervene.
    ///
    /// Carries no warnings, the report's own included: [`diagnostics`] adds
    /// [`report_warnings`] between the two.
    pub fn into_lines(mut self) -> Vec<String> {
        self.notes.extend(self.tips);
        self.notes
    }
}

/// Categorized messages for frontends that insert operational warnings before tips.
pub fn diagnostic_lines(report: &Report) -> DiagnosticLines {
    let (notes, tips) = collect(report);
    DiagnosticLines { notes, tips }
}

/// Factual notes, the report's own warnings, then actionable tips, for a frontend's
/// diagnostic stream.
pub fn diagnostics(report: &Report) -> Vec<String> {
    with_warnings(report, diagnostic_lines(report))
}

/// Conditions of the answer itself that a reader must not miss, which quiet output keeps.
///
/// Distinct from notes because `--quiet` suppresses notes, and distinct from a frontend's
/// operational warnings (a failed save, an unreadable path) because the engine owns this
/// fact: every surface prints the same line, naming its own option for a fresh answer.
/// Today that is one line, for an answer the snapshot gave without the filesystem being
/// consulted ([`ReportSource::CacheOnly`], which only a
/// [`Delivery::stale_ok`](crate::query::Delivery::stale_ok) request produces). A verified
/// answer, cold or warm, carries none.
pub fn report_warnings(report: &Report) -> Vec<String> {
    let mut warnings = Vec::new();
    if report.provenance.source == ReportSource::CacheOnly {
        warnings.push(format!(
            "warn: stale answer: served from the snapshot without filesystem verification; \
             drop {} for a fresh answer",
            report.axes.stale_ok
        ));
    }
    warnings
}

/// Notes, then the report's warnings, then tips: the category order with no frontend
/// warnings between them.
pub(super) fn with_warnings(report: &Report, lines: DiagnosticLines) -> Vec<String> {
    let DiagnosticLines { mut notes, tips } = lines;
    notes.extend(report_warnings(report));
    notes.extend(tips);
    notes
}

/// Facts about coverage and presentation, without suggestions or run telemetry.
pub fn report_notes(report: &Report) -> Vec<String> {
    diagnostic_lines(report).notes
}

/// Distinct actionable suggestions in stable order, using the caller's vocabulary.
pub fn report_tips(report: &Report) -> Vec<String> {
    diagnostic_lines(report).tips
}

fn unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn collect(report: &Report) -> (Vec<String>, Vec<String>) {
    let mut notes = Vec::new();
    let mut tips = Vec::new();
    let mut reasons = Vec::new();
    let mut tree_remainder_shown = false;
    let mut ignored_subset_shown = false;
    let mut code_rows_hidden = false;
    // Every display limit that hid something, as one list for one note.
    let mut limits_hit = Vec::new();
    let mut share_labels: Vec<(&'static str, &'static str)> = Vec::new();
    let mut coverage = Vec::new();
    let mut zero = false;
    let mut reason = |why| {
        if !reasons.contains(&why) {
            reasons.push(why);
        }
    };
    let single = report.sections.len() == 1;
    for section in &report.sections {
        if let Some((shown, total)) = super::bounded_rows(section) {
            reason(TreeOmissionReason::Rows);
            // A multi-view report states the bound on each section header.
            if single {
                unique(
                    &mut limits_hit,
                    format!(
                        "{} of {} rows shown",
                        super::human_count(shown as u64),
                        super::human_count(total as u64)
                    ),
                );
            }
        }
        match section {
            Section::Tree { root, omissions, limits, .. } => {
                tree_remainder_shown |=
                    crate::query::TreeRemainder::from_tree(root.as_deref(), omissions).is_some();
                let mut stack = root.iter().map(AsRef::as_ref).collect::<Vec<_>>();
                if let Some(root) = root {
                    let size = match report.size {
                        SizeMetric::Apparent => root.bytes,
                        SizeMetric::Allocated => root.allocated,
                    };
                    if size == 0 && !limits.min_share.admits(0, 1) {
                        zero = true;
                        reason(TreeOmissionReason::Share);
                    }
                }
                let mut local_reasons: Vec<_> = omissions.iter().map(|o| o.reason).collect();
                while let Some(node) = stack.pop() {
                    ignored_subset_shown |= node.ignored.is_some_and(|share| share.files > 0);
                    local_reasons.extend(node.omissions.iter().map(|omission| omission.reason));
                    stack.extend(node.children.iter());
                }
                for (why, description) in [
                    (
                        TreeOmissionReason::Share,
                        format!("below {} of root", limits.min_share.label()),
                    ),
                    (TreeOmissionReason::Depth, format!("depth {}", bound_label(limits.depth))),
                    (
                        TreeOmissionReason::Breadth,
                        format!("breadth {}", bound_label(limits.breadth)),
                    ),
                    (TreeOmissionReason::Rows, format!("row limit {}", bound_label(limits.rows))),
                ] {
                    if local_reasons.contains(&why) {
                        reason(why);
                        unique(&mut limits_hit, description);
                    }
                }
            }
            Section::Extensions { rows, share_omitted, .. } => {
                ignored_subset_shown |=
                    rows.iter().any(|row| row.ignored.is_some_and(|share| share.files > 0));
                share_hidden(&mut limits_hit, &mut reason, *share_omitted, section, single);
            }
            Section::Summary(row) => {
                ignored_subset_shown |= row.ignored.is_some_and(|share| share.files > 0);
            }
            Section::Metrics { view, summary } => {
                share_hidden(&mut limits_hit, &mut reason, summary.share_omitted, section, single);
                if let Some(label) = super::share_metric_label(summary.share_metric) {
                    share_labels.push((label, super::view_header(*view)));
                }
            }
            Section::Code(overview) => {
                code_rows_hidden |= overview.share_omitted > 0
                    || overview.languages.len() < overview.total_languages;
                share_hidden(&mut limits_hit, &mut reason, overview.share_omitted, section, single);
                // Alone, the table's `Code lines` heading names its share; beside other
                // views, the note must say which section each denominator belongs to.
                if !single {
                    share_labels.push(("code lines", super::view_header(ViewSpec::Code)));
                }
                code_coverage(&mut coverage, overview);
            }
            Section::Files { .. } => {}
        }
    }
    let mut includes = Vec::new();
    if report.ignored_entries == IgnoredEntries::Include && ignored_subset_shown {
        includes.push("gitignored sizes");
    }
    if tree_remainder_shown {
        includes.push("descendants");
    }
    if code_rows_hidden {
        includes.push("hidden languages");
    }
    if !includes.is_empty() {
        notes.push(format!("note: totals include {}", includes.join(" and ")));
    }
    // A single view needs no section name; beside other views, every non-byte denominator
    // names its section, so no table borrows another's (fdu-gda7 review A1).
    match share_labels.as_slice() {
        [] => {}
        [(label, _)] if single => notes.push(format!("note: percentages are shares of {label}")),
        labels => notes.push(format!(
            "note: percentages are shares of {}",
            labels
                .iter()
                .map(|(label, view)| format!("{label} ({view})"))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
    if let Some(metric) = report.sort_metric {
        notes.push(format!("note: ranked by {}", metric.replace('_', " ")));
    }
    notes.extend(coverage);
    if !limits_hit.is_empty() {
        notes.push(format!("note: display limits: {}", limits_hit.join(", ")));
    }
    if zero {
        notes.push("note: root size is zero, so shares are undefined".to_owned());
    }
    for note in &report.notes {
        unique(&mut notes, note.clone());
    }
    // One runnable tip lifts every bound that hid something, in a fixed order that is
    // independent of tree traversal and requested view order.
    let flags = [
        (TreeOmissionReason::Share, report.axes.min_share, "0%"),
        (TreeOmissionReason::Depth, report.axes.depth, "all"),
        (TreeOmissionReason::Breadth, report.axes.breadth, "all"),
        (TreeOmissionReason::Rows, report.axes.limit, "all"),
    ]
    .into_iter()
    .filter(|(why, ..)| reasons.contains(why))
    .map(|(_, axis, value)| format!("{axis}={value}"))
    .collect::<Vec<_>>();
    if !flags.is_empty() {
        tips.push(format!("tip: show more: {}", flags.join(report.axes.setting_separator)));
    }
    for tip in &report.tips {
        unique(&mut tips, tip.clone());
    }
    (notes, tips)
}

/// Record rows a share floor hid in a grouped section, naming the section beside others
/// so equal counts in different views stay distinct.
fn share_hidden(
    limits_hit: &mut Vec<String>,
    reason: &mut impl FnMut(TreeOmissionReason),
    omitted: usize,
    section: &Section,
    single: bool,
) {
    if omitted > 0 {
        reason(TreeOmissionReason::Share);
        let rows = format!(
            "{} {} below min share",
            super::human_count(omitted as u64),
            if omitted == 1 { "row" } else { "rows" }
        );
        unique(
            limits_hit,
            if single { rows } else { format!("{rows} in {}", super::view_header(section.view())) },
        );
    }
}

/// What the code overview analyzed and what it could not, as notes.
fn code_coverage(notes: &mut Vec<String>, overview: &crate::query::CodeOverview) {
    let mut analyzed = format!(
        "note: {} {} analyzed",
        super::human_count(overview.analyzed_languages),
        if overview.analyzed_languages == 1 { "language" } else { "languages" }
    );
    match overview.population {
        IgnoredEntries::Include => {}
        IgnoredEntries::Exclude => analyzed.push_str(", gitignored files excluded"),
        IgnoredEntries::Only => analyzed.push_str(", gitignored files only"),
    }
    unique(notes, analyzed);
    let selected = &overview.selected;
    let mut skipped = selected
        .coverage
        .iter()
        .filter(|(reason, _)| **reason != CoverageReason::Analyzed)
        .map(|(reason, files)| {
            format!("{} {}", super::human_count(*files), super::human_coverage_label(*reason))
        })
        .collect::<Vec<_>>();
    if selected.missing_records > 0 {
        skipped.push(format!(
            "{} without analyzer records",
            super::human_count(selected.missing_records)
        ));
    }
    if !skipped.is_empty() {
        unique(notes, format!("note: not analyzed: {}", skipped.join(", ")));
    }
    if overview.unknown.source_files > 0 {
        unique(
            notes,
            format!(
                "note: {} source files with unknown ignore classification",
                super::human_count(overview.unknown.source_files)
            ),
        );
    }
    if overview.unclassified_files > 0 {
        unique(
            notes,
            format!(
                "note: {} files with unclassified type",
                super::human_count(overview.unclassified_files)
            ),
        );
    }
}

fn bound_label(bound: crate::query::Bound) -> String {
    match bound {
        crate::query::Bound::All => "all".to_owned(),
        crate::query::Bound::Limit(value) => super::human_count_u128(value as u128),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displayed_bound_groups_large_counts() {
        assert_eq!(bound_label(crate::query::Bound::Limit(999)), "999");
        assert_eq!(bound_label(crate::query::Bound::Limit(1000)), "1,000");
    }
}
