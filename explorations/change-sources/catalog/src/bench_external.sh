#!/bin/bash
# Ported: REVIEW and EXTERNAL_VOLUME come from the environment (timing-lock on PATH) instead of a private env.sh and an absolute mount path.
# External-volume searchfs series: timings, completeness dumps for fixture f1, open-writer test.
# Each search is its own lock hold.
: "${REVIEW:?set REVIEW to the scratch working directory}"
A=$REVIEW/catalog/attic
C=$REVIEW/catalog
LOG=$C/data/searchfs-runs.txt
V="${EXTERNAL_VOLUME:?set EXTERNAL_VOLUME to the external mount point}"
run() { echo "# $(date '+%H:%M:%S') start $5 load=$(uptime | sed 's/.*load averages: //')" >> "$LOG"; timing-lock "$A/searchfs_ct" "$1" "$2" "$3" "$4" --label "$5" "${@:6}" >> "$LOG" 2>&1; }
now=$(date +%s)
T0=$(cat $C/fixtures/f1.t0)
run $V ctime $((now+100000)) $((now+200000)) ext-zero-1
run $V ctime $T0 $((now+600)) f1-ctime --dump $C/data/f1-ctime.tsv
run $V mtime $T0 $((now+600)) f1-mtime --dump $C/data/f1-mtime.tsv
run $V ctime $((now-3600)) $((now+60)) ext-1h-1
run $V ctime $((now-86400)) $((now+60)) ext-1d-1
run $V ctime $((now+100000)) $((now+200000)) ext-zero-2
echo "# $(date '+%H:%M:%S') openwriter (ctime only; two searches inside one lock hold)" >> "$LOG"
cd $C && OPENWRITER_CTIME_ONLY=1 timing-lock uv run --no-project python src/complete.py openwriter fixtures/f1 $A/searchfs_ct $V $C/data/openwriter > $C/data/openwriter-stdout.txt 2>&1
echo "# $(date '+%H:%M:%S') external series done" >> "$LOG"
