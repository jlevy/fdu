# pyright: reportUnknownArgumentType=false, reportUnknownVariableType=false
"""Immutable public values for fdu's Python API.

The native layer deliberately moves bulk dictionaries and JSON documents across the
language boundary.  This module converts each bulk result once into discoverable,
typed values; callers never need to know the private extension's wire shape.
"""

from __future__ import annotations

from collections.abc import Callable
from copy import deepcopy
from dataclasses import dataclass, field
from datetime import datetime
from enum import StrEnum
from pathlib import Path
from types import MappingProxyType
from typing import TYPE_CHECKING, Any, cast

if TYPE_CHECKING:
    # Typing only: `TypeRegistry` wraps a native handle and lives beside `Index` in
    # `_api`, while this module stays free of the extension. `from __future__ import
    # annotations` makes the reference lazy, so there is no runtime cycle.
    from ._api import TypeRegistry

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
    GROUPS = "groups"
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


class ScanOrder(StrEnum):
    """The order directories are visited in.

    Both orders visit every entry exactly once and leave an identical index behind, so
    this changes *when* observations are produced, never *which* ones. It matters only to
    a consumer reading while the walk runs.
    """

    BREADTH_FIRST = "breadth-first"
    DEPTH_FIRST = "depth-first"


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


@dataclass(frozen=True, slots=True)
class ScanOptions:
    """Filesystem scope for an initial scan and later refreshes."""

    max_depth: int | None = None
    one_filesystem: bool = False
    #: Directory visit order. Breadth-first is the default because it is the order whose
    #: partial results mean something: a consumer reading mid-walk sees top-level totals
    #: grow together rather than one subtree finishing while its siblings read zero.
    order: ScanOrder = ScanOrder.BREADTH_FIRST
    #: Walker threads, or ``None`` to choose automatically. ``1`` makes emission order
    #: depend only on the queue, which is what a reproducible recording needs.
    threads: int | None = None
    #: File-type rules to classify against, or ``None`` for the ones fdu ships.
    #:
    #: Scope rather than selection: the rules decide what every type row *means*, so a
    #: snapshot taken under one set is not answerable under another and the cache
    #: invalidates accordingly. Build one with :meth:`TypeRegistry.from_manifest` and
    #: reuse it across calls -- parsing is the cost, and a registry is read-only after.
    type_rules: TypeRegistry | None = None
    #: Tag rules to evaluate per entry, by name. Try ``("dotfile",)``.
    #:
    #: Scope rather than selection, for the same reason `type_rules` is: an index built
    #: without a rule carries no bit for it and cannot answer a question about it, so the
    #: cache invalidates accordingly. Enabling costs one branch per insert and nothing per
    #: query; the default evaluates none and fingerprints to zero, which is what every
    #: existing snapshot recorded.
    tag_rules: tuple[str, ...] = ()

    def __post_init__(self) -> None:
        if self.max_depth is not None and self.max_depth < 0:
            raise ValueError("max_depth must be non-negative")
        if self.threads is not None and self.threads < 1:
            raise ValueError("threads must be at least 1")
        if isinstance(self.tag_rules, str):
            raise TypeError("tag_rules takes a tuple of names; wrap the single value in a tuple")


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
    #: Tags an entry must carry at least one of. Any-of rather than all-of, matching
    #: `include`: naming a second tag widens, and the way to narrow is `not_tags`.
    #: Every name must be enabled on the index via `tag_rules=`; a rule that is off is
    #: refused rather than treated as a filter that matches nothing.
    tags: tuple[str, ...] = ()
    #: Tags that exclude an entry outright. Wins over `tags`, as `exclude` wins over
    #: `include`.
    not_tags: tuple[str, ...] = ()
    #: Accepts a raw token as well as an int or `Bound`, so a caller passing user input
    #: straight through gets the library's own grammar and wording rather than having to
    #: pre-validate and invent a second opinion about what is acceptable.
    depth: int | Bound | str | None = None
    limit: int | Bound | str | None = None
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
            ("tags", self.tags),
            ("not_tags", self.not_tags),
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
    poll_interval: float | None = None
    """Restat the tree every this many seconds instead of using native notifications.

    Network and FUSE filesystems accept a native watch and then deliver nothing, which is
    the worst failure available: no error is reported and the index quietly stops
    tracking. Polling trades a fixed cost per interval for the guarantee that a change is
    eventually seen, and change latency is then bounded by this rather than by
    :attr:`interval`.

    ``None`` uses the platform's native API, which is right for a local filesystem.
    Explicit rather than detected: choosing native on a filesystem that drops events
    fails silently, so the caller who knows what it mounted decides.
    """

    def __post_init__(self) -> None:
        if self.interval <= 0:
            raise ValueError("interval must be positive")
        if self.poll_interval is not None and self.poll_interval <= 0:
            raise ValueError("poll_interval must be positive")


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


class CoverageReason(StrEnum):
    """Why a value's coverage is partial.

    The interactive-client contract's vocabulary, declared whole. Two members are
    reachable from today's engine -- ``INACCESSIBLE`` and ``FAILED``. The other four are
    declared and currently unreachable: an in-progress walk and a cancellation both need
    the session, fdu has no walk budget, and a watcher gap marks a subtree *untrusted*
    rather than *uncovered* -- its totals still account for every entry, they may simply
    be wrong. Matching on all six today means not having to revisit the match when the
    engine learns to produce them.
    """

    BUILDING = "building"
    BUDGET = "budget"
    CANCELLED = "cancelled"
    INACCESSIBLE = "inaccessible"
    WATCHER_GAP = "watcher_gap"
    FAILED = "failed"


@dataclass(frozen=True, slots=True)
class Provenance:
    """Origin, observation time, and coverage for one retained value."""

    source: ValueSource
    observed_at_ns: int
    status: Coverage

    reason: CoverageReason | None = None
    """Why ``status`` is partial, and ``None`` whenever it is complete.

    Separate from ``status`` rather than folded into it, so a consumer that only branches
    on complete-or-not keeps working without learning six new values. The two cannot
    disagree: the engine spells the reason inside the partial variant itself.
    """


class DetectionSource(StrEnum):
    """Which bounded step established a classification."""

    EXACT_FILENAME = "exact_filename"
    COMPOUND_EXTENSION = "compound_extension"
    EXTENSION = "extension"
    SHEBANG = "shebang"
    MODELINE = "modeline"
    AMBIGUOUS_CONTENT = "ambiguous_content"
    FORMAT_SIGNATURE = "format_signature"
    CONTENT_PROBE = "content_probe"
    UNKNOWN = "unknown"


class DetectionConfidence(StrEnum):
    """Strength of the evidence a classification rests on."""

    CERTAIN = "certain"
    HIGH = "high"
    HEURISTIC = "heuristic"


class ContentFamily(StrEnum):
    """Broad analysis family: which analyzer may open a file.

    An analysis question, not a browsing one. Every image, video, PDF, and archive is
    ``BINARY`` here because none of them can be read as text, which is the only thing
    this axis decides.
    """

    CODE = "code"
    PROSE = "prose"
    MARKUP = "markup"
    DATA = "data"
    BINARY = "binary"
    UNKNOWN = "unknown"


@dataclass(frozen=True, slots=True)
class ClassificationFlags:
    """Origin and purpose attributes, orthogonal to the type itself."""

    generated: bool
    vendored: bool
    documentation: bool


@dataclass(frozen=True, slots=True)
class Classification:
    """What the type rules make of one path."""

    file_type: str
    """Stable identifier, or ``unknown:.ext`` for an extension no rule claims."""

    family: ContentFamily
    source: DetectionSource
    confidence: DetectionConfidence
    flags: ClassificationFlags
    group: str | None = None
    """Browsing group id, or ``None`` when the registry declares none."""


@dataclass(frozen=True, slots=True)
class ExtensionTally:
    files: int
    bytes: int
    allocated: int


@dataclass(frozen=True, slots=True)
class ExtensionRemainder:
    """Extension tallies a bound withheld from a roll-up.

    Same contract as a tree node's `Remainder`: the listed rows plus this account for
    every file in the subtree, so a listing that shows five extensions can label the rest
    instead of appearing to have shown them all.
    """

    extensions: int
    """Distinct extensions not listed."""

    files: int
    """Files carrying them."""

    bytes: int
    """Apparent bytes across those files."""

    allocated: int
    """Allocated bytes across those files."""


@dataclass(frozen=True, slots=True)
class RollUp:
    files: int
    dirs: int
    others: int
    """Descendant entries that are neither files nor directories: symlinks and the rest.

    Zero bytes each, and counted anyway, because otherwise a subtree of a hundred
    symlinks and one holding nothing at all are the same arithmetic. See ``is_empty``.
    """

    bytes: int
    allocated: int
    newest_mtime_ns: int
    by_extension: MappingProxyType[str, ExtensionTally]
    by_group: MappingProxyType[str, ExtensionTally] = field(
        default_factory=lambda: MappingProxyType({})
    )
    """Per-browsing-group tallies, keyed by group id.

    Maintained by the engine's reducer rather than derived from `by_extension`: an
    exact-filename rule (`Makefile`, `Dockerfile`) has no extension bucket to derive from.
    Empty when the active rule registry declares no groups.
    """

    extension_remainder: ExtensionRemainder | None = None
    """What an extension bound withheld from `by_extension`, or `None` when it holds all."""

    provenance: Provenance | None = None

    @property
    def entries(self) -> int:
        """Descendant entries of every kind.

        The sum the emptiness question is really about: ``bytes`` cannot answer it,
        because an empty file, a symlink and nothing at all all weigh nothing.
        """

        return self.files + self.dirs + self.others

    @property
    def is_empty(self) -> bool:
        """Whether this subtree holds no entries at all.

        Exact only for a value whose ``provenance.status`` is ``COMPLETE``. A partial
        roll-up has not accounted for its whole subtree, so zero here means "nothing
        found yet". ``Child.empty`` does that consulting for a listing row.
        """

        return self.entries == 0


@dataclass(frozen=True, slots=True)
class DirectoryTotals:
    """Subtree totals for one listing row: scalars, no per-extension breakdown.

    Not a ``SummaryRow``. A view's summary row answers "what does this query cover"; this
    answers "how big is this child, and is there anything in it", which is why it carries
    ``others`` and a summary row does not.
    """

    files: int
    dirs: int
    others: int
    """Descendant symlinks and other non-file, non-directory entries."""

    bytes: int
    allocated: int
    newest_mtime_ns: int | None

    @property
    def entries(self) -> int:
        """Descendant entries of every kind."""

        return self.files + self.dirs + self.others


@dataclass(frozen=True, slots=True)
class Child:
    name: str
    kind: EntryKind
    totals: DirectoryTotals | None
    """Subtree totals for a directory child, or ``None`` for anything else.

    Scalars, not a breakdown. A listing wants a size column per row; asking for the
    per-extension tallies per row cloned one map per child to render one number per
    child. Ask ``Index.rollup()`` for the breakdown of the one directory being inspected.
    """

    bytes: int | None
    allocated: int | None
    mtime_ns: int | None
    provenance: Provenance
    classification: Classification | None = None
    """What the index's rule registry makes of this child; ``None`` unless it is a file.

    Metadata-only: the name decides it and no file is opened. Here so a consumer can stop
    carrying a classifier of its own -- re-deriving it per row afterwards means answering
    in a second language, against a rule set with no way to stay in step with this one.
    """

    extension: str | None = None
    """The *logical* extension: this name's final two eligible components.

    The raw level of the shared format's two, which is the one a person reads off the
    name. It may differ from both the type and the parent's ``by_extension`` key:
    ``release.v2.zip`` is ``.v2.zip`` here, an ``archive`` by type, and on the ``.zip``
    pile. Filter on this; sum bytes by the breakdown's key.
    """

    tags: tuple[str, ...] = ()
    """Tags this child carries, in the enabled set's bit order.

    Empty unless the index was opened with ``tag_rules=``, which is the default. A name
    here is a named boolean fact about this entry alone -- never about its ancestors, so a
    file inside a ``dotfile`` directory is not itself tagged.
    """

    empty: bool | None = None
    """Whether this is a directory whose subtree is provably empty.

    ``None`` rather than ``False`` for anything undecidable: a non-directory, which has no
    subtree, and a directory whose roll-up is partial, which has not accounted for one. A
    partial subtree reporting zero entries means "nothing found yet", and a listing that
    greyed out such a row would be greying out a directory it had not finished reading.

    Decidable at all only because a roll-up counts symlinks and other non-file entries:
    before that, a subtree of a hundred symlinks was zero files, zero directories and zero
    bytes -- the same arithmetic as nothing.
    """


@dataclass(frozen=True, slots=True)
class Work:
    """What one read actually did, beside the answer rather than inside it.

    Execution telemetry, not a fact about the tree: two reads that answer identically can
    do very different amounts of work, and the difference is what a serving loop needs to
    see. It also turns "no hidden O(index) pass" into something a benchmark can assert --
    a frequent read must be proportional to its own output or to maintained state, and
    ``entries_visited`` is where a regression shows up first.

    Two things are deliberately absent. **CPU time**: a read on a maintained index does no
    I/O, so its wall time is CPU plus whatever it waited for the guard, and
    ``lock_wait_ns`` already separates those. **Bytes across the binding**: the engine
    cannot see a binding, and ``name_bytes`` is the one term in a result that grows
    without bound -- the rest is a fixed per-row schema that ``rows`` and ``tally_rows``
    multiply.
    """

    entries_visited: int
    """Index entries this read examined, including those walked past to find a path.

    The load-bearing number. A read of maintained state visits its path's depth plus the
    rows it returns; one that visits a subtree is doing an aggregate pass, and says so
    here whatever its answer looks like.
    """

    dirs_visited: int
    """Directories among them."""

    rows: int
    """Rows the result carries."""

    tally_rows: int
    """Extension and group tallies the result *examined*, which a bound may exceed.

    Bounding a roll-up's extension rows still ranks every tally to decide which survive,
    so a read whose rows look bounded can still be doing work that is not.
    """

    name_bytes: int
    """Bytes of entry and extension names the result carries."""

    lock_wait_ns: int
    """Nanoseconds spent waiting for the read guard.

    Separate from ``wall_ns`` because a slow read and a read behind a long write are
    different problems with different fixes.
    """

    wall_ns: int
    """Nanoseconds from entering the call to returning, guard wait included."""

    @property
    def wall_seconds(self) -> float:
        """``wall_ns`` as seconds, for reporting rather than for arithmetic."""

        return self.wall_ns / 1_000_000_000


@dataclass(frozen=True, slots=True)
class ChildRemainder:
    """The children a page does not carry, as their share of the directory's totals.

    A page's rows plus this account for the directory exactly, so a consumer showing
    fifty of eight hundred children can still say honestly what the other seven hundred
    and fifty come to.

    It is the complement of *this page*, not of everything delivered so far: on page two
    it counts page one's rows as well, which is what keeps it exact on every page without
    a cursor that has to carry a running total. ``ChildPage.next``, not this, says whether
    more pages remain.

    No newest-mtime field: a maximum cannot be subtracted back out, and a figure that is
    sometimes wrong is worse than one that is absent.
    """

    rows: int
    """Child rows this page does not carry."""

    files: int
    """Files those rows account for."""

    dirs: int
    """Directories those rows account for, counting a withheld directory row itself."""

    others: int
    """Symlinks and other non-file, non-directory entries those rows account for."""

    bytes: int
    allocated: int


@dataclass(frozen=True, slots=True)
class ChildPage:
    """One page of a directory's children, with the rest accounted for beside it."""

    rows: tuple[Child, ...]
    """The rows this page carries, in name order."""

    remainder: ChildRemainder | None = None
    """What this page does not carry, or ``None`` when it carries the whole directory."""

    next: str | None = None
    """Cursor to pass as ``after`` for the next page; ``None`` at the end.

    This, not ``remainder``, is what says whether paging continues: a later page's
    remainder counts earlier pages' rows too, so it stays present on the last page.
    """

    @property
    def truncated(self) -> bool:
        """Whether this page carries fewer than the directory's children."""

        return self.remainder is not None

    @property
    def has_next(self) -> bool:
        """Whether another page follows this one."""

        return self.next is not None


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
class GroupRow:
    """One browsing group's row.

    Carries the label as well as the id because a browsing view exists to be read: the id
    is the stable key to group by, the label is what goes on the row.
    """

    id: str
    label: str
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
    classification: Classification | None = None
    """What the active rule registry makes of this row; ``None`` unless it is a file.

    Metadata-only, and filled after the view's bound: a bounded preset classifies the rows
    it emits rather than every row it considered.
    """

    extension: str | None = None
    """The *logical* extension: this row's final two eligible components.

    The raw level, which may differ from the type and the roll-up bucket.
    """

    tags: tuple[str, ...] = ()
    """Tags this row carries, in the enabled set's bit order.

    Empty unless the index was opened with ``tag_rules=``, which is the default: a caller
    who does not ask for tags pays nothing for them.
    """


@dataclass(frozen=True, slots=True)
class Remainder:
    """What a depth or limit bound withheld from one tree node's children.

    "Truncate freely, never silently": a caller showing the rows it was given can also
    show what it was not given, which is what a treemap's "other" cell is. Fields are the
    withheld rows' own aggregates summed, so these bytes plus the emitted children's
    account for every directory beneath the node.
    """

    rows: int
    """Directory rows not emitted beneath this node."""

    files: int
    """Files in those withheld subtrees."""

    dirs: int
    """Directories nested inside those withheld subtrees.

    The withheld rows themselves are `rows`, so `rows + dirs` is every directory the
    bound hid.
    """

    bytes: int
    """Apparent bytes in those withheld subtrees."""

    allocated: int
    """Allocated bytes in those withheld subtrees."""


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
    """Whether any child row was withheld. `remainder` says how much."""

    remainder: Remainder | None
    """The withheld aggregate, or `None` when nothing was withheld."""

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
class GroupsSection:
    """One row per browsing group the active rule registry declares.

    A different question from :class:`MetricsSection` under ``View.FAMILIES``, not a
    coarser answer to the same one: a family says which analyzer may open a file, so every
    image, video, PDF, and archive is ``binary``. A group says where a reader would look
    for it.
    """

    view: View
    groups: tuple[GroupRow, ...]
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
    SummarySection | ExtensionsSection | GroupsSection | FilesSection | TreeSection | MetricsSection
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
    #: today, the views `full` had to drop for want of an analyzer. Carried as values
    #: rather than left inside the text rendering, because a caller reading `sections`
    #: would otherwise find one absent with no way to learn why (fdu-7wd1). Deliberately
    #: not in `as_dict`: the wire envelope excludes them, and a machine consumer reads
    #: the omission from which sections are present.
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
class ScanScope:
    """The scan semantics an index represents, and the rule sets behind them.

    The fingerprints are what a consumer cache key should derive from: a change to any of
    them means the same tree answers differently, and an answer cached across one is wrong
    in a way no field of the answer reveals.
    """

    max_depth: int | None
    follow_symlinks: bool
    one_filesystem: bool
    tag_rules_fingerprint: int
    type_rules_fingerprint: int
    reducers_fingerprint: int


@dataclass(frozen=True, slots=True)
class Bundle:
    """Several projections read under one guard, at one instant.

    A composed page must not straddle a commit. Answering a listing and its parent's
    totals with two calls lets a write land between them, and the page is then internally
    inconsistent in a way nothing in it reports -- the rows say one thing, the header
    another, and both are individually true.

    :attr:`clock` is the version every part of this bundle saw, so it is also the cursor
    to pass to :meth:`Index.since` next: a cache key derives from what was actually read
    rather than from a version sampled before dispatch.
    """

    clock: int
    """The version every part of this bundle saw, and the cursor to resume from."""

    root: Path
    entries: int
    """Live entries, including the root."""

    scope: ScanScope
    status: Status
    total: RollUp | None
    """Whole-tree totals, or ``None`` when they were not requested."""

    rollups: tuple[RollUp | None, ...]
    """One entry per requested path; ``None`` where it is absent or not a directory."""

    children: ChildPage | None
    """The requested directory's children, or ``None`` when no directory was named or it
    is absent -- distinct from a page with no rows, which means a directory with no
    children."""

    report: Report | None
    """The report the requested query produced, or ``None`` when none was requested.

    This is what makes a composed page whole. A listing beside a "recently changed" panel
    used to need two calls, and a write landing between them left the two halves
    describing different instants -- each individually true, and together wrong. Pass a
    :class:`Query` to :meth:`Index.read` and both come back from one guard, sharing this
    bundle's ``clock``.

    Not a second query language: it is the same :class:`Query` :meth:`Index.report` takes.
    """

    work: Work
    """What producing this bundle cost, in total.

    The guard wait is here and only here, because the projections waited together:
    attributing one wait to one of them would be inventing a number. It is also why
    measurement rides with the bundled read rather than with every accessor -- the
    bundled read is what an interactive client serves from.
    """

    projections: ProjectionWork
    """What each projection cost on its own.

    A total alone answers "this read was slow" without ever answering "which part of it",
    which is the question a serving loop has to act on. The parts ran in sequence inside
    one guard, so their wall times are genuinely theirs; ``lock_wait_ns`` stays zero on
    each, because that one is shared and stays on the bundle.
    """


@dataclass(frozen=True, slots=True)
class ProjectionWork:
    """What each projection of a bundled read cost on its own."""

    children: Work
    """Listing one directory's children."""

    total: Work
    """The whole-tree totals."""

    rollups: Work
    """Every requested per-directory roll-up, summed."""

    report: Work
    """Answering the requested query.

    Counts what the result carries -- rows and the bytes of their names -- and how long it
    took. It deliberately does not claim an ``entries_visited``: a report may serve from
    maintained roll-up state or re-aggregate by walking, and reporting a walk it did not
    do, or a zero for one it did, would be worse than reporting neither.
    """


@dataclass(frozen=True, slots=True)
class WalkTelemetry:
    """What one scan or refresh actually did, beside the report rather than inside it.

    Telemetry about a run, not a fact about the tree: two runs that answer identically
    can do very different amounts of work, and that difference is what an embedder
    running its own measured loop needs to see. These are the numbers ``fdu`` prints as
    its footer.

    Fields describe the most recent operation alone. They are not running totals, so a
    loop that refreshes on every change reads each refresh's own cost rather than a sum
    that grows without bound.
    """

    walked_files: int
    """Regular files whose metadata was observed."""

    walked_bytes: int
    """Apparent bytes represented by those walked files."""

    fresh_files: int
    """Content-analysis candidates processed from the filesystem."""

    bytes_read: int
    """Bytes actually returned by those fresh reads."""

    analysis_ns: int
    """Wall time spent on fresh analysis candidates."""

    cached_files: int
    """Content-analysis records restored from the sidecar instead of re-read."""

    cached_bytes: int
    """Apparent bytes represented by those restored records."""

    source: ReportSource
    """Which metadata cache tier produced the index."""

    @property
    def analysis_seconds(self) -> float:
        """`analysis_ns` as seconds, for reporting rather than for arithmetic."""

        return self.analysis_ns / 1e9


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
                format=str(format),
            ),
        )


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


def scan_scope_from_dict(value: dict[str, Any]) -> ScanScope:
    depth = value["max_depth"]
    return ScanScope(
        max_depth=None if depth is None else int(depth),
        follow_symlinks=bool(value["follow_symlinks"]),
        one_filesystem=bool(value["one_filesystem"]),
        tag_rules_fingerprint=int(value["tag_rules_fingerprint"]),
        type_rules_fingerprint=int(value["type_rules_fingerprint"]),
        reducers_fingerprint=int(value["reducers_fingerprint"]),
    )


def walk_telemetry_from_dict(value: dict[str, Any]) -> WalkTelemetry:
    return WalkTelemetry(
        walked_files=int(value["walked_files"]),
        walked_bytes=int(value["walked_bytes"]),
        fresh_files=int(value["fresh_files"]),
        bytes_read=int(value["bytes_read"]),
        analysis_ns=int(value["analysis_ns"]),
        cached_files=int(value["cached_files"]),
        cached_bytes=int(value["cached_bytes"]),
        source=ReportSource(str(value["source"])),
    )


def _file_row(row: dict[str, Any]) -> FileRow:
    classification = row.get("classification")
    extension = row.get("extension")
    return FileRow(
        path=Path(str(row["path"])),
        kind=EntryKind(str(row["kind"])),
        bytes=int(row["bytes"]),
        allocated=int(row["allocated"]),
        mtime_ns=int(row["mtime_ns"]),
        classification=(
            None if classification is None else classification_from_dict(classification)
        ),
        extension=None if extension is None else str(extension),
        tags=tuple(str(tag) for tag in row.get("tags", ())),
    )


def classification_from_dict(value: dict[str, Any]) -> Classification:
    flags = value["flags"]
    group = value.get("group")
    return Classification(
        group=None if group is None else str(group),
        file_type=str(value["file_type"]),
        family=ContentFamily(str(value["family"])),
        source=DetectionSource(str(value["source"])),
        confidence=DetectionConfidence(str(value["confidence"])),
        flags=ClassificationFlags(
            generated=bool(flags["generated"]),
            vendored=bool(flags["vendored"]),
            documentation=bool(flags["documentation"]),
        ),
    )


def provenance_from_dict(value: dict[str, Any]) -> Provenance:
    return Provenance(
        source=ValueSource(str(value["source"])),
        observed_at_ns=int(value["observed_at_ns"]),
        status=Coverage(str(value["status"])),
        reason=None if value.get("reason") is None else CoverageReason(str(value["reason"])),
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
        others=int(value.get("others", 0)),
        bytes=int(value["bytes"]),
        allocated=int(value["allocated"]),
        newest_mtime_ns=int(value["newest_mtime_ns"]),
        by_extension=MappingProxyType(tallies),
        by_group=MappingProxyType(
            {
                str(group): ExtensionTally(
                    files=int(tally["files"]),
                    bytes=int(tally["bytes"]),
                    allocated=int(tally["allocated"]),
                )
                for group, tally in value.get("by_group", {}).items()
            }
        ),
        extension_remainder=_extension_remainder(value.get("extension_remainder")),
        provenance=provenance,
    )


def _extension_remainder(value: object) -> ExtensionRemainder | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise TypeError("expected an extension remainder")
    return ExtensionRemainder(
        extensions=int(value["extensions"]),
        files=int(value["files"]),
        bytes=int(value["bytes"]),
        allocated=int(value["allocated"]),
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
        remainder=_remainder(value.get("remainder")),
        children=tuple(_tree(child) for child in value["children"]),
    )


def _remainder(value: object) -> Remainder | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        raise TypeError("expected a tree-node remainder")
    return Remainder(
        rows=int(value["rows"]),
        files=int(value["files"]),
        dirs=int(value["dirs"]),
        bytes=int(value["bytes"]),
        allocated=int(value["allocated"]),
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
            sections.append(SummarySection(view, SummaryRow(**row)))
        elif view is View.EXTENSIONS:
            rows = raw["extensions"]
            if not isinstance(rows, list):
                raise TypeError("extensions section must be a list")
            sections.append(
                ExtensionsSection(view, tuple(ExtensionRow(**row) for row in rows), _bound(raw))
            )
        elif view is View.GROUPS:
            rows = raw["groups"]
            if not isinstance(rows, list):
                raise TypeError("groups section must be a list")
            sections.append(
                GroupsSection(view, tuple(GroupRow(**row) for row in rows), _bound(raw))
            )
        elif view in (View.FILES, View.LARGEST, View.RECENT):
            rows = raw["files"]
            if not isinstance(rows, list):
                raise TypeError("files section must be a list")
            sections.append(
                FilesSection(
                    view,
                    tuple(_file_row(cast("dict[str, Any]", row)) for row in rows),
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
