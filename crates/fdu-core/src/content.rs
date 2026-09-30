//! Optional, versioned file-content analysis.
//!
//! Metadata-only scans allocate none of these structures and open no file content.
//!
//! An analysis pass reads each file in 64 KiB chunks, schedules its candidates in
//! bounded batches, and never truncates or size-skips an eligible file. Two things are
//! still retained whole per worker, because an exact answer needs them: the source of a
//! Markdown file under the words unit, since the `CommonMark` parser resolves list
//! tightness, headings, and references across the whole document (fdu-b2qz), and the
//! longest logical line of a supported code file under the code unit
//! ([`CodeAccumulator`], fdu-1zb6). A file of another type retains at most its 16 KiB
//! classification prefix and one chunk once the prefix has settled its type.

mod content_analysis;
mod content_basic_metrics;
mod content_cache;
mod content_code_metrics;
mod content_index;
mod content_markdown_metrics;
mod content_model;

pub(crate) use content_analysis::analyze_index_observed;
pub use content_analysis::{AnalysisReport, AnalyzerCoverage, analyze_index};
pub use content_basic_metrics::{BasicAccumulator, TextAdmission};
pub use content_cache::{
    ContentCacheLoad, content_cache_path, load_content_cache, save_content_cache,
};
pub(crate) use content_cache::{content_sidecar_bytes, identify_sidecar};
pub use content_code_metrics::CodeAccumulator;
pub(crate) use content_index::ContentTierState;
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
