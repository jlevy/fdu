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

pub use content_analysis::{AnalysisReport, AnalyzerCoverage, analyze_index};
pub use content_basic_metrics::{BasicAccumulator, TextAdmission};
pub use content_cache::{
    ContentCacheLoad, content_cache_path, load_content_cache, save_content_cache,
};
pub(crate) use content_cache::{content_sidecar_bytes, identify_sidecar};
pub use content_code_metrics::CodeAccumulator;
pub use content_index::{AnalyzerTally, ContentIndex, ContentRollUp, MetricTally};
pub(crate) use content_model::{
    AnalysisApplyOutcome, AnalysisCandidate, AnalysisObservation, RestoreCandidate,
};
pub use content_model::{
    AnalysisRequest, AnalysisSet, AnalyzerId, AnalyzerOutcome, AnalyzerVersion, BasicMetrics,
    CODE_SLOC, CONTENT_BASIC, CodeMetrics, ContentDetection, ContentProvenance, CoverageReason,
    FileAnalysis, LogicalWordStats, MARKDOWN_PROSE, METRICS, MetricDef, MetricValues,
    OptionsFingerprint, TEXT_LOGICAL, WordMetrics,
};
