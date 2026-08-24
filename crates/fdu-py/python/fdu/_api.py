"""Typed facade over the private native extension."""

from __future__ import annotations

import json
from collections.abc import Iterator, Sequence
from dataclasses import replace
from datetime import UTC, datetime
from pathlib import Path
from types import TracebackType
from typing import Any, cast

from . import _native
from ._models import (
    AnalysisOptions,
    Bound,
    Bundle,
    CachePolicy,
    CacheStatus,
    Change,
    ChangeKind,
    ChangeSet,
    Child,
    ChildPage,
    ChildRemainder,
    Classification,
    Cursor,
    DirectoryTotals,
    EntryKind,
    Format,
    ProjectionWork,
    Provenance,
    Query,
    RefreshResult,
    Report,
    RollUp,
    ScanOptions,
    Status,
    WalkTelemetry,
    WatchOptions,
    Work,
    classification_from_dict,
    provenance_from_dict,
    report_from_dict,
    rollup_from_dict,
    scan_scope_from_dict,
    status_from_dict,
    walk_telemetry_from_dict,
)

#: Named so the arithmetic in `_epoch_nanos` reads as units rather than as digits.
_SECONDS_PER_DAY = 86_400
_NANOS_PER_SECOND = 1_000_000_000
_NANOS_PER_MICROSECOND = 1_000
_EPOCH = datetime(1970, 1, 1, tzinfo=UTC)


class FduError(RuntimeError):
    """Base class for public fdu API errors."""


class InvalidArgumentError(FduError, ValueError):
    """A public option or query is invalid."""


class FilesystemError(OSError, FduError):
    """A filesystem failure that prevented an operation from producing a result.

    Reads the way the CLI reports the same failure, because it is the same failure::

        fdu: I/O error at missing: No such file or directory (os error 2)

    OSError's own rendering leads with ``[Errno None]`` for anything raised from the
    engine, since a Rust ``io::Error`` kind does not always carry an errno. That is worse
    than uninformative -- it states an error number that does not exist -- so the path and
    the reason are put first and the errno is shown only when there is one.
    """

    @property
    def path(self) -> Path | None:
        """The path whose operation failed, when the failure named one."""

        return Path(self.filename) if self.filename else None

    def __str__(self) -> str:
        # strerror already carries the operating system's own detail, so an errno
        # suffix would repeat it -- "No such file or directory (os error 2) (errno 2)".
        reason = self.strerror or "operation failed"
        if self.filename:
            return f"I/O error at {self.filename}: {reason}"
        return reason


def _bound(value: int | Bound | str | None) -> str | None:
    if value is None:
        return None
    return value.value if isinstance(value, Bound) else str(value)


def _when(value: datetime | str | None) -> str | None:
    if isinstance(value, datetime):
        # The engine's time grammar requires an explicit UTC offset because it has no
        # time-zone database of its own. Python does, and a naive datetime means local
        # time (as in `datetime.timestamp()`), so resolve it here instead of handing
        # the native layer a timestamp it must reject.
        aware = value.astimezone() if value.tzinfo is None else value
        return aware.isoformat()
    return value


def _call(function: Any, /, *args: object, **kwargs: object) -> Any:
    try:
        return function(*args, **kwargs)
    except OSError as error:
        raise FilesystemError(error.errno, error.strerror, error.filename) from error
    except ValueError as error:
        raise InvalidArgumentError(str(error)) from error
    except RuntimeError as error:
        raise FduError(str(error)) from error


def _query_kwargs(query: Query) -> dict[str, object]:
    selection = query.selection
    return {
        # `None` rather than an empty list: the binding distinguishes "the caller named
        # no view" -- derive one from the analyzers -- from "the caller named none", which
        # would be a report with no sections.
        # A raw spec goes through as one element: the binding joins the list with commas
        # before handing it to the library, so a spec with its own commas survives intact
        # and is parsed by the one grammar.
        "views": (
            [query.views]
            if isinstance(query.views, str)
            else [view.value for view in query.views] or None
        ),
        "include": list(selection.include),
        "exclude": list(selection.exclude),
        "min_size": str(selection.min_size) if selection.min_size is not None else None,
        "modified_since": _when(selection.modified_since),
        "modified_before": _when(selection.modified_before),
        "kind": [kind.value for kind in selection.kinds],
        "tags": list(selection.tags),
        "not_tags": list(selection.not_tags),
        "depth": _bound(selection.depth),
        "limit": _bound(selection.limit),
        "sort": selection.sort.value if selection.sort is not None else None,
        "reverse": selection.reverse,
        "size": selection.size.value,
        "words_per_page": query.words_per_page,
    }


def _cache_status(value: dict[str, Any]) -> CacheStatus:
    return CacheStatus(
        path=Path(value["path"]),
        bytes=int(value["bytes"]),
        content_bytes=(int(value["content_bytes"]) if value["content_bytes"] is not None else None),
        recognized=bool(value["recognized"]),
        root=Path(value["root"]) if value["root"] is not None else None,
        entries=int(value["entries"]) if value["entries"] is not None else None,
        max_depth=int(value["max_depth"]) if value["max_depth"] is not None else None,
        one_filesystem=(
            bool(value["one_filesystem"]) if value["one_filesystem"] is not None else None
        ),
    )


class Watch(Iterator[tuple[Change, ...]]):
    """Closable iterator of event-driven change batches."""

    __slots__ = ("_native",)

    def __init__(self, native: _native.Watch) -> None:
        self._native = native

    def __iter__(self) -> Watch:
        return self

    def __next__(self) -> tuple[Change, ...]:
        return tuple(_change(item) for item in next(self._native))

    @property
    def dirty_rollups(self) -> tuple[Path, ...]:
        """Directories whose roll-ups the batch just yielded may have moved.

        Root first, sorted, and never filtered by the selection: a change the selection
        hides still moves the totals its ancestors report. Invalidate exactly these and
        keep the rest, rather than dropping every cached per-directory answer.

        Scoped to the most recent batch, so read it after each iteration step.
        """

        return tuple(Path(path) for path in self._native.dirty_rollups)

    def report(self) -> Report:
        """The live answer, as of now, from the index this session has been updating.

        A watch run has no final answer: the aggregates are true only until the next
        change, so redrawing them needs the session's own index rather than the one it was
        opened from. Reporting the opened index repaints numbers that stopped being true
        at the first event -- a display that looks like it works and does not.
        """

        handle = _call(self._native.report)
        wire: dict[str, Any] = json.loads(_call(handle.render, "json", False))
        notes = tuple(_call(handle.notes))

        def renderer(format: str, color: bool) -> str:
            # A snapshot: rendering twice gives the same answer, where re-reporting the
            # session would quietly give a newer one.
            return cast(str, _call(handle.render, format, color))

        return replace(report_from_dict(wire, notes), _renderer=renderer)

    def close(self) -> None:
        self._native.close()

    def __enter__(self) -> Watch:
        return self

    def __exit__(
        self,
        _exc_type: type[BaseException] | None,
        _exc_value: BaseException | None,
        _traceback: TracebackType | None,
    ) -> bool:
        self.close()
        return False


class Index:
    """A retained in-process index serving many structured queries."""

    __slots__ = ("_native",)

    def __init__(self, native: _native.Index) -> None:
        self._native = native

    @property
    def root(self) -> Path:
        return Path(self._native.root)

    @property
    def clock(self) -> int:
        return self._native.clock

    @property
    def status(self) -> Status:
        return status_from_dict(_call(self._native.status))

    @property
    def telemetry(self) -> WalkTelemetry:
        """What the most recent scan or refresh actually did.

        Beside the report rather than inside it, so a machine-readable answer stays a
        fact about the tree and does not vary with how it happened to be produced.
        Read it right after the call whose cost you mean: a later refresh replaces it.
        """

        return walk_telemetry_from_dict(self._native.telemetry)

    def __len__(self) -> int:
        return len(self._native)

    def report(self, query: Query | None = None) -> Report:
        """A report answering `query` from the retained index, as of now.

        A snapshot. The index goes on changing underneath it -- that is what an index is
        for -- but the value returned does not, so `as_dict` and `render` are two
        serializations of one answer rather than two answers taken at different times.
        """

        selected = query if query is not None else Query()
        handle = _call(self._native.report_handle, **_query_kwargs(selected))
        wire: dict[str, Any] = json.loads(_call(handle.render, "json", False))
        report = report_from_dict(wire, tuple(_call(handle.notes)))
        errors = self.status.errors

        # Bound to the finished report, not to the query: re-projecting the index per
        # format would let one `Report` answer differently each time it was rendered
        # (fdu-4gno).
        def renderer(format: str, color: bool) -> str:
            return cast(str, _call(handle.render, format, color))

        return replace(
            report,
            status=replace(report.status, errors=errors),
            _renderer=renderer,
        )

    def total(self, extensions: int | None = None) -> RollUp:
        """Roll-up totals for the whole tree.

        `extensions` bounds the per-extension breakdown to that many rows, largest by
        apparent bytes; what it drops is aggregated into `extension_remainder` rather
        than discarded. `None` keeps every row.
        """

        return rollup_from_dict(_call(self._native.total, extensions), self.provenance())

    def rollup(self, path: str | Path, extensions: int | None = None) -> RollUp | None:
        """Roll-up totals for one directory, or `None` if it is absent or not a directory.

        `extensions` bounds the per-extension breakdown; see `total`.
        """

        raw = _call(self._native.rollup, path, extensions)
        return None if raw is None else rollup_from_dict(raw, self.provenance(path))

    def children(
        self, path: str | Path = Path(), *, after: str | None = None, limit: int | None = None
    ) -> ChildPage | None:
        """One page of a directory's children, in one call.

        `None` when the path is absent or is not a directory, which is distinct from a
        page with no rows.

        `after` resumes strictly past a child's name and `limit` bounds the rows, so a
        wide directory costs what is drawn rather than what it holds. Pass the returned
        `next` back as `after` to continue; it is `None` at the end.

        A name is the cursor rather than an offset because a directory that gains or
        loses an entry between two pages shifts every offset after it, which silently
        repeats or skips rows.

        Rows carry scalar subtree totals. For the per-extension breakdown of the one
        directory being inspected, call `rollup()`.
        """

        raw = _call(self._native.children, path, after, limit)
        return None if raw is None else _child_page(raw)

    def read(
        self,
        *,
        children_of: str | Path | None = None,
        rollups: Sequence[str | Path] = (),
        total: bool = False,
        extensions: int | None = None,
        after: str | None = None,
        limit: int | None = None,
        query: Query | None = None,
    ) -> Bundle:
        """Several projections read under one guard, at one instant.

        A composed page must not straddle a commit: answering a listing and its parent's
        totals with two calls lets a write land between them, and the page is then
        internally inconsistent in a way nothing in it reports.

        `query` adds a full report to the bundle, answered at the same instant as the
        rows. This is what a listing beside a "recently changed" panel needs: two calls
        would let a write land between them and leave the halves describing different
        moments, each individually true and together wrong. It is the same `Query`
        `report()` takes, not a second grammar.

        The returned `clock` is the version every part saw, so it is also the cursor to
        pass to `since()` next -- a cache key derives from what was actually read rather
        than from a version sampled before dispatch.

        It is also one crossing of the language boundary and one lock acquisition,
        instead of one of each per call.
        """

        # The report's own row bound is `limit_rows`: `limit` on this call already means
        # the child page's, and one name for two bounds is how a caller ends up bounding
        # the listing when they meant the report.
        report_kwargs: dict[str, object] = {"report": False}
        if query is not None:
            report_kwargs = dict(_query_kwargs(query))
            report_kwargs["limit_rows"] = report_kwargs.pop("limit")
            report_kwargs["report"] = True

        value = _call(
            self._native.read,
            children_of,
            [str(path) for path in rollups],
            total,
            extensions,
            after,
            limit,
            **report_kwargs,
        )
        children = value["children"]
        report = value["report"]
        projections = value["projections"]
        return Bundle(
            clock=int(value["clock"]),
            root=Path(str(value["root"])),
            entries=int(value["entries"]),
            scope=scan_scope_from_dict(value["scope"]),
            status=status_from_dict(value),
            total=None if value["total"] is None else rollup_from_dict(value["total"]),
            rollups=tuple(
                None if roll is None else rollup_from_dict(roll) for roll in value["rollups"]
            ),
            children=None if children is None else _child_page(children),
            # Parsed by the same function `report()` uses, from the same wire shape, so a
            # bundled report cannot describe the tree differently from a standalone one.
            report=None if report is None else report_from_dict(report),
            work=_work(value["work"]),
            projections=ProjectionWork(
                children=_work(projections["children"]),
                total=_work(projections["total"]),
                rollups=_work(projections["rollups"]),
                report=_work(projections["report"]),
            ),
        )

    def provenance(self, path: str | Path = Path()) -> Provenance | None:
        value = _call(self._native.provenance, path)
        return None if value is None else provenance_from_dict(value)

    def refresh(self, path: str | Path | None = None) -> RefreshResult:
        """Re-reconcile against the filesystem, optionally scoped to one subtree.

        A scoped refresh is how a caller feeds hints from a watcher of its own: the
        mutation still arrives through the engine's one delta contract, and it costs the
        subtree rather than the whole tree.
        """

        value = _call(self._native.refresh, path)
        return RefreshResult(
            inserted=int(value["inserted"]),
            updated=int(value["updated"]),
            removed=int(value["removed"]),
            unchanged=int(value["unchanged"]),
            stale=int(value["stale"]),
            clock=int(value["clock"]),
            status=status_from_dict(value),
        )

    def cursor(self) -> Cursor:
        """The current resume position, to hand back to `since()`."""
        return _cursor(_call(self._native.cursor))

    def since(self, cursor: Cursor) -> ChangeSet:
        """Changes applied after `cursor`, and the position to resume from next.

        The returned cursor comes from the same read that produced the changes, so
        resuming from it is exact. A cursor from another opened index -- or from a
        previous run over the same tree -- raises rather than returning an empty set,
        because "nothing changed" and "I cannot place you" are different answers and only
        one of them is safe to believe.
        """
        value = _call(self._native.since, {"session": cursor.session, "clock": cursor.clock})
        return ChangeSet(
            truncated=bool(value["truncated"]),
            cursor=_cursor(value["cursor"]),
            changes=tuple(_change(item) for item in value["ops"]),
        )

    def watch(self, options: WatchOptions | None = None) -> Watch:
        selected = options if options is not None else WatchOptions()
        arguments = _query_kwargs(selected.query)
        arguments["interval"] = selected.interval
        arguments["poll_interval"] = selected.poll_interval
        native = _call(
            self._native.watch,
            **arguments,
        )
        return Watch(native)


def _work(value: dict[str, Any]) -> Work:
    """What one read cost, parsed beside what it answered."""

    return Work(
        entries_visited=int(value["entries_visited"]),
        dirs_visited=int(value["dirs_visited"]),
        rows=int(value["rows"]),
        tally_rows=int(value["tally_rows"]),
        name_bytes=int(value["name_bytes"]),
        lock_wait_ns=int(value["lock_wait_ns"]),
        wall_ns=int(value["wall_ns"]),
    )


def _child_page(value: dict[str, Any]) -> ChildPage:
    """One page of children, parsed the same way wherever it came from."""

    rows: list[dict[str, Any]] = value["rows"]
    remainder: dict[str, Any] | None = value.get("remainder")
    cursor = value.get("next")
    return ChildPage(
        rows=tuple(_child(item) for item in rows),
        remainder=None if remainder is None else _child_remainder(remainder),
        next=None if cursor is None else str(cursor),
    )


def _child_remainder(value: dict[str, Any]) -> ChildRemainder:
    return ChildRemainder(
        rows=int(value["rows"]),
        files=int(value["files"]),
        dirs=int(value["dirs"]),
        others=int(value["others"]),
        bytes=int(value["bytes"]),
        allocated=int(value["allocated"]),
    )


def _child(item: dict[str, Any]) -> Child:
    """One listing row, parsed the same way wherever it came from."""

    provenance = provenance_from_dict(item["provenance"])
    classification = item.get("classification")
    extension = item.get("extension")
    # A directory row carries subtree totals and a non-directory carries its own attrs;
    # the two field sets are disjoint, so presence of `files` is what tells them apart.
    totals = (
        DirectoryTotals(
            files=int(item["files"]),
            dirs=int(item["dirs"]),
            others=int(item["others"]),
            bytes=int(item["bytes"]),
            allocated=int(item["allocated"]),
            newest_mtime_ns=int(item["newest_mtime_ns"]) or None,
        )
        if item.get("files") is not None
        else None
    )
    empty = item.get("empty")
    return Child(
        name=str(item["name"]),
        kind=EntryKind(item["kind"]),
        totals=totals,
        bytes=None if totals is not None else int(item["bytes"]),
        allocated=None if totals is not None else int(item["allocated"]),
        mtime_ns=None if totals is not None else int(item["mtime_ns"]),
        provenance=provenance,
        classification=(
            None if classification is None else classification_from_dict(classification)
        ),
        extension=None if extension is None else str(extension),
        tags=tuple(str(tag) for tag in item.get("tags", ())),
        empty=None if empty is None else bool(empty),
    )


def _cursor(value: object) -> Cursor:
    if not isinstance(value, dict):
        raise TypeError("expected a cursor mapping")
    mapping = cast("dict[str, Any]", value)
    return Cursor(session=int(mapping["session"]), clock=int(mapping["clock"]))


def _change(value: dict[str, Any]) -> Change:
    raw_kind = str(value["op"])
    return Change(
        clock=int(value["clock"]),
        path=Path(value["path"]),
        kind=ChangeKind(raw_kind),
        entry_kind=EntryKind(value["kind"]) if value.get("kind") is not None else None,
        bytes=int(value["bytes"]) if value.get("bytes") is not None else None,
        allocated=int(value["allocated"]) if value.get("allocated") is not None else None,
        mtime_ns=int(value["mtime_ns"]) if value.get("mtime_ns") is not None else None,
        reason=str(value["reason"]) if value.get("reason") is not None else None,
    )


class TypeRegistry:
    """A compiled set of file-type rules.

    fdu ships a taxonomy; this is how a caller supplies its own. Rules are parsed,
    validated, indexed, and fingerprinted in Rust -- the same code that reads fdu's own
    manifest at build time -- so a manifest this accepts is one that would have compiled,
    and classification runs at the engine's speed rather than the caller's.

    Build one and reuse it: parsing is the cost, and a registry is read-only afterwards.
    Pass it as :attr:`ScanOptions.type_rules`.

    Rules are scope, not selection. They decide what every type row *means*, so an index
    built under one registry is not answerable under another and the snapshot cache
    invalidates on :attr:`fingerprint`.
    """

    __slots__ = ("_native",)

    def __init__(self, native: _native.TypeRegistry) -> None:
        self._native = native

    @staticmethod
    def from_manifest(source: str, expect_fingerprint: int | None = None) -> TypeRegistry:
        """Parse rules in the ``[[kind]]`` manifest dialect.

        Raises :class:`fdu.InvalidArgumentError` with the parser's own line-numbered
        message for a manifest that would classify ambiguously -- a duplicate id, an
        unknown family, or a key two rules both claim.

        Pass ``expect_fingerprint`` when the packet's supplier recorded an identity for
        it, and the open fails on disagreement rather than classifying under rules nobody
        chose. Both fingerprints were always readable, so a caller could always have
        compared them; what it could not do was be unable to skip the comparison.
        """

        return TypeRegistry(_call(_native.TypeRegistry.from_manifest, source, expect_fingerprint))

    @staticmethod
    def compiled() -> TypeRegistry:
        """The rules fdu ships, used when a scan names no others."""

        return TypeRegistry(_call(_native.TypeRegistry.compiled))

    @property
    def fingerprint(self) -> int:
        """Identity of these rules, which a snapshot and a content sidecar both record."""

        return self._native.fingerprint

    @property
    def rule_count(self) -> int:
        return self._native.rule_count

    @property
    def extension_count(self) -> int:
        return self._native.extension_count

    @property
    def filename_count(self) -> int:
        return self._native.filename_count

    def type_ids(self) -> tuple[str, ...]:
        """Every stable type identifier these rules can produce, in manifest order."""

        return tuple(_call(self._native.type_ids))

    def classify(self, path: str | Path) -> Classification:
        """What these rules make of one path, from its name alone.

        No file is opened, so this answers for a path that does not exist -- which is
        what makes it usable for checking a taxonomy before scanning with it.
        """

        return classification_from_dict(_call(self._native.classify, path))

    def __repr__(self) -> str:
        return repr(self._native)


def _native_rules(registry: TypeRegistry | None) -> _native.TypeRegistry | None:
    return None if registry is None else registry._native


def open(
    root: str | Path,
    *,
    cache: CachePolicy = CachePolicy.AUTO,
    scan: ScanOptions | None = None,
    analysis: AnalysisOptions | None = None,
) -> Index:
    """Open a root using the requested cache policy, then return a retained index."""

    scan_options = scan if scan is not None else ScanOptions()
    analysis_options = analysis if analysis is not None else AnalysisOptions()
    native = _call(
        _native.open,
        root,
        cache=cache.value,
        max_depth=scan_options.max_depth,
        one_filesystem=scan_options.one_filesystem,
        order=str(scan_options.order),
        threads=scan_options.threads,
        type_rules=_native_rules(scan_options.type_rules),
        tag_rules=list(scan_options.tag_rules),
        analyze=str(analysis_options.analyze),
        analysis_workers=analysis_options.workers,
    )
    return Index(native)


def scan(
    root: str | Path,
    *,
    scan: ScanOptions | None = None,
    analysis: AnalysisOptions | None = None,
) -> Index:
    """Walk a root without reading or writing a snapshot cache."""

    scan_options = scan if scan is not None else ScanOptions()
    analysis_options = analysis if analysis is not None else AnalysisOptions()
    native = _call(
        _native.scan,
        root,
        max_depth=scan_options.max_depth,
        one_filesystem=scan_options.one_filesystem,
        order=str(scan_options.order),
        threads=scan_options.threads,
        type_rules=_native_rules(scan_options.type_rules),
        tag_rules=list(scan_options.tag_rules),
        analyze=str(analysis_options.analyze),
        analysis_workers=analysis_options.workers,
    )
    return Index(native)


def report(
    root: str | Path,
    query: Query | None = None,
    *,
    cache: CachePolicy = CachePolicy.AUTO,
    scan: ScanOptions | None = None,
    analysis: AnalysisOptions | None = None,
) -> Report:
    """One report, retaining the least state the request needs.

    The contract the command line runs under, and until now the only way to get it was to
    be the command line. :func:`open` takes the session path: it retains an index and
    writes a snapshot, which is right for a caller asking many questions and wrong for one
    asking a single question. An unfiltered summary is answered by a transient tier that
    retains nothing, so writing a snapshot for it caches state the walk never saved --
    which meant a Python caller left cache state on a tree that the same command would
    not have, visible to a later cache-only read.

    Use :func:`open` when you will ask more than one question; the index is the point.
    """

    scan_options = scan if scan is not None else ScanOptions()
    analysis_options = analysis if analysis is not None else AnalysisOptions()
    selected = query if query is not None else Query()
    handle = _call(
        _native.report_once,
        str(root),
        cache=str(cache),
        max_depth=scan_options.max_depth,
        one_filesystem=scan_options.one_filesystem,
        order=str(scan_options.order),
        threads=scan_options.threads,
        type_rules=_native_rules(scan_options.type_rules),
        tag_rules=list(scan_options.tag_rules),
        analyze=str(analysis_options.analyze),
        analysis_workers=analysis_options.workers,
        **_query_kwargs(selected),
    )
    wire: dict[str, Any] = json.loads(_call(handle.render, "json", False))
    parsed = report_from_dict(wire, tuple(_call(handle.notes)))

    def renderer(format: str, color: bool) -> str:
        # The handle owns the finished report, so a second format costs no second walk.
        # Rebuilding it from the query would mean rescanning, which is the cost a one-shot
        # exists to avoid.
        return cast(str, _call(handle.render, format, color))

    return replace(parsed, _renderer=renderer)


def watch_rule(at: datetime | int) -> str:
    """The rule that separates one watch repaint from the one before it.

    A watch run has no final answer and so no performance footer, which leaves consecutive
    text repaints with nothing between them. A blank line will not do -- that already
    separates two views inside one report -- so the rule carries the instant it was drawn.

    Takes nanoseconds since the epoch, which is what `Change.mtime_ns` already carries, so
    a caller repainting after a batch has the instant to hand without converting through
    anything. A `datetime` is accepted too and is exact to its own microsecond resolution.

    An aware `datetime` only: a naive one has no instant to render, and reading it as local
    time would silently move the rule by the machine's UTC offset while still printing `Z`.
    """

    return _call(_native.watch_rule, _epoch_nanos(at))


def _epoch_nanos(at: datetime | int) -> int:
    """Nanoseconds since the epoch, without a float in the path.

    `datetime.timestamp()` returns a float, and at present-day magnitudes a float64 cannot
    resolve nanoseconds -- it quantizes to a few hundred of them, so `mtime_ns` round-tripped
    through one comes back a different instant. Integer arithmetic on the timedelta is exact
    to the microsecond a `datetime` actually holds.
    """

    if isinstance(at, int):
        return at
    if at.tzinfo is None or at.tzinfo.utcoffset(at) is None:
        raise InvalidArgumentError(
            "watch_rule needs an aware datetime or nanoseconds since the epoch; a naive "
            "datetime names no instant, and reading it as local time would move the rule "
            "while still printing UTC"
        )
    since_epoch = at - _EPOCH
    return (
        since_epoch.days * _SECONDS_PER_DAY + since_epoch.seconds
    ) * _NANOS_PER_SECOND + since_epoch.microseconds * _NANOS_PER_MICROSECOND


def render_cache_status(
    caches: Sequence[CacheStatus | Path | str], format: Format = Format.TEXT
) -> str:
    """Render cache files exactly as ``fdu --cache-status`` prints them.

    The same renderer the CLI uses, in every format, so a caller can print what fdu prints
    instead of inventing a layout that will drift from it.

    Named for the files rather than for the values: each entry is only a way of naming a
    cache file, and the file is **re-read at render time**. Passing a :class:`CacheStatus`
    obtained earlier therefore renders what that file says now, not the fields the value
    carries -- a status held across a `clear_cache` renders as no snapshots at all. That
    keeps one definition of what a status is, and means a caller cannot hand the renderer a
    status the engine never produced, but it is I/O rather than formatting and the name says
    so (fdu-bogi).
    """

    paths = [str(cache.path) if isinstance(cache, CacheStatus) else str(cache) for cache in caches]
    return cast(str, _call(_native.render_cache_status, paths, str(format)))


def cache_path(root: str | Path) -> Path | None:
    value = _call(_native.cache_path, root)
    return None if value is None else Path(value)


def cache_status(root: str | Path) -> CacheStatus | None:
    value = _call(_native.cache_status, root)
    return None if value is None else _cache_status(value)


def list_caches(root: str | Path = Path()) -> tuple[CacheStatus, ...]:
    return tuple(_cache_status(value) for value in _call(_native.list_caches, root))


def clear_cache(root: str | Path) -> bool:
    return bool(_call(_native.clear_cache, root))


def clear_all_caches(root: str | Path = Path()) -> int:
    return int(_call(_native.clear_all_caches, root))


def _main() -> int:
    """Console-script boundary; argument parsing remains in the native CLI."""

    return _native.main()
