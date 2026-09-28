#!/bin/bash
# Ported: REVIEW and FDU come from the environment (timing-lock on PATH) instead of a private env.sh and an absolute binary path.
# Full-scan baseline for the FSEvents architecture review (read-only on real trees).
# Each fdu invocation runs under timing-lock (one short hold per command).
# Usage: bench.sh <label> <root> <rounds>
set -u
: "${REVIEW:?set REVIEW to the scratch working directory}"
LABEL="$1"; ROOT="$2"; ROUNDS="${3:-3}"
ARCH="$REVIEW/architecture"
RAW="$ARCH/raw"
CACHE="$ARCH/cache"
mkdir -p "$RAW" "$CACHE"
LOG="$RAW/bench-$LABEL.log"
TSV="$RAW/bench-$LABEL.tsv"
FDU="${FDU:-fdu}"

now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }

run_one() {
  # $1 round, $2 policy-name, rest: extra flags
  local round="$1"; local policy="$2"; shift 2
  local out="$RAW/bench-$LABEL-r${round}-${policy}.txt"
  local load; load="$(sysctl -n vm.loadavg | tr -d '{}' | awk '{print $1}')"
  local fse0; fse0="$(ps -o time= -p "$(pgrep -x fseventsd)" | tr -d ' ')"
  local t0; t0="$(now_ms)"
  timing-lock env FDU_COUNTERS=1 /usr/bin/time -l \
    "$FDU" "$ROOT" --view summary --color never --progress never --cache-dir "$CACHE" "$@" \
    >"$out" 2>&1
  local rc=$?
  local t1; t1="$(now_ms)"
  local wall=$(( t1 - t0 ))
  local rss; rss="$(awk '/maximum resident set size/{print $1}' "$out")"
  local real; real="$(awk '/ real /{print $1}' "$out")"
  local user; user="$(awk '/ real /{print $3}' "$out")"
  local sys;  sys="$(awk '/ real /{print $5}' "$out")"
  local perf; perf="$(grep -m1 '^perf:' "$out" | sed 's/^perf: //')"
  local summ; summ="$(grep -m1 -E '^ *[0-9.]+ [KMGT]?i?B ' "$out" | sed 's/^ *//')"
  local files; files="$(echo "$perf" | sed -n 's/.*walk \([0-9,]*\) files.*/\1/p' | tr -d ,)"
  local fps; fps="$(echo "$perf" | sed -n 's/.* at \([0-9,]*\) files\/s.*/\1/p' | tr -d ,)"
  local tier; tier="$(echo "$perf" | sed 's/.*; //')"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$LABEL" "$round" "$policy" "$rc" "$real" "$user" "$sys" "$rss" "$wall" "$load" "$files" "$fps" "$tier" "$summ" \
    | tee -a "$TSV"
  echo "[$(date +%H:%M:%S)] $LABEL r$round $policy rc=$rc real=${real}s rss=$rss load=$load fse_cpu=$fse0 :: $perf" >>"$LOG"
}

echo "=== $(date) label=$LABEL root=$ROOT rounds=$ROUNDS ===" >>"$LOG"
printf 'label\tround\tpolicy\trc\treal_s\tuser_s\tsys_s\tmax_rss_bytes\twall_ms_incl_lockwait\tload1\tfiles_walked\tfiles_per_s\ttier\tsummary\n' >>"$TSV"

# Round 0: first touch, "as found" regime; also creates the snapshot that `only` needs.
run_one 0 auto --cache auto

for r in $(seq 1 "$ROUNDS"); do
  run_one "$r" off-nogitignore --cache off --no-gitignore
  run_one "$r" off             --cache off
  run_one "$r" auto            --cache auto
  run_one "$r" only            --cache only
  run_one "$r" refresh         --cache refresh
done
echo "=== done $(date) ===" >>"$LOG"
