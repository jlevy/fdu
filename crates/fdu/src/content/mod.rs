//! Optional, versioned file-content analysis.
//!
//! Metadata-only scans allocate none of these structures and open no file content.

mod index;
mod types;

pub use index::{ContentIndex, ContentRollUp, MetricTally};
pub use types::{
    AnalysisApplyOutcome, AnalysisCandidate, AnalysisObservation, AnalysisProfile, AnalysisRequest,
    AnalyzerId, AnalyzerVersion, CODE_SLOC, CONTENT_BASIC, ContentProvenance, CoverageReason,
    FileAnalysis, LogicalWordStats, MARKDOWN_PROSE, MetricSlotId, MetricValues, OptionsFingerprint,
    TEXT_LOGICAL,
};
