//! Shared diagnostic design system; rendering data never writes diagnostics.
//!
//! A completed report ends with short, self-contained lines in category order:
//! `note:` explains coverage or interpretation; `warn:` identifies an operational
//! failure; `tip:` gives an actionable existing option; `perf:` summarizes run cost.
//! No section headings or empty categories. The engine supplies notes and tips as
//! distinct values; frontends insert operational warnings and human-report telemetry.
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
//! Quiet frontends suppress notes, tips, and performance, while retaining warnings and
//! fatal errors. The collector remains a pure description of report facts.
//! See `docs/project/architecture/fdu-output-design.md` for examples and test coverage.

use crate::query::{IgnoredEntries, Report, Section, SizeMetric, TreeOmissionReason};

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

/// Factual notes followed by actionable tips, for a frontend's diagnostic stream.
pub fn diagnostics(report: &Report) -> Vec<String> {
    diagnostic_lines(report).into_lines()
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
    let mut tree_omitted = false;
    let mut tree_remainder_shown = false;
    let mut ignored_subset_shown = false;
    let mut code_rows_hidden = false;
    let mut tree_bounds = Vec::new();
    let mut zero = false;
    let mut reason = |why| {
        if !reasons.contains(&why) {
            reasons.push(why);
        }
    };
    for section in &report.sections {
        if super::bounded_rows(section).is_some() {
            reason(TreeOmissionReason::Rows);
        }
        match section {
            Section::Tree { root, omissions, limits, .. } => {
                tree_remainder_shown |=
                    crate::query::TreeRemainder::from_tree(root.as_deref(), omissions).is_some();
                for omission in omissions {
                    tree_omitted = true;
                    reason(omission.reason);
                }
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
                    for omission in &node.omissions {
                        local_reasons.push(omission.reason);
                        tree_omitted = true;
                        reason(omission.reason);
                    }
                    stack.extend(node.children.iter());
                }
                for (why, description) in [
                    (
                        TreeOmissionReason::Share,
                        format!("below {} of selected root", limits.min_share.label()),
                    ),
                    (TreeOmissionReason::Depth, format!("depth {}", bound_label(limits.depth))),
                    (
                        TreeOmissionReason::Breadth,
                        format!("breadth {}", bound_label(limits.breadth)),
                    ),
                    (TreeOmissionReason::Rows, format!("row limit {}", bound_label(limits.rows))),
                ] {
                    if local_reasons.contains(&why) {
                        unique(&mut tree_bounds, description);
                    }
                }
            }
            Section::Extensions { rows, share_omitted, .. } => {
                ignored_subset_shown |=
                    rows.iter().any(|row| row.ignored.is_some_and(|share| share.files > 0));
                if *share_omitted > 0 {
                    reason(TreeOmissionReason::Share);
                }
            }
            Section::Summary(row) => {
                ignored_subset_shown |= row.ignored.is_some_and(|share| share.files > 0);
            }
            Section::Metrics { summary, .. } if summary.share_omitted > 0 => {
                reason(TreeOmissionReason::Share);
            }
            Section::Code(overview) => {
                code_rows_hidden |= overview.share_omitted > 0
                    || overview.languages.len() < overview.total_languages;
                if overview.share_omitted > 0 {
                    reason(TreeOmissionReason::Share);
                }
            }
            _ => {}
        }
    }
    if report.ignored_entries == IgnoredEntries::Include && ignored_subset_shown {
        notes.push("note: gitignored sizes are included in row totals".to_owned());
    }
    if code_rows_hidden {
        notes.push("note: code totals include languages hidden by display limits".to_owned());
    }
    if tree_remainder_shown {
        notes.push("note: more covers unlisted root branches; listed directory totals already include their descendants".to_owned());
    }
    if tree_omitted && !tree_bounds.is_empty() {
        notes.push(format!("note: display limits: {}", tree_bounds.join(", ")));
    }
    if zero {
        notes.push("note: no size denominator: selected root size is zero".to_owned());
    }
    for note in &report.notes {
        unique(&mut notes, note.clone());
    }
    // Fixed category order is independent of tree traversal and requested view order.
    for (why, action, axis, value) in [
        (TreeOmissionReason::Share, "show smaller entries", report.axes.min_share, "0%"),
        (TreeOmissionReason::Depth, "expand deeper", report.axes.depth, "all"),
        (TreeOmissionReason::Breadth, "show more children", report.axes.breadth, "all"),
        (TreeOmissionReason::Rows, "show more rows", report.axes.limit, "all"),
    ] {
        if reasons.contains(&why) {
            tips.push(format!("tip: {action}: {axis}={value}"));
        }
    }
    for tip in &report.tips {
        unique(&mut tips, tip.clone());
    }
    (notes, tips)
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
