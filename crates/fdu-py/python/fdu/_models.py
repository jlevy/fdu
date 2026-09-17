# pyright: reportUnknownArgumentType=false, reportUnknownVariableType=false
"""Immutable public values for fdu's Python API.

The native layer deliberately moves bulk dictionaries and JSON documents across the
language boundary.  This module converts each bulk result once into discoverable,
typed values; callers never need to know the private extension's wire shape.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
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
    """A report projection over the retained index.

    Declared in `ViewSpec::ALL` order, which is the order the CLI lists views in and the
    order `--view full` renders them: the summary first, then the roll-up ladder from
    coarse to fine, then the per-file views. Iterating this enum and iterating the CLI's
    own view list must give the same sequence, and a parity run compares them (fdu-ggux).
    """

    SUMMARY = "summary"
    TREE = "tree"
    FAMILIES = "families"
    TYPES = "types"
    EXTENSIONS = "extensions"
    LANGUAGES = "languages"
    DOCUMENTS = "documents"
    LARGEST = "largest"
    RECENT = "recent"
    FILES = "files"
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


class CacheScope(StrEnum):
    """Which snapshots a cache-lifecycle request covers.

    The Python API distinguishes these by function -- `cache_status` and `clear_cache`
    for one root, `list_caches` and `clear_all_caches` for the directory -- but the
    vocabulary is shared with the CLI's `--cache-status` and `--cache-clear`, so it is
    named here and asserted against `contract()` like every other shared vocabulary.
    """

    ROOT = "root"
    ALL = "all"


class CacheState(StrEnum):
    """What a path in the snapshot cache holds.

    `STALE` is one of fdu's snapshots that this build cannot serve -- an older or newer
    format, another engine, or a header this build cannot read -- and clearing removes it
    like a `CURRENT` one. `LEFTOVER` is a file fdu wrote that is not a snapshot in place,
    which `clear_all_caches` reclaims under the rules on `LeftoverKind`. `UNRECOGNIZED` is
    anything fdu cannot identify as its own, and clearing never removes it. `ABSENT` means
    nothing is there, which only a status for one root can report.
    """

    CURRENT = "current"
    STALE = "stale"
    LEFTOVER = "leftover"
    UNRECOGNIZED = "unrecognized"
    ABSENT = "absent"


class StaleReason(StrEnum):
    """Why a snapshot fdu wrote cannot be served by this build."""

    OLDER_FORMAT = "older_format"
    NEWER_FORMAT = "newer_format"
    OTHER_ENGINE = "other_engine"
    UNREADABLE = "unreadable"


class LeftoverKind(StrEnum):
    """Which of fdu's own files a `CacheState.LEFTOVER` is.

    `STAGING_TEMPORARY` is the file a killed writer left beside its target, never renamed
    into place; it is reclaimed only once it is too old to belong to a running writer.
    `ORPHANED_CONTENT` is a content sidecar whose snapshot is gone; it is reclaimed only
    while no snapshot claims it.
    """

    STAGING_TEMPORARY = "staging_temporary"
    ORPHANED_CONTENT = "orphaned_content"


class Format(StrEnum):
    """How a report is serialized.

    `TEXT` is the human rendering the command line prints, minus its performance footer:
    that footer is transient telemetry the report schema deliberately excludes, and the
    walk counts behind it are not part of a `Report`.
    """

    TEXT = "text"
    JSON = "json"
    JSONL = "jsonl"
    YAML = "yaml"


class ChangeKind(StrEnum):
    """A retained-index or watch-feed mutation."""

    UPSERT = "upsert"
    REMOVE = "remove"
    INVALIDATE = "invalidate"
    INVALIDATE_SUBTREE = "invalidate_subtree"


class Bound(StrEnum):
    """An explicitly unbounded report depth or row limit."""

    ALL = "all"


class IgnoredEntries(StrEnum):
    """Which entries a report selects by their ``.gitignore`` classification.

    A selection rather than a scan setting: every entry is classified, so choosing a side
    never rescans. The command line spells ``EXCLUDE`` and ``ONLY`` as
    ``--exclude-ignored`` and ``--only-ignored``. Anything but ``INCLUDE`` needs a scan
    that reads ``.gitignore``, and is refused under ``ScanOptions(read_controls=False)``.
    """

    #: Every entry; rows carry their ignored share.
    INCLUDE = "include"
    #: Only entries no ``.gitignore`` rule ignores.
    EXCLUDE = "exclude"
    #: Only entries a ``.gitignore`` rule ignores.
    ONLY = "only"


class ControlRefusalReason(StrEnum):
    """Which of the :class:`ControlLimits` refused a ``.gitignore`` instead of applying it.

    Each value is also the name of the limit that fired.
    """

    #: Retaining it would have taken the index past its budget.
    BUDGET = "budget"
    #: One of its lines is longer than the line limit.
    LINE_LIMIT = "line_limit"


@dataclass(frozen=True, slots=True)
class RefusedControl:
    """One ``.gitignore`` whose rules an index refused, relative to the root."""

    path: Path
    reason: ControlRefusalReason


@dataclass(frozen=True, slots=True)
class ControlLimits:
    """The two independent bounds an index applied ``.gitignore`` files under.

    The budget bounds how much control state the whole index retains; the line limit bounds
    what one pattern costs to match. Each is a byte count, or ``None`` when unbounded.
    """

    budget: int | None
    line_limit: int | None


@dataclass(frozen=True, slots=True)
class ControlObservation:
    """The ``.gitignore`` files an index applied and refused.

    Sizes and counts never depend on this. Below a refused file the ignored and unignored
    split is not exact in either direction, because the file may have held negations.
    """

    limits: ControlLimits
    applied: int
    #: Counted exactly, even when ``refusals`` is truncated.
    refused: int
    #: The first refused files in path order; shorter than ``refused`` when truncated.
    refusals: tuple[RefusedControl, ...] = ()

    @property
    def is_complete(self) -> bool:
        return self.refused == 0

    @property
    def lists_every_refusal(self) -> bool:
        return len(self.refusals) == self.refused


def _limit(value: object) -> int | None:
    return None if value is None else int(cast(int, value))


def _check_control_limits(budget: object, line_limit: object) -> None:
    """Reject a negative control limit before it crosses the native boundary."""

    for name, value in (("control_budget", budget), ("control_line_limit", line_limit)):
        if isinstance(value, int) and value < 0:
            raise ValueError(f"{name} must be non-negative or Bound.ALL")


def control_observation_from_dict(value: Mapping[str, Any]) -> ControlObservation:
    raw_limits = value["limits"]
    limits = ControlLimits(
        budget=_limit(raw_limits["budget"]), line_limit=_limit(raw_limits["line_limit"])
    )
    return ControlObservation(
        limits=limits,
        applied=int(value["applied"]),
        refused=int(value["refused"]),
        refusals=tuple(
            RefusedControl(path=Path(item["path"]), reason=ControlRefusalReason(item["reason"]))
            for item in value["refusals"]
        ),
    )


@dataclass(frozen=True, slots=True)
class ScanOptions:
    """Filesystem scope for an initial scan and later refreshes."""

    max_depth: int | None = None
    one_filesystem: bool = False
    #: Observe ``.gitignore`` control state, as the engine's ``ScanConfig.read_controls``.
    #: On by default for :func:`fdu.open`, :func:`fdu.scan`, :func:`fdu.report`, and a
    #: watch, so an index keeps the exact control state and every report row carries its
    #: ignored share. Off, no control file is read, rows carry ``ignored=None`` rather than
    #: a zero share, a selection by ``IgnoredEntries`` is refused, and the snapshot is of a
    #: separate scope. The command line spells it ``--no-gitignore``.
    read_controls: bool = True
    #: Bytes of retained ``.gitignore`` charge before further files are refused, as the
    #: engine's ``ControlLimits.budget``: an int, a size such as ``"16MiB"``, ``Bound.ALL``
    #: for no bound, which also reads every ``.gitignore`` whole however large, or ``None``
    #: for the default of 4 MiB. It never changes the line limit. A refused file ends
    #: nothing; its report's ``status.ignore_rules`` names it. Part of the snapshot scope,
    #: so a different budget scans cold once.
    control_budget: int | Bound | str | None = None
    #: Longest ``.gitignore`` line applied before its file is refused, as the engine's
    #: ``ControlLimits.line_limit``: an int, a size such as ``"64KiB"``, ``Bound.ALL`` for no
    #: bound, or ``None`` for the default of 16 KiB. It never changes the budget. Part of the
    #: snapshot scope, like ``control_budget``.
    control_line_limit: int | Bound | str | None = None

    def __post_init__(self) -> None:
        if self.max_depth is not None and self.max_depth < 0:
            raise ValueError("max_depth must be non-negative")
        _check_control_limits(self.control_budget, self.control_line_limit)


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
    #: Accepts a raw token as well as an int or `Bound`, so a caller passing user input
    #: straight through gets the library's own grammar and wording rather than having to
    #: pre-validate and invent a second opinion about what is acceptable.
    depth: int | Bound | str | None = None
    limit: int | Bound | str | None = None
    sort: SortKey | None = None
    reverse: bool = False
    size: SizeMetric = SizeMetric.ALLOCATED
    #: Entries to consider by ``.gitignore`` classification. Sizes, ordering, and
    #: ``min_size`` follow the entries selected.
    ignored: IgnoredEntries = IgnoredEntries.INCLUDE

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
    #: A raw comma-separated spec is accepted as well as a tuple, so a caller passing
    #: user input through gets the library's list grammar -- duplicate and empty-entry
    #: rejection, and `full` expansion -- rather than having to reimplement it and get a
    #: different answer than the CLI for the same string.
    views: tuple[View, ...] | str = ()
    selection: Selection = field(default_factory=Selection)
    words_per_page: int = 250

    def __post_init__(self) -> None:
        # A lone `View` is a `StrEnum` and therefore an iterable string, so passing one
        # unwrapped would iterate its characters; rejecting it here gives a clear error
        # instead of a later per-character failure. A plain `str` is something else --
        # a deliberate raw spec -- and goes to the library to be parsed by the one
        # grammar, which is why the check names the enum rather than the type it inherits.
        if isinstance(self.views, View):
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
    #: Which ``.gitignore`` files apply, or ``None`` when none was read. A refused file
    #: leaves ``complete`` true and every size exact; only the ignored and unignored split
    #: below it is not.
    ignore_rules: ControlObservation | None = None


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
    #: Keyed at the raw extension level, as ``fdu_core::classify::ext_bucket`` names it:
    #: ``file.c++`` is ``.c++``, ``release.v2.zip`` is ``.zip``, and ``Makefile`` is
    #: ``(none)``. The levels are tabulated under "Classification and Extension Levels" in
    #: docs/project/architecture/fdu-engine-architecture.md and in the ``classify`` module.
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
class IgnoredTally:
    """The part of a row's tallies that ``.gitignore`` rules ignore.

    An entry is ignored when a rule matches it or any directory above it. Counted over the
    selected entries, like the row itself. Below a ``.gitignore`` that a control limit
    refused (the budget or the line limit; ``Status.ignore_rules`` names which), the split
    is not exact in either direction; the sizes it divides are.
    """

    files: int
    dirs: int
    bytes: int
    allocated: int


@dataclass(frozen=True, slots=True)
class SummaryRow:
    files: int
    dirs: int
    bytes: int
    allocated: int
    newest_mtime_ns: int | None
    #: The ignored share, zero when nothing is ignored, or ``None`` when no ``.gitignore``
    #: was read.
    ignored: IgnoredTally | None = None


@dataclass(frozen=True, slots=True)
class ExtensionRow:
    #: The raw extension level, the same key as ``RollUp.by_extension``: ``file.c++`` is
    #: ``.c++`` and ``release.v2.zip`` is ``.zip``. See "Classification and Extension
    #: Levels" in docs/project/architecture/fdu-engine-architecture.md.
    extension: str
    files: int
    bytes: int
    allocated: int
    #: The ignored share of this extension's files, or ``None`` when no ``.gitignore`` was
    #: read.
    ignored: ExtensionTally | None = None


@dataclass(frozen=True, slots=True)
class FileRow:
    path: Path
    kind: EntryKind
    bytes: int
    allocated: int
    mtime_ns: int
    #: Whether ``.gitignore`` rules ignore this entry, or ``None`` when none was read.
    ignored: bool | None = None


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
    #: The ignored share of this subtree, or ``None`` when no ``.gitignore`` was read.
    ignored: IgnoredTally | None = None


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
    #: Remarks the report makes about itself, in the order a renderer prints them --
    #: the views `full` had to drop for want of an analyzer, and the ``.gitignore`` files a
    #: control limit refused. Carried as values rather than left inside the text
    #: rendering, because a caller reading `sections` would otherwise find one absent with
    #: no way to learn why (fdu-7wd1). Deliberately not in `as_dict`: the wire envelope
    #: excludes them, and a machine consumer reads the omission from which sections are
    #: present and the refusals from ``status.ignore_rules``.
    notes: tuple[str, ...]
    _wire: dict[str, JsonValue] = field(repr=False, compare=False)
    #: Bound renderer, supplied by `Index.report`. Absent on a report built by hand.
    _renderer: Callable[[str, bool], str] | None = field(default=None, repr=False, compare=False)

    def as_dict(self) -> dict[str, JsonValue]:
        """Return an independent copy of the exact CLI JSON schema."""

        return deepcopy(self._wire)

    def render(self, format: Format = Format.TEXT, *, color: bool = False) -> str:
        """Serialize this report the way the command line does.

        Beside `as_dict` because both are serializations of the same value -- the one this
        report was built from, not whatever the index holds now -- and splitting them
        across a method and a module function would make the pair harder to find than
        either alone.

        `color` is a plain bool rather than the CLI's `auto | always | never`: resolving
        `auto` means asking whether stdout is a terminal, and a library does not own
        stdout. The caller decides and passes the answer in.

        The report only. The command line appends a performance footer, which is transient
        telemetry the schema excludes and whose counts are not on a `Report`.
        """

        if self._renderer is None:
            # Deferred: `_api` imports this module, so its exception types cannot be
            # imported at module scope.
            from ._api import InvalidArgumentError

            raise InvalidArgumentError(
                "this report carries no renderer; only a report from Index.report, "
                "fdu.report, or Watch.report can be rendered"
            )
        return self._renderer(str(format), color)


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
    #: Whether ignore rules ignore the entry, or ``None`` when the run read none.
    ignored: bool | None = None
    reason: str | None = None

    def render(self, format: Format = Format.JSONL) -> str:
        """Render this record as ``fdu --watch`` streams it.

        The same renderer the command line uses, so a caller streaming changes emits the
        bytes fdu emits rather than a format of its own that will drift from them.
        """

        # Through `_call` like every other native call, so a bad format raises the
        # package's own `InvalidArgumentError` rather than the bare `ValueError` pyo3
        # produces -- a caller writing `except FduError` should not miss this one
        # (fdu-dygl). Deferred for the same reason as above.
        from . import _native
        from ._api import _call

        return cast(
            str,
            _call(
                _native.render_change,
                path=str(self.path),
                op=str(self.kind),
                clock=self.clock,
                kind=str(self.entry_kind) if self.entry_kind is not None else None,
                bytes=self.bytes,
                allocated=self.allocated,
                mtime_ns=self.mtime_ns,
                ignored=self.ignored,
                format=str(format),
            ),
        )


@dataclass(frozen=True, slots=True)
class ChangeSet:
    truncated: bool
    clock: int
    changes: tuple[Change, ...]


@dataclass(frozen=True, slots=True)
class EntryTierIdentity:
    """Which entries a store holds: the engine that built it, the scope it retained, and
    the type rules and reducer set its roll-ups were tallied under.

    `.gitignore` observation is not part of it, because reading rules changes which entries
    are ignored, never which exist or what they measure.
    """

    engine: int
    max_depth: int | None
    follow_symlinks: bool
    one_filesystem: bool
    hidden_fingerprint: int
    exclude_special: bool
    type_rules_fingerprint: int
    reducers_fingerprint: int


@dataclass(frozen=True, slots=True)
class ControlTierIdentity:
    """The ``.gitignore`` control tier of a store that observed rules, under its limits."""

    limits: ControlLimits


@dataclass(frozen=True, slots=True)
class SnapshotIdentity:
    """The identity of every tier a snapshot holds.

    `ignore_rules` is ``None`` when the snapshot read no ``.gitignore`` file.
    """

    entries: EntryTierIdentity
    ignore_rules: ControlTierIdentity | None


@dataclass(frozen=True, slots=True)
class ContentTierIdentity:
    """The identity of a content sidecar's records: the entry tier they were analyzed
    over, which alone holds their type rules, then the analyzer set, options, and analyzers
    as a report's `AnalysisMetadata` names them.
    """

    entries: EntryTierIdentity
    analyze: tuple[Analysis, ...]
    options_fingerprint: int
    analyzers: tuple[Analyzer, ...]


@dataclass(frozen=True, slots=True)
class ContentStatus:
    """The content sidecar fdu wrote beside a snapshot.

    `state` is `CURRENT` or `STALE`. `records` and `identity` come from the header of a
    `CURRENT` sidecar and are `None` otherwise; `stale_reason` is set only for a `STALE`
    one, and `format_version` only when the version is the reason.
    """

    bytes: int
    state: CacheState
    stale_reason: StaleReason | None
    format_version: int | None
    records: int | None
    identity: ContentTierIdentity | None


@dataclass(frozen=True, slots=True)
class CacheStatus:
    """One file in the snapshot cache.

    `root`, `entries`, and `identity` come from the header of a `CURRENT` snapshot and are
    `None` otherwise; `stale_reason` is set only for a `STALE` one, `format_version` only
    when the version is the reason, and `leftover_kind` only for a `LEFTOVER` one.
    `content` describes the sidecar beside a snapshot, current or stale, and is `None`
    when there is none.
    """

    path: Path
    bytes: int
    state: CacheState
    stale_reason: StaleReason | None
    format_version: int | None
    leftover_kind: LeftoverKind | None
    root: Path | None
    entries: int | None
    identity: SnapshotIdentity | None
    content: ContentStatus | None


@dataclass(frozen=True, slots=True)
class ClearSummary:
    """What one `clear_all_caches` removed.

    Two counts rather than one total: snapshots someone could have used, and files fdu
    itself left behind. Reporting them as one number would overstate what was cleared.
    """

    snapshots: int
    leftovers: int


def _datetime(value: object) -> datetime | None:
    if value is None:
        return None
    if not isinstance(value, str):
        raise TypeError(f"expected timestamp string, got {type(value).__name__}")
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def _entry_tier_identity(value: Mapping[str, Any]) -> EntryTierIdentity:
    return EntryTierIdentity(
        engine=int(value["engine"]),
        max_depth=_limit(value["max_depth"]),
        follow_symlinks=bool(value["follow_symlinks"]),
        one_filesystem=bool(value["one_filesystem"]),
        hidden_fingerprint=int(value["hidden_fingerprint"]),
        exclude_special=bool(value["exclude_special"]),
        type_rules_fingerprint=int(value["type_rules_fingerprint"]),
        reducers_fingerprint=int(value["reducers_fingerprint"]),
    )


def _snapshot_identity(value: Mapping[str, Any] | None) -> SnapshotIdentity | None:
    if value is None:
        return None
    raw_controls = value["ignore_rules"]
    controls = None
    if raw_controls is not None:
        raw_limits = raw_controls["limits"]
        controls = ControlTierIdentity(
            ControlLimits(
                budget=_limit(raw_limits["budget"]), line_limit=_limit(raw_limits["line_limit"])
            )
        )
    return SnapshotIdentity(entries=_entry_tier_identity(value["entries"]), ignore_rules=controls)


def _content_status(value: Mapping[str, Any] | None) -> ContentStatus | None:
    if value is None:
        return None
    raw_identity = value["identity"]
    identity = None
    if raw_identity is not None:
        identity = ContentTierIdentity(
            entries=_entry_tier_identity(raw_identity["entries"]),
            analyze=tuple(Analysis(str(name)) for name in raw_identity["analyze"]),
            options_fingerprint=int(raw_identity["options_fingerprint"]),
            analyzers=tuple(
                Analyzer(str(item["id"]), int(item["version"]))
                for item in raw_identity["analyzers"]
            ),
        )
    return ContentStatus(
        bytes=int(value["bytes"]),
        state=CacheState(value["state"]),
        stale_reason=_stale_reason(value["stale_reason"]),
        format_version=_limit(value["format_version"]),
        records=_limit(value["records"]),
        identity=identity,
    )


def _stale_reason(value: object) -> StaleReason | None:
    return None if value is None else StaleReason(str(value))


def cache_status_from_dict(value: Mapping[str, Any]) -> CacheStatus:
    return CacheStatus(
        path=Path(value["path"]),
        bytes=int(value["bytes"]),
        state=CacheState(value["state"]),
        stale_reason=_stale_reason(value["stale_reason"]),
        format_version=_limit(value["format_version"]),
        leftover_kind=(
            LeftoverKind(value["leftover_kind"]) if value["leftover_kind"] is not None else None
        ),
        root=Path(value["root"]) if value["root"] is not None else None,
        entries=_limit(value["entries"]),
        identity=_snapshot_identity(value["identity"]),
        content=_content_status(value["content"]),
    )


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


def _ignore_rules(value: object) -> ControlObservation | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise TypeError("ignore_rules must be an object or null")
    return control_observation_from_dict(cast(dict[str, Any], value))


def status_from_dict(value: dict[str, Any]) -> Status:
    return Status(
        complete=bool(value["complete"]),
        freshness=Freshness(str(value["freshness"])),
        source=ReportSource(str(value["source"])),
        errors=tuple(_operation_error(item) for item in value.get("errors", [])),
        ignore_rules=_ignore_rules(value["ignore_rules"]),
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


def _ignored_tally(value: object) -> IgnoredTally | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise TypeError("an ignored share must be an object or null")
    share = cast(dict[str, Any], value)
    return IgnoredTally(
        files=int(share["files"]),
        dirs=int(share["dirs"]),
        bytes=int(share["bytes"]),
        allocated=int(share["allocated"]),
    )


def _ignored_files(value: object) -> ExtensionTally | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise TypeError("an ignored share must be an object or null")
    share = cast(dict[str, Any], value)
    return ExtensionTally(
        files=int(share["files"]), bytes=int(share["bytes"]), allocated=int(share["allocated"])
    )


def _ignored_flag(value: object) -> bool | None:
    if value is None:
        return None
    if not isinstance(value, bool):
        raise TypeError("a file row's ignored flag must be a boolean or null")
    return value


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
        ignored=_ignored_tally(value["ignored"]),
    )


def report_from_dict(wire: dict[str, Any], notes: tuple[str, ...] = ()) -> Report:
    """
    Parse the exact CLI JSON object into immutable public values.

    Takes ownership of `wire`: the report retains it as its wire form, so the caller
    must not mutate it afterwards. `Report.as_dict()` hands out independent copies.

    `notes` comes from the report itself rather than from `wire`, because the wire
    envelope deliberately excludes them; the producer reads them off the same handle it
    rendered from and passes them in.
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
            sections.append(
                SummarySection(
                    view,
                    SummaryRow(
                        files=int(row["files"]),
                        dirs=int(row["dirs"]),
                        bytes=int(row["bytes"]),
                        allocated=int(row["allocated"]),
                        newest_mtime_ns=(
                            int(row["newest_mtime_ns"])
                            if row["newest_mtime_ns"] is not None
                            else None
                        ),
                        ignored=_ignored_tally(row["ignored"]),
                    ),
                )
            )
        elif view is View.EXTENSIONS:
            rows = raw["extensions"]
            if not isinstance(rows, list):
                raise TypeError("extensions section must be a list")
            sections.append(
                ExtensionsSection(
                    view,
                    tuple(
                        ExtensionRow(
                            extension=str(row["extension"]),
                            files=int(row["files"]),
                            bytes=int(row["bytes"]),
                            allocated=int(row["allocated"]),
                            ignored=_ignored_files(row["ignored"]),
                        )
                        for row in rows
                    ),
                    _bound(raw),
                )
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
                            ignored=_ignored_flag(row["ignored"]),
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
        ignore_rules=_ignore_rules(wire["ignore_rules"]),
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
        notes=notes,
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
