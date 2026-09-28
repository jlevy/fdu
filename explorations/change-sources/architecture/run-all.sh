#!/bin/bash
# Ported: REVIEW, PROJECT_ROOT and FDU come from the environment; the private project scope is labelled 'project'.
# Driver: full-scan baseline matrix over the review scopes, then watch-mode
# load+reconcile timings, then a bounded whole-home feasibility attempt.
set -u
: "${REVIEW:?set REVIEW to the scratch working directory}" "${PROJECT_ROOT:?set PROJECT_ROOT to a large project checkout}"
ARCH="$REVIEW/architecture"; RAW="$ARCH/raw"; CACHE="$ARCH/cache"
mkdir -p "$RAW" "$CACHE"
USERTMP="$(getconf DARWIN_USER_TEMP_DIR)"
ROUNDS=3
{
  echo "=== run-all start $(date) ==="
  sysctl -n vm.loadavg; sysctl -n kern.maxvnodes kern.num_vnodes
} >>"$RAW/run-all.log"

"$ARCH/bench.sh" codex   "$HOME/.codex"            "$ROUNDS"
"$ARCH/bench.sh" claude  "$HOME/.claude"           "$ROUNDS"
"$ARCH/bench.sh" project "$PROJECT_ROOT"  "$ROUNDS"
"$ARCH/bench.sh" usertmp "$USERTMP"                "$ROUNDS"
"$ARCH/bench.sh" privtmp /private/tmp              "$ROUNDS"

# Phase 2: watch-mode first report = snapshot load + full reconcile (read-only, no write).
printf 'label\troot\tfirst_report_s\tsource\trss_kib\tsummary\tstderr\n' >"$RAW/watch-first.tsv"
for rep in 1 2; do
  for pair in "codex:$HOME/.codex" "claude:$HOME/.claude" "project:$PROJECT_ROOT"; do
    lbl="${pair%%:*}"; rt="${pair#*:}"
    timing-lock python3 "$ARCH/watch_first.py" "$lbl-rep$rep" "$rt" "$CACHE" >>"$RAW/watch-first.tsv" 2>>"$RAW/run-all.log"
  done
done

# Phase 3: whole-home feasibility, cheapest route (summary reducer), bounded to 200 s.
{
  echo "=== home attempt $(date) load=$(sysctl -n vm.loadavg) ==="
  timing-lock timeout 200 /usr/bin/time -l "${FDU:-fdu}" "$HOME" --view summary \
    --cache off --no-gitignore --color never --progress never --cache-dir "$CACHE"
  echo "rc=$?"
  echo "=== home attempt end $(date) ==="
} >"$RAW/home-attempt.txt" 2>&1
echo "=== run-all done $(date) ===" >>"$RAW/run-all.log"
