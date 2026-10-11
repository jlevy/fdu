# pyright: reportUnknownArgumentType=false, reportUnknownVariableType=false
"""Immutable public values for fdu's Python API.

The native layer deliberately moves bulk dictionaries and JSON documents across the
language boundary.  This module converts each bulk result once into discoverable,
typed values; callers never need to know the private extension's wire shape.
"""

from __future__ import annotations

import math
import os
from collections.abc import Callable, Mapping
from dataclasses import dataclass, field
from datetime import UTC, datetime, timedelta
from enum import StrEnum
from pathlib import Path
from types import MappingProxyType
from typing import Any, cast

from . import _native

type JsonScalar = bool | int | float | str | None
type JsonValue = JsonScalar | list[JsonValue] | dict[str, JsonValue]


def _wire_path(value: Mapping[str, Any], name: str = "path") -> Path:
    """Decode a display path, preferring its lossless machine companion when present."""

    raw = value.get(f"{name}_raw")
    if raw is None:
        return Path(str(value[name]))
    if not isinstance(raw, Mapping):
        raise TypeError(f"{name}_raw must be an object")
    raw = cast(Mapping[str, Any], raw)
    encoding = str(raw.get("encoding"))
    try:
        payload = bytes.fromhex(str(raw["hex"]))
    except (KeyError, ValueError) as error:
        raise ValueError(f"{name}_raw must contain hexadecimal bytes") from error
    if encoding == "unix-bytes":
        return Path(os.fsdecode(payload))
    if encoding == "windows-wtf16le":
        if len(payload) % 2:
            raise ValueError(f"{name}_raw has an odd-length UTF-16 payload")
        return Path(payload.decode("utf-16-le", errors="surrogatepass"))
    raise ValueError(f"unsupported {name}_raw encoding: {encoding}")


def _copy_json(value: JsonValue) -> JsonValue:
    """Copy a JSON value without consuming Python's call stack."""

    if not isinstance(value, (dict, list)):
        return value
    root: dict[str, JsonValue] | list[JsonValue] = {} if isinstance(value, dict) else []
    stack: list[
        tuple[dict[str, JsonValue] | list[JsonValue], dict[str, JsonValue] | list[JsonValue]]
    ] = [(value, root)]
    while stack:
        source, target = stack.pop()
        items = source.items() if isinstance(source, dict) else enumerate(source)
        for key, item in items:
            if isinstance(item, dict):
                copied: JsonValue = {}
            elif isinstance(item, list):
                copied = []
            else:
                copied = item
            if isinstance(target, dict):
                target[str(key)] = copied
            else:
                target.append(copied)
            if isinstance(item, (dict, list)):
                stack.append((item, cast(dict[str, JsonValue] | list[JsonValue], copied)))
    return root


class CachePolicy(StrEnum):
    """Whether :func:`fdu.open` and :func:`fdu.report` may read and write the snapshot cache.

    ``AUTO`` uses the cache where the kind of request gains from it: a one-shot metadata
    report neither reads nor writes it, while content analysis and an index from
    :func:`fdu.open` read, revalidate, and write it. ``ON`` also writes after a one-shot
    report, leaving a snapshot for a later ``stale_ok`` answer. ``OFF`` neither reads nor
    writes.
    """

    AUTO = "auto"
    ON = "on"
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

    LIST = "list"
    SUMMARY = "summary"
    TREE = "tree"
    FAMILIES = "families"
    TYPES = "types"
    EXTENSIONS = "extensions"
    LANGUAGES = "languages"
    CODE = "code"
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


#: The metric a selection answers in when the caller names none, from the request model's
#: defaults table. Read rather than written out, so one table decides what every surface
#: answers: this default was the one place a Rust caller got another metric than everyone
#: else.
_DEFAULT_SIZE = SizeMetric(_native.DEFAULT_SIZE)


class SortKey(StrEnum):
    """Report row ordering."""

    SIZE = "size"
    COUNT = "count"
    MTIME = "mtime"
    NAME = "name"

    PHYSICAL_LINES = "physical_lines"
    BLANK_LINES = "blank_lines"
    NONBLANK_LINES = "nonblank_lines"
    RAW_WORDS = "raw_words"
    CODE_LINES = "code_lines"
    COMMENT_LINES = "comment_lines"
    CODE_BLANK_LINES = "code_blank_lines"
    LOGICAL_WORDS = "logical_words"
    PARAGRAPHS = "paragraphs"
    VISIBLE_WORDS = "visible_words"
    VISIBLE_LOGICAL_WORDS = "visible_logical_words"
    DOCUMENT_WORDS = "document_words"


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


class ContentState(StrEnum):
    """What the content sidecar beside a snapshot holds.

    Narrower than `CacheState`, and the two values are spelled the same: a sidecar is
    reported only where one was found beside a snapshot, so it is never `LEFTOVER`,
    `UNRECOGNIZED`, or `ABSENT` here. A sidecar with no snapshot is a file of its own in
    the cache listing, and that one is `CacheState.LEFTOVER`.
    """

    CURRENT = "current"
    STALE = "stale"


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
    TREE = "tree"
    PATHS = "paths"
    LONG = "long"
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

    The one-shot report derives discovery and analysis from this population. Exclusion
    prunes safely ignored subtrees; ``ONLY`` discovers matches through ordinary ancestors.
    A retained Include index can answer either narrower selection without scanning.
    The command line spells this ``--ignored=include|exclude|only``. Anything but
    ``INCLUDE`` requires observed ignore controls.
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
    #: Position of the file's root in :attr:`Report.roots` over several roots, which
    #: ``path`` is relative to; ``None`` over one root.
    root: int | None = None


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
    rules: int
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
        rules=int(value["rules"]),
        refused=int(value["refused"]),
        refusals=tuple(
            RefusedControl(
                path=_wire_path(item),
                reason=ControlRefusalReason(item["reason"]),
                root=_root_position(item),
            )
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
    read_controls: bool = _native.DEFAULT_READ_CONTROLS
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
    """Content analysis requested while opening or scanning.

    The control axis for file contents. :func:`fdu.report` builds its own basis, so a
    content view in its query requests its analyzer and these options add only what the
    views do not imply: analysis shown in a metadata view such as ``View.LANGUAGES``, or
    run without display to warm the sidecar. :func:`fdu.open` and :func:`fdu.scan` fix a
    basis that serves many reads, so they run exactly the analyzers named here, and a later
    ``index.report`` never widens it.
    """

    #: Analyzers to run beyond what the views imply, as one :class:`Analysis` value or a
    #: comma-separated set.
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
    breadth: int | Bound | str | None = None
    min_share: str | None = None
    sort: SortKey | None = None
    reverse: bool = False
    size: SizeMetric = _DEFAULT_SIZE
    #: Entries to consider by ``.gitignore`` classification. Sizes, ordering, and
    #: ``min_size`` follow the entries selected.
    ignored: IgnoredEntries | None = None

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
        for name, value in (
            ("depth", self.depth),
            ("limit", self.limit),
            ("breadth", self.breadth),
        ):
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
    #: ``View.CODE`` and ``View.DOCUMENTS`` have no metadata meaning, so naming one asks
    #: for its analysis: :func:`fdu.report` runs the ``code`` or ``words`` analyzer for it,
    #: while ``index.report`` on an index opened without that analyzer raises
    #: :class:`InvalidArgumentError` naming it. Every other view, ``full`` included, reads
    #: no file contents of its own accord.
    #: A raw comma-separated spec is accepted as well as a tuple, so a caller passing
    #: user input through gets the library's list grammar -- duplicate and empty-entry
    #: rejection, and `full` expansion -- rather than having to reimplement it and get a
    #: different answer than the CLI for the same string.
    views: tuple[View, ...] | str = ()
    selection: Selection = field(default_factory=Selection)
    words_per_page: int = _native.DEFAULT_WORDS_PER_PAGE
    #: Select the report projection before reading; the default retains the directory tree.
    format: Format = Format.TEXT

    def __post_init__(self) -> None:
        # A lone `View` is a `StrEnum` and therefore an iterable string, so passing one
        # unwrapped would iterate its characters; rejecting it here gives a clear error
        # instead of a later per-character failure. A plain `str` is something else --
        # a deliberate raw spec -- and goes to the library to be parsed by the one
        # grammar, which is why the check names the enum rather than the type it inherits.
        if isinstance(self.views, View):
            raise TypeError("views takes a tuple of View values; wrap the single view in a tuple")


@dataclass(frozen=True, slots=True)
class WatchOptions:
    """Configuration for an event-driven change feed."""

    interval: float = _native.DEFAULT_WATCH_INTERVAL_SECONDS
    #: What the watch answers. A default `Query` takes the request model's own defaults, so
    #: a watch shows what a report of the same index shows; this named `files` of its own,
    #: which made one request mean two things depending on which door it came through.
    query: Query = field(default_factory=Query)

    def __post_init__(self) -> None:
        if (
            not math.isfinite(self.interval)
            or self.interval < _native.MIN_WATCH_INTERVAL_SECONDS
            or self.interval > _native.MAX_WATCH_INTERVAL_SECONDS
        ):
            raise ValueError("interval must be finite, positive, and within the supported range")


@dataclass(frozen=True, slots=True)
class OperationError:
    """One non-fatal operational condition that made a result partial."""

    path: Path | None
    kind: str
    message: str
    os_error: int | None = None
    #: Position of the error's root in :attr:`Report.roots` over several roots, which
    #: ``path`` is relative to; ``None`` over one root.
    root: int | None = None


@dataclass(frozen=True, slots=True)
class Status:
    """Completeness and bounded operational failure detail."""

    complete: bool
    coverage: Coverage
    coverage_reason: str | None
    errors: tuple[OperationError, ...] = ()
    errors_omitted: int = 0
    #: Which ``.gitignore`` files apply, or ``None`` when none was read. A refused file
    #: leaves ``complete`` true and every size exact; only the ignored and unignored split
    #: below it is not.
    ignore_rules: ControlObservation | None = None


@dataclass(frozen=True, slots=True)
class TierState:
    """Source, currency, and observation time for one retained tier."""

    source: ValueSource
    freshness: Freshness
    observed_at_ns: int | None


@dataclass(frozen=True, slots=True)
class TierProvenance:
    entries: TierState
    content: TierState | None


@dataclass(frozen=True, slots=True)
class ReportProvenance:
    """Delivery facts for one coherent report answer."""

    source: ReportSource
    freshness: Freshness
    scan_started_at: datetime | None
    generated_at: datetime
    tiers: TierProvenance


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
class FileClassification:
    file_type: str
    family: str
    source: str
    confidence: str
    generated: bool
    vendored: bool
    documentation: bool


@dataclass(frozen=True, slots=True)
class FileRow:
    path: Path
    kind: EntryKind
    bytes: int
    allocated: int
    mtime_ns: int
    #: Whether ``.gitignore`` rules ignore this entry, or ``None`` when none was read.
    ignored: bool | None = None
    sort_value: int | None = None
    classification: FileClassification | None = None
    #: Directory subtree counts, excluding its root; absent for other entry kinds.
    files: int | None = None
    dirs: int | None = None
    #: Whether a directory's eligible subtree was listed in full. ``False`` makes its
    #: bytes, counts, and ``mtime_ns`` lower bounds and its ``age_ns`` ``None``: a
    #: scan-depth boundary, a partial scan, or a directory discovery has not listed yet.
    #: Absent for other entry kinds.
    complete: bool | None = None
    #: Signed modification age relative to Report.age_reference_ns; future is negative,
    #: and ``None`` when the reference is unrepresentable or the subtree is incomplete.
    age_ns: int | None = None
    #: ``mtime_ns`` as a timezone-aware UTC ``datetime``, floored to the microsecond, or
    #: ``None`` when the subtree is incomplete and ``mtime_ns`` only a lower bound.
    modified_at: datetime | None = None
    #: Position of the row's root in :attr:`Report.roots` over several roots, so
    #: ``report.roots[row.root].path / row.path`` names the entry; ``None`` over one root,
    #: where ``path`` is relative to :attr:`Report.root`.
    root: int | None = None


class TreeOmissionReason(StrEnum):
    SHARE = "share"
    BREADTH = "breadth"
    DEPTH = "depth"
    ROWS = "rows"


@dataclass(frozen=True, slots=True)
class TreeOmission:
    reason: TreeOmissionReason
    #: Directly hidden child entries; one omitted directory counts as one entry.
    entries: int
    bytes: int | None
    allocated: int | None
    #: Recursive regular-file count hidden by this omission, when known.
    files: int | None = None


@dataclass(frozen=True, slots=True)
class TreeRemainder:
    #: Recursive regular-file count hidden across this tree, when known.
    files: int | None
    bytes: int | None
    allocated: int | None
    reasons: tuple[TreeOmissionReason, ...]


@dataclass(frozen=True, slots=True)
class TreeNode:
    name: str
    path: Path
    kind: EntryKind
    bytes: int
    allocated: int
    files: int
    dirs: int
    #: The newest regular file's modification time in this subtree, or ``None`` with none.
    newest_mtime_ns: int | None
    truncated: bool
    children: tuple[TreeNode, ...]
    omissions: tuple[TreeOmission, ...] = ()
    #: The ignored share of this subtree, or ``None`` when no ``.gitignore`` was read.
    ignored: IgnoredTally | None = None
    #: Whether this entry itself is gitignored, or ``None`` when classification is unknown.
    entry_ignored: bool | None = None
    #: The newest modification time among the entries this row counts, of any kind: its
    #: own entry when the selection admits it and every counted entry beneath it, never
    #: the report root's own time. ``None`` when the row counts nothing; a lower bound
    #: when ``complete`` is false. The age the text tree shows.
    mtime_ns: int | None = None
    #: Whether the directory's subtree was listed in full; ``None`` for a file.
    complete: bool | None = None
    #: Signed age relative to Report.age_reference_ns; future is negative, and ``None``
    #: when the row counts nothing, the subtree is incomplete, or the reference is
    #: unrepresentable.
    age_ns: int | None = None
    #: ``mtime_ns`` as a timezone-aware UTC ``datetime``, floored to the microsecond, or
    #: ``None`` when the row counts nothing or its subtree is incomplete.
    modified_at: datetime | None = None


@dataclass(frozen=True, slots=True)
class MetricValues:
    physical_lines: int | None = None
    blank_lines: int | None = None
    nonblank_lines: int | None = None
    raw_words: int | None = None
    code_lines: int | None = None
    comment_lines: int | None = None
    code_blank_lines: int | None = None
    logical_words: int | None = None
    paragraphs: int | None = None
    visible_words: int | None = None
    visible_logical_words: int | None = None
    document_words: int | None = None


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
class Pages:
    words: int
    words_per_page: int


@dataclass(frozen=True, slots=True)
class MetricRow:
    id: str
    family: str
    files: int
    bytes: int
    allocated: int
    share: MetricShare
    metrics: MetricValues
    lines_coverage: MappingProxyType[str, int] | None
    code_coverage: MappingProxyType[str, int] | None
    words_coverage: MappingProxyType[str, int] | None
    detection: Detection
    pages: Pages | None


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
class ReportScope:
    max_depth: int | None
    follow_symlinks: bool
    one_filesystem: bool
    exclude_special: bool
    read_controls: bool
    population: IgnoredEntries


@dataclass(frozen=True, slots=True)
class ReportRequest:
    sort_metric: str | None
    scope: ReportScope
    analyze: tuple[Analysis, ...]
    size: SizeMetric
    views: tuple[View, ...]
    omitted_views: tuple[View, ...]


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
    share_omitted: int = 0


@dataclass(frozen=True, slots=True)
class FilesSection:
    """A flat list of matching files and directories, with subtree metrics for directories.

    ``view`` preserves the requested list or legacy preset. ``largest`` and ``recent``
    produce this shape with their bounded regular-file selection.
    """

    view: View
    files: tuple[FileRow, ...]
    bound: SectionBound | None = None


@dataclass(frozen=True, slots=True)
class TreeDisplayLimits:
    depth: int | None
    min_share: str
    breadth: int | None
    rows: int | None


@dataclass(frozen=True, slots=True)
class ReportRoot:
    """One root of a report over several: the path as the caller named it, normalized by
    its components, and the directory it is."""

    #: What text prints before each path under this root.
    label: Path
    #: The canonical directory every path under this root is relative to.
    path: Path


@dataclass(frozen=True, slots=True)
class TreeTotal:
    """The total row of a tree over several roots: every root's totals together.

    A tree node's values without the identity of an entry, since no entry is the total.
    """

    bytes: int
    allocated: int
    files: int
    dirs: int
    #: The ignored share, or ``None`` when any root's is unknown.
    ignored: IgnoredTally | None
    #: The newest regular file under any root.
    newest_mtime_ns: int | None
    #: The newest activity any root's row counts.
    mtime_ns: int | None
    #: Whether every root was listed in full.
    complete: bool
    age_ns: int | None = None
    modified_at: datetime | None = None


@dataclass(frozen=True, slots=True)
class RootTree:
    """One root's tree in a report over several roots, named by the root's label."""

    #: Position of the tree's root in :attr:`Report.roots`.
    root: int
    tree: TreeNode
    #: What this tree's bounds hid, as one root's section carries it.
    remainder: TreeRemainder | None = None


@dataclass(frozen=True, slots=True)
class TreeSection:
    view: View
    #: The tree of a report over one root; ``None`` over several, or when the row limit
    #: is zero.
    tree: TreeNode | None
    limits: TreeDisplayLimits
    omissions: tuple[TreeOmission, ...] = ()
    #: Hidden contents across this tree, or ``None`` when nothing is omitted.
    #: Parent totals already include these values; do not add them again. Over several
    #: roots, what the row limit cut at the top: root rows, or every root under a zero
    #: limit.
    remainder: TreeRemainder | None = None
    #: The total row over several roots, ``None`` over one root or under a zero row limit.
    total: TreeTotal | None = None
    #: One tree per root over several roots, in display order; empty over one root.
    trees: tuple[RootTree, ...] = ()


@dataclass(frozen=True, slots=True)
class MetricsSection:
    view: View
    group: str
    share_metric: str
    total: MetricRow
    rows: tuple[MetricRow, ...]
    bound: SectionBound | None = None
    share_omitted: int = 0


@dataclass(frozen=True, slots=True)
class CodeTally:
    source_files: int
    analyzed_files: int
    code_lines: int
    comment_lines: int
    blank_lines: int
    missing_records: int
    coverage: MappingProxyType[str, int]


@dataclass(frozen=True, slots=True)
class CodeLanguageRow:
    language: str
    share: MetricShare
    selected: CodeTally
    non_ignored: CodeTally | None
    ignored: CodeTally | None
    unknown: CodeTally


@dataclass(frozen=True, slots=True)
class CodeOverview:
    unclassified_files: int
    population: IgnoredEntries
    share_metric: str
    analyzed_languages: int
    selected: CodeTally
    non_ignored: CodeTally | None
    ignored: CodeTally | None
    unknown: CodeTally
    languages: tuple[CodeLanguageRow, ...]
    bound: SectionBound | None = None
    share_omitted: int = 0


@dataclass(frozen=True, slots=True)
class CodeSection:
    view: View
    code: CodeOverview


type ReportSection = (
    SummarySection | ExtensionsSection | FilesSection | TreeSection | MetricsSection | CodeSection
)


@dataclass(frozen=True, slots=True)
class Report:
    """One immutable multi-view report plus its exact CLI wire representation."""

    schema: str
    generator: str
    #: The root of a report over one root; ``None`` over several, which :attr:`roots`
    #: names. Exactly one of the two is set.
    root: Path | None
    request: ReportRequest
    status: Status
    provenance: ReportProvenance
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
    #: The instant every age in the report is measured from, in epoch nanoseconds, or
    #: ``None`` when it cannot be represented.
    age_reference_ns: int | None = None
    #: ``age_reference_ns`` as a timezone-aware UTC ``datetime``, floored to the
    #: microsecond.
    age_reference_at: datetime | None = None
    #: Actionable suggestions, separate from facts and formatted data.
    tips: tuple[str, ...] = ()
    #: What a reader must not miss about the answer itself, in the order a renderer
    #: prints them, between `notes` and `tips`. A ``stale_ok`` answer that no filesystem
    #: check verified carries one, naming ``stale_ok`` as the way to a fresh answer; a
    #: verified answer carries none. The command line prints these as ``warn:`` lines
    #: even under ``--quiet``, which suppresses notes and tips. Like them, not in
    #: `as_dict`: the wire envelope states the same fact as ``provenance.source`` and
    #: ``provenance.freshness``.
    warnings: tuple[str, ...] = ()
    #: Bound renderer, supplied by `Index.report`. Absent on a report built by hand.
    _renderer: Callable[[str, bool, int], str] | None = field(
        default=None, repr=False, compare=False
    )
    #: The roots of a report over several, in the caller's order, each with its label and
    #: canonical path; ``None`` over one root.
    roots: tuple[ReportRoot, ...] | None = None

    def as_dict(self) -> dict[str, JsonValue]:
        """Return an independent copy of the exact CLI JSON schema."""

        return cast(dict[str, JsonValue], _copy_json(self._wire))

    def render(
        self, format: Format = Format.TEXT, *, color: bool = False, bar_size: int = 10
    ) -> str:
        """Serialize this report the way the command line does.

        Beside `as_dict` because both are serializations of the same value -- the one this
        report was built from, not whatever the index holds now -- and splitting them
        across a method and a module function would make the pair harder to find than
        either alone.

        `color` is a plain bool rather than the CLI's `auto | always | never`: resolving
        `auto` means asking whether stdout is a terminal, and a library does not own
        stdout. The caller decides and passes the answer in. ``bar_size`` controls
        tree bar width (default 10); zero or negative values hide the bar column.
        This changes presentation only, not the report or machine output.

        The report only. The command line appends a performance footer, which is transient
        telemetry the schema excludes and whose counts are not on a `Report`.

        ``TEXT`` uses the query's requested presentation. Machine formats serialize this
        stored projection; PATHS and LONG require a flat projection, and TREE requires
        a tree. To change between a folded tree and a complete list, request another
        report from the retained index with the desired ``Query.format``.
        """

        if self._renderer is None:
            # Deferred: `_api` imports this module, so its exception types cannot be
            # imported at module scope.
            from ._api import InvalidArgumentError

            raise InvalidArgumentError(
                "this report carries no renderer; only a report from Index.report, "
                "fdu.report, or Watch.report can be rendered"
            )
        return self._renderer(str(format), color, bar_size)


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
    #: Whether rules ignore the entry, or ``None`` when classification is unavailable.
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

    Include populations retain the same entries with or without `.gitignore`
    observation. Narrowed populations carry the governing control policy because it
    determines which entries are retained.
    """

    engine: int
    max_depth: int | None
    follow_symlinks: bool
    one_filesystem: bool
    hidden_fingerprint: int
    population: IgnoredEntries
    control_fingerprint: int
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

    `state` is one of the two values a sidecar takes, `ContentState.CURRENT` or
    `ContentState.STALE`. `records` and `identity` come from the header of a `CURRENT`
    sidecar and are `None` otherwise; `stale_reason` is set only for a `STALE` one, and
    `format_version` only when the version is the reason.
    """

    bytes: int
    state: ContentState
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


_EPOCH = datetime(1970, 1, 1, tzinfo=UTC)


def _instant(nanos: int | None) -> datetime | None:
    """A timezone-aware UTC `datetime` for epoch nanoseconds, floored to the microsecond.

    Derived from the exact integer rather than parsed from the wire's RFC 3339 string, so
    nothing depends on how a string rounds. Floor division floors toward the past, so an
    instant before the epoch lands on the microsecond at or before it, as one after the
    epoch does; truncating toward zero would move it later.
    """

    if nanos is None:
        return None
    return _EPOCH + timedelta(microseconds=nanos // 1_000)


def _entry_tier_identity(value: Mapping[str, Any]) -> EntryTierIdentity:
    return EntryTierIdentity(
        engine=int(value["engine"]),
        max_depth=_limit(value["max_depth"]),
        follow_symlinks=bool(value["follow_symlinks"]),
        one_filesystem=bool(value["one_filesystem"]),
        hidden_fingerprint=int(value["hidden_fingerprint"]),
        population=IgnoredEntries(value["population"]),
        control_fingerprint=int(value["control_fingerprint"]),
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
    raw_identity = value.get("identity")
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
        state=ContentState(value["state"]),
        stale_reason=_stale_reason(value.get("stale_reason")),
        format_version=_limit(value.get("format_version")),
        records=_limit(value.get("records")),
        identity=identity,
    )


def _stale_reason(value: object) -> StaleReason | None:
    return None if value is None else StaleReason(str(value))


def cache_status_from_dict(value: Mapping[str, Any]) -> CacheStatus:
    return CacheStatus(
        path=_wire_path(value),
        bytes=int(value["bytes"]),
        state=CacheState(value["state"]),
        stale_reason=_stale_reason(value.get("stale_reason")),
        format_version=_limit(value.get("format_version")),
        leftover_kind=(
            LeftoverKind(value.get("leftover_kind"))
            if value.get("leftover_kind") is not None
            else None
        ),
        root=_wire_path(value, "root") if value.get("root") is not None else None,
        entries=_limit(value.get("entries")),
        identity=_snapshot_identity(value.get("identity")),
        content=_content_status(value.get("content")),
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
        _wire_path(error) if path is not None else None,
        str(error.get("kind", "operation")),
        str(error.get("message", "")),
        int(error["os_error"]) if error.get("os_error") is not None else None,
        _root_position(error),
    )


def _root_position(value: Mapping[str, Any]) -> int | None:
    """The position of a row's, an error's, or a refusal's root in ``Report.roots``,
    present only over several roots."""

    root = value.get("root")
    return None if root is None else int(root)


def _ignore_rules(value: object) -> ControlObservation | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise TypeError("ignore_rules must be an object or null")
    return control_observation_from_dict(cast(dict[str, Any], value))


def status_from_dict(value: dict[str, Any]) -> Status:
    raw_coverage = value["coverage"]
    if not isinstance(raw_coverage, dict):
        raise TypeError("status coverage must be an object")
    raw_coverage = cast(dict[str, Any], raw_coverage)
    return Status(
        complete=bool(value["complete"]),
        coverage=Coverage(str(raw_coverage["kind"])),
        coverage_reason=(
            str(raw_coverage["reason"]) if raw_coverage.get("reason") is not None else None
        ),
        errors=tuple(_operation_error(item) for item in value.get("errors", [])),
        errors_omitted=int(value.get("errors_omitted", 0)),
        ignore_rules=_ignore_rules(value.get("ignore_rules")),
    )


def _tier_state(value: dict[str, Any]) -> TierState:
    return TierState(
        source=ValueSource(str(value["source"])),
        freshness=Freshness(str(value["freshness"])),
        observed_at_ns=(
            int(value["observed_at_ns"]) if value.get("observed_at_ns") is not None else None
        ),
    )


def _report_provenance(value: dict[str, Any]) -> ReportProvenance:
    tiers = value["tiers"]
    if not isinstance(tiers, dict):
        raise TypeError("report provenance tiers must contain entries")
    tiers = cast(dict[str, Any], tiers)
    if not isinstance(tiers.get("entries"), dict):
        raise TypeError("report provenance tiers must contain entries")
    generated_at = _datetime(value["generated_at"])
    if generated_at is None:
        raise TypeError("report generated_at must be present")
    raw_content = tiers.get("content")
    return ReportProvenance(
        source=ReportSource(str(value["source"])),
        freshness=Freshness(str(value["freshness"])),
        scan_started_at=_datetime(value.get("scan_started_at")),
        generated_at=generated_at,
        tiers=TierProvenance(
            entries=_tier_state(tiers["entries"]),
            content=_tier_state(raw_content) if isinstance(raw_content, dict) else None,
        ),
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


def _code_tally(value: dict[str, Any]) -> CodeTally:
    return CodeTally(
        source_files=int(value["source_files"]),
        analyzed_files=int(value["analyzed_files"]),
        code_lines=int(value["code_lines"]),
        comment_lines=int(value["comment_lines"]),
        blank_lines=int(value["blank_lines"]),
        missing_records=int(value["missing_records"]),
        coverage=_int_map(value["coverage"]),
    )


def _optional_code_tally(value: dict[str, Any] | None) -> CodeTally | None:
    return None if value is None else _code_tally(value)


def _code_overview(value: dict[str, Any]) -> CodeOverview:
    return CodeOverview(
        unclassified_files=int(value["unclassified_files"]),
        share_omitted=int(value["share_omitted"]),
        bound=None
        if value["bound"] is None
        else SectionBound(shown=int(value["bound"]["shown"]), total=int(value["bound"]["total"])),
        population=IgnoredEntries(value["population"]),
        share_metric=str(value["share_metric"]),
        analyzed_languages=int(value["analyzed_languages"]),
        selected=_code_tally(value["selected"]),
        non_ignored=_optional_code_tally(value["non_ignored"]),
        ignored=_optional_code_tally(value["ignored"]),
        unknown=_code_tally(value["unknown"]),
        languages=tuple(
            CodeLanguageRow(
                language=str(row["language"]),
                share=MetricShare(int(row["share"]["numerator"]), int(row["share"]["denominator"])),
                selected=_code_tally(row["selected"]),
                non_ignored=_optional_code_tally(row["non_ignored"]),
                ignored=_optional_code_tally(row["ignored"]),
                unknown=_code_tally(row["unknown"]),
            )
            for row in value["languages"]
        ),
    )


def _metric_row(value: dict[str, Any]) -> MetricRow:
    metrics = value["metrics"]
    coverage = value["coverage"]
    detection = value["detection"]
    flags = detection["flags"]
    raw_pages = value.get("pages")
    names = (
        "physical_lines",
        "blank_lines",
        "nonblank_lines",
        "raw_words",
        "code_lines",
        "comment_lines",
        "code_blank_lines",
        "logical_words",
        "paragraphs",
        "visible_words",
        "visible_logical_words",
        "document_words",
    )
    return MetricRow(
        id=str(value["id"]),
        family=str(value["family"]),
        files=int(value["files"]),
        bytes=int(value["bytes"]),
        allocated=int(value["allocated"]),
        share=MetricShare(int(value["share"]["numerator"]), int(value["share"]["denominator"])),
        metrics=MetricValues(
            **{name: int(metrics[name]) if name in metrics else None for name in names}
        ),
        lines_coverage=_int_map(coverage["lines"]) if "lines" in coverage else None,
        code_coverage=_int_map(coverage["code"]) if "code" in coverage else None,
        words_coverage=_int_map(coverage["words"]) if "words" in coverage else None,
        detection=Detection(
            sources=_int_map(detection["sources"]),
            confidence=_int_map(detection["confidence"]),
            generated=int(flags["generated"]),
            vendored=int(flags["vendored"]),
            documentation=int(flags["documentation"]),
        ),
        pages=(
            Pages(words=int(raw_pages["words"]), words_per_page=int(raw_pages["words_per_page"]))
            if raw_pages is not None
            else None
        ),
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


def _file_classification(value: dict[str, Any] | None) -> FileClassification | None:
    if value is None:
        return None
    flags = value["flags"]
    return FileClassification(
        file_type=str(value["file_type"]),
        family=str(value["family"]),
        source=str(value["source"]),
        confidence=str(value["confidence"]),
        generated=bool(flags["generated"]),
        vendored=bool(flags["vendored"]),
        documentation=bool(flags["documentation"]),
    )


def _ignored_flag(value: object) -> bool | None:
    if value is None:
        return None
    if not isinstance(value, bool):
        raise TypeError("a file row's ignored flag must be a boolean or null")
    return value


def _tree_omissions(values: list[dict[str, Any]]) -> tuple[TreeOmission, ...]:
    return tuple(
        TreeOmission(
            reason=TreeOmissionReason(value["reason"]),
            entries=int(value["entries"]),
            bytes=_optional_int(value["bytes"]),
            allocated=_optional_int(value["allocated"]),
            files=_optional_int(value.get("files")),
        )
        for value in values
    )


def _tree_remainder(value: object) -> TreeRemainder | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise TypeError("tree remainder must be an object or null")
    reasons = value["reasons"]
    if not isinstance(reasons, list):
        raise TypeError("tree remainder reasons must be a list")
    return TreeRemainder(
        files=_optional_int(value["files"]),
        bytes=_optional_int(value["bytes"]),
        allocated=_optional_int(value["allocated"]),
        reasons=tuple(TreeOmissionReason(reason) for reason in reasons),
    )


def _tree(value: dict[str, Any]) -> TreeNode:
    stack: list[tuple[dict[str, Any], bool]] = [(value, False)]
    built: dict[int, TreeNode] = {}
    while stack:
        raw, visited = stack.pop()
        children = raw["children"]
        if not isinstance(children, list):
            raise TypeError("tree children must be a list")
        if not visited:
            stack.append((raw, True))
            for child in reversed(children):
                if not isinstance(child, dict):
                    raise TypeError("tree child must be an object")
                stack.append((child, False))
            continue
        built[id(raw)] = TreeNode(
            name=str(raw["name"]),
            path=_wire_path(raw),
            kind=EntryKind(str(raw["kind"])),
            entry_ignored=_optional_bool(raw["entry_ignored"]),
            bytes=int(raw["bytes"]),
            allocated=int(raw["allocated"]),
            files=int(raw["files"]),
            dirs=int(raw["dirs"]),
            newest_mtime_ns=(
                int(raw["newest_mtime_ns"]) if raw.get("newest_mtime_ns") is not None else None
            ),
            truncated=bool(raw["truncated"]),
            children=tuple(built[id(child)] for child in children),
            omissions=_tree_omissions(raw["omissions"]),
            ignored=_ignored_tally(raw["ignored"]),
            mtime_ns=_optional_int(raw["mtime_ns"]),
            complete=_optional_bool(raw["complete"]),
            age_ns=_optional_int(raw["age_ns"]),
            modified_at=_modified_at(raw["mtime_ns"], raw["complete"]),
        )
    return built[id(value)]


def _tree_total(value: object) -> TreeTotal | None:
    """The total row of a tree over several roots, or ``None``."""

    if value is None:
        return None
    if not isinstance(value, dict):
        raise TypeError("tree total must be an object or null")
    raw = cast(dict[str, Any], value)
    complete = _optional_bool(raw["complete"])
    return TreeTotal(
        bytes=int(raw["bytes"]),
        allocated=int(raw["allocated"]),
        files=int(raw["files"]),
        dirs=int(raw["dirs"]),
        ignored=_ignored_tally(raw["ignored"]),
        newest_mtime_ns=_optional_int(raw["newest_mtime_ns"]),
        mtime_ns=_optional_int(raw["mtime_ns"]),
        complete=bool(complete),
        age_ns=_optional_int(raw["age_ns"]),
        modified_at=_modified_at(raw["mtime_ns"], complete),
    )


def _root_trees(value: object) -> tuple[RootTree, ...]:
    """One tree per root over several roots; none over one root."""

    if value is None:
        return ()
    if not isinstance(value, list):
        raise TypeError("root trees must be a list")
    trees: list[RootTree] = []
    for item in cast(list[Any], value):
        if not isinstance(item, dict):
            raise TypeError("a root tree must be an object")
        raw = cast(dict[str, Any], item)
        if not isinstance(raw.get("tree"), dict):
            raise TypeError("a root tree must carry a tree")
        trees.append(
            RootTree(
                root=int(raw["root"]),
                tree=_tree(cast(dict[str, Any], raw["tree"])),
                remainder=_tree_remainder(raw.get("remainder")),
            )
        )
    return tuple(trees)


def _report_roots(value: object) -> tuple[ReportRoot, ...] | None:
    """The envelope's roots over several roots, or ``None`` over one."""

    if value is None:
        return None
    if not isinstance(value, list):
        raise TypeError("report roots must be a list")
    roots: list[ReportRoot] = []
    for item in cast(list[Any], value):
        if not isinstance(item, dict):
            raise TypeError("a report root must be an object")
        raw = cast(dict[str, Any], item)
        roots.append(ReportRoot(label=_wire_path(raw, "label"), path=_wire_path(raw, "path")))
    return tuple(roots)


def _modified_at(mtime_ns: object, complete: object) -> datetime | None:
    """A row's ``modified_at``: its time, unless the subtree's time is only a lower bound."""

    if mtime_ns is None or complete is False:
        return None
    return _instant(int(cast(int, mtime_ns)))


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
        elif view is View.CODE:
            sections.append(CodeSection(view, _code_overview(raw["code"])))
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
                    share_omitted=int(raw["share_omitted"]),
                )
            )
        elif "files" in raw and view in (
            View.LIST,
            View.TREE,
            View.FILES,
            View.LARGEST,
            View.RECENT,
        ):
            rows = raw["files"]
            if not isinstance(rows, list):
                raise TypeError("files section must be a list")
            sections.append(
                FilesSection(
                    view,
                    tuple(
                        FileRow(
                            path=_wire_path(row),
                            kind=EntryKind(str(row["kind"])),
                            bytes=int(row["bytes"]),
                            allocated=int(row["allocated"]),
                            mtime_ns=int(row["mtime_ns"]),
                            files=_optional_int(row["files"]) if "files" in row else None,
                            dirs=_optional_int(row["dirs"]) if "dirs" in row else None,
                            complete=_optional_bool(row["complete"]) if "complete" in row else None,
                            age_ns=_optional_int(row["age_ns"]) if "age_ns" in row else None,
                            modified_at=_modified_at(
                                row["mtime_ns"],
                                _optional_bool(row["complete"]) if "complete" in row else None,
                            ),
                            ignored=_ignored_flag(row["ignored"]),
                            sort_value=_optional_int(row["sort_value"]),
                            classification=_file_classification(row["classification"]),
                            root=_root_position(row),
                        )
                        for row in rows
                    ),
                    _bound(raw),
                )
            )
        elif "tree" in raw and view in (View.LIST, View.TREE, View.FILES):
            tree = raw["tree"]
            if tree is not None and not isinstance(tree, dict):
                raise TypeError("tree section must be an object or null")
            limits = raw["limits"]
            sections.append(
                TreeSection(
                    view,
                    None if tree is None else _tree(tree),
                    TreeDisplayLimits(
                        _limit(limits["depth"]),
                        str(limits["min_share"]),
                        _limit(limits["breadth"]),
                        _limit(limits["rows"]),
                    ),
                    _tree_omissions(raw["omissions"]),
                    _tree_remainder(cast(dict[str, Any], raw).get("remainder")),
                    total=_tree_total(cast(dict[str, Any], raw).get("total")),
                    trees=_root_trees(cast(dict[str, Any], raw).get("trees")),
                )
            )
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
                    total=_metric_row(total),
                    rows=tuple(_metric_row(row) for row in rows),
                    bound=_bound(metrics),
                    share_omitted=int(metrics["share_omitted"]),
                )
            )

    raw_status = wire["status"]
    raw_provenance = wire["provenance"]
    raw_request = wire["request"]
    if not isinstance(raw_status, dict) or not isinstance(raw_provenance, dict):
        raise TypeError("report status and provenance must be objects")
    if not isinstance(raw_request, dict):
        raise TypeError("report request and scope must be objects")
    raw_request = cast(dict[str, Any], raw_request)
    if not isinstance(raw_request.get("scope"), dict):
        raise TypeError("report request and scope must be objects")
    raw_status = cast(dict[str, Any], raw_status)
    raw_provenance = cast(dict[str, Any], raw_provenance)
    status = status_from_dict({**raw_status, "ignore_rules": wire["ignore_rules"]})
    provenance = _report_provenance(raw_provenance)
    raw_scope = cast(dict[str, Any], raw_request["scope"])
    request = ReportRequest(
        sort_metric=str(raw_request["sort_metric"])
        if raw_request["sort_metric"] is not None
        else None,
        scope=ReportScope(
            max_depth=(
                int(raw_scope["max_depth"]) if raw_scope.get("max_depth") is not None else None
            ),
            follow_symlinks=bool(raw_scope["follow_symlinks"]),
            one_filesystem=bool(raw_scope["one_filesystem"]),
            exclude_special=bool(raw_scope["exclude_special"]),
            read_controls=bool(raw_scope["read_controls"]),
            population=IgnoredEntries(raw_scope["population"]),
        ),
        analyze=tuple(Analysis(str(name)) for name in raw_request["analyze"]),
        size=SizeMetric(str(raw_request["size"])),
        views=tuple(View(str(name)) for name in raw_request["views"]),
        omitted_views=tuple(View(str(name)) for name in raw_request["omitted_views"]),
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
    return Report(
        notes=notes,
        schema=str(wire["schema"]),
        generator=str(wire["generator"]),
        root=_wire_path(wire, "root") if wire.get("root") is not None else None,
        roots=_report_roots(wire.get("roots")),
        request=request,
        age_reference_ns=_optional_int(wire.get("age_reference_ns")),
        age_reference_at=_instant(_optional_int(wire.get("age_reference_ns"))),
        status=status,
        provenance=provenance,
        analysis=analysis,
        sections=tuple(sections),
        _wire=cast(dict[str, JsonValue], wire),
    )


def _optional_int(value: Any) -> int | None:
    """Decode a nullable exact integer from the native wire report."""
    return None if value is None else int(value)


def _optional_bool(value: Any) -> bool | None:
    """Decode a nullable boolean, refusing anything that merely looks true or false."""
    if value is None:
        return None
    if not isinstance(value, bool):
        raise TypeError("a file row's complete flag must be a boolean or null")
    return value
