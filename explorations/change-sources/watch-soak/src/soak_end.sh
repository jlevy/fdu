#!/bin/bash
# Ported: reads $REVIEW and $FDU from the environment instead of host paths.
# End the watch soak: oracle walk O1 (fresh, --cache off), wait for the watcher's next
# persisted snapshot and copy it aside (W), oracle walk O2, then stop the watcher and
# sampler, and record whether the snapshot changed at exit. Runs under timing-lock.
# usage: soak_end.sh ROOT
set -u
: "${REVIEW:?set REVIEW to the working directory (see ../README.md)}"
ROOT=$1
WS=$REVIEW/watch-soak
PRIV=$WS/private
CACHE=$WS/cache
FDU=${FDU:-fdu}
WPID=$(cat "$PRIV/watcher.pid")
TPID=$(sed -n 's/^time_pid=//p' "$PRIV/start.txt")
mkdir -p "$PRIV/W"
END=$PRIV/end.txt
: > "$END"
log() { echo "$(date +%s.%N)	$*" | tee -a "$END"; }

oracle() { # $1 = label
  local t0 t1
  t0=$(date +%s.%N)
  /usr/bin/time -l "$FDU" "$ROOT" --cache off --view files --format jsonl --full --quiet --progress never --color never \
      > "$PRIV/$1.jsonl" 2> "$PRIV/$1.err"
  t1=$(date +%s.%N)
  log "$1 walk seconds=$(echo "$t1 - $t0" | bc) bytes=$(stat -f %z "$PRIV/$1.jsonl") exit=$?"
}

endgame() {
  log "endgame begin; watcher pid $WPID alive=$(kill -0 $WPID 2>/dev/null && echo yes || echo no)"
  snap_before=$(stat -f '%m' "$CACHE"/*.metadata.bin)
  log "snapshot mtime before O1: $snap_before"
  oracle O1
  # wait for the next persisted snapshot (mtime change), up to 120 s
  for i in $(seq 1 240); do
    m=$(stat -f '%m' "$CACHE"/*.metadata.bin)
    if [ "$m" != "$snap_before" ]; then break; fi
    sleep 0.5
  done
  log "snapshot mtime after wait: $m (changed=$([ "$m" != "$snap_before" ] && echo yes || echo no), waited ~$((i/2)) s)"
  # copy the snapshot aside; retry if a rename lands mid-copy (size/mtime must be stable)
  for try in 1 2 3; do
    f=$(ls "$CACHE"/*.metadata.bin)
    m1=$(stat -f '%m' "$f"); cp "$f" "$PRIV/W/$(basename "$f")"; m2=$(stat -f '%m' "$f")
    [ "$m1" = "$m2" ] && break
  done
  log "W copied: snapshot mtime=$m1 size=$(stat -f %z "$PRIV/W/"*.metadata.bin) stream_bytes=$(stat -f %z "$PRIV/soak.jsonl")"
  oracle O2
  log "stopping watcher (SIGTERM to $WPID)"
  before=$(stat -f '%z %m' "$CACHE"/*.metadata.bin)
  kill -TERM "$WPID"
  for i in $(seq 1 100); do kill -0 "$WPID" 2>/dev/null || break; sleep 0.1; done
  sleep 1
  after=$(stat -f '%z %m' "$CACHE"/*.metadata.bin)
  log "snapshot at stop: before='$before' after='$after' (changed=$([ "$before" != "$after" ] && echo yes || echo no))"
  log "watcher alive after TERM: $(kill -0 $WPID 2>/dev/null && echo yes || echo no); time wrapper alive: $(kill -0 $TPID 2>/dev/null && echo yes || echo no)"
  # extract W from the copied snapshot
  t0=$(date +%s.%N)
  "$FDU" "$ROOT" --cache only --cache-dir "$PRIV/W" --view files --format jsonl --full --quiet --progress never --color never \
      > "$PRIV/W.jsonl" 2> "$PRIV/W.err"
  log "W extract exit=$? seconds=$(echo "$(date +%s.%N) - $t0" | bc) bytes=$(stat -f %z "$PRIV/W.jsonl")"
}
export -f endgame oracle log
export ROOT WS PRIV CACHE FDU WPID TPID END
"$WS/bin/lockwait" "$REVIEW/timing.lock" "${LOCK_TIMEOUT:-300}" bash -c endgame
sleep 6
echo "sampler alive: $(pgrep -f sampler.sh | tr '\n' ' ')"
pkill -f "$WS/src/sampler.sh"
echo "leftover fdu watch processes: $(pgrep -f 'fdu .*--watch' | tr '\n' ' ')"
tail -5 "$PRIV/soak.err"
