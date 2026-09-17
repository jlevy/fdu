"""Run one fdu Python-surface request and print a JSON envelope.

Usage: pyrun.py '<json job>'
job = {"root": str, "mode": "report"|"open"|"scan", "cache": "auto"|..., "spec": {...}}
XDG_CACHE_HOME must already be set in the environment by the caller.
"""

import json
import sys

import fdu


def build(spec):
    sc = spec.get("scope", {})
    se = spec.get("sel", {})
    scan = fdu.ScanOptions(
        max_depth=sc.get("scan_depth"),
        one_filesystem=bool(sc.get("one_fs", False)),
        read_controls=not sc.get("no_gitignore", False),
        control_budget=sc.get("budget"),
        control_line_limit=sc.get("line_limit"),
    )
    kw = {}
    if "include" in se:
        kw["include"] = tuple(se["include"])
    if "exclude" in se:
        kw["exclude"] = tuple(se["exclude"])
    if "min_size" in se:
        kw["min_size"] = se["min_size"]
    if "modified_since" in se:
        kw["modified_since"] = se["modified_since"]
    if "kind" in se:
        kw["kinds"] = tuple(fdu.EntryKind(k) for k in se["kind"])
    if "ignored" in se:
        kw["ignored"] = fdu.IgnoredEntries(se["ignored"])
    if "depth" in se:
        kw["depth"] = se["depth"]
    if "limit" in se:
        kw["limit"] = se["limit"]
    if "sort" in se:
        kw["sort"] = fdu.SortKey(se["sort"])
    if "reverse" in se:
        kw["reverse"] = se["reverse"]
    if "size" in se:
        kw["size"] = fdu.SizeMetric(se["size"])
    sel = fdu.Selection(**kw)
    views = tuple(fdu.View(v) for v in spec.get("views", []))
    query = fdu.Query(views=views, selection=sel)
    analysis = fdu.AnalysisOptions(analyze=spec.get("analyze", "none"))
    return scan, query, analysis


def main():
    job = json.loads(sys.argv[1])
    try:
        scan, query, analysis = build(job["spec"])
        cache = fdu.CachePolicy(job.get("cache", "auto"))
        mode = job["mode"]
        if mode == "report":
            rep = fdu.report(job["root"], query, cache=cache, scan=scan, analysis=analysis)
        elif mode == "open":
            idx = fdu.open(job["root"], cache=cache, scan=scan, analysis=analysis)
            rep = idx.report(query)
        elif mode == "scan":
            idx = fdu.scan(job["root"], scan=scan, analysis=analysis)
            rep = idx.report(query)
        else:
            raise SystemExit("bad mode")
        out = {"ok": True, "dict": rep.as_dict(), "notes": list(rep.notes)}
    except Exception as e:  # noqa: BLE001
        out = {"ok": False, "error": f"{type(e).__name__}: {e}"}
    print(json.dumps(out, default=str))


main()
