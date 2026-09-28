#!/bin/bash
# Ported: reads $REVIEW and $FDU from the environment instead of host paths.
# Start the watch soak on ROOT (read-only for us): fdu --watch under /usr/bin/time -l,
# stream (initial full files view = W0, then change records) to PRIVATE/soak.jsonl,
# with the cache on the external volume. Runs the startup under timing-lock (the initial
# scan and revalidation walk are heavy), releases it once the initial report is out,
# then starts the sampler detached. Prints startup timing and pids.
# usage: soak_start.sh ROOT INTERVAL
set -u
: "${REVIEW:?set REVIEW to the working directory (see ../README.md)}"
ROOT=$1; INTERVAL=${2:-10s}
WS=$REVIEW/watch-soak
PRIV=$WS/private
CACHE=$WS/cache
FDU=${FDU:-fdu}
export FDU_COUNTERS=1
mkdir -p "$PRIV" "$CACHE"
STREAM=$PRIV/soak.jsonl
ERR=$PRIV/soak.err
TIMEOUT=$PRIV/soak.time
rm -f "$STREAM" "$ERR" "$TIMEOUT" "$CACHE"/*.metadata.bin "$PRIV/watcher.pid" "$PRIV/start.txt"

start_watcher() {
  t0=$(date +%s.%N)
  echo "start_wall=$t0" > "$PRIV/start.txt"
  nohup /usr/bin/time -l "$FDU" "$ROOT" --watch --interval "$INTERVAL" --cache-dir "$CACHE" \
      --view files --format jsonl --full --quiet --progress never --color never \
      > "$STREAM" 2> "$ERR" < /dev/null &
  TPID=$!
  disown $TPID
  # the child of /usr/bin/time is the fdu (python) process
  for i in $(seq 1 100); do WPID=$(pgrep -P $TPID | head -1); [ -n "$WPID" ] && break; sleep 0.1; done
  echo "time_pid=$TPID" >> "$PRIV/start.txt"
  echo "watcher_pid=$WPID" >> "$PRIV/start.txt"
  echo "$WPID" > "$PRIV/watcher.pid"
  # wait for the initial report (the files view line), sampling RSS meanwhile
  for i in $(seq 1 3000); do
    if grep -q '"view": "files"' "$STREAM" 2>/dev/null; then break; fi
    if ! kill -0 "$WPID" 2>/dev/null; then echo "watcher died during startup" >> "$PRIV/start.txt"; break; fi
    if (( i % 10 == 0 )); then echo "startup_sample	$(date +%s.%N)	$(ps -o rss=,time= -p "$WPID" | awk '{print $1"\t"$2}')" >> "$PRIV/start.txt"; fi
    sleep 0.2
  done
  t1=$(date +%s.%N)
  echo "initial_report_wall=$t1" >> "$PRIV/start.txt"
  echo "startup_seconds=$(echo "$t1 - $t0" | bc)" >> "$PRIV/start.txt"
  echo "rss_kb_after_start=$(ps -o rss= -p "$WPID" | tr -d ' ')" >> "$PRIV/start.txt"
  echo "cputime_after_start=$(ps -o time= -p "$WPID" | tr -d ' ')" >> "$PRIV/start.txt"
  echo "stream_bytes_after_start=$(stat -f %z "$STREAM")" >> "$PRIV/start.txt"
  echo "snapshot_after_start=$(stat -f '%z %m' "$CACHE"/*.metadata.bin 2>/dev/null)" >> "$PRIV/start.txt"
  # give persistence one interval to settle before releasing the lock
  sleep 3
  echo "snapshot_after_start_plus3=$(stat -f '%z %m' "$CACHE"/*.metadata.bin 2>/dev/null)" >> "$PRIV/start.txt"
}
export -f start_watcher
export ROOT INTERVAL PRIV CACHE FDU STREAM ERR
"$WS/bin/lockwait" "$REVIEW/timing.lock" "${LOCK_TIMEOUT:-300}" bash -c start_watcher
cat "$PRIV/start.txt"
WPID=$(cat "$PRIV/watcher.pid")
nohup "$WS/src/sampler.sh" "$WPID" "$ROOT" "$CACHE" "$STREAM" "$PRIV" "$(sed -n "s/^stream_bytes_after_start=//p" "$PRIV/start.txt")" > "$PRIV/sampler.out" 2>&1 < /dev/null &
SPID=$!
disown $SPID
echo "sampler_pid=$SPID" | tee -a "$PRIV/start.txt"
