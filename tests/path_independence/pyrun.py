"""Ask one request through the installed fdu Python package and print a JSON envelope.

Usage: pyrun.py '<job>', where job is
    {"root": str, "mode": "report" | "open" | "scan", "cache": str, "spec": {...}}
and the caller has set XDG_CACHE_HOME.

The envelope is {"ok": true, "answer": {...}}; {"ok": false, "kind": ..., "error": "..."}
when the request raised, where kind is "fdu" for fdu.FduError, "refused" for ValueError,
and "unexpected" for anything else; or {"refused": "..."} when this interpreter would
test the wrong fdu.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

SOURCE_PACKAGE = Path(__file__).resolve().parents[2] / "crates" / "fdu-py" / "python"


def _build(spec: dict[str, Any]) -> tuple[Any, Any, Any]:
    import fdu

    scope, selection = spec.get("scope", {}), spec.get("sel", {})
    scan = fdu.ScanOptions(
        max_depth=scope.get("scan_depth"),
        one_filesystem=bool(scope.get("one_fs", False)),
        read_controls=not scope.get("no_gitignore", False),
        control_budget=scope.get("budget"),
        control_line_limit=scope.get("line_limit"),
    )
    fields: dict[str, Any] = {}
    for name in ("include", "exclude"):
        if name in selection:
            fields[name] = tuple(selection[name])
    for name in ("min_size", "modified_since", "depth", "limit", "reverse"):
        if name in selection:
            fields[name] = selection[name]
    if "kind" in selection:
        fields["kinds"] = tuple(fdu.EntryKind(kind) for kind in selection["kind"])
    if "ignored" in selection:
        fields["ignored"] = fdu.IgnoredEntries(selection["ignored"])
    if "sort" in selection:
        fields["sort"] = fdu.SortKey(selection["sort"])
    if "size" in selection:
        fields["size"] = fdu.SizeMetric(selection["size"])
    views = tuple(fdu.View(view) for view in spec.get("views", []))
    query = fdu.Query(views=views, selection=fdu.Selection(**fields), format=fdu.Format.JSON)
    analysis = fdu.AnalysisOptions(analyze=spec.get("analyze", "none"))
    return scan, query, analysis


def main() -> None:
    import fdu

    # A parity safety property: the source tree's package would pass while the built
    # wheel is broken, so only an installed fdu is ever measured.
    if Path(fdu.__file__).resolve().is_relative_to(SOURCE_PACKAGE):
        print(json.dumps({"refused": f"fdu imports from the source tree: {fdu.__file__}"}))
        return
    job = json.loads(sys.argv[1])
    try:
        scan, query, analysis = _build(job["spec"])
        cache = fdu.CachePolicy(job["cache"])
        mode = job["mode"]
        if mode == "report":
            report = fdu.report(job["root"], query, cache=cache, scan=scan, analysis=analysis)
        elif mode == "open":
            report = fdu.open(job["root"], cache=cache, scan=scan, analysis=analysis).report(query)
        elif mode == "scan":
            report = fdu.scan(job["root"], scan=scan, analysis=analysis).report(query)
        else:
            raise RuntimeError(f"unknown mode {mode!r}")
        envelope: dict[str, Any] = {"ok": True, "answer": report.as_dict()}
    except fdu.InvalidArgumentError as error:
        # How the Python API refuses a request it cannot answer, in its own names. It is
        # both an `FduError` and a `ValueError`, so which arm catches it is the question
        # this order answers: asking `FduError` first reported every refused request as an
        # operation that failed, and the command line's refusal then looked like a
        # different outcome than the same refusal one door over.
        envelope = {"ok": False, "kind": "refused", "error": f"ValueError: {error}"}
    except fdu.FduError as error:
        envelope = {"ok": False, "kind": "fdu", "error": f"FduError: {error}"}
    except ValueError as error:
        envelope = {"ok": False, "kind": "refused", "error": f"ValueError: {error}"}
    except Exception as error:
        envelope = {"ok": False, "kind": "unexpected", "error": f"{type(error).__name__}: {error}"}
    print(json.dumps(envelope, default=str))


if __name__ == "__main__":
    main()
