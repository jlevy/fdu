//! Direct `PyO3` binding for the long-lived opened-root engine.
//!
//! This module is intentionally a thin boundary. It parses fdu-native request values,
//! releases the GIL for every engine operation, and converts complete engine results
//! back to ordinary Python values. It owns no scheduler, cache, or duplicate lifecycle.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyModule};

use fdu_core::content::AnalysisSet;
use fdu_core::query::{AxisNames, EntrySelection, Query, Selection};
use fdu_core::{
    ChangeOutcome, ChangePoll, ChangeRequest, ContinuationId, CountResult, Coverage,
    CoverageReason, EffectiveChange, EngineVersion, EntryKind, EntryValue, Freshness, Impact,
    ImpactDomain, IndexState, Issue, IssueKind, Knowledge, LifecyclePhase, LimitedProjection,
    OpenOptions, OpenedIndex, PageRequest, ProjectionResult, ReadDiagnostics, ReadProjection,
    ReadRequest, ReadResponse, RefreshResult, RollUpSummary, RowShape, ScopeIdentity,
    SemanticIdentity, Source, StateTransition, Work,
};

create_exception!(fdu, OpenedIndexError, PyRuntimeError);
create_exception!(fdu, OpenedIndexClosedError, OpenedIndexError);
create_exception!(fdu, OpenedIndexStoppedError, OpenedIndexError);
create_exception!(fdu, VersionUnavailableError, OpenedIndexError);
create_exception!(fdu, ContinuationUnavailableError, OpenedIndexError);
create_exception!(fdu, ChangeCursorUnavailableError, OpenedIndexError);
create_exception!(fdu, OpenedIndexLimitError, OpenedIndexError);

fn opened_py_err(error: fdu_core::Error) -> PyErr {
    match error {
        fdu_core::Error::OpenedIndexClosed => OpenedIndexClosedError::new_err(error.to_string()),
        fdu_core::Error::OpenedIndexStopped => OpenedIndexStoppedError::new_err(error.to_string()),
        fdu_core::Error::VersionUnavailable { .. } => {
            VersionUnavailableError::new_err(error.to_string())
        }
        fdu_core::Error::ContinuationUnavailable | fdu_core::Error::ContinuationStale { .. } => {
            ContinuationUnavailableError::new_err(error.to_string())
        }
        fdu_core::Error::ChangeCursorUnavailable { .. } => {
            ChangeCursorUnavailableError::new_err(error.to_string())
        }
        fdu_core::Error::PriorityPathLimit { .. }
        | fdu_core::Error::RefreshPathLimit { .. }
        | fdu_core::Error::ReadProjectionLimit { .. }
        | fdu_core::Error::PageRowLimit { .. }
        | fdu_core::Error::PageWorkLimit { .. }
        | fdu_core::Error::CountCapLimit { .. }
        | fdu_core::Error::ReportViewLimit { .. }
        | fdu_core::Error::ContinuationRecordLimit { .. } => {
            OpenedIndexLimitError::new_err(error.to_string())
        }
        fdu_core::Error::ContinuationIdentityExhausted
        | fdu_core::Error::OpenedIdentityExhausted
        | fdu_core::Error::OpenedJournalPoisoned
        | fdu_core::Error::OpenedLifecyclePoisoned
        | fdu_core::Error::OpenedWorkerPanicked { .. }
        | fdu_core::Error::OpenedWorkerFailed { .. }
        | fdu_core::Error::OpenedWorkerSpawn { .. } => OpenedIndexError::new_err(error.to_string()),
        fdu_core::Error::UnsupportedFlatSelection => PyValueError::new_err(error.to_string()),
        other => super::to_py_err(other),
    }
}

fn required<'py>(dict: &Bound<'py, PyDict>, key: &str) -> PyResult<Bound<'py, PyAny>> {
    dict.get_item(key)?
        .ok_or_else(|| PyValueError::new_err(format!("missing required field {key:?}")))
}

fn mapping<'py>(value: Bound<'py, PyAny>, label: &str) -> PyResult<Bound<'py, PyDict>> {
    value.cast_into().map_err(|_| PyValueError::new_err(format!("{label} must be a mapping")))
}

fn optional_string(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<String>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => value.extract().map(Some),
        _ => Ok(None),
    }
}

fn optional_strings(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<Vec<String>>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => value.extract().map(Some),
        _ => Ok(None),
    }
}

fn parse_selection(dict: Option<&Bound<'_, PyDict>>, now: SystemTime) -> PyResult<Selection> {
    let Some(dict) = dict else {
        return Ok(Selection::default());
    };
    let include = optional_strings(dict, "include")?;
    let exclude = optional_strings(dict, "exclude")?;
    let min_size = optional_string(dict, "min_size")?;
    let modified_since = optional_string(dict, "modified_since")?;
    let modified_before = optional_string(dict, "modified_before")?;
    let kind = optional_strings(dict, "kind")?;
    let depth = optional_string(dict, "depth")?;
    let limit = optional_string(dict, "limit")?;
    let sort = optional_string(dict, "sort")?;
    let reverse = dict
        .get_item("reverse")?
        .map(|value| value.extract::<bool>())
        .transpose()?
        .unwrap_or(false);
    let size = dict
        .get_item("size")?
        .map(|value| value.extract::<String>())
        .transpose()?
        .unwrap_or_else(|| "apparent".to_owned());
    Ok(super::build_query_at(
        now,
        AnalysisSet::NONE,
        None,
        None,
        include,
        exclude,
        min_size.as_deref(),
        modified_since.as_deref(),
        modified_before.as_deref(),
        kind,
        depth.as_deref(),
        limit.as_deref(),
        sort.as_deref(),
        reverse,
        &size,
        250,
    )?
    .selection)
}

fn parse_entry_selection(
    dict: Option<&Bound<'_, PyDict>>,
    now: SystemTime,
) -> PyResult<EntrySelection> {
    let Some(dict) = dict else {
        return Ok(EntrySelection::default());
    };
    Ok(EntrySelection {
        query: parse_selection(Some(dict), now)?,
        max_size: dict
            .get_item("max_size")?
            .filter(|value| !value.is_none())
            .map(|value| value.extract())
            .transpose()?,
        exclude_ignored: dict
            .get_item("exclude_ignored")?
            .map(|value| value.extract())
            .transpose()?
            .unwrap_or(false),
        logical_extensions: optional_strings(dict, "logical_extensions")?.unwrap_or_default(),
        exact_names: optional_strings(dict, "exact_names")?.unwrap_or_default(),
        terminal_extensions: optional_strings(dict, "terminal_extensions")?.unwrap_or_default(),
        ancestor_names: optional_strings(dict, "ancestor_names")?.unwrap_or_default(),
    })
}

fn parse_scope(dict: &Bound<'_, PyDict>) -> PyResult<ScopeIdentity> {
    let max_depth = match dict.get_item("max_depth")? {
        Some(value) if !value.is_none() => Some(value.extract()?),
        _ => None,
    };
    Ok(ScopeIdentity {
        max_depth,
        follow_symlinks: required(dict, "follow_symlinks")?.extract()?,
        one_filesystem: required(dict, "one_filesystem")?.extract()?,
        hidden_fingerprint: required(dict, "hidden_fingerprint")?.extract()?,
        exclude_special: required(dict, "exclude_special")?.extract()?,
    })
}

fn parse_semantics(dict: &Bound<'_, PyDict>) -> PyResult<SemanticIdentity> {
    Ok(SemanticIdentity {
        ignore_rules_fingerprint: required(dict, "ignore_rules_fingerprint")?.extract()?,
        type_rules_fingerprint: required(dict, "type_rules_fingerprint")?.extract()?,
        reducers_fingerprint: required(dict, "reducers_fingerprint")?.extract()?,
    })
}

fn parse_version(dict: &Bound<'_, PyDict>) -> PyResult<EngineVersion> {
    let raw_session: u64 = required(dict, "session")?.extract()?;
    let session = fdu_core::SessionId::from_opaque(raw_session)
        .ok_or_else(|| PyValueError::new_err("session must be a nonzero integer"))?;
    let scope = mapping(required(dict, "scope")?, "scope")?;
    let semantics = mapping(required(dict, "semantics")?, "semantics")?;
    Ok(EngineVersion {
        session,
        sequence: fdu_core::Clock(required(dict, "sequence")?.extract()?),
        scope: parse_scope(&scope)?,
        semantics: parse_semantics(&semantics)?,
    })
}

fn parse_continuation(dict: &Bound<'_, PyDict>) -> PyResult<ContinuationId> {
    let session = required(dict, "session")?.extract()?;
    let ordinal = required(dict, "ordinal")?.extract()?;
    ContinuationId::from_opaque_parts(session, ordinal)
        .ok_or_else(|| PyValueError::new_err("continuation parts must be nonzero integers"))
}

fn parse_page(dict: &Bound<'_, PyDict>) -> PyResult<PageRequest> {
    Ok(PageRequest {
        limit: required(dict, "limit")?.extract()?,
        max_work: required(dict, "max_work")?.extract()?,
    })
}

fn system_time_from_nanos(nanos: i64) -> PyResult<SystemTime> {
    if nanos >= 0 {
        UNIX_EPOCH
            .checked_add(Duration::from_nanos(nanos.unsigned_abs()))
            .ok_or_else(|| PyValueError::new_err("generated_at_ns is outside SystemTime range"))
    } else {
        UNIX_EPOCH
            .checked_sub(Duration::from_nanos(nanos.unsigned_abs()))
            .ok_or_else(|| PyValueError::new_err("generated_at_ns is outside SystemTime range"))
    }
}

fn parse_report(dict: &Bound<'_, PyDict>) -> PyResult<fdu_core::ReportRequest> {
    let generated_at = system_time_from_nanos(required(dict, "generated_at_ns")?.extract()?)?;
    let selection =
        dict.get_item("selection")?.map(|value| mapping(value, "selection")).transpose()?;
    let selection = parse_selection(selection.as_ref(), generated_at)?;
    let views = optional_strings(dict, "views")?;
    let words_per_page = dict
        .get_item("words_per_page")?
        .map(|value| value.extract::<u64>())
        .transpose()?
        .unwrap_or(250);
    let (views, omitted_views) = fdu_core::query::ViewSpec::resolve(
        views.as_ref().map(|values| values.join(",")).as_deref(),
        AnalysisSet::NONE,
        "view",
    )
    .map_err(PyValueError::new_err)?;
    if words_per_page == 0 {
        return Err(PyValueError::new_err("words_per_page must be positive"));
    }
    let query = Query { selection, views, omitted_views, axes: AxisNames::FIELDS, words_per_page };
    Ok(fdu_core::ReportRequest {
        query,
        generated_at,
        max_work: required(dict, "max_work")?.extract()?,
    })
}

fn parse_projection(value: Bound<'_, PyAny>, now: SystemTime) -> PyResult<ReadProjection> {
    let dict = mapping(value, "projection")?;
    let kind: String = required(&dict, "kind")?.extract()?;
    let path = || required(&dict, "path").and_then(|value| value.extract::<PathBuf>());
    let page = || {
        let value = mapping(required(&dict, "page")?, "page")?;
        parse_page(&value)
    };
    match kind.as_str() {
        "lookup" => Ok(ReadProjection::Lookup { path: path()? }),
        "rollup" => Ok(ReadProjection::RollUp { path: path()? }),
        "tree" => Ok(ReadProjection::Tree { path: path()?, page: page()? }),
        "flat" => {
            let selection =
                dict.get_item("selection")?.map(|value| mapping(value, "selection")).transpose()?;
            let shape = match dict
                .get_item("shape")?
                .map(|value| value.extract::<String>())
                .transpose()?
                .as_deref()
            {
                None | Some("compact") => RowShape::Compact,
                Some("full") => RowShape::Full,
                Some(other) => {
                    return Err(PyValueError::new_err(format!(
                        "invalid row shape {other:?}: expected compact or full"
                    )));
                }
            };
            Ok(ReadProjection::Flat {
                selection: parse_entry_selection(selection.as_ref(), now)?,
                shape,
                page: page()?,
            })
        }
        "aggregate" => {
            let selection =
                dict.get_item("selection")?.map(|value| mapping(value, "selection")).transpose()?;
            Ok(ReadProjection::Aggregate {
                selection: parse_entry_selection(selection.as_ref(), now)?,
                count_cap: required(&dict, "count_cap")?.extract()?,
                max_work: required(&dict, "max_work")?.extract()?,
            })
        }
        "report" => {
            let request = mapping(required(&dict, "request")?, "report request")?;
            Ok(ReadProjection::Report(parse_report(&request)?))
        }
        "continue" => {
            let continuation = mapping(required(&dict, "continuation")?, "continuation")?;
            Ok(ReadProjection::Continue {
                continuation: parse_continuation(&continuation)?,
                page: page()?,
            })
        }
        "diagnostics" => Ok(ReadProjection::Diagnostics),
        other => Err(PyValueError::new_err(format!(
            "invalid projection kind {other:?}: expected lookup, rollup, tree, flat, aggregate, report, continue, or diagnostics"
        ))),
    }
}

fn parse_projections(values: &Bound<'_, PyList>) -> PyResult<Vec<ReadProjection>> {
    let now = SystemTime::now();
    values.iter().map(|value| parse_projection(value, now)).collect()
}

fn source_label(value: Source) -> &'static str {
    match value {
        Source::Scanned => "scanned",
        Source::Revalidated => "revalidated",
        Source::JournalScoped => "journal_scoped",
        Source::Cached => "cached",
    }
}

fn freshness_label(value: Freshness) -> &'static str {
    match value {
        Freshness::Fresh => "fresh",
        Freshness::Reconciling => "reconciling",
        Freshness::Stale => "stale",
        Freshness::Partial => "partial",
    }
}

fn phase_label(value: LifecyclePhase) -> &'static str {
    match value {
        LifecyclePhase::Discovering => "discovering",
        LifecyclePhase::Reconciling => "reconciling",
        LifecyclePhase::Ready => "ready",
        LifecyclePhase::Watching => "watching",
        LifecyclePhase::Stopped => "stopped",
        LifecyclePhase::Failed => "failed",
    }
}

fn coverage_reason_label(value: CoverageReason) -> &'static str {
    match value {
        CoverageReason::Building => "building",
        CoverageReason::Budget => "budget",
        CoverageReason::Cancelled => "cancelled",
        CoverageReason::Inaccessible => "inaccessible",
        CoverageReason::Failed => "failed",
    }
}

fn issue_kind_label(value: IssueKind) -> &'static str {
    match value {
        IssueKind::Permission => "permission",
        IssueKind::Disappeared => "disappeared",
        IssueKind::InvalidMetadata => "invalid_metadata",
        IssueKind::ResourceBudget => "resource_budget",
        IssueKind::ObservationGap => "observation_gap",
        IssueKind::ProviderFailure => "provider_failure",
    }
}

fn impact_domain_label(value: ImpactDomain) -> &'static str {
    match value {
        ImpactDomain::Topology => "topology",
        ImpactDomain::Metadata => "metadata",
        ImpactDomain::Classification => "classification",
        ImpactDomain::Aggregates => "aggregates",
        ImpactDomain::Content => "content",
        ImpactDomain::State => "state",
    }
}

fn invalidation_reason_label(value: fdu_core::InvalidateReason) -> &'static str {
    match value {
        fdu_core::InvalidateReason::WatchOverflow => "watch_overflow",
        fdu_core::InvalidateReason::UnpairedRename => "unpaired_rename",
        fdu_core::InvalidateReason::WatchSetupRace => "watch_setup_race",
        fdu_core::InvalidateReason::PeriodicSweep => "periodic_sweep",
        fdu_core::InvalidateReason::VerificationFailed => "verification_failed",
        fdu_core::InvalidateReason::UnknownAncestry => "unknown_ancestry",
        fdu_core::InvalidateReason::WatchContention => "watch_contention",
        fdu_core::InvalidateReason::Requested => "requested",
    }
}

fn entry_kind_label(value: EntryKind) -> &'static str {
    super::entry_kind_label(value)
}

fn limited_projection_label(value: LimitedProjection) -> &'static str {
    match value {
        LimitedProjection::Tree => "tree",
        LimitedProjection::Flat => "flat",
        LimitedProjection::Report => "report",
        LimitedProjection::Aggregate => "aggregate",
    }
}

fn scope_dict(py: Python<'_>, scope: ScopeIdentity) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("max_depth", scope.max_depth)?;
    out.set_item("follow_symlinks", scope.follow_symlinks)?;
    out.set_item("one_filesystem", scope.one_filesystem)?;
    out.set_item("hidden_fingerprint", scope.hidden_fingerprint)?;
    out.set_item("exclude_special", scope.exclude_special)?;
    Ok(out)
}

fn semantics_dict(py: Python<'_>, semantics: SemanticIdentity) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("ignore_rules_fingerprint", semantics.ignore_rules_fingerprint)?;
    out.set_item("type_rules_fingerprint", semantics.type_rules_fingerprint)?;
    out.set_item("reducers_fingerprint", semantics.reducers_fingerprint)?;
    Ok(out)
}

fn version_dict(py: Python<'_>, version: EngineVersion) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("session", version.session.opaque())?;
    out.set_item("sequence", version.sequence.0)?;
    out.set_item("scope", scope_dict(py, version.scope)?)?;
    out.set_item("semantics", semantics_dict(py, version.semantics)?)?;
    Ok(out)
}

fn continuation_dict(py: Python<'_>, continuation: ContinuationId) -> PyResult<Bound<'_, PyDict>> {
    let (session, ordinal) = continuation.opaque_parts();
    let out = PyDict::new(py);
    out.set_item("session", session)?;
    out.set_item("ordinal", ordinal)?;
    Ok(out)
}

fn scan_scope_dict(py: Python<'_>, scope: fdu_core::ScanScope) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("max_depth", scope.max_depth)?;
    out.set_item("follow_symlinks", scope.follow_symlinks)?;
    out.set_item("one_filesystem", scope.one_filesystem)?;
    out.set_item("hidden_fingerprint", scope.hidden_fingerprint)?;
    out.set_item("exclude_special", scope.exclude_special)?;
    out.set_item("ignore_rules_fingerprint", scope.ignore_rules_fingerprint)?;
    out.set_item("type_rules_fingerprint", scope.type_rules_fingerprint)?;
    out.set_item("reducers_fingerprint", scope.reducers_fingerprint)?;
    Ok(out)
}

fn coverage_dict(py: Python<'_>, coverage: Coverage) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    match coverage {
        Coverage::Complete => {
            out.set_item("kind", "complete")?;
            out.set_item("reason", py.None())?;
        }
        Coverage::Partial(reason) => {
            out.set_item("kind", "partial")?;
            out.set_item("reason", coverage_reason_label(reason))?;
        }
    }
    Ok(out)
}

fn state_dict(py: Python<'_>, state: IndexState) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("phase", phase_label(state.phase))?;
    out.set_item("coverage", coverage_dict(py, state.coverage)?)?;
    out.set_item("freshness", freshness_label(state.freshness))?;
    out.set_item("source", source_label(state.source))?;
    let progress = PyDict::new(py);
    progress.set_item("files_retained", state.progress.files_retained)?;
    progress.set_item("directories_complete", state.progress.directories_complete)?;
    out.set_item("progress", progress)?;
    let issues = PyDict::new(py);
    issues.set_item("retained", state.issues.retained)?;
    issues.set_item("omitted", state.issues.omitted)?;
    out.set_item("issues", issues)?;
    Ok(out)
}

fn work_dict(py: Python<'_>, work: Work) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("observations", work.observations)?;
    out.set_item("unchanged", work.unchanged)?;
    out.set_item("stale", work.stale)?;
    out.set_item("resource_refused", work.resource_refused)?;
    out.set_item("rows_visited", work.rows_visited)?;
    out.set_item("rows_returned", work.rows_returned)?;
    out.set_item("maintained_index_work", work.maintained_index_work)?;
    out.set_item("commits_visited", work.commits_visited)?;
    out.set_item("commits_returned", work.commits_returned)?;
    out.set_item("directories_read", work.directories_read)?;
    out.set_item("entries_visited", work.entries_visited)?;
    out.set_item("files_visited", work.files_visited)?;
    out.set_item("bytes_visited", work.bytes_visited)?;
    Ok(out)
}

fn attrs_dict(py: Python<'_>, attrs: fdu_core::Attrs) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("size", attrs.size)?;
    out.set_item("allocated", attrs.allocated)?;
    out.set_item("mtime_ns", attrs.mtime_ns)?;
    out.set_item("ctime_ns", attrs.ctime_ns)?;
    out.set_item("inode", attrs.inode)?;
    out.set_item("dev", attrs.dev)?;
    Ok(out)
}

fn rollup_summary_dict(py: Python<'_>, rollup: RollUpSummary) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("files", rollup.files)?;
    out.set_item("dirs", rollup.dirs)?;
    out.set_item("bytes", rollup.bytes)?;
    out.set_item("allocated", rollup.allocated)?;
    out.set_item("newest_mtime_ns", rollup.newest_mtime_ns)?;
    Ok(out)
}

fn partition_rollup_dict(
    py: Python<'_>,
    rollup: fdu_core::PartitionRollUpSummary,
) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("all", rollup_summary_dict(py, rollup.all)?)?;
    out.set_item("unignored", rollup_summary_dict(py, rollup.unignored)?)?;
    Ok(out)
}

fn entry_dict<'py>(py: Python<'py>, entry: &EntryValue) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("path", entry.path.as_os_str())?;
    out.set_item("portable_path", entry.portable_path.as_str())?;
    out.set_item("kind", entry_kind_label(entry.kind))?;
    out.set_item("attrs", attrs_dict(py, entry.attrs)?)?;
    out.set_item("ignored", entry.ignored)?;
    out.set_item(
        "classification",
        entry
            .classification
            .as_ref()
            .map(|classification| classification_dict(py, classification))
            .transpose()?,
    )?;
    out.set_item(
        "rollup",
        entry.rollup.map(|rollup| partition_rollup_dict(py, rollup)).transpose()?,
    )?;
    out.set_item("children_complete", entry.children_complete)?;
    Ok(out)
}

fn classification_dict<'py>(
    py: Python<'py>,
    classification: &fdu_core::classify::NameClassification,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("logical_extension", classification.logical_extension())?;
    out.set_item("canonical_extension", classification.canonical_extension())?;
    out.set_item("kind_id", classification.kind_id())?;
    out.set_item("family_id", classification.family_id())?;
    out.set_item("group_id", classification.group_id())?;
    out.set_item("content_family", classification.content_family().as_str())?;
    Ok(out)
}

fn issue_dict<'py>(py: Python<'py>, issue: &Issue) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("kind", issue_kind_label(issue.kind))?;
    out.set_item("path", issue.path.as_ref().map(|path| path.as_os_str()))?;
    out.set_item("message", &issue.message)?;
    out.set_item("os_error", issue.os_error)?;
    Ok(out)
}

fn impact_dict<'py>(py: Python<'py>, impact: &Impact) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item(
        "domains",
        impact.domains.iter().copied().map(impact_domain_label).collect::<Vec<_>>(),
    )?;
    let dirty_paths: Vec<&Path> = impact.dirty_paths.iter().map(PathBuf::as_path).collect();
    out.set_item("dirty_paths", dirty_paths)?;
    out.set_item("all_dirty", impact.all_dirty)?;
    Ok(out)
}

fn control_identity_dict(
    py: Python<'_>,
    identity: fdu_core::control::ControlIdentity,
) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("bytes", identity.bytes)?;
    out.set_item("fingerprint", identity.fingerprint)?;
    Ok(out)
}

fn effective_change_dict<'py>(
    py: Python<'py>,
    change: &EffectiveChange,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    match change {
        EffectiveChange::Inserted { path, kind, attrs } => {
            out.set_item("kind", "inserted")?;
            out.set_item("path", path.as_os_str())?;
            out.set_item("entry_kind", entry_kind_label(*kind))?;
            out.set_item("attrs", attrs_dict(py, *attrs)?)?;
        }
        EffectiveChange::Updated { path, kind, previous, current } => {
            out.set_item("kind", "updated")?;
            out.set_item("path", path.as_os_str())?;
            out.set_item("entry_kind", entry_kind_label(*kind))?;
            out.set_item("previous", attrs_dict(py, *previous)?)?;
            out.set_item("current", attrs_dict(py, *current)?)?;
        }
        EffectiveChange::Removed { path, kind, attrs } => {
            out.set_item("kind", "removed")?;
            out.set_item("path", path.as_os_str())?;
            out.set_item("entry_kind", entry_kind_label(*kind))?;
            out.set_item("attrs", attrs_dict(py, *attrs)?)?;
        }
        EffectiveChange::ControlUpdated { path, previous, current } => {
            out.set_item("kind", "control_updated")?;
            out.set_item("path", path.as_os_str())?;
            out.set_item(
                "previous",
                previous.map(|value| control_identity_dict(py, value)).transpose()?,
            )?;
            out.set_item(
                "current",
                current.map(|value| control_identity_dict(py, value)).transpose()?,
            )?;
        }
        EffectiveChange::Reclassified { path, previous_ignored, current_ignored } => {
            out.set_item("kind", "reclassified")?;
            out.set_item("path", path.as_os_str())?;
            out.set_item("previous_ignored", previous_ignored)?;
            out.set_item("current_ignored", current_ignored)?;
        }
        EffectiveChange::Invalidated { path, reason } => {
            out.set_item("kind", "invalidated")?;
            out.set_item("path", path.as_os_str())?;
            out.set_item("reason", invalidation_reason_label(*reason))?;
        }
    }
    Ok(out)
}

fn transition_dict<'py>(
    py: Python<'py>,
    transition: &StateTransition,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    match transition {
        StateTransition::Freshness { path, previous, current } => {
            out.set_item("kind", "freshness")?;
            out.set_item("path", path.as_os_str())?;
            out.set_item("previous", freshness_label(*previous))?;
            out.set_item("current", freshness_label(*current))?;
        }
        StateTransition::Verified { path } => {
            out.set_item("kind", "verified")?;
            out.set_item("path", path.as_os_str())?;
        }
        StateTransition::DirectoryComplete { path } => {
            out.set_item("kind", "directory_complete")?;
            out.set_item("path", path.as_os_str())?;
        }
        StateTransition::IndexState { previous, current } => {
            out.set_item("kind", "index_state")?;
            out.set_item("previous", state_dict(py, *previous)?)?;
            out.set_item("current", state_dict(py, *current)?)?;
        }
    }
    Ok(out)
}

fn commit_dict<'py>(py: Python<'py>, commit: &fdu_core::Commit) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("sequence", commit.clock.0)?;
    let changes = PyList::empty(py);
    for change in &commit.changes {
        changes.append(effective_change_dict(py, change)?)?;
    }
    out.set_item("changes", changes)?;
    out.set_item("impact", impact_dict(py, &commit.impact)?)?;
    let transitions = PyList::empty(py);
    for transition in &commit.state {
        transitions.append(transition_dict(py, transition)?)?;
    }
    out.set_item("state", transitions)?;
    out.set_item("work", work_dict(py, commit.work)?)?;
    Ok(out)
}

fn tree_page_dict<'py>(py: Python<'py>, page: &fdu_core::TreePage) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("directory", entry_dict(py, &page.directory)?)?;
    let rows = PyList::empty(py);
    for row in &page.rows {
        rows.append(entry_dict(py, row)?)?;
    }
    out.set_item("rows", rows)?;
    out.set_item("next", page.next.map(|value| continuation_dict(py, value)).transpose()?)?;
    out.set_item("complete", page.complete)?;
    Ok(out)
}

fn flat_page_dict<'py>(py: Python<'py>, page: &fdu_core::FlatPage) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    let rows = PyList::empty(py);
    for row in &page.rows {
        rows.append(entry_dict(py, row)?)?;
    }
    out.set_item("rows", rows)?;
    out.set_item("next", page.next.map(|value| continuation_dict(py, value)).transpose()?)?;
    Ok(out)
}

fn diagnostics_dict<'py>(
    py: Python<'py>,
    diagnostics: &ReadDiagnostics,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("root", diagnostics.root.as_os_str())?;
    out.set_item("scope", scan_scope_dict(py, diagnostics.scope)?)?;
    out.set_item("entries", diagnostics.entries)?;
    let issues = PyList::empty(py);
    for issue in &diagnostics.issues {
        issues.append(issue_dict(py, issue)?)?;
    }
    out.set_item("issues", issues)?;
    Ok(out)
}

fn lookup_result_dict<'py>(
    py: Python<'py>,
    knowledge: &Knowledge<EntryValue>,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    match knowledge {
        Knowledge::Present(entry) => {
            out.set_item("knowledge", "present")?;
            out.set_item("value", entry_dict(py, entry)?)?;
        }
        Knowledge::Absent => {
            out.set_item("knowledge", "absent")?;
            out.set_item("value", py.None())?;
        }
        Knowledge::Unknown { reason } => {
            out.set_item("knowledge", "unknown")?;
            out.set_item("value", py.None())?;
            out.set_item("reason", coverage_reason_label(*reason))?;
        }
    }
    Ok(out)
}

fn rollup_result_dict<'py>(
    py: Python<'py>,
    knowledge: &Knowledge<fdu_core::PartitionRollUpSummary>,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    match knowledge {
        Knowledge::Present(rollup) => {
            out.set_item("knowledge", "present")?;
            out.set_item("value", partition_rollup_dict(py, *rollup)?)?;
        }
        Knowledge::Absent => {
            out.set_item("knowledge", "absent")?;
            out.set_item("value", py.None())?;
        }
        Knowledge::Unknown { reason } => {
            out.set_item("knowledge", "unknown")?;
            out.set_item("value", py.None())?;
            out.set_item("reason", coverage_reason_label(*reason))?;
        }
    }
    Ok(out)
}

fn tree_result_dict<'py>(
    py: Python<'py>,
    knowledge: &Knowledge<fdu_core::TreePage>,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    match knowledge {
        Knowledge::Present(page) => {
            out.set_item("knowledge", "present")?;
            out.set_item("value", tree_page_dict(py, page)?)?;
        }
        Knowledge::Absent => {
            out.set_item("knowledge", "absent")?;
            out.set_item("value", py.None())?;
        }
        Knowledge::Unknown { reason } => {
            out.set_item("knowledge", "unknown")?;
            out.set_item("value", py.None())?;
            out.set_item("reason", coverage_reason_label(*reason))?;
        }
    }
    Ok(out)
}

fn projection_result_dict<'py>(
    py: Python<'py>,
    result: &ProjectionResult,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    match result {
        ProjectionResult::Lookup(value) => {
            out.set_item("kind", "lookup")?;
            out.set_item("value", lookup_result_dict(py, value)?)?;
        }
        ProjectionResult::RollUp(value) => {
            out.set_item("kind", "rollup")?;
            out.set_item("value", rollup_result_dict(py, value)?)?;
        }
        ProjectionResult::Tree(value) => {
            out.set_item("kind", "tree")?;
            out.set_item("value", tree_result_dict(py, value)?)?;
        }
        ProjectionResult::Flat(value) => {
            out.set_item("kind", "flat")?;
            out.set_item("value", flat_page_dict(py, value)?)?;
        }
        ProjectionResult::Aggregate(value) => {
            out.set_item("kind", "aggregate")?;
            let count = PyDict::new(py);
            match value {
                CountResult::Exact(value) => {
                    count.set_item("kind", "exact")?;
                    count.set_item("value", value)?;
                }
                CountResult::AtLeast(value) => {
                    count.set_item("kind", "at_least")?;
                    count.set_item("value", value)?;
                }
            }
            out.set_item("value", count)?;
        }
        ProjectionResult::Report(value) => {
            out.set_item("kind", "report")?;
            let rendered = fdu_core::report_format::render(
                value,
                fdu_core::report_format::Format::Json,
                false,
            );
            let wire = py.import("json")?.call_method1("loads", (rendered,))?;
            let report = PyDict::new(py);
            report.set_item("wire", wire)?;
            report.set_item("notes", &value.notes)?;
            report
                .set_item("renderer", Py::new(py, super::PyOneShot { report: value.clone() })?)?;
            out.set_item("value", report)?;
        }
        ProjectionResult::Diagnostics(value) => {
            out.set_item("kind", "diagnostics")?;
            out.set_item("value", diagnostics_dict(py, value)?)?;
        }
        ProjectionResult::Limit(value) => {
            out.set_item("kind", "limit")?;
            let limit = PyDict::new(py);
            limit.set_item("projection", limited_projection_label(value.projection))?;
            limit.set_item("max_work", value.max_work)?;
            limit.set_item("rows_visited", value.rows_visited)?;
            out.set_item("value", limit)?;
        }
    }
    Ok(out)
}

fn read_response_dict<'py>(
    py: Python<'py>,
    response: &ReadResponse,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("version", version_dict(py, response.version)?)?;
    out.set_item("state", state_dict(py, response.state)?)?;
    let results = PyList::empty(py);
    for result in &response.results {
        results.append(projection_result_dict(py, result)?)?;
    }
    out.set_item("results", results)?;
    out.set_item("work", work_dict(py, response.work)?)?;
    out.set_item("change_cursor", version_dict(py, response.change_cursor)?)?;
    Ok(out)
}

fn change_poll_dict<'py>(py: Python<'py>, poll: &ChangePoll) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("cursor", version_dict(py, poll.cursor)?)?;
    out.set_item("version", version_dict(py, poll.version)?)?;
    out.set_item("state", state_dict(py, poll.state)?)?;
    let outcome = PyDict::new(py);
    match &poll.outcome {
        ChangeOutcome::Changes { commits, impact } => {
            outcome.set_item("kind", "changes")?;
            let items = PyList::empty(py);
            for commit in commits {
                items.append(commit_dict(py, commit)?)?;
            }
            outcome.set_item("commits", items)?;
            outcome.set_item("impact", impact_dict(py, impact)?)?;
        }
        ChangeOutcome::Idle => {
            outcome.set_item("kind", "idle")?;
        }
        ChangeOutcome::Reset { impact } => {
            outcome.set_item("kind", "reset")?;
            outcome.set_item("impact", impact_dict(py, impact)?)?;
        }
    }
    out.set_item("outcome", outcome)?;
    out.set_item("work", work_dict(py, poll.work)?)?;
    Ok(out)
}

fn refresh_result_dict<'py>(
    py: Python<'py>,
    result: &RefreshResult,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("after", version_dict(py, result.after)?)?;
    out.set_item("version", version_dict(py, result.version)?)?;
    out.set_item("state", state_dict(py, result.state)?)?;
    let accepted: Vec<&Path> = result.accepted.iter().map(PathBuf::as_path).collect();
    out.set_item("accepted", accepted)?;
    let rejected = PyList::empty(py);
    for value in &result.rejected {
        let item = PyDict::new(py);
        item.set_item("path", value.path.as_os_str())?;
        item.set_item("reason", value.reason.as_str())?;
        rejected.append(item)?;
    }
    out.set_item("rejected", rejected)?;
    out.set_item("impact", impact_dict(py, &result.impact)?)?;
    out.set_item("work", work_dict(py, result.work)?)?;
    let issues = PyList::empty(py);
    for issue in &result.issues {
        issues.append(issue_dict(py, issue)?)?;
    }
    out.set_item("issues", issues)?;
    out.set_item("omitted_issues", result.omitted_issues)?;
    Ok(out)
}

/// Native owner for one long-lived fdu root.
///
/// The Python package wraps the dictionaries this class exchanges in immutable public
/// values. The native object itself stores only the shared Rust handle.
#[pyclass(name = "OpenedIndex", module = "fdu._native", frozen)]
pub(super) struct PyOpenedIndex {
    inner: OpenedIndex,
}

#[pymethods]
impl PyOpenedIndex {
    /// Open one root and start progressive discovery.
    #[staticmethod]
    #[pyo3(signature = (
        root,
        *,
        batch_size = None,
        follow_symlinks = false,
        one_filesystem = false,
        prune_hidden = false,
        hidden_allow = None,
        exclude_special = false,
        max_files = None,
        observe = false,
        journal_capacity = None
    ))]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn open(
        py: Python<'_>,
        root: PathBuf,
        batch_size: Option<usize>,
        follow_symlinks: bool,
        one_filesystem: bool,
        prune_hidden: bool,
        hidden_allow: Option<Vec<OsString>>,
        exclude_special: bool,
        max_files: Option<u64>,
        observe: bool,
        journal_capacity: Option<usize>,
    ) -> PyResult<Self> {
        let allowed = hidden_allow.unwrap_or_default();
        if !prune_hidden && !allowed.is_empty() {
            return Err(PyValueError::new_err(
                "hidden_allow requires prune_hidden=True so the allowlist has a policy to modify",
            ));
        }
        let mut options = OpenOptions::default();
        if let Some(value) = batch_size {
            options.batch_size = value;
        }
        options.follow_symlinks = follow_symlinks;
        options.one_filesystem = one_filesystem;
        options.hidden =
            prune_hidden.then(|| Arc::new(fdu_core::HiddenPolicy::prune_hidden(allowed)));
        options.exclude_special = exclude_special;
        options.budget.max_files = max_files;
        options.observation = observe.then(fdu_core::watch::WatchConfig::default);
        if let Some(value) = journal_capacity {
            options.journal_capacity = value;
        }
        let inner = py.detach(move || OpenedIndex::open(&root, options)).map_err(opened_py_err)?;
        Ok(Self { inner })
    }

    /// Return one coherent set of fdu-native projections.
    #[pyo3(signature = (projections, *, expected = None))]
    fn read<'py>(
        &self,
        py: Python<'py>,
        projections: &Bound<'py, PyList>,
        expected: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let projections = parse_projections(projections)?;
        let expected = expected.map(parse_version).transpose()?;
        let response = py
            .detach(|| self.inner.read(ReadRequest { projections, expected }))
            .map_err(opened_py_err)?;
        read_response_dict(py, &response)
    }

    /// Return the current version and state without requesting a projection.
    fn state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let response =
            py.detach(|| self.inner.read(ReadRequest::default())).map_err(opened_py_err)?;
        read_response_dict(py, &response)
    }

    /// Poll exact commits after a previously returned engine version.
    #[pyo3(signature = (after, *, timeout_ms = 0))]
    fn changes<'py>(
        &self,
        py: Python<'py>,
        after: &Bound<'py, PyDict>,
        timeout_ms: u64,
    ) -> PyResult<Bound<'py, PyDict>> {
        let after = parse_version(after)?;
        let poll = py
            .detach(|| {
                self.inner
                    .changes(ChangeRequest { after, timeout: Duration::from_millis(timeout_ms) })
            })
            .map_err(opened_py_err)?;
        change_poll_dict(py, &poll)
    }

    /// Verify a bounded set of relative paths and return one coherent receipt.
    fn refresh<'py>(&self, py: Python<'py>, paths: Vec<PathBuf>) -> PyResult<Bound<'py, PyDict>> {
        let result = py.detach(move || self.inner.refresh(&paths)).map_err(opened_py_err)?;
        refresh_result_dict(py, &result)
    }

    /// Reorder pending discovery toward a bounded set of relative paths.
    fn prioritize(&self, py: Python<'_>, paths: Vec<PathBuf>) -> PyResult<()> {
        py.detach(move || self.inner.prioritize(&paths)).map_err(opened_py_err)
    }

    /// Cancel and join every worker owned by this handle.
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.inner.close()).map_err(opened_py_err)
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __exit__(
        &self,
        py: Python<'_>,
        _exception_type: &Bound<'_, PyAny>,
        _exception: &Bound<'_, PyAny>,
        _traceback: &Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        self.close(py)?;
        Ok(false)
    }
}

/// Register the opened-root class and its typed native errors.
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyOpenedIndex>()?;
    module.add("OpenedIndexError", module.py().get_type::<OpenedIndexError>())?;
    module.add("OpenedIndexClosedError", module.py().get_type::<OpenedIndexClosedError>())?;
    module.add("OpenedIndexStoppedError", module.py().get_type::<OpenedIndexStoppedError>())?;
    module.add("VersionUnavailableError", module.py().get_type::<VersionUnavailableError>())?;
    module.add(
        "ContinuationUnavailableError",
        module.py().get_type::<ContinuationUnavailableError>(),
    )?;
    module.add(
        "ChangeCursorUnavailableError",
        module.py().get_type::<ChangeCursorUnavailableError>(),
    )?;
    module.add("OpenedIndexLimitError", module.py().get_type::<OpenedIndexLimitError>())?;
    Ok(())
}

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(1);
            let ordinal = NEXT.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir()
                .join(format!("fdu-py-opened-{}-{ordinal}", std::process::id()));
            std::fs::create_dir(&root).expect("create test root");
            Self(root)
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn ready(opened: &OpenedIndex) -> ReadResponse {
        for _ in 0..10_000 {
            let response = opened.read(ReadRequest::default()).expect("read opened state");
            if response.state.coverage == Coverage::Complete {
                return response;
            }
            std::thread::yield_now();
        }
        panic!("tiny opened root did not complete discovery")
    }

    #[test]
    fn blocking_change_poll_releases_the_gil_for_a_python_thread() {
        Python::initialize();
        let root = TestRoot::new();
        std::fs::write(root.0.join("seed.txt"), b"seed").expect("write seed");
        let opened = OpenedIndex::open(&root.0, OpenOptions::default()).expect("open test root");
        let cursor = ready(&opened).change_cursor;
        let producer = opened.clone();
        let changed_path = root.0.join("changed.txt");

        Python::attach(|py| {
            let index =
                Py::new(py, PyOpenedIndex { inner: opened }).expect("allocate opened class");
            let worker = std::thread::spawn(move || {
                Python::attach(|_| {
                    std::fs::write(changed_path, b"changed").expect("write changed file");
                    producer
                        .refresh(&[PathBuf::from("changed.txt")])
                        .expect("refresh after acquiring the GIL");
                });
            });

            let cursor = version_dict(py, cursor).expect("convert cursor");
            let poll =
                index.borrow(py).changes(py, &cursor, 2_000).expect("poll through Python binding");
            let outcome = poll
                .get_item("outcome")
                .expect("read outcome")
                .expect("outcome present")
                .cast_into::<PyDict>()
                .expect("outcome mapping");
            assert_eq!(
                outcome
                    .get_item("kind")
                    .expect("read kind")
                    .expect("kind present")
                    .extract::<String>()
                    .expect("kind string"),
                "changes"
            );
            worker.join().expect("Python producer thread");
            index.borrow(py).close(py).expect("close opened index");
        });
    }
}
