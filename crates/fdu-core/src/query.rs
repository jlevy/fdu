//! Queries over a built index: what to select, which roll-ups to report, and the value
//! grammars both are written in.
//!
//! The module is deliberately free of filesystem access and of the `cli` feature. A query
//! is a pure function of an [`Index`](crate::Index) and a request, so the CLI, the Rust
//! API, and the Python bindings all compose the same types rather than reimplementing
//! selection three times — and so a report can never quietly become a producer of state.

mod query_glob;
mod query_report;
mod query_selection;
mod query_values;

pub use query_glob::Pattern;
pub use query_report::{
    AxisNames, ContentReportMetadata, FileRow, IgnoredTally, MetricGroup, MetricRow, MetricShare,
    MetricSummary, Provenance, Query, Report, ReportSource, Section, ShareMetric, SummaryRow,
    TreeNode, TypeRow, ViewSpec, document_words, report,
};
pub(crate) use query_report::{
    display_notes, forget_ignore_classification, report_in, report_summary,
};
pub(crate) use query_selection::NameIdentity;
pub use query_selection::{
    Bound, Candidate, EntrySelection, IgnoredEntries, ModifiedWindow, Selection, SizeMetric,
    SortKey,
};
pub use query_values::{
    format_rfc3339, parse_control_budget, parse_size, parse_when, system_time_to_nanos,
};
