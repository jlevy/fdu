#!/bin/bash
# Usage: seqprobe.sh "<step>;<step>;..."   step = analyze profile run with --cache auto, or "touch" (touch src/lib.py)
# Prints, after the sequence, what `--analyze lines --view families` reports under auto (header + totals),
# and which profiles `--cache only` can then serve.
DM=$(cd "$(dirname "$0")" && pwd); F=${FDU_BIN:?set FDU_BIN to the fdu command under test}; mkdir -p "$DM/manual"
W=$(mktemp -d "$DM/manual/seq.XXXX"); cp -Rp "$DM/tree" "$W/tree"; T=$W/tree; export XDG_CACHE_HOME=$W/xdg; mkdir -p $XDG_CACHE_HOME
IFS=';' read -ra steps <<< "$1"
for s in "${steps[@]}"; do
  if [ "$s" = touch ]; then touch $T/src/lib.py; else $F $T --analyze "$s" --format json >/dev/null; fi
done
avail=""; for p in lines code words all; do $F $T --analyze $p --format json --cache only >/dev/null 2>&1 && avail="$avail $p"; done
$F $T --analyze lines --view families --format json | python3 -c "
import json,sys; d=json.load(sys.stdin); t=d['reports'][0]['metrics']['total']
print('%-28s'%sys.argv[1], 'hdr=',','.join(d['analysis']['analyze']), {k:t['metrics'][k] for k in ['physical_lines','code_lines','logical_words','document_words']}, 'cov=',t['coverage'], 'only-ok:%s'%sys.argv[2])" "$1" "$avail"
rm -rf "$W"
