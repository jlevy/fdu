//! Optional, versioned file-content analysis.
//!
//! Metadata-only scans allocate none of these structures and open no file content.

mod content_analysis;
mod content_basic_metrics;
mod content_cache;
mod content_code_metrics;
mod content_index;
mod content_markdown_metrics;
mod content_model;

pub use content_analysis::{AnalysisReport, analyze_index};
pub use content_basic_metrics::{BasicAccumulator, TextAdmission};
pub(crate) use content_cache::is_recognized_content_cache;
pub use content_cache::{
    ContentCacheLoad, content_cache_path, load_content_cache, save_content_cache,
};
pub use content_code_metrics::CodeAccumulator;
pub use content_index::{ContentIndex, ContentRollUp, MetricTally};
pub use content_model::{
    AnalysisApplyOutcome, AnalysisCandidate, AnalysisObservation, AnalysisProfile, AnalysisRequest,
    AnalyzerId, AnalyzerVersion, CODE_SLOC, CONTENT_BASIC, ContentProvenance, CoverageReason,
    FileAnalysis, LogicalWordStats, MARKDOWN_PROSE, MetricSlotId, MetricValues, OptionsFingerprint,
    TEXT_LOGICAL,
};
