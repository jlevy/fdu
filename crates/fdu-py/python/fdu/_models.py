# pyright: reportUnknownArgumentType=false, reportUnknownVariableType=false
"""Immutable public values for fdu's Python API.

The native layer deliberately moves bulk dictionaries and JSON documents across the
language boundary.  This module converts each bulk result once into discoverable,
typed values; callers never need to know the private extension's wire shape.
"""

from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass, field
from datetime import datetime
from enum import StrEnum
from pathlib import Path
from types import MappingProxyType
from typing import Any, cast

type JsonScalar = bool | int | float | str | None
type JsonValue = JsonScalar | list[JsonValue] | dict[str, JsonValue]


class CachePolicy(StrEnum):
    """How :func:`fdu.open` may use the on-disk snapshot cache."""

    AUTO = "auto"
    REFRESH = "refresh"
    READ_ONLY = "read-only"
    ONLY = "only"
    OFF = "off"


class Freshness(StrEnum):
    """Whether indexed state has been verified against the filesystem now."""

    FRESH = "fresh"
    RECONCILING = "reconciling"
    STALE = "stale"
    PARTIAL = "partial"


class ReportSource(StrEnum):
    """The cache tier that produced an index or report."""

    COLD_SCAN = "cold_scan"
    WARM_REVALIDATE = "warm_revalidate"
    CACHE_ONLY = "cache_only"


class ValueSource(StrEnum):
    """Where one retained entry's value came from."""

    SCANNED = "scanned"
    REVALIDATED = "revalidated"
    JOURNAL_SCOPED = "journal_scoped"
    CACHED = "cached"


class Coverage(StrEnum):
    """Whether a value covers everything beneath its path that is in scope."""

    COMPLETE = "complete"
    PARTIAL = "partial"


class View(StrEnum):
    """A report projection over the retained index."""

    TREE = "tree"
    EXTENSIONS = "extensions"
    TYPES = "types"
    FAMILIES = "families"
    LANGUAGES = "languages"
    DOCUMENTS = "documents"
    LARGEST = "largest"
    RECENT = "recent"
    FILES = "files"
    SUMMARY = "summary"
    FULL = "full"


class EntryKind(StrEnum):
    """Filesystem entry kind."""

    FILE = "file"
    DIR = "dir"
    SYMLINK = "symlink"
    OTHER = "other"


class SizeMetric(StrEnum):
    """Size used for ordering, limits, and percentages."""

    ALLOCATED = "allocated"
    APPARENT = "apparent"


class SortKey(StrEnum):
    """Report row ordering."""

    SIZE = "size"
    COUNT = "count"
    MTIME = "mtime"
    NAME = "name"


class Analysis(StrEnum):
    """A value the content axis accepts: one analyzer, or a total naming the whole axis.

    Analyzers compose, so a request is a comma-separated set -- ``"code,words"`` runs
    both. ``NONE`` and ``ALL`` name the whole axis and cannot be combined with anything
    else. ``LINES`` comes free with any analyzer, because a file being read for one
    metric is already being counted for the other.
    """

    NONE = "none"
    LINES = "lines"
    CODE = "code"
    WORDS = "words"
    ALL = "all"


class ChangeKind(StrEnum):
    """A retained-index or watch-feed mutation."""

    UPSERT = "upsert"
    REMOVE = "remove"
    INVALIDATE = "invalidate"
    INVALIDATE_SUBTREE = "invalidate_subtree"


class Bound(StrEnum):
    """An explicitly unbounded report depth or row limit."""

    ALL = "all"


@dataclass(frozen=True, slots=True)
class ScanOptions:
    """Filesystem scope for an initial scan and later refreshes."""

    max_depth: int | None = None
    one_filesystem: bool = False

    def __post_init__(self) -> None:
        if self.max_depth is not None and self.max_depth < 0:
            raise ValueError("max_depth must be non-negative")


@dataclass(frozen=True, slots=True)
class AnalysisOptions:
    """Content analysis requested while opening or scanning."""

    #: Analyzers to run, as one :class:`Analysis` value or a comma-separated set.
    analyze: str = Analysis.NONE
    workers: int = 0

    def __post_init__(self) -> None:
        if self.workers < 0:
            raise ValueError("workers must be non-negative")


@dataclass(frozen=True, slots=True)
class Selection:
    """Rows selected from an already-built index."""

    include: tuple[str, ...] = ()
    exclude: tuple[str, ...] = ()
    min_size: int | str | None = None
    modified_since: datetime | str | None = None
    modified_before: datetime | str | None = None
    kinds: tuple[EntryKind, ...] = ()
    depth: int | Bound | None = None
    limit: int | Bound | None = None
    sort: SortKey | None = None
    reverse: bool = False
    size: SizeMetric = SizeMetric.ALLOCATED

    def __post_init__(self) -> None:
        # A bare string is iterable, so without this guard `include="*.rs"` would run
        # as the per-character patterns `*`, `.`, `r`, `s` and silently match far too
        # much. `StrEnum` members are strings, so this also catches a bare kind.
        for name, value in (
            ("include", self.include),
            ("exclude", self.exclude),
            ("kinds", self.kinds),
        ):
            if isinstance(value, str):
                raise TypeError(f"{name} takes a tuple of values; wrap the single value in a tuple")
        for name, value in (("depth", self.depth), ("limit", self.limit)):
            if isinstance(value, int) and value < 0:
                raise ValueError(f"{name} must be non-negative or Bound.ALL")
        if isinstance(self.min_size, int) and self.min_size < 0:
            raise ValueError("min_size must be non-negative")


@dataclass(frozen=True, slots=True)
class Query:
    """One or more report views generated without rescanning."""

    #: Views to report. Empty means "let the requested analyzers choose", which is what
    #: the command line does: asking to read files and then printing a directory tree
    #: containing none of the results is the defect the content axis removed.
    views: tuple[View, ...] = ()
    selection: Selection = field(default_factory=Selection)
    words_per_page: int = 250

    def __post_init__(self) -> None:
        # A lone `View` is a `StrEnum` and therefore an iterable string; rejecting it
        # here gives a clear error instead of a later per-character failure.
        if isinstance(self.views, str):
            raise TypeError("views takes a tuple of View values; wrap the single view in a tuple")
        if self.words_per_page <= 0:
            raise ValueError("words_per_page must be positive")


@dataclass(frozen=True, slots=True)
class WatchOptions:
    """Configuration for an event-driven change feed."""

    interval: float = 2.0
    query: Query = field(default_factory=lambda: Query(views=(View.FILES,)))

    def __post_init__(self) -> None:
        if self.interval <= 0:
            raise ValueError("interval must be positive")


@dataclass(frozen=True, slots=True)
class OperationError:
    """One non-fatal operational condition that made a result partial."""

    path: Path | None
    kind: str
    message: str
    os_error: int | None = None


@dataclass(frozen=True, slots=True)
class Status:
    """Independent coverage, currency, origin, and error facts."""

    complete: bool
    freshness: Freshness
    source: ReportSource
    errors: tuple[OperationError, ...] = ()


@dataclass(frozen=True, slots=True)
class Provenance:
    """Origin, observation time, and coverage for one retained value."""

    source: ValueSource
    observed_at_ns: int
    status: Coverage


@dataclass(frozen=True, slots=True)
class ExtensionTally:
    files: int
    bytes: int
    allocated: int


@dataclass(frozen=True, slots=True)
class RollUp:
    files: int
    dirs: int
    bytes: int
    allocated: int
    newest_mtime_ns: int
    by_extension: MappingProxyType[str, ExtensionTally]
    provenance: Provenance | None = None


@dataclass(frozen=True, slots=True)
class Child:
    name: str
    kind: EntryKind
    rollup: RollUp | None
    bytes: int | None
    allocated: int | None
    mtime_ns: int | None
    provenance: Provenance


@dataclass(frozen=True, slots=True)
class SummaryRow:
    files: int
    dirs: int
    bytes: int
    allocated: int
    newest_mtime_ns: int | None


@dataclass(frozen=True, slots=True)
class ExtensionRow:
    extension: str
    files: int
    bytes: int
    allocated: int


@dataclass(frozen=True, slots=True)
class FileRow:
    path: Path
    kind: EntryKind
    bytes: int
    allocated: int
    mtime_ns: int


@dataclass(frozen=True, slots=True)
class TreeNode:
    name: str
    path: Path
    kind: EntryKind
    bytes: int
    allocated: int
    files: int
    dirs: int
    newest_mtime_ns: int | None
    truncated: bool
    children: tuple[TreeNode, ...]


@dataclass(frozen=True, slots=True)
class MetricValues:
    physical_lines: int
    blank_lines: int
    nonblank_lines: int
    code_lines: int
    comment_lines: int
    code_blank_lines: int
    raw_words: int
    logical_words: int
    paragraphs: int
    visible_words: int
    visible_logical_words: int
    document_words: int


@dataclass(frozen=True, slots=True)
class MetricShare:
    numerator: int
    denominator: int


@dataclass(frozen=True, slots=True)
class Detection:
    sources: MappingProxyType[str, int]
    confidence: MappingProxyType[str, int]
    generated: int
    vendored: int
    documentation: int


@dataclass(frozen=True, slots=True)
class MetricRow:
    id: str
    family: str
    files: int
    bytes: int
    allocated: int
    analyzed_files: int
    share: MetricShare
    metrics: MetricValues
    coverage: MappingProxyType[str, int]
    detection: Detection
    page_words: int
    words_per_page: int


@dataclass(frozen=True, slots=True)
class Analyzer:
    id: str
    version: int


@dataclass(frozen=True, slots=True)
class AnalysisMetadata:
    #: The analyzers this report requested, in canonical order.
    analyze: tuple[Analysis, ...]
    type_rules_fingerprint: int
    options_fingerprint: int
    analyzers: tuple[Analyzer, ...]


@dataclass(frozen=True, slots=True)
class SummarySection:
    view: View
    summary: SummaryRow


@dataclass(frozen=True, slots=True)
class SectionBound:
    """What a section dropped when a limit applied.

    Named against the existing `Bound`, which is the *requested* limit on the selection
    axis. Rust keeps the two in separate modules; Python's namespace is flat, so the
    distinction has to be in the name.

    ``None`` on a section rather than an absent attribute, so a consumer branches on the
    value: twenty rows of 192,871 look complete unless the report says otherwise.
    """

    shown: int
    total: int


@dataclass(frozen=True, slots=True)
class ExtensionsSection:
    view: View
    extensions: tuple[ExtensionRow, ...]
    bound: SectionBound | None = None


@dataclass(frozen=True, slots=True)
class FilesSection:
    """A flat listing: ``files``, or one of its bounded presets.

    ``view`` distinguishes them, because ``largest`` and ``recent`` produce this shape too.
    """

    view: View
    files: tuple[FileRow, ...]
    bound: SectionBound | None = None


@dataclass(frozen=True, slots=True)
class TreeSection:
    view: View
    tree: TreeNode


@dataclass(frozen=True, slots=True)
class MetricsSection:
    view: View
    group: str
    share_metric: str
    words_per_page: int
    total: MetricRow
    rows: tuple[MetricRow, ...]
    bound: SectionBound | None = None


type ReportSection = (
    SummarySection | ExtensionsSection | FilesSection | TreeSection | MetricsSection
)


@dataclass(frozen=True, slots=True)
class Report:
    """One immutable multi-view report plus its exact CLI wire representation."""

    schema: str
    generator: str
    root: Path
    scan_started_at: datetime | None
    generated_at: datetime
    status: Status
    analysis: AnalysisMetadata | None
    sections: tuple[ReportSection, ...]
    _wire: dict[str, JsonValue] = field(repr=False, compare=False)

    def as_dict(self) -> dict[str, JsonValue]:
        """Return an independent copy of the exact CLI JSON schema."""

        return deepcopy(self._wire)


@dataclass(frozen=True, slots=True)
class RefreshResult:
    inserted: int
    updated: int
    removed: int
    unchanged: int
    stale: int
    clock: int
    status: Status


@dataclass(frozen=True, slots=True)
class Change:
    clock: int
    path: Path
    kind: ChangeKind
    entry_kind: EntryKind | None = None
    bytes: int | None = None
    allocated: int | None = None
    mtime_ns: int | None = None
    reason: str | None = None


@dataclass(frozen=True, slots=True)
class ChangeSet:
    truncated: bool
    clock: int
    changes: tuple[Change, ...]


@dataclass(frozen=True, slots=True)
class CacheStatus:
    path: Path
    bytes: int
    content_bytes: int | None
    recognized: bool
    root: Path | None
    entries: int | None
    max_depth: int | None
    one_filesystem: bool | None


def _datetime(value: object) -> datetime | None:
    if value is None:
        return None
    if not isinstance(value, str):
        raise TypeError(f"expected timestamp string, got {type(value).__name__}")
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def _int_map(value: dict[str, Any]) -> MappingProxyType[str, int]:
    return MappingProxyType({str(key): int(item) for key, item in value.items()})


def _operation_error(value: object) -> OperationError:
    if isinstance(value, str):
        return OperationError(None, "operation", value)
    if not isinstance(value, dict):
        raise TypeError("expected an operation error")
    error = cast(dict[str, Any], value)
    path = error.get("path")
    return OperationError(
        Path(str(path)) if path is not None else None,
        str(error.get("kind", "operation")),
        str(error.get("message", "")),
        int(error["os_error"]) if error.get("os_error") is not None else None,
    )


def status_from_dict(value: dict[str, Any]) -> Status:
    return Status(
        complete=bool(value["complete"]),
        freshness=Freshness(str(value["freshness"])),
        source=ReportSource(str(value["source"])),
        errors=tuple(_operation_error(item) for item in value.get("errors", [])),
    )


def provenance_from_dict(value: dict[str, Any]) -> Provenance:
    return Provenance(
        source=ValueSource(str(value["source"])),
        observed_at_ns=int(value["observed_at_ns"]),
        status=Coverage(str(value["status"])),
    )


def rollup_from_dict(value: dict[str, Any], provenance: Provenance | None = None) -> RollUp:
    tallies = {
        str(extension): ExtensionTally(
            files=int(tally["files"]),
            bytes=int(tally["bytes"]),
            allocated=int(tally["allocated"]),
        )
        for extension, tally in value["by_extension"].items()
    }
    return RollUp(
        files=int(value["files"]),
        dirs=int(value["dirs"]),
        bytes=int(value["bytes"]),
        allocated=int(value["allocated"]),
        newest_mtime_ns=int(value["newest_mtime_ns"]),
        by_extension=MappingProxyType(tallies),
        provenance=provenance,
    )


def _metric_row(value: dict[str, Any]) -> MetricRow:
    metrics = value["metrics"]
    detection = value["detection"]
    flags = detection["flags"]
    pages = value["pages"]
    return MetricRow(
        id=str(value["id"]),
        family=str(value["family"]),
        files=int(value["files"]),
        bytes=int(value["bytes"]),
        allocated=int(value["allocated"]),
        analyzed_files=int(value["analyzed_files"]),
        share=MetricShare(int(value["share"]["numerator"]), int(value["share"]["denominator"])),
        metrics=MetricValues(**{name: int(item) for name, item in metrics.items()}),
        coverage=_int_map(value["coverage"]),
        detection=Detection(
            sources=_int_map(detection["sources"]),
            confidence=_int_map(detection["confidence"]),
            generated=int(flags["generated"]),
            vendored=int(flags["vendored"]),
            documentation=int(flags["documentation"]),
        ),
        page_words=int(pages["words"]),
        words_per_page=int(pages["words_per_page"]),
    )


def _tree(value: dict[str, Any]) -> TreeNode:
    return TreeNode(
        name=str(value["name"]),
        path=Path(str(value["path"])),
        kind=EntryKind(str(value["kind"])),
        bytes=int(value["bytes"]),
        allocated=int(value["allocated"]),
        files=int(value["files"]),
        dirs=int(value["dirs"]),
        newest_mtime_ns=(
            int(value["newest_mtime_ns"]) if value.get("newest_mtime_ns") is not None else None
        ),
        truncated=bool(value["truncated"]),
        children=tuple(_tree(child) for child in value["children"]),
    )


def report_from_dict(wire: dict[str, Any]) -> Report:
    """
    Parse the exact CLI JSON object into immutable public values.

    Takes ownership of `wire`: the report retains it as its wire form, so the caller
    must not mutate it afterwards. `Report.as_dict()` hands out independent copies.
    """

    def _bound(raw: dict[str, Any]) -> SectionBound | None:
        value = raw.get("bound")
        if value is None:
            return None
        if not isinstance(value, dict):
            raise TypeError("section bound must be an object or null")
        return SectionBound(shown=int(value["shown"]), total=int(value["total"]))

    sections: list[ReportSection] = []
    raw_sections = wire["reports"]
    if not isinstance(raw_sections, list):
        raise TypeError("report sections must be a list")
    for raw in raw_sections:
        if not isinstance(raw, dict):
            raise TypeError("report section must be an object")
        view = View(str(raw["view"]))
        if view is View.SUMMARY:
            row = raw["summary"]
            if not isinstance(row, dict):
                raise TypeError("summary section must be an object")
            sections.append(SummarySection(view, SummaryRow(**row)))
        elif view is View.EXTENSIONS:
            rows = raw["extensions"]
            if not isinstance(rows, list):
                raise TypeError("extensions section must be a list")
            sections.append(
                ExtensionsSection(view, tuple(ExtensionRow(**row) for row in rows), _bound(raw))
            )
        elif view in (View.FILES, View.LARGEST, View.RECENT):
            rows = raw["files"]
            if not isinstance(rows, list):
                raise TypeError("files section must be a list")
            sections.append(
                FilesSection(
                    view,
                    tuple(
                        FileRow(
                            path=Path(str(row["path"])),
                            kind=EntryKind(str(row["kind"])),
                            bytes=int(row["bytes"]),
                            allocated=int(row["allocated"]),
                            mtime_ns=int(row["mtime_ns"]),
                        )
                        for row in rows
                    ),
                    _bound(raw),
                )
            )
        elif view is View.TREE:
            tree = raw["tree"]
            if not isinstance(tree, dict):
                raise TypeError("tree section must be an object")
            sections.append(TreeSection(view, _tree(tree)))
        else:
            metrics = raw["metrics"]
            if not isinstance(metrics, dict):
                raise TypeError("metrics section must be an object")
            rows = metrics["rows"]
            total = metrics["total"]
            if not isinstance(rows, list) or not isinstance(total, dict):
                raise TypeError("metrics section must carry a rows list and a total object")
            sections.append(
                MetricsSection(
                    view=view,
                    group=str(metrics["group"]),
                    share_metric=str(metrics["share_metric"]),
                    words_per_page=int(metrics["words_per_page"]),
                    total=_metric_row(total),
                    rows=tuple(_metric_row(row) for row in rows),
                    bound=_bound(metrics),
                )
            )

    raw_errors = wire.get("errors", [])
    if not isinstance(raw_errors, list):
        raise TypeError("report errors must be a list")
    status = Status(
        complete=bool(wire["complete"]),
        freshness=Freshness(str(wire["freshness"])),
        source=ReportSource(str(wire["source"])),
        errors=tuple(_operation_error(item) for item in raw_errors),
    )
    raw_analysis = wire.get("analysis")
    analysis = None
    if isinstance(raw_analysis, dict):
        raw_analyzers = raw_analysis["analyzers"]
        if not isinstance(raw_analyzers, list):
            raise TypeError("analysis analyzers must be a list")
        analysis = AnalysisMetadata(
            analyze=tuple(Analysis(str(name)) for name in raw_analysis["analyze"]),
            type_rules_fingerprint=int(raw_analysis["type_rules_fingerprint"]),
            options_fingerprint=int(raw_analysis["options_fingerprint"]),
            analyzers=tuple(
                Analyzer(str(item["id"]), int(item["version"])) for item in raw_analyzers
            ),
        )
    generated_at = _datetime(wire["generated_at"])
    if generated_at is None:
        raise TypeError("report generated_at must be present")
    return Report(
        schema=str(wire["schema"]),
        generator=str(wire["generator"]),
        root=Path(str(wire["root"])),
        scan_started_at=_datetime(wire.get("scan_started_at")),
        generated_at=generated_at,
        status=status,
        analysis=analysis,
        sections=tuple(sections),
        _wire=cast(dict[str, JsonValue], wire),
    )
