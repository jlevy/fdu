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


def parse_views(value: str) -> tuple[fdu.View, ...]:
    views: list[fdu.View] = []
    for part in (p.strip() for p in value.split(",") if p.strip()):
        try:
            # `full` is a View member, not a shim-side expansion: the library owns view
            # resolution and default derivation, and reimplementing either here is the
            # drift this harness exists to catch.
            views.append(fdu.View(part))
        except ValueError:
            known = ", ".join(v.value for v in fdu.View)
            raise UsageError(f'invalid --view "{part}": expected one of {known}') from None
    return tuple(views)


def parse_scope(value: str, flag: str) -> str:
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
        self.include: list[str] = []
        self.exclude: list[str] = []
        self.min_size: str | None = None
        self.modified_since: str | None = None
        self.modified_before: str | None = None
        self.kinds: list[fdu.EntryKind] = []
        self.depth: fdu.Bound | int | None = None
        self.limit: fdu.Bound | int | None = None
        self.sort: fdu.SortKey | None = None
        self.reverse = False
        self.size = fdu.SizeMetric.ALLOCATED
        self.views: tuple[fdu.View, ...] = ()
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
        elif flag == "--include":
            args.include.append(take())
        elif flag == "--exclude":
            args.exclude.append(take())
        elif flag == "--min-size":
            args.min_size = take()
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
        modified_since=args.modified_since,
        modified_before=args.modified_before,
        kinds=tuple(args.kinds),
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
    options = fdu.WatchOptions(interval=args.interval, query=build_query(args))
    with index.watch(options) as watch:
        for change in watch:
            print(change)
    return 0


def _open(args: Args) -> fdu.Index:
    scan = fdu.ScanOptions(max_depth=args.scan_depth, one_filesystem=args.one_filesystem)
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

    index = _open(args)
    report = index.report(build_query(args))
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
