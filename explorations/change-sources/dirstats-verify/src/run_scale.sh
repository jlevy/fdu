#!/bin/bash
# Orchestrate the scale stages on the two identical fixtures, each stage under its own
# short timing-lock hold. Usage: run_scale.sh FIXTURE_A FIXTURE_B PROBE_DIR OUTSIDE_DIR DATA_DIR ROUNDS
set -u
A=$1; B=$2; PROBE=$3; OUTSIDE=$4; DATA=$5; ROUNDS=${6:-7}
SRC=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$DATA"
run() { echo "=== $(date +%H:%M:%S) $*"; timing-lock "$@" 2>&1 | cut -c1-2500; }
run python3 "$SRC/mark_cost.py" "$A" "$PROBE" "$DATA/mark-root.json" --stage root
run python3 "$SRC/bench_wa.py" "$A" "$B" one-origin "$ROUNDS" "$DATA/wa-one-origin.json"
run python3 "$SRC/mark_cost.py" "$A" "$PROBE" "$DATA/mark-depth3.json" --stage depth3
run python3 "$SRC/bench_wa.py" "$A" "$B" depth3 "$ROUNDS" "$DATA/wa-depth3.json"
run python3 "$SRC/refresh.py" "$A" "$OUTSIDE" "$DATA/refresh-depth3.json" depth3
run python3 "$SRC/mark_cost.py" "$A" "$PROBE" "$DATA/mark-all.json" --stage all
run python3 "$SRC/bench_wa.py" "$A" "$B" all-levels "$ROUNDS" "$DATA/wa-all-levels.json"
run python3 "$SRC/refresh.py" "$A" "$OUTSIDE" "$DATA/refresh-all.json" all
echo "=== $(date +%H:%M:%S) done"
