#!/bin/bash
# Ported: reads $REVIEW and $FDU from the environment instead of host paths.
# Periodic sampler for the watch soak. Every 5 s: stream size and snapshot file stat
# (persistence timeline). Every 30 s: watcher RSS/CPU/threads, fseventsd CPU time, load,
# change-record counts in the stream region appended since the initial report, and a
# libproc open-writer enumeration under ROOT (fd records every sample, writable shared
# mappings every 10th). All output is PRIVATE (paths inside).
# usage: sampler.sh WATCHER_PID ROOT CACHE_DIR STREAM_LOG PRIVATE_DIR STREAM_BASE_BYTES
set -u
WPID=$1; ROOT=$2; CACHE=$3; STREAM=$4; PRIV=$5; BASE=${6:-0}
BIN=$(cd "$(dirname "$0")/../bin" && pwd)
FSE=$(pgrep -x fseventsd | head -1)
mkdir -p "$PRIV/writers"
FAST="$PRIV/timeline_5s.tsv"
SLOW="$PRIV/samples_30s.tsv"
echo -e "ts\tstream_bytes\tsnap_size\tsnap_mtime" > "$FAST"
echo -e "ts\twatcher_rss_kb\twatcher_vsz_kb\twatcher_cputime\twatcher_pcpu\twatcher_threads\tfseventsd_cputime\tload1\trecords\tupserts\tremoves\tinvalidates\troot_invalidates\tunder_fds\tunder_files\tunder_pids\tunder_deleted_files\tunder_deleted_alloc\tunder_map_files\twriters_ms" > "$SLOW"
n=0
while kill -0 "$WPID" 2>/dev/null; do
  ts=$(date +%s)
  sbytes=$(stat -f %z "$STREAM" 2>/dev/null || echo 0)
  snap=$(stat -f '%z	%m' "$CACHE"/*.metadata.bin 2>/dev/null || echo -e "0\t0")
  echo -e "$ts\t$sbytes\t$snap" >> "$FAST"
  if (( n % 6 == 0 )); then
    read -r rss vsz cput pcpu <<< "$(ps -o rss=,vsz=,time=,%cpu= -p "$WPID" | awk '{print $1, $2, $3, $4}')"
    threads=$(ps -M -p "$WPID" 2>/dev/null | tail -n +2 | wc -l | tr -d ' ')
    fse=$(ps -o time= -p "$FSE" | tr -d ' ')
    load1=$(sysctl -n vm.loadavg | awk '{print $2}')
    counts=$(tail -c +$((BASE+1)) "$STREAM" 2>/dev/null | awk '/"record": "change"/{r++} /"op": "upsert"/{u++} /"op": "remove"/{d++} /"op": "invalidate"/{i++; if ($0 ~ /"path": ""/) ri++} END{printf "%d\t%d\t%d\t%d\t%d", r, u, d, i, ri}')
    if (( n % 60 == 0 )); then mapflag=--maps; else mapflag=; fi
    out="$PRIV/writers/w_$ts.tsv"
    "$BIN/writers_under" "$ROOT" $mapflag > "$out" 2>/dev/null
    summary=$(tail -1 "$out")
    ufds=$(sed -n 's/.* under_fds=\([0-9]*\).*/\1/p' <<< "$summary")
    ufiles=$(sed -n 's/.* under_files=\([0-9]*\).*/\1/p' <<< "$summary")
    upids=$(sed -n 's/.* under_pids=\([0-9]*\).*/\1/p' <<< "$summary")
    udel=$(sed -n 's/.* under_deleted_files=\([0-9]*\).*/\1/p' <<< "$summary")
    udela=$(sed -n 's/.* under_deleted_alloc=\([0-9]*\).*/\1/p' <<< "$summary")
    umap=$(sed -n 's/.* under_map_files=\([0-9]*\).*/\1/p' <<< "$summary")
    wms=$(sed -n 's/.* total_ms=\([0-9.]*\).*/\1/p' <<< "$summary")
    echo -e "$ts\t$rss\t$vsz\t$cput\t$pcpu\t$threads\t$fse\t$load1\t$counts\t$ufds\t$ufiles\t$upids\t$udel\t$udela\t$umap\t$wms" >> "$SLOW"
  fi
  n=$((n+1))
  sleep 5
done
echo "sampler: watcher $WPID gone at $(date +%s)" >> "$SLOW"
