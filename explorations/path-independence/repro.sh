#!/bin/bash
# Minimal repros for each distinct path-dependence found in fdu 0.1.0.
# Usage: ./repro.sh [1..8]   (no argument runs all)
# Each case builds a fresh fixture copy and a fresh XDG_CACHE_HOME.
set -u
DM=$(cd "$(dirname "$0")" && pwd)
FDU=${FDU_BIN:?set FDU_BIN to the fdu command under test}
PY=${FDU_PYTHON:?set FDU_PYTHON to a Python with the fdu package}

setup() {
  mkdir -p "$DM/manual"; W=$(mktemp -d "$DM/manual/repro.XXXX")
  "$DM/make_fixture.sh" "$W/tree" >/dev/null
  T=$W/tree
  export XDG_CACHE_HOME=$W/xdg
  mkdir -p "$XDG_CACHE_HOME"
}
# summarize: source, analyze header, share metric, first-report totals and row ids
show() {
  python3 -c '
import json, sys
raw = sys.stdin.read()
try:
    d = json.loads(raw)
except Exception:
    print("   (no JSON on stdout)"); sys.exit()
rep = d["reports"][0]
m = rep.get("metrics")
hdr = (d.get("analysis") or {}).get("analyze")
if m:
    t = m["total"]["metrics"]
    keys = ["physical_lines", "code_lines", "raw_words", "logical_words", "document_words"]
    print("   source=%s analyze=%s view=%s share=%s rows=%s" % (d["source"], hdr, rep["view"], m["share_metric"], [r["id"] for r in m["rows"]]))
    print("   total", {k: t.get(k) for k in keys}, "coverage", m["total"]["coverage"])
else:
    print("   source=%s analyze=%s view=%s" % (d["source"], hdr, rep["view"]))
'
}
f() { "$FDU" "$T" --format json "$@"; }

case1() {
  echo "== 1. Wider cached analyzers leak into a narrower request (header, unrequested metrics, and requested metrics)"
  setup
  echo " cold: fdu T --analyze lines --cache off"; f --analyze lines --cache off | show
  f --analyze all >/dev/null
  echo " after fdu T --analyze all: fdu T --analyze lines (auto / read-only / only)"
  f --analyze lines | show; f --analyze lines --cache read-only | show; f --analyze lines --cache only | show
}
case2() {
  echo "== 2. Cached code analysis changes a lines request's share metric and row order (languages view)"
  setup
  echo " cold:"; f --analyze lines --view languages --cache off | show
  f --analyze code >/dev/null
  echo " after --analyze code:"; f --analyze lines --view languages | show
}
case3() {
  echo "== 3. Wide cache + in-place edit + narrow request = mixed per-file analyzer sets (sticky)"
  setup
  f --analyze all >/dev/null
  touch "$T/src/lib.py"
  echo " cold narrow (code):"; f --analyze code --cache off | show
  echo " cold wide (all, languages view):"; f --analyze all --view languages --cache off | show
  echo " warm narrow after [all; touch lib.py]:"; f --analyze code | show
  echo " --cache only for each profile now:"
  for p in lines code words all; do f --analyze $p --cache only >/dev/null 2>"$W/err"; echo "   $p exit=$? $(head -c 110 "$W/err")"; done
  echo " a later plain lines request (auto) still reports mixed totals:"; f --analyze lines | show
  echo " healing requires a full-width auto run:"; f --analyze all >/dev/null; f --analyze lines --cache only | show
}
case4() {
  echo "== 4. Scope compatibility differs by policy and by surface (--no-gitignore from a controls-on snapshot)"
  setup; f >/dev/null
  echo " [default] then CLI --no-gitignore --cache only (succeeds, projected):"; f --no-gitignore --cache only | show
  echo " [default] then Python report(ONLY, read_controls=False) vs open(ONLY, read_controls=False):"
  "$PY" - "$T" <<'PYEOF'
import sys, fdu
root = sys.argv[1]
so = fdu.ScanOptions(read_controls=False)
for name, fn in [("report", lambda: fdu.report(root, cache=fdu.CachePolicy.ONLY, scan=so)),
                 ("open", lambda: fdu.open(root, cache=fdu.CachePolicy.ONLY, scan=so).report())]:
    try:
        d = fn().as_dict(); print("   %-6s ok source=%s" % (name, d["source"]))
    except Exception as e:
        print("   %-6s %s: %s" % (name, type(e).__name__, str(e)[:150]))
PYEOF
  setup; f --analyze all >/dev/null
  echo " [--analyze all] then --no-gitignore --analyze all --cache only (succeeds):"; f --no-gitignore --analyze all --cache only | show
  echo " [--analyze all] then same request --cache auto (treated as a miss: cold_scan):"; f --no-gitignore --analyze all | show
}
case5() {
  echo "== 5. --view summary --no-gitignore never writes a snapshot, so --cache only fails even after running it twice"
  setup
  f --view summary --no-gitignore >/dev/null; f --view summary --no-gitignore >/dev/null
  echo "   cache files: $(ls "$XDG_CACHE_HOME/fdu" 2>/dev/null | wc -l)"
  f --view summary --no-gitignore --cache only >/dev/null 2>"$W/err"; echo "   only exit=$? $(head -c 120 "$W/err")"
  setup
  f --view summary >/dev/null; echo "   (controls on) --view summary cache files: $(ls "$XDG_CACHE_HOME/fdu" | wc -l)"
}
case6() {
  echo "== 6. Content sidecar is reused across scan scopes: metadata cold_scan, analysis header widened"
  setup; echo " cold:"; f --scan-depth 1 --analyze code --cache off | show
  f --analyze all >/dev/null; echo " after --analyze all:"; f --scan-depth 1 --analyze code | show
  setup; f --analyze all >/dev/null
  echo " --gitignore-budget 1KiB --analyze lines after --analyze all (fully widened):"; f --gitignore-budget 1KiB --analyze lines | show
}
case7() {
  echo "== 7. Root cause of 1-3 in cold answers: lines/words metrics depend on which other analyzers ran"
  setup
  for p in lines code words all; do echo " cold --analyze $p --view families:"; f --analyze $p --view families --cache off | show; done
}

case8() {
  echo "== 8. No mutation needed: one differently scoped request evicts the snapshot and leaves a mixed sidecar"
  setup; f --analyze all >/dev/null
  f --scan-depth 1 --analyze lines >/dev/null
  echo " [all; --scan-depth 1 --analyze lines] then --analyze lines (cold lines: 259 document_words; cold all: 647):"
  f --analyze lines | show
  setup; f --analyze all >/dev/null; f --no-gitignore >/dev/null
  echo " [all; --no-gitignore] then --cache only for default / --analyze all:"
  for a in "" "--analyze all"; do f $a --cache only >/dev/null 2>"$W/err"; echo "   [${a:-default}] exit=$? $(head -c 150 "$W/err")"; done
}

if [ $# -eq 0 ]; then set -- 1 2 3 4 5 6 7 8; fi
for n in "$@"; do "case$n"; echo; done
