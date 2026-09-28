# Ported: imports real_tree from the repository's explorations/fsevents-replay instead of an absolute checkout path.
"""Read-only audit of a preserved real_tree.py state directory.

Prints only redacted path labels (depth, suffix, short hash) so output can be quoted.
Usage: analyze_real_root.py STATE_DIR LABEL [LABEL...] [--lsof] [--stat]
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path, PurePosixPath

FLAG_NAMES = {
    0x1: "MustScanSubDirs", 0x2: "UserDropped", 0x4: "KernelDropped", 0x8: "EventIdsWrapped",
    0x10: "HistoryDone", 0x20: "RootChanged", 0x40: "Mount", 0x80: "Unmount",
    0x100: "ItemCreated", 0x200: "ItemRemoved", 0x400: "ItemInodeMetaMod", 0x800: "ItemRenamed",
    0x1000: "ItemModified", 0x2000: "ItemFinderInfoMod", 0x4000: "ItemChangeOwner",
    0x8000: "ItemXattrMod", 0x10000: "ItemIsFile", 0x20000: "ItemIsDir", 0x40000: "ItemIsSymlink",
    0x80000: "OwnEvent", 0x100000: "ItemIsHardlink", 0x200000: "ItemIsLastHardlink",
    0x400000: "ItemCloned",
}


def flag_names(flags: int) -> str:
    names = [name for bit, name in FLAG_NAMES.items() if flags & bit]
    return "|".join(names) or "0"


def redact(path: str) -> str:
    parts = PurePosixPath(path).parts
    suffix = PurePosixPath(path).suffix or "(none)"
    digest = hashlib.sha256(path.encode("utf-8", "surrogateescape")).hexdigest()[:8]
    return f"d{len(parts)}:{suffix}:{digest}"


def load_inventory(path: Path) -> dict[str, tuple[int, ...]]:
    data = json.loads(path.read_text())
    return {k: tuple(v) for k, v in data["inventory"].items()}


def contains(parent: str, path: str) -> bool:
    return parent == "." or path == parent or path.startswith(parent + "/")


def parent_of(path: str) -> str:
    return str(PurePosixPath(path).parent)


def facts_row(row: tuple[int, ...] | None) -> dict | None:
    if row is None:
        return None
    mode, size, blocks, mtime, ctime, dev, ino, nlink = row
    return {
        "mode": oct(mode), "size": size, "blocks": blocks, "mtime": mtime / 1e9,
        "ctime": ctime / 1e9, "ino": ino, "nlink": nlink,
    }


def main() -> None:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    do_lsof = "--lsof" in sys.argv
    do_stat = "--stat" in sys.argv
    state = Path(args[0])
    labels = args[1:]
    base = json.loads((state / "baseline.json").read_text())
    fence = int(base["captured"]["device_fence"])
    global_now = int(base["captured"]["global_now"])
    root = Path(base["root"])
    baseline = {k: tuple(v) for k, v in base["inventory"].items()}
    print(json.dumps({
        "alias": base["alias"], "entries": len(baseline), "device_fence": fence,
        "global_now_at_capture": global_now, "fence_minus_now": fence - global_now,
        "capture_started_at": base["started_at"], "capture_completed_at": base["completed_at"],
        "capture_started_iso": time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(base["started_at"])),
    }, indent=1))
    for label in labels:
        trial = state / label
        details = json.loads((trial / "details-private.json").read_text())
        summary = json.loads((trial / "summary-shareable.json").read_text())
        records = details["native_records"]
        plan = details["scope_plan"]
        comparison = details["comparison"]
        cstats = details["candidate_stats"]
        events = [r for r in records if r.get("type") == "event"]
        native_summary = [r for r in records if r.get("type") == "summary"][-1]
        decoded = []
        for e in events:
            p = os.fsdecode(bytes.fromhex(e["path_hex"])) if e.get("inside") else ""
            p = str(PurePosixPath(p)) if p else "."
            decoded.append((p, e))
        ids = [e["id"] for e in events if not e["flags"] & 0x10]
        hd = [e for e in events if e["flags"] & 0x10]
        print("\n==== trial", label)
        print(json.dumps({
            "refresh_started_at": summary["refresh_started_at"],
            "refresh_started_iso": time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(summary["refresh_started_at"])),
            "gap_s": summary["gap_since_baseline_completion_seconds"],
            "native": {k: native_summary[k] for k in ("cursor", "history_done", "timeout", "truncated", "latest_id", "history_id", "history_ms", "flush_ms", "callbacks", "events", "first_callback_ms", "last_callback_ms")},
            "event_count": len(events), "min_id": min(ids) if ids else None, "max_id": max(ids) if ids else None,
            "min_id_minus_fence": (min(ids) - fence) if ids else None,
            "history_done_records": [(e["id"], flag_names(e["flags"]), e.get("inside"), e.get("ancestor")) for e in hd],
            "after_history_done_events": sum(1 for e in events if e.get("after_history_done") and not e["flags"] & 0x10),
            "overlap_events": sum(1 for e in events if e["id"] <= fence and not e["flags"] & 0x10),
            "outside_events": sum(1 for e in events if not e.get("inside") and not e.get("ancestor") and not e["flags"] & 0x10),
            "ancestor_events": sum(1 for e in events if e.get("ancestor")),
            "plan_relist": [redact(p) if p != "." else "." for p in plan["relist"]],
            "plan_relist_depths": sorted(len(PurePosixPath(p).parts) for p in plan["relist"]),
            "plan_recursive": plan["recursive"], "fallback": plan["fallback_reasons"],
            "effective_scopes_n": len(cstats["effective_scopes"]),
            "relisted_n": len(cstats["relisted_scopes"]), "recursive_n": len(cstats["recursive_scopes"]),
            "errors": cstats["errors"],
        }, indent=1))
        # Flag histogram
        hist: dict[str, int] = {}
        for p, e in decoded:
            if e["flags"] & 0x10:
                continue
            hist[flag_names(e["flags"])] = hist.get(flag_names(e["flags"]), 0) + 1
        print("flag histogram:")
        for k, v in sorted(hist.items(), key=lambda kv: -kv[1]):
            print(f"  {v:5d}  {k}")
        # Distinct event paths and their parents
        distinct_paths = sorted({p for p, e in decoded if not e["flags"] & 0x10})
        distinct_parents = sorted({parent_of(p) if not (e["flags"] & 0x20000) else p for p, e in decoded if not e["flags"] & 0x10})
        print(f"distinct event paths: {len(distinct_paths)}; distinct nominated dirs (file->parent, dir->self): {len(distinct_parents)}")
        # Re-run normalization from the repo copy to confirm plan reproduces
        sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "fsevents-replay"))
        import real_tree  # noqa: E402
        replan = real_tree.normalize_events(records, summary["create_flags"])
        print("normalize_events reproduces saved plan:", list(replan.relist) == list(plan["relist"]) and list(replan.recursive) == list(plan["recursive"]) and list(replan.fallback_reasons) == list(plan["fallback_reasons"]))
        before = load_inventory(trial / "oracle-before.json")
        after = load_inventory(trial / "oracle-after.json")
        candidate = load_inventory(trial / "candidate.json")

        def nominated(path: str) -> dict:
            par = parent_of(path)
            direct = par in plan["relist"] or path in plan["relist"]
            rec = any(contains(r, path) for r in plan["recursive"])
            relisted_effective = par in cstats["relisted_scopes"] or path in cstats["relisted_scopes"]
            rec_effective = any(contains(r, path) for r in cstats["recursive_scopes"])
            own_events = [(e["id"] - fence, flag_names(e["flags"]), e.get("after_history_done")) for p, e in decoded if p == path and not e["flags"] & 0x10]
            parent_events = [(e["id"] - fence, flag_names(e["flags"])) for p, e in decoded if p == par and not e["flags"] & 0x10]
            ancestor_events = [(redact(p) if p != "." else ".", e["id"] - fence, flag_names(e["flags"])) for p, e in decoded if p != path and p != par and contains(p, path) and not e["flags"] & 0x10]
            descendant_events = sum(1 for p, e in decoded if p != path and contains(path, p) and not e["flags"] & 0x10)
            return {
                "parent_in_plan_relist": direct, "in_plan_recursive": rec,
                "parent_relisted_effective": relisted_effective, "recursive_effective": rec_effective,
                "own_events": own_events, "parent_dir_events": parent_events,
                "ancestor_events": ancestor_events, "descendant_events": descendant_events,
            }

        def describe(path: str, with_lsof: bool) -> dict:
            b, bf, af, c = baseline.get(path), before.get(path), after.get(path), candidate.get(path)
            out = {
                "path": redact(path), "depth": len(PurePosixPath(path).parts),
                "baseline": facts_row(b), "before": facts_row(bf), "after": facts_row(af), "candidate": facts_row(c),
                "candidate_equals": ("baseline" if c == b else "before" if c == bf else "after" if c == af else "none"),
                "nomination": nominated(path),
            }
            if do_stat:
                try:
                    st = os.lstat(root / path)
                    out["now"] = {
                        "size": st.st_size, "mtime": st.st_mtime, "ctime": st.st_ctime,
                        "birthtime": getattr(st, "st_birthtime", None), "ino": st.st_ino, "nlink": st.st_nlink,
                        "mode": oct(st.st_mode),
                        "mtime_minus_fence_time": st.st_mtime - base["started_at"],
                        "birth_minus_fence_time": (getattr(st, "st_birthtime", 0) - base["started_at"]),
                    }
                except OSError as err:
                    out["now"] = {"error": type(err).__name__}
            if with_lsof and do_lsof:
                try:
                    res = subprocess.run(["lsof", "-F", "pcfa", "--", str(root / path)], capture_output=True, text=True, timeout=60)
                    procs = []
                    cur = {}
                    for line in res.stdout.splitlines():
                        tag, val = line[0], line[1:]
                        if tag == "p":
                            cur = {"pid": val}
                            procs.append(cur)
                        elif tag == "c":
                            cur["cmd"] = val
                        elif tag == "f":
                            cur.setdefault("fds", []).append({"fd": val})
                        elif tag == "a":
                            cur["fds"][-1]["access"] = val
                    out["lsof"] = {"returncode": res.returncode, "procs": procs, "stderr": res.stderr.strip()[:200]}
                except Exception as err:  # noqa: BLE001
                    out["lsof"] = {"error": type(err).__name__}
            return out

        print("\n-- stable mismatches:", len(comparison["stable_mismatches"]))
        for p in comparison["stable_mismatches"]:
            print(json.dumps(describe(p, True), indent=1, default=str))
        print("\n-- concurrent paths:", len(comparison["concurrent_paths"]))
        cats: dict[str, int] = {}
        rows = []
        gap_rows = []
        for p in comparison["concurrent_paths"]:
            n = nominated(p)
            nom = n["parent_in_plan_relist"] or n["in_plan_recursive"] or n["parent_relisted_effective"] or n["recursive_effective"]
            own = bool(n["own_events"])
            changed_before_replay = baseline.get(p) != before.get(p)
            kind_row = after.get(p) or before.get(p)
            kind = "dir" if kind_row and (kind_row[0] & 0o170000) == 0o040000 else "file"
            exists_before = before.get(p) is not None
            exists_after = after.get(p) is not None
            life = "modified" if exists_before and exists_after else "created" if exists_after else "deleted"
            if not nom and changed_before_replay:
                mech = "HIDDEN-GAP(changed before replay, never nominated)"
            elif not nom:
                mech = "pure-concurrency(changed after replay, not nominated)"
            elif nom and not own and changed_before_replay:
                mech = "MASKED-GAP(changed before replay, no own event, parent relisted for a sibling)"
            elif nom and own:
                mech = "nominated-by-own-event"
            else:
                mech = "nominated-via-sibling, changed after replay"
            key = f"{kind}/{life}/{mech}"
            cats[key] = cats.get(key, 0) + 1
            d = describe(p, kind == "file" and ("GAP" in mech))
            rows.append((mech, kind, life, d["path"], d["candidate_equals"], n["own_events"][:2]))
            if "GAP" in mech:
                gap_rows.append((mech, d["path"], {k: (v["size"], round(v["mtime"] - base["started_at"], 1)) for k, v in (("baseline", d["baseline"]), ("before", d["before"]), ("after", d["after"]), ("candidate", d["candidate"])) if v}, d.get("lsof", {}).get("procs"), d.get("now", {}).get("mtime_minus_fence_time")))
        for k, v in sorted(cats.items()):
            print(f"  {v:3d}  {k}")
        print("  rows (mechanism, kind, life, redacted, candidate_equals, own_events[:2]):")
        for r in rows:
            print("   ", r)
        print("  gap rows (mechanism, redacted, {stage:(size, mtime-fence_s)}, lsof procs, now mtime-fence_s):")
        for r in gap_rows:
            print("   ", json.dumps(r, default=str))
        # Unresolved
        print("\n-- unresolved concurrent mismatches:", len(comparison["unresolved_concurrent_mismatches"]))
        # Sibling masking: among changed_paths in candidate vs baseline, how many had no own event?
        changed = [p for p in baseline.keys() | candidate.keys() if baseline.get(p) != candidate.get(p)]
        own = {p for p, e in decoded if not e["flags"] & 0x10}
        masked = [p for p in changed if p not in own and p != "." and parent_of(p) not in own]
        print(f"\n-- candidate changed entries: {len(changed)}; with no own event and no event on the path itself as dir: {len(masked)} (refreshed via sibling relist or ancestor refresh)")
        kinds = {}
        for p in masked:
            row = candidate.get(p) or baseline.get(p)
            k = "dir" if row and (row[0] & 0o170000) == 0o040000 else "file"
            kinds[k] = kinds.get(k, 0) + 1
        print("   masked kinds:", kinds)
        masked_files = [p for p in masked if (candidate.get(p) or baseline.get(p)) and ((candidate.get(p) or baseline.get(p))[0] & 0o170000) == 0o100000]
        print("   masked regular files (sibling-relist refreshed without own event); redacted, depth, baseline->candidate size, in_before_already, lsof:")
        for p in masked_files:
            d = describe(p, True)
            already = baseline.get(p) != before.get(p)
            print("    ", d["path"], d["depth"], (d["baseline"] or {}).get("size"), "->", (d["candidate"] or {}).get("size"), "changed_before_replay=", already, "lsof=", json.dumps(d.get("lsof", {}).get("procs")))


if __name__ == "__main__":
    main()
