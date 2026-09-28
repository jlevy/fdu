#!/usr/bin/env python3
"""Condense the verification data files into one summary JSON plus a printed digest.

  summarize.py DATA_DIR OUT.json

Reads (when present): semantics-ext.json, semantics-internal.json, dmg-tests.json,
flag-unset.json, accounting.json, hardlink-primary.json, ow-lag.json, scale/mark-*.json,
scale/wa-*.json, scale/refresh-*.json, smoke/*.json.
"""
import json
import os
import sys


def load(d, name):
    p = os.path.join(d, name)
    return json.load(open(p)) if os.path.exists(p) else None


def semantics_table(S):
    reps = S["replicates"]
    by = [{e["step"]: e for e in r["log"]} for r in reps]
    names = [e["step"] for e in reps[0]["log"]]
    rows = []
    for n in names:
        entries = [b[n] for b in by if n in b]
        deltas = [e["delta"] for e in entries]
        d = deltas[-1]
        rf = d["root.fsctl"]
        rf = tuple(rf) if isinstance(rf, list) else None
        first = [e["first_read"]["root"] for e in entries]
        consistent = len({json.dumps(x, sort_keys=True, default=str) for x in deltas}) == 1
        rows.append({"step": n, "note": entries[0]["note"], "root_dgen": d.get("root.gen"), "root_dphys": rf[2] if rf else None, "root_ddesc": rf[1] if rf else None,
                     "A_dgen": d.get("A.gen"), "B_dgen": d.get("B.gen"), "C_dgen": d.get("C.gen"),
                     "visible_in_first_read": all(f["changed"] for f in first) if any(f["changed"] for f in first) else False,
                     "first_read_us_after_return": max(f["since_op_return_us"] for f in first), "replicates": len(entries), "consistent": consistent,
                     "info": entries[0]["info"]})
    return rows


def main():
    d, out = sys.argv[1], sys.argv[2]
    S = {}
    ext = load(d, "semantics-ext.json")
    if ext:
        S["semantics_external"] = {"uid": ext["uid"], "replicates": len(ext["replicates"]), "table": semantics_table(ext)}
    it = load(d, "semantics-internal.json")
    if it:
        S["semantics_internal_volume"] = {"uid": it["uid"], "replicates": len(it["replicates"]), "table": semantics_table(it)}
        if ext:
            ea = {r["step"]: (r["root_dgen"], r["A_dgen"], r["B_dgen"], r["C_dgen"]) for r in S["semantics_external"]["table"]}
            ia = {r["step"]: (r["root_dgen"], r["A_dgen"], r["B_dgen"], r["C_dgen"]) for r in S["semantics_internal_volume"]["table"]}
            S["semantics_internal_volume"]["gencount_deltas_differ_from_external"] = [k for k in ia if k in ea and ia[k] != ea[k]]
    for name in ("dmg-tests.json", "flag-unset.json", "accounting.json", "hardlink-primary.json", "ow-lag.json", "research-notes.json"):
        v = load(d, name)
        if v is not None:
            key = name[:-5].replace("-", "_")
            if name == "dmg-tests.json":
                v = {k: (x if k not in ("fsck_before_mark", "fsck_after_mark") else {"rc": x["rc"], "last_line": x["stdout"].strip().splitlines()[-1]}) for k, x in v.items()}
                v["forced_detach"] = {k: (x if k != "fsck_after_force" else {"rc": x["rc"], "last_line": x["stdout"].strip().splitlines()[-1]}) for k, x in v["forced_detach"].items()}
            S[key] = v
    for sub in ("scale", "smoke"):
        sd = os.path.join(d, sub)
        if os.path.isdir(sd):
            S[sub] = {}
            for name in sorted(os.listdir(sd)):
                if name.endswith(".json") and not name.startswith("walk-") and not name.startswith("ckpt-"):
                    v = json.load(open(os.path.join(sd, name)))
                    if name.startswith("wa-"):
                        v = {"label": v["label"], "rounds": len(v["rounds"]), "summary": v["summary"]}
                    if name.startswith("refresh-"):
                        v = {k: x for k, x in v.items()}
                        v["totals_only"] = {"immediate": v["totals_only"]["immediate"], "after_sync": {"changed_origins": v["totals_only"]["after_sync"]["changed_origins"], "ms": v["totals_only"]["after_sync"]["ms"]}}
                    S[sub][name[:-5]] = v
    json.dump(S, open(out, "w"), indent=1, default=str)
    # digest
    if ext:
        print("== semantics (external, %d replicates): step  root dgen/dphys/ddesc  A B C  first-read" % len(ext["replicates"]))
        for r in S["semantics_external"]["table"]:
            print(f"  {r['step']:28s} {str(r['root_dgen']):>5s} {str(r['root_dphys']):>9s} {str(r['root_ddesc']):>4s}  {str(r['A_dgen']):>4s} {str(r['B_dgen']):>4s} {str(r['C_dgen']):>3s}  {'sync' if r['visible_in_first_read'] else '-':>5s} {'' if r['consistent'] else 'INCONSISTENT'}")
    if "scale" in S:
        for k, v in S["scale"].items():
            print("== scale", k)
            print("  ", json.dumps(v, default=str)[:1500])
    print("wrote", out)


if __name__ == "__main__":
    main()
