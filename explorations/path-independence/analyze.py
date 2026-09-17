#!/usr/bin/env python3
"""Summarize out/*/results.jsonl from matrix.py into out/summary.txt."""
import collections, json, sys
from pathlib import Path
DM = Path(__file__).resolve().parent
sys.path.insert(0, str(DM))
from matrix import R, W  # noqa: E402
OUT = DM / "out"
lines = []
p = lines.append
def load(ph): return [json.loads(l) for l in open(OUT / ph / "results.jsonl")]
def cell(v):
    return {"both-error": "E", "error-mismatch": "F"}.get(v["kind"], "=" if v["same"] else "≠")

cold = load("cold")
p(f"# cold: {len(cold)} requests; auto-on-empty-cache differs from off: {sum(not r['auto_vs_off']['same'] for r in cold)}")
p("  requests whose auto run left no cache file: " + str([r["R"] for r in cold if not r["cache_after_auto"]]))

sw = load("selfwarm")
p(f"\n# selfwarm: {len(sw)} steps; differing: " + str([(r['R'], r['step'], r['verdict']['kind']) for r in sw if not r['verdict']['same']]))

wm = load("warm")
p(f"\n# warm: {len(wm)} (W,R,policy) cases")
groups = collections.defaultdict(list)
for r in wm:
    v = r["verdict"]
    if v["same"] or (r["policy"] == "only" and v["kind"] == "error-mismatch"):
        continue
    groups[tuple(v.get("paths", []))].append(f"{r['W']}>{r['R']}:{r['policy']}")
p(f"  differing (excluding cache-only failures): {sum(len(c) for c in groups.values())}")
for k, c in sorted(groups.items(), key=lambda kv: -len(kv[1])):
    p(f"  - {len(c)} cases; paths={list(k)}\n      {c}")
rows = collections.defaultdict(dict); prov = collections.defaultdict(dict)
for r in wm:
    if r["policy"] == "only":
        rows[r["R"]][r["W"]] = cell(r["verdict"])
    else:
        pv = r["verdict"].get("prov")
        s = {"cold_scan": "c", "warm_revalidate": "w"}.get(pv["source"], "?") if pv else "-"
        prov[r["R"]].setdefault(r["W"], {})[r["policy"]] = s + ("" if r["verdict"]["same"] else "!")
p("\n  cache-only matrix after W (= same as cold, ≠ differs, F fails, E error in cold too)")
p("  %-24s " % "" + " ".join("%-6s" % w[2:8] for w in W))
for rid in R:
    p("  %-24s " % rid + " ".join("%-6s" % rows[rid][w] for w in W))
p("\n  source under auto/read-only after W (c cold_scan, w warm_revalidate, ! differs)")
for rid in R:
    p("  %-24s " % rid + " ".join("%-6s" % (prov[rid][w]["auto"] + "/" + prov[rid][w]["read-only"]) for w in W))

mu = load("mutation")
warm = {(r["W"], r["R"], r["policy"]): r["verdict"] for r in wm}
new = [r for r in mu if not r["verdict"]["same"] and warm[(r["W"], r["R"], r["policy"])]["same"]]
extra = collections.Counter()
for r in mu:
    v = r["verdict"]; wv = warm[(r["W"], r["R"], r["policy"])]
    if not v["same"] and not wv["same"]:
        ex = set(v.get("paths", [])) - set(wv.get("paths", []))
        if ex:
            extra[(r["M"], r["W"], r["R"], r["policy"], tuple(sorted(ex)))] += 1
p(f"\n# mutation: {len(mu)} cases; differing={sum(not r['verdict']['same'] for r in mu)}; "
  f"differing where unmutated warm was identical={len(new)}")
p("  differing with paths beyond the unmutated widening diff (mixed-provenance):")
for k in sorted(extra):
    p(f"   {k}")

cx = load("cross")
p(f"\n# cross: {len(cx)} cases")
cli_only = {(r["W"], r["R"]): cell(r["verdict"]) for r in wm if r["policy"] == "only"}
for label in sorted({r["label"] for r in cx}):
    rs = [r for r in cx if r["label"] == label]
    nd = sum(1 for r in rs if not r["verdict"]["same"] and r["verdict"]["kind"] != "both-error"
             and not (label.endswith("-only") and r["verdict"]["kind"] == "error-mismatch"))
    msg = f"  {label:28} n={len(rs):3} differing-from-cold={nd}"
    if label.endswith("-only"):
        dd = [(r["W"], r["R"], cell(r["verdict"])) for r in rs if cli_only[(r["W"], r["R"])] != cell(r["verdict"])]
        msg += f" cache-only outcome != CLI(W)->CLI only: {dd}"
    p(msg)
(OUT / "summary.txt").write_text("\n".join(lines) + "\n")
print("\n".join(lines[:3]))
