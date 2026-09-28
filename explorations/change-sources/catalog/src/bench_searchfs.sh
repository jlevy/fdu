#!/bin/bash
# Ported: REVIEW and EXTERNAL_VOLUME come from the environment (timing-lock on PATH) instead of a private env.sh and an absolute mount path.
# Run a series of volume-wide searchfs timings, each in its own timing-lock hold.
# Usage: bench_searchfs.sh SERIES   (series: internal | external | interleave)
: "${REVIEW:?set REVIEW to the scratch working directory}"
A=$REVIEW/catalog/attic
LOG=$REVIEW/catalog/data/searchfs-runs.txt
run() { # vol attr lower upper label
    local now; now=$(date +%s)
    echo "# $(date '+%H:%M:%S') start $5 load=$(uptime | sed 's/.*load averages: //')" >> "$LOG"
    timing-lock "$A/searchfs_ct" "$1" "$2" "$3" "$4" --label "$5" >> "$LOG" 2>&1
}
now=$(date +%s)
case "$1" in
internal)
    V=/System/Volumes/Data
    run $V ctime $((now+100000)) $((now+200000)) int-zero-1
    run $V ctime $((now-3600)) $((now+60)) int-1h-1
    run $V ctime $((now-86400)) $((now+60)) int-1d-1
    run $V ctime $((now+100000)) $((now+200000)) int-zero-2
    run $V ctime $((now-3600)) $((now+60)) int-1h-2
    run $V mtime $((now-3600)) $((now+60)) int-1h-mtime-1
    run $V ctime $((now-86400)) $((now+60)) int-1d-2
    ;;
external)
    V="${EXTERNAL_VOLUME:?set EXTERNAL_VOLUME to the external mount point}"
    run $V ctime $((now+100000)) $((now+200000)) ext-zero-1
    run $V ctime $((now-3600)) $((now+60)) ext-1h-1
    run $V ctime $((now-86400)) $((now+60)) ext-1d-1
    run $V ctime $((now+100000)) $((now+200000)) ext-zero-2
    run $V ctime $((now-3600)) $((now+60)) ext-1h-2
    ;;
*) echo "unknown series"; exit 2;;
esac
echo "# $(date '+%H:%M:%S') series $1 done" >> "$LOG"
