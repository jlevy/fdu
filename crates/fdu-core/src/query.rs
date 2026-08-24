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
pub(crate) use query_report::report_summary;
pub use query_report::{
    AxisNames, ContentReportMetadata, FileRow, GroupRow, MetricGroup, MetricRow, MetricShare,
    MetricSummary, Provenance, Query, Remainder, Report, ReportSource, RunFacts, Section,
    ShareMetric, SummaryRow, TreeNode, TypeRow, ViewSpec, document_words, report, report_measured,
};
// One bound vocabulary, defined in the shared contract layer and re-exported here so
// `query::Bound` keeps naming it: the report's depth and row limits and a roll-up's
// extension rows are the same question asked in three places.
pub use crate::engine_contract::Bound;
pub use query_selection::{Candidate, ModifiedWindow, Selection, SizeMetric, SortKey, TagFilter};
pub use query_values::{format_rfc3339, parse_size, parse_when, system_time_to_nanos};
