"""Long-lived opened-root API over the native fdu engine.

The public objects in this module are immutable value carriers. ``OpenedIndex`` owns
only the native shared handle; it adds validation and conversion but no scheduler,
journal, cache, or application-specific query layer.
"""

from __future__ import annotations

import math
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field, replace
from datetime import UTC, datetime
from enum import StrEnum
from pathlib import Path
from types import TracebackType
from typing import Any, Literal, cast

from . import _native
from ._api import FduError, FilesystemError, InvalidArgumentError, _epoch_nanos, _query_kwargs
from ._models import EntryKind, Freshness, Query, Report, Selection, ValueSource, report_from_dict

__all__ = [
    "Aggregate",
    "AggregateResult",
    "Attributes",
    "ChangeCursorUnavailableError",
    "ChangeOutcome",
    "ChangeOutcomeKind",
    "ChangePoll",
    "Commit",
    "Continuation",
    "ContinuationUnavailableError",
    "Continue",
    "ControlIdentity",
    "Count",
    "CountKind",
    "CoverageKind",
    "CoverageReason",
    "CoverageState",
    "Diagnostics",
    "DiagnosticsResult",
    "DirectoryRollUp",
    "DiscoveryProgress",
    "EffectiveChange",
    "EffectiveChangeKind",
    "EngineVersion",
    "Entry",
    "EntryKind",
    "EntrySelection",
    "Flat",
    "FlatPage",
    "FlatResult",
    "Freshness",
    "Impact",
    "ImpactDomain",
    "InvalidateReason",
    "Issue",
    "IssueKind",
    "IssueSummary",
    "Knowledge",
    "KnowledgeKind",
    "LifecyclePhase",
    "LimitResult",
    "LimitedProjection",
    "Lookup",
    "LookupResult",
    "NameClassification",
    "OpenedIndex",
    "OpenedIndexClosedError",
    "OpenedIndexError",
    "OpenedIndexLimitError",
    "OpenedIndexStoppedError",
    "OpenedOptions",
    "OpenedState",
    "Page",
    "PartitionRollUpSummary",
    "PortablePathEncoding",
    "PortablePathExample",
    "PortablePathIssue",
    "Projection",
    "ProjectionResult",
    "Query",
    "ReadDiagnostics",
    "ReadResponse",
    "RefreshReceipt",
    "RefreshRejection",
    "RejectedRefreshPath",
    "Report",
    "ReportProjection",
    "ReportResult",
    "RollUpResult",
    "RollUpSummary",
    "RowShape",
    "ScanScope",
    "ScopeIdentity",
    "Selection",
    "SemanticIdentity",
    "StateTransition",
    "StateTransitionKind",
    "Tree",
    "TreePage",
    "TreeResult",
    "ValueSource",
    "VersionUnavailableError",
    "Work",
]


class OpenedIndexError(FduError):
    """An opened-root operation failed after its arguments were accepted."""


class OpenedIndexClosedError(OpenedIndexError):
    """The shared opened root has begun or completed shutdown."""


class OpenedIndexStoppedError(OpenedIndexError):
    """A resource-stopped root cannot perform expanding work."""


class VersionUnavailableError(OpenedIndexError):
    """A coherent read version is foreign, stale, or otherwise unavailable."""


class ContinuationUnavailableError(OpenedIndexError):
    """A page continuation is foreign, consumed, evicted, or unavailable."""


class ChangeCursorUnavailableError(OpenedIndexError):
    """A change cursor is foreign, incompatible, or in the future."""


class OpenedIndexLimitError(OpenedIndexError):
    """A request exceeded one of the opened engine's explicit public bounds."""


class LifecyclePhase(StrEnum):
    DISCOVERING = "discovering"
    RECONCILING = "reconciling"
    READY = "ready"
    WATCHING = "watching"
    STOPPED = "stopped"
    FAILED = "failed"


class CoverageKind(StrEnum):
    COMPLETE = "complete"
    PARTIAL = "partial"


class CoverageReason(StrEnum):
    BUILDING = "building"
    BUDGET = "budget"
    CANCELLED = "cancelled"
    INACCESSIBLE = "inaccessible"
    FAILED = "failed"


class IssueKind(StrEnum):
    PERMISSION = "permission"
    DISAPPEARED = "disappeared"
    INVALID_METADATA = "invalid_metadata"
    RESOURCE_BUDGET = "resource_budget"
    OBSERVATION_GAP = "observation_gap"
    PROVIDER_FAILURE = "provider_failure"


class ImpactDomain(StrEnum):
    TOPOLOGY = "topology"
    METADATA = "metadata"
    CLASSIFICATION = "classification"
    AGGREGATES = "aggregates"
    CONTENT = "content"
    STATE = "state"


class RowShape(StrEnum):
    COMPACT = "compact"
    FULL = "full"


class KnowledgeKind(StrEnum):
    PRESENT = "present"
    ABSENT = "absent"
    UNKNOWN = "unknown"


class CountKind(StrEnum):
    EXACT = "exact"
    AT_LEAST = "at_least"


class ChangeOutcomeKind(StrEnum):
    CHANGES = "changes"
    IDLE = "idle"
    RESET = "reset"


class PortablePathEncoding(StrEnum):
    UNIX_BYTES = "unix_bytes"
    WINDOWS_WTF16_LE = "windows_wtf16_le"
    PLATFORM_BYTES = "platform_bytes"


class LimitedProjection(StrEnum):
    TREE = "tree"
    FLAT = "flat"
    REPORT = "report"
    AGGREGATE = "aggregate"


class EffectiveChangeKind(StrEnum):
    INSERTED = "inserted"
    UPDATED = "updated"
    REMOVED = "removed"
    CONTROL_UPDATED = "control_updated"
    RECLASSIFIED = "reclassified"
    INVALIDATED = "invalidated"


class InvalidateReason(StrEnum):
    WATCH_OVERFLOW = "watch_overflow"
    UNPAIRED_RENAME = "unpaired_rename"
    WATCH_SETUP_RACE = "watch_setup_race"
    PERIODIC_SWEEP = "periodic_sweep"
    VERIFICATION_FAILED = "verification_failed"
    UNKNOWN_ANCESTRY = "unknown_ancestry"
    WATCH_CONTENTION = "watch_contention"
    REQUESTED = "requested"


class StateTransitionKind(StrEnum):
    FRESHNESS = "freshness"
    VERIFIED = "verified"
    DIRECTORY_COMPLETE = "directory_complete"
    INDEX_STATE = "index_state"


class RefreshRejection(StrEnum):
    OUTSIDE_ROOT = "outside_root"
    BEYOND_DEPTH = "beyond_depth"
    NOT_ADMITTED = "not_admitted"
    UNSAFE_ANCESTRY = "unsafe_ancestry"
    RESOURCE_BUDGET = "resource_budget"


@dataclass(frozen=True, slots=True)
class OpenedOptions:
    """Scope, resource, observation, and journal policy for one opened root."""

    batch_size: int | None = None
    follow_symlinks: bool = False
    one_filesystem: bool = False
    prune_hidden: bool = False
    hidden_allow: tuple[str, ...] = ()
    exclude_special: bool = False
    max_files: int | None = None
    observe: bool = False
    journal_capacity: int | None = None

    def __post_init__(self) -> None:
        if isinstance(self.hidden_allow, str):
            raise TypeError("hidden_allow must be a tuple of names, not a string")
        for name, value in (
            ("batch_size", self.batch_size),
            ("journal_capacity", self.journal_capacity),
        ):
            if value is not None and value <= 0:
                raise ValueError(f"{name} must be positive")
        if self.max_files is not None and self.max_files <= 0:
            raise ValueError("max_files must be positive")
        if self.hidden_allow and not self.prune_hidden:
            raise ValueError("hidden_allow requires prune_hidden=True")


@dataclass(frozen=True, slots=True)
class ScopeIdentity:
    max_depth: int | None
    follow_symlinks: bool
    one_filesystem: bool
    hidden_fingerprint: int
    exclude_special: bool


@dataclass(frozen=True, slots=True)
class SemanticIdentity:
    ignore_rules_fingerprint: int
    type_rules_fingerprint: int
    reducers_fingerprint: int


@dataclass(frozen=True, slots=True)
class EngineVersion:
    session: int
    sequence: int
    scope: ScopeIdentity
    semantics: SemanticIdentity


@dataclass(frozen=True, slots=True)
class CoverageState:
    kind: CoverageKind
    reason: CoverageReason | None


@dataclass(frozen=True, slots=True)
class DiscoveryProgress:
    files_retained: int
    directories_complete: int


@dataclass(frozen=True, slots=True)
class IssueSummary:
    retained: int
    omitted: int


@dataclass(frozen=True, slots=True)
class OpenedState:
    phase: LifecyclePhase
    coverage: CoverageState
    freshness: Freshness
    source: ValueSource
    progress: DiscoveryProgress
    issues: IssueSummary


@dataclass(frozen=True, slots=True)
class Work:
    observations: int
    unchanged: int
    stale: int
    resource_refused: int
    rows_visited: int
    rows_returned: int
    maintained_index_work: int
    commits_visited: int
    commits_returned: int
    directories_read: int
    entries_visited: int
    files_visited: int
    bytes_visited: int


@dataclass(frozen=True, slots=True)
class Attributes:
    size: int
    allocated: int
    mtime_ns: int
    ctime_ns: int
    inode: int
    dev: int


@dataclass(frozen=True, slots=True)
class RollUpSummary:
    files: int
    dirs: int
    bytes: int
    allocated: int
    newest_mtime_ns: int | None


@dataclass(frozen=True, slots=True)
class PartitionRollUpSummary:
    all: RollUpSummary
    unignored: RollUpSummary


@dataclass(frozen=True, slots=True)
class NameClassification:
    logical_extension: str | None
    canonical_extension: str | None
    kind_id: str | None
    family_id: str | None
    group_id: str | None
    content_family: str


@dataclass(frozen=True, slots=True)
class Entry:
    path: Path
    portable_path: str | None
    kind: EntryKind
    attrs: Attributes
    ignored: bool
    classification: NameClassification | None
    rollup: PartitionRollUpSummary | None
    children_complete: bool | None


@dataclass(frozen=True, slots=True)
class Issue:
    kind: IssueKind
    path: Path | None
    message: str
    os_error: int | None


@dataclass(frozen=True, slots=True)
class Continuation:
    session: int
    ordinal: int


@dataclass(frozen=True, slots=True)
class PortablePathExample:
    encoding: PortablePathEncoding
    encoded_hex: str
    truncated: bool


@dataclass(frozen=True, slots=True)
class PortablePathIssue:
    omitted: int
    examples: tuple[PortablePathExample, ...]


@dataclass(frozen=True, slots=True)
class Page:
    limit: int = 256
    max_work: int = 100_000

    def __post_init__(self) -> None:
        if self.limit <= 0:
            raise ValueError("page limit must be positive")
        if self.max_work <= 0:
            raise ValueError("page max_work must be positive")


@dataclass(frozen=True, slots=True)
class Lookup:
    path: Path | str


@dataclass(frozen=True, slots=True)
class DirectoryRollUp:
    path: Path | str = ""


@dataclass(frozen=True, slots=True)
class Tree:
    path: Path | str = ""
    page: Page = field(default_factory=Page)


@dataclass(frozen=True, slots=True)
class EntrySelection:
    """Portable opened-root row predicates composed with the one-shot query selection."""

    query: Selection = field(default_factory=Selection)
    max_size: int | None = None
    exclude_ignored: bool = False
    logical_extensions: tuple[str, ...] = ()
    exact_names: tuple[str, ...] = ()
    terminal_extensions: tuple[str, ...] = ()
    ancestor_names: tuple[str, ...] = ()

    def __post_init__(self) -> None:
        if self.max_size is not None and self.max_size < 0:
            raise ValueError("entry selection max_size must be nonnegative")


@dataclass(frozen=True, slots=True)
class Flat:
    selection: EntrySelection = field(default_factory=EntrySelection)
    shape: RowShape = RowShape.COMPACT
    page: Page = field(default_factory=Page)


@dataclass(frozen=True, slots=True)
class Aggregate:
    selection: EntrySelection = field(default_factory=EntrySelection)
    count_cap: int = 10_000
    max_work: int = 100_000

    def __post_init__(self) -> None:
        if self.count_cap <= 0 or self.max_work <= 0:
            raise ValueError("aggregate count_cap and max_work must be positive")


@dataclass(frozen=True, slots=True)
class ReportProjection:
    query: Query = field(default_factory=Query)
    generated_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    max_work: int = 100_000

    def __post_init__(self) -> None:
        if self.max_work <= 0:
            raise ValueError("report max_work must be positive")


@dataclass(frozen=True, slots=True)
class Continue:
    continuation: Continuation
    page: Page = field(default_factory=Page)


@dataclass(frozen=True, slots=True)
class Diagnostics:
    """Request fixed-size owner, scope, and issue diagnostics."""


type Projection = (
    Lookup | DirectoryRollUp | Tree | Flat | Aggregate | ReportProjection | Continue | Diagnostics
)


@dataclass(frozen=True, slots=True)
class Knowledge[T]:
    kind: KnowledgeKind
    value: T | None
    reason: CoverageReason | None = None


@dataclass(frozen=True, slots=True)
class TreePage:
    directory: Entry
    rows: tuple[Entry, ...]
    next: Continuation | None
    native_complete: bool
    portable_complete: bool
    portable_issue: PortablePathIssue | None


@dataclass(frozen=True, slots=True)
class FlatPage:
    rows: tuple[Entry, ...]
    next: Continuation | None
    portable_issue: PortablePathIssue | None


@dataclass(frozen=True, slots=True)
class Count:
    kind: CountKind
    value: int


@dataclass(frozen=True, slots=True)
class ScanScope:
    max_depth: int | None
    follow_symlinks: bool
    one_filesystem: bool
    hidden_fingerprint: int
    exclude_special: bool
    ignore_rules_fingerprint: int
    type_rules_fingerprint: int
    reducers_fingerprint: int


@dataclass(frozen=True, slots=True)
class ReadDiagnostics:
    root: Path
    scope: ScanScope
    entries: int
    issues: tuple[Issue, ...]


@dataclass(frozen=True, slots=True)
class LookupResult:
    kind: Literal["lookup"]
    value: Knowledge[Entry]


@dataclass(frozen=True, slots=True)
class RollUpResult:
    kind: Literal["rollup"]
    value: Knowledge[PartitionRollUpSummary]


@dataclass(frozen=True, slots=True)
class TreeResult:
    kind: Literal["tree"]
    value: Knowledge[TreePage]


@dataclass(frozen=True, slots=True)
class FlatResult:
    kind: Literal["flat"]
    value: FlatPage


@dataclass(frozen=True, slots=True)
class AggregateResult:
    kind: Literal["aggregate"]
    value: Count


@dataclass(frozen=True, slots=True)
class ReportResult:
    kind: Literal["report"]
    value: Report


@dataclass(frozen=True, slots=True)
class DiagnosticsResult:
    kind: Literal["diagnostics"]
    value: ReadDiagnostics


@dataclass(frozen=True, slots=True)
class LimitResult:
    kind: Literal["limit"]
    projection: LimitedProjection
    max_work: int
    rows_visited: int


type ProjectionResult = (
    LookupResult
    | RollUpResult
    | TreeResult
    | FlatResult
    | AggregateResult
    | ReportResult
    | DiagnosticsResult
    | LimitResult
)


@dataclass(frozen=True, slots=True)
class ReadResponse:
    version: EngineVersion
    state: OpenedState
    results: tuple[ProjectionResult, ...]
    work: Work
    change_cursor: EngineVersion


@dataclass(frozen=True, slots=True)
class Impact:
    domains: tuple[ImpactDomain, ...]
    dirty_paths: tuple[Path, ...]
    all_dirty: bool


@dataclass(frozen=True, slots=True)
class ControlIdentity:
    bytes: int
    fingerprint: int


@dataclass(frozen=True, slots=True)
class EffectiveChange:
    kind: EffectiveChangeKind
    path: Path
    entry_kind: EntryKind | None = None
    attrs: Attributes | None = None
    previous_attrs: Attributes | None = None
    current_attrs: Attributes | None = None
    previous_control: ControlIdentity | None = None
    current_control: ControlIdentity | None = None
    previous_ignored: bool | None = None
    current_ignored: bool | None = None
    reason: InvalidateReason | None = None


@dataclass(frozen=True, slots=True)
class StateTransition:
    kind: StateTransitionKind
    path: Path | None = None
    previous_freshness: Freshness | None = None
    current_freshness: Freshness | None = None
    previous_state: OpenedState | None = None
    current_state: OpenedState | None = None


@dataclass(frozen=True, slots=True)
class Commit:
    sequence: int
    changes: tuple[EffectiveChange, ...]
    impact: Impact
    state: tuple[StateTransition, ...]
    work: Work


@dataclass(frozen=True, slots=True)
class ChangeOutcome:
    kind: ChangeOutcomeKind
    commits: tuple[Commit, ...] = ()
    impact: Impact | None = None


@dataclass(frozen=True, slots=True)
class ChangePoll:
    cursor: EngineVersion
    version: EngineVersion
    state: OpenedState
    outcome: ChangeOutcome
    work: Work


@dataclass(frozen=True, slots=True)
class RejectedRefreshPath:
    path: Path
    reason: RefreshRejection


@dataclass(frozen=True, slots=True)
class RefreshReceipt:
    after: EngineVersion
    version: EngineVersion
    state: OpenedState
    accepted: tuple[Path, ...]
    rejected: tuple[RejectedRefreshPath, ...]
    impact: Impact
    work: Work
    issues: tuple[Issue, ...]
    omitted_issues: int


def _mapping(value: object, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{label} must be a mapping")
    return cast(dict[str, Any], value)


def _sequence(value: object, label: str) -> list[Any]:
    if not isinstance(value, list):
        raise TypeError(f"{label} must be a list")
    return cast(list[Any], value)


def _scope(value: object) -> ScopeIdentity:
    raw = _mapping(value, "scope identity")
    return ScopeIdentity(
        max_depth=int(raw["max_depth"]) if raw["max_depth"] is not None else None,
        follow_symlinks=bool(raw["follow_symlinks"]),
        one_filesystem=bool(raw["one_filesystem"]),
        hidden_fingerprint=int(raw["hidden_fingerprint"]),
        exclude_special=bool(raw["exclude_special"]),
    )


def _semantics(value: object) -> SemanticIdentity:
    raw = _mapping(value, "semantic identity")
    return SemanticIdentity(
        ignore_rules_fingerprint=int(raw["ignore_rules_fingerprint"]),
        type_rules_fingerprint=int(raw["type_rules_fingerprint"]),
        reducers_fingerprint=int(raw["reducers_fingerprint"]),
    )


def _version(value: object) -> EngineVersion:
    raw = _mapping(value, "engine version")
    return EngineVersion(
        session=int(raw["session"]),
        sequence=int(raw["sequence"]),
        scope=_scope(raw["scope"]),
        semantics=_semantics(raw["semantics"]),
    )


def _coverage(value: object) -> CoverageState:
    raw = _mapping(value, "coverage")
    reason = raw.get("reason")
    return CoverageState(
        kind=CoverageKind(str(raw["kind"])),
        reason=CoverageReason(str(reason)) if reason is not None else None,
    )


def _state(value: object) -> OpenedState:
    raw = _mapping(value, "opened state")
    progress = _mapping(raw["progress"], "discovery progress")
    issues = _mapping(raw["issues"], "issue summary")
    return OpenedState(
        phase=LifecyclePhase(str(raw["phase"])),
        coverage=_coverage(raw["coverage"]),
        freshness=Freshness(str(raw["freshness"])),
        source=ValueSource(str(raw["source"])),
        progress=DiscoveryProgress(
            files_retained=int(progress["files_retained"]),
            directories_complete=int(progress["directories_complete"]),
        ),
        issues=IssueSummary(retained=int(issues["retained"]), omitted=int(issues["omitted"])),
    )


def _work(value: object) -> Work:
    raw = _mapping(value, "work")
    return Work(**{name: int(raw[name]) for name in Work.__dataclass_fields__})


def _attrs(value: object) -> Attributes:
    raw = _mapping(value, "attributes")
    return Attributes(**{name: int(raw[name]) for name in Attributes.__dataclass_fields__})


def _rollup(value: object) -> RollUpSummary:
    raw = _mapping(value, "roll-up summary")
    return RollUpSummary(
        files=int(raw["files"]),
        dirs=int(raw["dirs"]),
        bytes=int(raw["bytes"]),
        allocated=int(raw["allocated"]),
        newest_mtime_ns=(
            int(raw["newest_mtime_ns"]) if raw["newest_mtime_ns"] is not None else None
        ),
    )


def _partition_rollup(value: object) -> PartitionRollUpSummary:
    raw = _mapping(value, "partition roll-up")
    return PartitionRollUpSummary(all=_rollup(raw["all"]), unignored=_rollup(raw["unignored"]))


def _entry(value: object) -> Entry:
    raw = _mapping(value, "entry")
    rollup = raw.get("rollup")
    raw_classification = raw.get("classification")
    classification = (
        _name_classification(raw_classification) if raw_classification is not None else None
    )
    return Entry(
        path=Path(raw["path"]),
        portable_path=str(raw["portable_path"]) if raw["portable_path"] is not None else None,
        kind=EntryKind(str(raw["kind"])),
        attrs=_attrs(raw["attrs"]),
        ignored=bool(raw["ignored"]),
        classification=classification,
        rollup=_partition_rollup(rollup) if rollup is not None else None,
        children_complete=(
            bool(raw["children_complete"]) if raw["children_complete"] is not None else None
        ),
    )


def _name_classification(value: object) -> NameClassification:
    raw = _mapping(value, "name classification")
    return NameClassification(
        logical_extension=(
            str(raw["logical_extension"]) if raw["logical_extension"] is not None else None
        ),
        canonical_extension=(
            str(raw["canonical_extension"]) if raw["canonical_extension"] is not None else None
        ),
        kind_id=str(raw["kind_id"]) if raw["kind_id"] is not None else None,
        family_id=str(raw["family_id"]) if raw["family_id"] is not None else None,
        group_id=str(raw["group_id"]) if raw["group_id"] is not None else None,
        content_family=str(raw["content_family"]),
    )


def _issue(value: object) -> Issue:
    raw = _mapping(value, "issue")
    return Issue(
        kind=IssueKind(str(raw["kind"])),
        path=Path(raw["path"]) if raw["path"] is not None else None,
        message=str(raw["message"]),
        os_error=int(raw["os_error"]) if raw["os_error"] is not None else None,
    )


def _continuation(value: object) -> Continuation:
    raw = _mapping(value, "continuation")
    return Continuation(session=int(raw["session"]), ordinal=int(raw["ordinal"]))


def _portable_issue(value: object) -> PortablePathIssue:
    raw = _mapping(value, "portable path issue")
    return PortablePathIssue(
        omitted=int(raw["omitted"]),
        examples=tuple(
            PortablePathExample(
                encoding=PortablePathEncoding(str(item["encoding"])),
                encoded_hex=str(item["encoded_hex"]),
                truncated=bool(item["truncated"]),
            )
            for item in (_mapping(entry, "portable path example") for entry in raw["examples"])
        ),
    )


def _knowledge[T](value: object, converter: Callable[[object], T]) -> Knowledge[T]:
    raw = _mapping(value, "knowledge")
    kind = KnowledgeKind(str(raw["knowledge"]))
    reason = raw.get("reason")
    converted = converter(raw["value"]) if kind is KnowledgeKind.PRESENT else None
    return Knowledge(
        kind=kind,
        value=converted,
        reason=CoverageReason(str(reason)) if reason is not None else None,
    )


def _tree_page(value: object) -> TreePage:
    raw = _mapping(value, "tree page")
    portable_issue = raw.get("portable_issue")
    continuation = raw.get("next")
    return TreePage(
        directory=_entry(raw["directory"]),
        rows=tuple(_entry(row) for row in _sequence(raw["rows"], "tree rows")),
        next=_continuation(continuation) if continuation is not None else None,
        native_complete=bool(raw["native_complete"]),
        portable_complete=bool(raw["portable_complete"]),
        portable_issue=_portable_issue(portable_issue) if portable_issue is not None else None,
    )


def _flat_page(value: object) -> FlatPage:
    raw = _mapping(value, "flat page")
    portable_issue = raw.get("portable_issue")
    continuation = raw.get("next")
    return FlatPage(
        rows=tuple(_entry(row) for row in _sequence(raw["rows"], "flat rows")),
        next=_continuation(continuation) if continuation is not None else None,
        portable_issue=_portable_issue(portable_issue) if portable_issue is not None else None,
    )


def _scan_scope(value: object) -> ScanScope:
    raw = _mapping(value, "scan scope")
    return ScanScope(
        max_depth=int(raw["max_depth"]) if raw["max_depth"] is not None else None,
        follow_symlinks=bool(raw["follow_symlinks"]),
        one_filesystem=bool(raw["one_filesystem"]),
        hidden_fingerprint=int(raw["hidden_fingerprint"]),
        exclude_special=bool(raw["exclude_special"]),
        ignore_rules_fingerprint=int(raw["ignore_rules_fingerprint"]),
        type_rules_fingerprint=int(raw["type_rules_fingerprint"]),
        reducers_fingerprint=int(raw["reducers_fingerprint"]),
    )


def _diagnostics(value: object) -> ReadDiagnostics:
    raw = _mapping(value, "read diagnostics")
    return ReadDiagnostics(
        root=Path(raw["root"]),
        scope=_scan_scope(raw["scope"]),
        entries=int(raw["entries"]),
        issues=tuple(_issue(issue) for issue in _sequence(raw["issues"], "diagnostic issues")),
    )


def _projection_result(value: object) -> ProjectionResult:
    raw = _mapping(value, "projection result")
    kind = str(raw["kind"])
    payload = raw["value"]
    if kind == "lookup":
        return LookupResult("lookup", _knowledge(payload, _entry))
    if kind == "rollup":
        return RollUpResult("rollup", _knowledge(payload, _partition_rollup))
    if kind == "tree":
        return TreeResult("tree", _knowledge(payload, _tree_page))
    if kind == "flat":
        return FlatResult("flat", _flat_page(payload))
    if kind == "aggregate":
        count = _mapping(payload, "count")
        return AggregateResult(
            "aggregate", Count(CountKind(str(count["kind"])), int(count["value"]))
        )
    if kind == "report":
        report = _mapping(payload, "report result")
        wire = _mapping(report["wire"], "report wire value")
        notes = tuple(str(note) for note in _sequence(report["notes"], "report notes"))
        handle = report["renderer"]

        def renderer(format: str, color: bool) -> str:
            return cast(str, _opened_call(handle.render, format, color))

        return ReportResult(
            "report",
            replace(report_from_dict(wire, notes), _renderer=renderer),
        )
    if kind == "diagnostics":
        return DiagnosticsResult("diagnostics", _diagnostics(payload))
    if kind == "limit":
        limit = _mapping(payload, "query limit")
        return LimitResult(
            "limit",
            projection=LimitedProjection(str(limit["projection"])),
            max_work=int(limit["max_work"]),
            rows_visited=int(limit["rows_visited"]),
        )
    raise TypeError(f"unknown native projection result kind {kind!r}")


def _read_response(value: object) -> ReadResponse:
    raw = _mapping(value, "read response")
    return ReadResponse(
        version=_version(raw["version"]),
        state=_state(raw["state"]),
        results=tuple(
            _projection_result(result) for result in _sequence(raw["results"], "projection results")
        ),
        work=_work(raw["work"]),
        change_cursor=_version(raw["change_cursor"]),
    )


def _impact(value: object) -> Impact:
    raw = _mapping(value, "impact")
    return Impact(
        domains=tuple(ImpactDomain(str(domain)) for domain in raw["domains"]),
        dirty_paths=tuple(Path(path) for path in raw["dirty_paths"]),
        all_dirty=bool(raw["all_dirty"]),
    )


def _control_identity(value: object) -> ControlIdentity:
    raw = _mapping(value, "control identity")
    return ControlIdentity(bytes=int(raw["bytes"]), fingerprint=int(raw["fingerprint"]))


def _effective_change(value: object) -> EffectiveChange:
    raw = _mapping(value, "effective change")
    kind = EffectiveChangeKind(str(raw["kind"]))
    return EffectiveChange(
        kind=kind,
        path=Path(raw["path"]),
        entry_kind=EntryKind(str(raw["entry_kind"])) if "entry_kind" in raw else None,
        attrs=_attrs(raw["attrs"]) if "attrs" in raw else None,
        previous_attrs=_attrs(raw["previous"]) if kind is EffectiveChangeKind.UPDATED else None,
        current_attrs=_attrs(raw["current"]) if kind is EffectiveChangeKind.UPDATED else None,
        previous_control=(
            _control_identity(raw["previous"])
            if kind is EffectiveChangeKind.CONTROL_UPDATED and raw["previous"] is not None
            else None
        ),
        current_control=(
            _control_identity(raw["current"])
            if kind is EffectiveChangeKind.CONTROL_UPDATED and raw["current"] is not None
            else None
        ),
        previous_ignored=(
            bool(raw["previous_ignored"]) if kind is EffectiveChangeKind.RECLASSIFIED else None
        ),
        current_ignored=(
            bool(raw["current_ignored"]) if kind is EffectiveChangeKind.RECLASSIFIED else None
        ),
        reason=(
            InvalidateReason(str(raw["reason"]))
            if kind is EffectiveChangeKind.INVALIDATED
            else None
        ),
    )


def _transition(value: object) -> StateTransition:
    raw = _mapping(value, "state transition")
    kind = StateTransitionKind(str(raw["kind"]))
    return StateTransition(
        kind=kind,
        path=Path(raw["path"]) if "path" in raw else None,
        previous_freshness=(
            Freshness(str(raw["previous"])) if kind is StateTransitionKind.FRESHNESS else None
        ),
        current_freshness=(
            Freshness(str(raw["current"])) if kind is StateTransitionKind.FRESHNESS else None
        ),
        previous_state=(
            _state(raw["previous"]) if kind is StateTransitionKind.INDEX_STATE else None
        ),
        current_state=(_state(raw["current"]) if kind is StateTransitionKind.INDEX_STATE else None),
    )


def _commit(value: object) -> Commit:
    raw = _mapping(value, "commit")
    return Commit(
        sequence=int(raw["sequence"]),
        changes=tuple(
            _effective_change(change) for change in _sequence(raw["changes"], "effective changes")
        ),
        impact=_impact(raw["impact"]),
        state=tuple(
            _transition(transition) for transition in _sequence(raw["state"], "state transitions")
        ),
        work=_work(raw["work"]),
    )


def _change_poll(value: object) -> ChangePoll:
    raw = _mapping(value, "change poll")
    outcome = _mapping(raw["outcome"], "change outcome")
    kind = ChangeOutcomeKind(str(outcome["kind"]))
    impact = outcome.get("impact")
    return ChangePoll(
        cursor=_version(raw["cursor"]),
        version=_version(raw["version"]),
        state=_state(raw["state"]),
        outcome=ChangeOutcome(
            kind=kind,
            commits=tuple(
                _commit(commit)
                for commit in _sequence(outcome.get("commits", []), "change commits")
            ),
            impact=_impact(impact) if impact is not None else None,
        ),
        work=_work(raw["work"]),
    )


def _refresh_receipt(value: object) -> RefreshReceipt:
    raw = _mapping(value, "refresh receipt")
    return RefreshReceipt(
        after=_version(raw["after"]),
        version=_version(raw["version"]),
        state=_state(raw["state"]),
        accepted=tuple(Path(path) for path in raw["accepted"]),
        rejected=tuple(
            RejectedRefreshPath(Path(item["path"]), RefreshRejection(str(item["reason"])))
            for item in (
                _mapping(rejected, "rejected refresh path")
                for rejected in _sequence(raw["rejected"], "rejected refresh paths")
            )
        ),
        impact=_impact(raw["impact"]),
        work=_work(raw["work"]),
        issues=tuple(_issue(issue) for issue in _sequence(raw["issues"], "refresh issues")),
        omitted_issues=int(raw["omitted_issues"]),
    )


def _version_wire(version: EngineVersion) -> dict[str, object]:
    return {
        "session": version.session,
        "sequence": version.sequence,
        "scope": {
            "max_depth": version.scope.max_depth,
            "follow_symlinks": version.scope.follow_symlinks,
            "one_filesystem": version.scope.one_filesystem,
            "hidden_fingerprint": version.scope.hidden_fingerprint,
            "exclude_special": version.scope.exclude_special,
        },
        "semantics": {
            "ignore_rules_fingerprint": version.semantics.ignore_rules_fingerprint,
            "type_rules_fingerprint": version.semantics.type_rules_fingerprint,
            "reducers_fingerprint": version.semantics.reducers_fingerprint,
        },
    }


def _page_wire(page: Page) -> dict[str, int]:
    return {"limit": page.limit, "max_work": page.max_work}


def _selection_wire(selection: Selection) -> dict[str, object]:
    values = _query_kwargs(Query(selection=selection))
    values.pop("views")
    values.pop("words_per_page")
    return values


def _entry_selection_wire(selection: EntrySelection) -> dict[str, object]:
    values = _selection_wire(selection.query)
    values.update(
        {
            "max_size": selection.max_size,
            "exclude_ignored": selection.exclude_ignored,
            "logical_extensions": selection.logical_extensions,
            "exact_names": selection.exact_names,
            "terminal_extensions": selection.terminal_extensions,
            "ancestor_names": selection.ancestor_names,
        }
    )
    return values


def _projection_wire(projection: Projection) -> dict[str, object]:
    if isinstance(projection, Lookup):
        return {"kind": "lookup", "path": projection.path}
    if isinstance(projection, DirectoryRollUp):
        return {"kind": "rollup", "path": projection.path}
    if isinstance(projection, Tree):
        return {"kind": "tree", "path": projection.path, "page": _page_wire(projection.page)}
    if isinstance(projection, Flat):
        return {
            "kind": "flat",
            "selection": _entry_selection_wire(projection.selection),
            "shape": projection.shape.value,
            "page": _page_wire(projection.page),
        }
    if isinstance(projection, Aggregate):
        return {
            "kind": "aggregate",
            "selection": _entry_selection_wire(projection.selection),
            "count_cap": projection.count_cap,
            "max_work": projection.max_work,
        }
    if isinstance(projection, ReportProjection):
        query = _query_kwargs(projection.query)
        views = query.pop("views")
        words_per_page = query.pop("words_per_page")
        return {
            "kind": "report",
            "request": {
                "selection": query,
                "views": views,
                "words_per_page": words_per_page,
                "generated_at_ns": _epoch_nanos(projection.generated_at),
                "max_work": projection.max_work,
            },
        }
    if isinstance(projection, Continue):
        return {
            "kind": "continue",
            "continuation": {
                "session": projection.continuation.session,
                "ordinal": projection.continuation.ordinal,
            },
            "page": _page_wire(projection.page),
        }
    return {"kind": "diagnostics"}


def _opened_call(function: Callable[..., Any], /, *args: object, **kwargs: object) -> Any:
    try:
        return function(*args, **kwargs)
    except _native.OpenedIndexClosedError as error:
        raise OpenedIndexClosedError(str(error)) from error
    except _native.OpenedIndexStoppedError as error:
        raise OpenedIndexStoppedError(str(error)) from error
    except _native.VersionUnavailableError as error:
        raise VersionUnavailableError(str(error)) from error
    except _native.ContinuationUnavailableError as error:
        raise ContinuationUnavailableError(str(error)) from error
    except _native.ChangeCursorUnavailableError as error:
        raise ChangeCursorUnavailableError(str(error)) from error
    except _native.OpenedIndexLimitError as error:
        raise OpenedIndexLimitError(str(error)) from error
    except _native.OpenedIndexError as error:
        raise OpenedIndexError(str(error)) from error
    except OSError as error:
        raise FilesystemError(error.errno, error.strerror, error.filename) from error
    except ValueError as error:
        raise InvalidArgumentError(str(error)) from error


class OpenedIndex:
    """Synchronous, long-lived inventory over one filesystem root.

    All substantial native work releases the GIL. Async applications should adapt the
    blocking methods with their own executor policy so task lifetime remains owned by
    the application rather than hidden inside this package.
    """

    __slots__ = ("_native",)

    def __init__(self, native: _native.OpenedIndex) -> None:
        self._native = native

    @classmethod
    def open(
        cls,
        root: str | Path,
        options: OpenedOptions | None = None,
    ) -> OpenedIndex:
        """Open ``root`` and begin progressive discovery."""

        selected = options if options is not None else OpenedOptions()
        native = _opened_call(
            _native.OpenedIndex.open,
            root,
            batch_size=selected.batch_size,
            follow_symlinks=selected.follow_symlinks,
            one_filesystem=selected.one_filesystem,
            prune_hidden=selected.prune_hidden,
            hidden_allow=list(selected.hidden_allow),
            exclude_special=selected.exclude_special,
            max_files=selected.max_files,
            observe=selected.observe,
            journal_capacity=selected.journal_capacity,
        )
        return cls(cast(_native.OpenedIndex, native))

    def read(
        self,
        *projections: Projection,
        expected: EngineVersion | None = None,
    ) -> ReadResponse:
        """Return all requested projections from one coherent committed boundary."""

        raw = _opened_call(
            self._native.read,
            [_projection_wire(projection) for projection in projections],
            expected=_version_wire(expected) if expected is not None else None,
        )
        return _read_response(raw)

    def state(self) -> ReadResponse:
        """Return the current version and state without a variable-size projection."""

        return _read_response(_opened_call(self._native.state))

    def changes(self, after: EngineVersion, *, timeout: float = 0.0) -> ChangePoll:
        """Wait up to ``timeout`` seconds for exact commits after ``after``."""

        if not math.isfinite(timeout) or timeout < 0:
            raise ValueError("timeout must be finite and non-negative")
        timeout_ms = math.ceil(timeout * 1_000)
        if timeout_ms > 2**64 - 1:
            raise ValueError("timeout is too large")
        return _change_poll(
            _opened_call(self._native.changes, _version_wire(after), timeout_ms=timeout_ms)
        )

    def refresh(self, paths: Sequence[str | Path]) -> RefreshReceipt:
        """Verify a bounded path set and return one coherent interval receipt."""

        if isinstance(paths, (str, Path)):
            raise TypeError("refresh takes a sequence of paths; wrap one path in a tuple")
        return _refresh_receipt(_opened_call(self._native.refresh, list(paths)))

    def prioritize(self, paths: Sequence[str | Path]) -> None:
        """Move pending progressive discovery toward a bounded path set."""

        if isinstance(paths, (str, Path)):
            raise TypeError("prioritize takes a sequence of paths; wrap one path in a tuple")
        _opened_call(self._native.prioritize, list(paths))

    def close(self) -> None:
        """Cancel and join all work owned by this shared opened root."""

        _opened_call(self._native.close)

    def __enter__(self) -> OpenedIndex:
        return self

    def __exit__(
        self,
        _exception_type: type[BaseException] | None,
        _exception: BaseException | None,
        _traceback: TracebackType | None,
    ) -> bool:
        self.close()
        return False
