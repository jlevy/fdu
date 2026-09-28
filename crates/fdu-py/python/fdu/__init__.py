"""Typed Python interface to fdu's retained file roll-up engine."""

# The `fdu` command the wheel installs enters through `_main` below and hands its
# arguments to the native command line, which needs none of the typed API. Importing that
# API here made every such command pay for it: defining the frozen models, with the
# `dataclasses`, `json`, and `pathlib` imports behind them, was about 55 ms of an 88 ms
# `fdu --version` (fdu-03yn). So each public name is imported on first access (PEP 562),
# and type checkers read the same names from the imports under `TYPE_CHECKING`. That is
# a local constant, which type checkers treat as `typing.TYPE_CHECKING`, because
# importing `typing` for it would cost the command several milliseconds more.

from . import _native
from ._native import __version__

TYPE_CHECKING = False

if TYPE_CHECKING:
    from ._api import (
        FduError,
        FilesystemError,
        Index,
        InvalidArgumentError,
        Watch,
        cache_directory,
        cache_path,
        cache_status,
        clear_all_caches,
        clear_cache,
        list_caches,
        open,
        render_cache_status,
        report,
        scan,
        watch_rule,
    )
    from ._models import (
        Analysis,
        AnalysisMetadata,
        AnalysisOptions,
        Analyzer,
        Bound,
        CachePolicy,
        CacheScope,
        CacheState,
        CacheStatus,
        Change,
        ChangeKind,
        ChangeSet,
        Child,
        ClearSummary,
        CodeLanguageRow,
        CodeOverview,
        CodeSection,
        CodeTally,
        ContentState,
        ContentStatus,
        ContentTierIdentity,
        ControlLimits,
        ControlObservation,
        ControlRefusalReason,
        ControlTierIdentity,
        Coverage,
        Detection,
        EntryKind,
        EntryTierIdentity,
        ExtensionRow,
        ExtensionsSection,
        ExtensionTally,
        FileClassification,
        FileRow,
        FilesSection,
        Format,
        Freshness,
        IgnoredEntries,
        IgnoredTally,
        LeftoverKind,
        MetricRow,
        MetricShare,
        MetricsSection,
        MetricValues,
        OperationError,
        Pages,
        Provenance,
        Query,
        RefreshResult,
        RefusedControl,
        Report,
        ReportProvenance,
        ReportRequest,
        ReportScope,
        ReportSection,
        ReportSource,
        RollUp,
        ScanOptions,
        SectionBound,
        Selection,
        SizeMetric,
        SnapshotIdentity,
        SortKey,
        StaleReason,
        Status,
        SummaryRow,
        SummarySection,
        TierProvenance,
        TierState,
        TreeDisplayLimits,
        TreeNode,
        TreeOmission,
        TreeOmissionReason,
        TreeRemainder,
        TreeSection,
        ValueSource,
        View,
        WatchOptions,
    )
else:
    # Type checkers never see this branch. One that saw a module `__getattr__` would
    # accept any attribute of `fdu`, misspellings included.

    # The names in `__all__` that `_api` defines. Every other one but `__version__` is a
    # `_models` value type; tests/test_startup.py holds this set to the imports above.
    _API_NAMES = frozenset(
        {
            "FduError",
            "FilesystemError",
            "Index",
            "InvalidArgumentError",
            "Watch",
            "cache_directory",
            "cache_path",
            "cache_status",
            "clear_all_caches",
            "clear_cache",
            "list_caches",
            "open",
            "render_cache_status",
            "report",
            "scan",
            "watch_rule",
        }
    )

    def __getattr__(name: str) -> object:
        """Import a public name on its first access (PEP 562)."""

        if name in _API_NAMES:
            from . import _api as source
        elif name in __all__:
            from . import _models as source
        else:
            raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
        value = getattr(source, name)
        globals()[name] = value  # later reads are ordinary module lookups
        return value

    def __dir__() -> list[str]:
        """List the public names too, before their first access imports them."""

        return sorted({*globals(), *__all__})


def _main() -> int:
    """Console-script boundary; argument parsing remains in the native CLI."""

    # Imported here so that `import fdu` loads the native module and nothing else.
    import signal

    # Python's SIGINT handler only sets a flag the native CLI never checks, so a
    # `--watch` console script would ignore Ctrl-C until the native call returned.
    # Restore the default disposition so the process dies on interrupt the way the
    # cargo-installed binary does (fdu-18vk).
    signal.signal(signal.SIGINT, signal.SIG_DFL)
    return _native.main()


__all__ = [
    "Analysis",
    "AnalysisMetadata",
    "AnalysisOptions",
    "Analyzer",
    "Bound",
    "CachePolicy",
    "CacheScope",
    "CacheState",
    "CacheStatus",
    "Change",
    "ChangeKind",
    "ChangeSet",
    "Child",
    "ClearSummary",
    "CodeLanguageRow",
    "CodeOverview",
    "CodeSection",
    "CodeTally",
    "ContentState",
    "ContentStatus",
    "ContentTierIdentity",
    "ControlLimits",
    "ControlObservation",
    "ControlRefusalReason",
    "ControlTierIdentity",
    "Coverage",
    "Detection",
    "EntryKind",
    "EntryTierIdentity",
    "ExtensionRow",
    "ExtensionTally",
    "ExtensionsSection",
    "FduError",
    "FileClassification",
    "FileRow",
    "FilesSection",
    "FilesystemError",
    "Format",
    "Freshness",
    "IgnoredEntries",
    "IgnoredTally",
    "Index",
    "InvalidArgumentError",
    "LeftoverKind",
    "MetricRow",
    "MetricShare",
    "MetricValues",
    "MetricsSection",
    "OperationError",
    "Pages",
    "Provenance",
    "Query",
    "RefreshResult",
    "RefusedControl",
    "Report",
    "ReportProvenance",
    "ReportRequest",
    "ReportScope",
    "ReportSection",
    "ReportSource",
    "RollUp",
    "ScanOptions",
    "SectionBound",
    "Selection",
    "SizeMetric",
    "SnapshotIdentity",
    "SortKey",
    "StaleReason",
    "Status",
    "SummaryRow",
    "SummarySection",
    "TierProvenance",
    "TierState",
    "TreeDisplayLimits",
    "TreeNode",
    "TreeOmission",
    "TreeOmissionReason",
    "TreeRemainder",
    "TreeSection",
    "ValueSource",
    "View",
    "Watch",
    "WatchOptions",
    "__version__",
    "cache_directory",
    "cache_path",
    "cache_status",
    "clear_all_caches",
    "clear_cache",
    "list_caches",
    "open",
    "render_cache_status",
    "report",
    "scan",
    "watch_rule",
]

del TYPE_CHECKING
