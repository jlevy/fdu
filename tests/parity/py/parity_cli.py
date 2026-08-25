"""fdu's command line, served entirely by the public Python package.

This is the instrument *and* the measurement. It exists so the golden corpus can be
replayed against the Python API rather than the Rust binary, and every session that
still produces fdu's bytes is a claim the binding holds. It is deliberately not a
wrapper around the binary, which would test nothing.

Two properties it inherits from the name it stands in for. Its diagnostics say ``fdu:``
rather than ``fdu-py:``, because the corpus pins those strings and it is impersonating
fdu rather than announcing itself. And it reimplements nothing the library owns: view
resolution, default derivation, ``full`` expansion, and every value grammar live behind
the API, and are called rather than copied. If any of those had to be reimplemented
here, that would be exactly the drift this harness exists to catch.

The one place it does announce itself is ``--version``, which names the surface. That
keeps the committed deviation file non-empty by construction: an empty parity diff means
the shim never ran, which is the failure mode a naive harness reports as success.

Mirrors ``crates/fdu/src/cli.rs`` function for function so a reader can check the
mapping by name.
"""

from __future__ import annotations

import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import NoReturn

import fdu

PROGRAM = "fdu"

# Discovery surfaces the package does not carry: clap's own help rendering, and two
# static documents that live in the binary. Declining them is a decision, not an
# oversight, and any growth in this list is a regression worth arguing about.
DECLINED = frozenset({"--help", "-h", "--docs", "--skill"})


class UsageError(Exception):
    """A value the CLI grammar rejects; exit 2, like the binary."""


def _scan_options(args: Args) -> fdu.ScanOptions:
    """The scope axis as one typed value, matching the flags the command line takes."""
    order = parse_order(args.order) if args.order is not None else fdu.ScanOrder.BREADTH_FIRST
    return fdu.ScanOptions(
        max_depth=args.scan_depth,
        one_filesystem=args.one_filesystem,
        order=order,
        threads=args.threads,
        type_rules=load_type_rules(args.type_rules),
        tag_rules=tuple(args.tag_rules),
        promote=tuple(args.promote),
    )


def load_type_rules(path: str | None) -> fdu.TypeRegistry | None:
    """Read `--type-rules`, failing the way the command line fails.

    Both surfaces reject a bad manifest before any walk starts, with the engine parser's
    own message; the shim only has to prefix the flag the same way.
    """

    if path is None:
        return None
    try:
        source = Path(path).read_text(encoding="utf-8")
    except OSError as error:
        fail(f"{path}: {error.strerror} (os error {error.errno})")
    try:
        return fdu.TypeRegistry.from_manifest(source)
    except fdu.FduError as error:
        raise UsageError(f"{path}: {error}") from None


def fail(message: str, code: int = 2) -> NoReturn:
    print(f"{PROGRAM}: {message}", file=sys.stderr)
    raise SystemExit(code)


def parse_format(value: str) -> fdu.Format:
    try:
        return fdu.Format(value)
    except ValueError:
        known = ", ".join(f.value for f in fdu.Format)
        raise UsageError(f'invalid --format "{value}": expected one of {known}') from None


def parse_analysis(value: str) -> str:
    # The library owns this grammar, and AnalysisOptions takes the same comma list the
    # CLI does, so the value is passed through untouched: a new analyzer needs no change
    # here, and an invalid one is rejected by the library with the library's wording.
    return value


def parse_cache(value: str) -> fdu.CachePolicy:
    try:
        return fdu.CachePolicy(value)
    except ValueError:
        known = ", ".join(p.value for p in fdu.CachePolicy)
        raise UsageError(f'invalid --cache "{value}": expected one of {known}') from None


def parse_order(value: str) -> fdu.ScanOrder:
    try:
        return fdu.ScanOrder(value)
    except ValueError:
        raise UsageError(
            f"unknown --order {value}; expected breadth-first or depth-first"
        ) from None


def parse_views(value: str) -> str:
    """Hand the spec over untouched.

    This used to split and filter empty tokens here, which quietly accepted
    ``--view tree,,types`` that the CLI rejects -- the shim disagreeing with the surface
    it stands in for. The library owns the list grammar, `full` expansion, and the
    default; the only correct move is to pass the string it was given.
    """

    return value


def parse_scope(value: str, flag: str) -> str:
    """Validate a cache scope against the vocabulary both surfaces share."""

    try:
        return str(fdu.CacheScope(value))
    except ValueError:
        known = " or ".join(scope.value for scope in fdu.CacheScope)
        raise UsageError(f'invalid {flag} "{value}": expected {known}') from None


def parse_size(value: str) -> fdu.SizeMetric:
    try:
        return fdu.SizeMetric(value)
    except ValueError:
        known = ", ".join(m.value for m in fdu.SizeMetric)
        raise UsageError(f'invalid --size "{value}": expected one of {known}') from None


def parse_sort(value: str) -> fdu.SortKey:
    try:
        return fdu.SortKey(value)
    except ValueError:
        known = ", ".join(k.value for k in fdu.SortKey)
        raise UsageError(f'invalid --sort "{value}": expected one of {known}') from None


def parse_bound(value: str) -> fdu.Bound | int | str:
    """Pass the token through; the library owns the grammar and the wording.

    This used to validate here and say "expected a number or all", which is neither what
    the library says nor what the CLI says -- a third opinion invented by the shim. The
    only correct move is to hand the token over and let the one grammar reject it.
    """

    if value == "all":
        return fdu.Bound.ALL
    return value


class Args:
    """argv, decomposed along fdu's six axes."""

    def __init__(self) -> None:
        self.root: str | None = None
        self.scan_depth: int | None = None
        self.one_filesystem = False
        self.order: str | None = None
        self.threads: int | None = None
        self.type_rules: str | None = None
        self.tag_rules: list[str] = []
        self.promote: list[str] = []
        self.tags: list[str] = []
        self.not_tags: list[str] = []
        self.plane: str | None = None
        self.include: list[str] = []
        self.exclude: list[str] = []
        self.min_size: str | None = None
        self.max_size: str | None = None
        self.modified_since: str | None = None
        self.modified_before: str | None = None
        self.kinds: list[fdu.EntryKind] = []
        self.depth: fdu.Bound | int | None = None
        self.limit: fdu.Bound | int | None = None
        self.sort: fdu.SortKey | None = None
        self.reverse = False
        self.size = fdu.SizeMetric.ALLOCATED
        self.views: tuple[fdu.View, ...] | str = ()
        self.words_per_page = 250
        self.analyze = "none"
        self.analysis_workers = 0
        self.format = fdu.Format.TEXT
        self.color = "auto"
        self.cache = fdu.CachePolicy.AUTO
        self.allow_partial = False
        self.watch = False
        self.interval = 2.0
        self.cache_status: str | None = None
        self.cache_clear: str | None = None
        self.version = False


# Flags taking a value, mapped to the Args attribute and the parser that validates it.
def parse_args(argv: list[str]) -> Args:
    args = Args()
    rest = list(argv)
    positional: list[str] = []

    def value_for(flag: str) -> str:
        if not rest:
            raise UsageError(f"a value is required for '{flag}' but none was supplied")
        return rest.pop(0)

    while rest:
        token = rest.pop(0)
        if token in DECLINED:
            raise SystemExit(_decline(token))
        # `--flag=value` and `--flag value` are both accepted, as clap accepts both.
        flag, separator, inline = token.partition("=")

        # Bound as defaults rather than captured: a closure over the loop variables
        # would read whatever the last iteration left behind, which happens to work
        # only because every call site runs inside the same iteration.
        def take(name: str = flag, value: str = inline, inlined: str = separator) -> str:
            return value if inlined else value_for(name)

        if token == "--":
            positional.extend(rest)
            break
        if not token.startswith("-"):
            positional.append(token)
        elif flag in ("--version", "-V"):
            args.version = True
        elif flag == "--scan-depth":
            args.scan_depth = int(take())
        elif flag == "--one-filesystem":
            args.one_filesystem = True
        elif flag == "--order":
            args.order = take()
        elif flag == "--threads":
            args.threads = int(take())
        elif flag == "--type-rules":
            args.type_rules = take()
        elif flag == "--tag-rules":
            args.tag_rules = [name.strip() for name in take().split(",") if name.strip()]
        elif flag == "--promote":
            args.promote = [name.strip() for name in take().split(",") if name.strip()]
        elif flag == "--tag":
            args.tags.append(take())
        elif flag == "--not-tag":
            args.not_tags.append(take())
        elif flag == "--plane":
            args.plane = take()
        elif flag == "--include":
            args.include.append(take())
        elif flag == "--exclude":
            args.exclude.append(take())
        elif flag == "--min-size":
            args.min_size = take()
        elif flag == "--max-size":
            args.max_size = take()
        elif flag == "--modified-since":
            args.modified_since = take()
        elif flag == "--modified-before":
            args.modified_before = take()
        elif flag == "--kind":
            args.kinds = [fdu.EntryKind(k.strip()) for k in take().split(",") if k.strip()]
        elif flag in ("--depth", "-d"):
            args.depth = parse_bound(take())
        elif flag in ("--limit", "-n"):
            args.limit = parse_bound(take())
        elif flag == "--sort":
            args.sort = parse_sort(take())
        elif flag == "--reverse":
            args.reverse = True
        elif flag == "--size":
            args.size = parse_size(take())
        elif flag == "--view":
            args.views = parse_views(take())
        elif flag == "--words-per-page":
            args.words_per_page = int(take())
        elif flag == "--analyze":
            args.analyze = parse_analysis(take())
        elif flag == "--analysis-workers":
            args.analysis_workers = int(take())
        elif flag == "--format":
            args.format = parse_format(take())
        elif flag == "--color":
            args.color = take()
        elif flag == "--cache":
            args.cache = parse_cache(take())
        elif flag == "--allow-partial":
            args.allow_partial = True
        elif flag == "--watch":
            args.watch = True
        elif flag == "--interval":
            args.interval = _duration(take())
        elif flag == "--cache-status":
            args.cache_status = parse_scope(inline if separator else "root", flag)
        elif flag == "--cache-clear":
            args.cache_clear = parse_scope(inline if separator else "root", flag)
        else:
            raise UsageError(f"unexpected argument '{token}' found")

    if len(positional) > 1:
        raise UsageError(f"unexpected argument '{positional[1]}' found")
    args.root = positional[0] if positional else None
    return args


def _duration(value: str) -> float:
    units = {"ms": 0.001, "s": 1.0, "m": 60.0, "h": 3600.0}
    for suffix, scale in sorted(units.items(), key=lambda kv: -len(kv[0])):
        if value.endswith(suffix):
            return float(value[: -len(suffix)]) * scale
    return float(value)


def _decline(flag: str) -> int:
    # A declined surface says so on stderr and in one line. It must never look like
    # success, and it must never look like fdu's own output either.
    print(
        f"{PROGRAM}: {flag} is a discovery surface the Python package does not carry",
        file=sys.stderr,
    )
    return 2


def build_query(args: Args) -> fdu.Query:
    selection = fdu.Selection(
        include=tuple(args.include),
        exclude=tuple(args.exclude),
        min_size=args.min_size,
        max_size=args.max_size,
        modified_since=args.modified_since,
        modified_before=args.modified_before,
        kinds=tuple(args.kinds),
        tags=tuple(args.tags),
        not_tags=tuple(args.not_tags),
        plane=args.plane,
        depth=args.depth,
        limit=args.limit,
        sort=args.sort,
        reverse=args.reverse,
        size=args.size,
    )
    # An empty view tuple means "let the requested analyzers choose", which is the
    # library's own default derivation rather than a default spelled out here.
    return fdu.Query(
        views=args.views,
        selection=selection,
        words_per_page=args.words_per_page,
    )


def run_cache_lifecycle(args: Args) -> int:
    root = Path(args.root) if args.root else Path(".")

    if args.cache_clear is not None:
        if args.cache_clear == "all":
            directory = _cache_directory(root)
            if directory is None:
                print("Cache already empty.")
            else:
                # Echoed before acting, so a destructive flag always says where it points.
                print(f"Cache directory: {directory}")
                removed = fdu.clear_all_caches(directory)
                if removed == 0:
                    print("Cache already empty.")
                else:
                    noun = "snapshot" if removed == 1 else "snapshots"
                    print(f"Cache cleared: {removed} {noun}.")
        else:
            path = fdu.cache_path(root)
            removed = fdu.clear_cache(root)
            if path is not None:
                print(f"Cache file: {path}")
            print("Cache cleared." if removed else "Cache already empty.")

    if args.cache_status is not None:
        statuses = _statuses(root, args.cache_status)
        # The one renderer, in every format. A shim formatting these itself would be
        # testing its own layout rather than the API's.
        print(fdu.render_cache_status(statuses, args.format))

    return 0


def _cache_directory(root: Path) -> Path | None:
    path = fdu.cache_path(root)
    return path.parent if path is not None else None


def _statuses(root: Path, scope: str) -> tuple[fdu.CacheStatus, ...]:
    if scope == "all":
        directory = _cache_directory(root)
        return fdu.list_caches(directory) if directory is not None else ()
    status = fdu.cache_status(root)
    return (status,) if status is not None else ()


def run_watch(args: Args) -> int:
    index = _open(args)
    query = build_query(args)

    options = fdu.WatchOptions(interval=args.interval, query=query)
    # The session opens first, so a scope it cannot watch is refused before anything is
    # written. Printing the initial answer first meant a rejected request still emitted a
    # report, which is the opposite of what refusing means.
    with index.watch(options) as watch:
        # Then the initial answer, identical to a run without --watch. A stream that opens
        # with its changes tells a reader nothing about what it is watching.
        sys.stdout.write(render(args, index.report(query)))
        sys.stdout.flush()
        # Views that stream per entry are emitted as records; anything aggregate has to be
        # repainted, because a total cannot be expressed as a change. Both come from the
        # one query, so nothing here is a second grammar.
        streams_changes = fdu.View.FILES in query.views
        has_aggregates = any(view != fdu.View.FILES for view in query.views)
        dirty = False

        for batch in watch:
            if not batch.dirty:
                # The idle tick: repaint only if something moved since the last one.
                if has_aggregates and dirty:
                    _repaint(args, watch)
                    dirty = False
                continue
            for change in batch.changes:
                if streams_changes:
                    # The one renderer, so the stream is fdu's bytes and not the shim's
                    # idea of them. This printed repr() until Change.render (fdu-m66a).
                    print(change.render(args.format), flush=True)
            # `batch.dirty`, not "the change list was non-empty". A mutation the selection
            # filters out still moves the aggregates, and deriving this from the changes
            # meant an aggregate view could go stale with no tick ever saying so.
            dirty = True
    return 0


def _repaint(args: Args, watch: fdu.Watch) -> None:
    """Redraw the aggregate views, separated from the repaint before them.

    From the session's report, which is the opened index: a watch shares the handle it was
    opened from, so there is no second index to prefer.
    """

    if args.format is fdu.Format.TEXT:
        print(f"\n{fdu.watch_rule(datetime.now(tz=UTC))}", flush=True)
    sys.stdout.write(render(args, watch.report()))
    sys.stdout.flush()


def _open(args: Args) -> fdu.Index:
    scan = _scan_options(args)
    analysis = fdu.AnalysisOptions(analyze=args.analyze, workers=args.analysis_workers)
    return fdu.open(args.root or ".", cache=args.cache, scan=scan, analysis=analysis)


def render(args: Args, report: fdu.Report) -> str:
    # The one renderer, reached through the API rather than reimplemented. A shim that
    # drew its own bars and padding would be testing the reimplementation.
    color = args.color == "always"
    return report.render(args.format, color=color)


def exit_code(args: Args, status: fdu.Status) -> int:
    if status.complete or args.allow_partial:
        return 0
    return 1


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    try:
        args = parse_args(argv)
    except UsageError as error:
        fail(str(error))

    if args.version:
        # The one place the shim announces itself. This keeps the committed deviation
        # file non-empty by construction: an empty parity diff means the shim never ran,
        # which is the failure a naive harness reports as success.
        print(f"{PROGRAM} {fdu.__version__} (python parity surface)")
        return 0

    if args.cache_status is not None or args.cache_clear is not None:
        return run_cache_lifecycle(args)

    if args.root is None:
        # The binary prints help here. The shim has no help to print, and must not
        # silently succeed, so it declines the same way it declines --help.
        return _decline("a bare invocation")

    if args.watch:
        return run_watch(args)

    # fdu.report, not fdu.open().report(): the command line runs one-shot, retaining the
    # least state the request needs, and a session would retain an index and write a
    # snapshot the command would not have left behind (fdu-4msv).
    report = fdu.report(
        args.root or ".",
        build_query(args),
        cache=args.cache,
        scan=_scan_options(args),
        analysis=fdu.AnalysisOptions(analyze=args.analyze, workers=args.analysis_workers),
    )
    sys.stdout.write(render(args, report))
    return exit_code(args, report.status)


def run() -> int:
    try:
        return main()
    except UsageError as error:
        fail(str(error))
    except fdu.InvalidArgumentError as error:
        fail(str(error))
    except fdu.FilesystemError as error:
        fail(str(error), code=1)
    except fdu.FduError as error:
        fail(str(error), code=1)
    except BrokenPipeError:
        return 0


if __name__ == "__main__":
    raise SystemExit(run())
