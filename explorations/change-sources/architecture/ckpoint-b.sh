#!/bin/bash
# Ported: REVIEW and FDU come from the environment; the 22:00:30 start gate is the original one-off timing.
set -u
: "${REVIEW:?set REVIEW to the scratch working directory}"
A="$REVIEW/architecture"; R="$A/raw"
# wait until 22:00:30 local (>= 10 min after checkpoint A at 21:48)
while [ "$(date +%H%M%S)" -lt 220030 ]; do sleep 20; done
date +%T > "$R/codex-dirs-B.stamp"
timing-lock /usr/bin/time -l "${FDU:-fdu}" "$HOME/.codex" --full --view tree --kind dir --format json --cache off --color never --progress never --cache-dir "$A/cache" > "$R/codex-dirs-B.json" 2> "$R/codex-dirs-B.time"
date +%T >> "$R/codex-dirs-B.stamp"
python3 "$A/ckdiff.py" "$R/codex-dirs-A.json" "$R/codex-dirs-B.json" "$R/codex-diff.txt" > /dev/null 2>"$R/codex-diff.err"
echo "ckpoint-b done $(date)" >> "$R/run-all.log"
