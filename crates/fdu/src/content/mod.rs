//! Optional, versioned file-content analysis.
//!
//! Metadata-only scans allocate none of these structures and open no file content.

mod analyze;
mod basic;
mod cache;
mod code;
mod index;
mod types;

pub use analyze::{AnalysisReport, analyze_index};
pub use basic::{BasicAccumulator, TextAdmission};
pub(crate) use cache::is_recognized_content_cache;
pub use cache::{ContentCacheLoad, content_cache_path, load_content_cache, save_content_cache};
pub use code::CodeAccumulator;
pub use index::{ContentIndex, ContentRollUp, MetricTally};
pub use types::{
    AnalysisApplyOutcome, AnalysisCandidate, AnalysisObservation, AnalysisProfile, AnalysisRequest,
    AnalyzerId, AnalyzerVersion, CODE_SLOC, CONTENT_BASIC, ContentProvenance, CoverageReason,
    FileAnalysis, LogicalWordStats, MARKDOWN_PROSE, MetricSlotId, MetricValues, OptionsFingerprint,
    TEXT_LOGICAL,
};
