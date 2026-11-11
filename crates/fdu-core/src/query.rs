//! Queries over a built index: what to select, which roll-ups to report, and the value
//! grammars both are written in.
//!
//! The module is deliberately free of filesystem access and of the `cli` feature. A query
//! is a pure function of an [`Index`](crate::Index) and a request, so the CLI, the Rust
//! API, and the Python bindings all compose the same types rather than reimplementing
//! selection three times — and so a report can never quietly become a producer of state.
//! Two reads, each storing nothing: [`Roots::resolve`], which validates a request's roots
//! before anything is scanned, as a scan would on reaching each one, and
//! [`Delivery::for_root`], which resolves a root to name its snapshot in a cache
//! directory.

mod query_ages;
mod query_glob;
mod query_report;
mod query_request;
mod query_selection;
mod query_status;
mod query_subtrees;
mod query_values;
mod query_volumes;

pub use crate::engine_contract::{TimeBasis, Times};
pub use query_ages::{AgeBand, AgeProfile, AgeTally};
pub use query_glob::Pattern;
pub use query_report::{
    AxisNames, CodeLanguageRow, CodeOverview, CodeTally, ContentReportMetadata, EntryTotals,
    FileRow, IgnoredSize, IgnoredTally, MetricGroup, MetricRow, MetricShare, MetricSummary, Pages,
    Query, Report, ReportMetricValues, ReportSource, RootTree, RootTrees, Section,
    SelectionCoverage, ShareMetric, SummaryRow, TreeDisplayLimits, TreeNode, TreeOmission,
    TreeOmissionReason, TreeRemainder, TreeTotal, TypeRow, ViewSpec, document_words, pages, report,
    report_roots, report_roots_with_capacity, report_with_capacity,
};
pub(crate) use query_report::{
    SummaryPart, read_indexes, report_in, report_summary, summary_totals_fit,
};
pub use query_request::{
    Basis, BasisHolder, Delivery, NamedRoot, ReadSpec, Request, RequestDefaults, RequestError,
    RequestSpec, Roots, RootsRequest, Scope, ScopeAxis, WatchDelivery, Workers, bound_nanos,
    labelled_path, parse_bound, parse_cache_policy, parse_kind, parse_kinds, parse_size_metric,
    parse_sort, parse_time_basis,
};
pub(crate) use query_request::{Rejection, RootAliases};
pub(crate) use query_selection::NameIdentity;
pub use query_selection::{
    Bound, Candidate, EntrySelection, IgnoredEntries, Selection, ShareThreshold, SizeMetric,
    SortKey, TimeWindow,
};
pub use query_status::{ReportProvenance, StatusIssue, TierProvenance, TierState, TreeStatus};
#[cfg(test)]
pub(crate) use query_subtrees::{assert_maintained_activity, assert_same_as_cold_walk};
pub(crate) use query_values::{
    MONTH_SECONDS, YEAR_SECONDS, format_rfc3339_nanos, with_rfc3339_nanos,
};
pub use query_values::{
    format_rfc3339, parse_control_budget, parse_control_line_limit, parse_size, parse_when,
    system_time_to_nanos,
};
pub use query_volumes::{
    VolumeCounted, VolumeEntry, VolumeRow, VolumesObservation, VolumesSection, observe_volumes,
    volume_entries,
};
pub(crate) use query_volumes::{observe_entries, root_devices};
