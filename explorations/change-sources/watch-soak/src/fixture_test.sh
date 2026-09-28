#!/bin/bash
# Ported: reads $REVIEW and $FDU from the environment instead of host paths.
# Fixture check of the W-extraction method: start `fdu --watch` on an owned fixture,
# mutate it (create, append while open, rename, close, delete), and compare the stream,
# the persisted snapshot (--cache only), and a fresh walk (--cache off) at each step.
# Usage: fixture_test.sh  (works under $REVIEW/watch-soak/fixture)
set -u
: "${REVIEW:?set REVIEW to the working directory (see ../README.md)}"
FDU=${FDU:-fdu}
FX=$REVIEW/watch-soak/fixture
ROOT=$FX/root
CACHE=$FX/fcache
LOG=$FX/watch.jsonl
ERR=$FX/watch.err
rm -rf "$ROOT" "$CACHE" "$LOG" "$ERR"
mkdir -p "$ROOT/sub" "$CACHE"
printf 'seed\n' > "$ROOT/seed.txt"
printf 'x%.0s' $(seq 1 3000) > "$ROOT/sub/b.bin"

walk() { "$FDU" "$ROOT" --cache off --view files --format jsonl --full --quiet 2>/dev/null | tail -1; }
cached() { "$FDU" "$ROOT" --cache only --cache-dir "$CACHE" --view files --format jsonl --full --quiet 2>&1 | tail -1; }
rows() { # print path bytes allocated mtime_ns per row from a files-view line on stdin
  "$HOME/.local/share/uv/tools/fdu/bin/python" -c '
import json,sys
line=sys.stdin.read().strip()
if not line.startswith("{"): print("  (no rows:", line[:120], ")"); sys.exit()
d=json.loads(line)
for r in d.get("files",[]):
    print("  %-14s %-8s bytes=%-6d alloc=%-6d mtime=%d" % (r["path"], r["kind"], r["bytes"], r["allocated"], r["mtime_ns"]))
'
}
snap_stat() { stat -f 'snapshot: size=%z mtime=%m' "$CACHE"/*.metadata.bin 2>/dev/null || echo "snapshot: absent"; }
stream_since() { # print stream records after line $1
  tail -n +"$1" "$LOG" | grep '"record": "change"' | sed 's/^/  stream: /'
}

echo "== start watcher (interval 2s)"
"$FDU" "$ROOT" --watch --interval 2s --cache-dir "$CACHE" --view files --format jsonl --full --quiet > "$LOG" 2> "$ERR" &
WPID=$!
for i in $(seq 1 100); do grep -q '"view": "files"' "$LOG" 2>/dev/null && break; sleep 0.1; done
echo "watcher pid $WPID, initial report after ~$((i*100)) ms; lines=$(wc -l < "$LOG")"
sleep 3
echo "-- after startup: $(snap_stat)"
mark=$(( $(wc -l < "$LOG") + 1 ))

echo "== T1 create new file"
printf 'created\n' > "$ROOT/new.txt"
sleep 3
stream_since $mark; mark=$(( $(wc -l < "$LOG") + 1 ))
echo "-- $(snap_stat)"
echo "-- cached view:"; cached | rows
echo "-- walk:"; walk | rows

echo "== T2 open seed.txt for append, hold open, append 4096 bytes (no close)"
exec 7>>"$ROOT/seed.txt"
head -c 4096 /dev/zero | tr '\0' 'a' >&7
sleep 3
stream_since $mark; mark=$(( $(wc -l < "$LOG") + 1 ))
echo "-- $(snap_stat)"
echo "-- cached view (expect seed.txt still 5 bytes):"; cached | rows
echo "-- walk (expect seed.txt 4101 bytes):"; walk | rows

echo "== T3 rename new.txt -> moved.txt (still holding seed.txt open)"
mv "$ROOT/new.txt" "$ROOT/moved.txt"
sleep 3
stream_since $mark; mark=$(( $(wc -l < "$LOG") + 1 ))
echo "-- $(snap_stat)"
echo "-- cached view (does the rename-triggered reconcile pick up seed.txt growth?):"; cached | rows

echo "== T4 close seed.txt"
exec 7>&-
sleep 3
stream_since $mark; mark=$(( $(wc -l < "$LOG") + 1 ))
echo "-- $(snap_stat)"
echo "-- cached view (expect seed.txt 4101):"; cached | rows

echo "== T5 delete sub/b.bin and moved.txt"
rm "$ROOT/sub/b.bin" "$ROOT/moved.txt"
sleep 3
stream_since $mark; mark=$(( $(wc -l < "$LOG") + 1 ))
echo "-- $(snap_stat)"
echo "-- cached view:"; cached | rows
echo "-- walk:"; walk | rows

echo "== T6 SIGTERM the watcher; does the snapshot change afterwards?"
before=$(snap_stat)
kill -TERM $WPID; wait $WPID 2>/dev/null; echo "watcher exit status $?"
sleep 1
echo "-- before: $before"
echo "-- after:  $(snap_stat)"
echo "-- stderr:"; sed 's/^/  /' "$ERR"
echo "-- total stream records: $(grep -c '"record": "change"' "$LOG"); invalidates: $(grep -c '"op": "invalidate"' "$LOG")"
