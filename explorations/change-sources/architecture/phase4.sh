#!/bin/bash
# Ported: REVIEW, PROJECT_ROOT and FDU come from the environment; the private project scope is labelled 'project'.
# Phase 4 (after run-all): isolate index-retention cost from .gitignore cost.
#   off-nogitignore-tree: full index retained, no control files read (--view tree forces the index)
#   off-nogitignore (summary reducer) and off (index + controls) already exist in bench-*.tsv.
set -u
: "${REVIEW:?set REVIEW to the scratch working directory}" "${PROJECT_ROOT:?set PROJECT_ROOT to a large project checkout}"
ARCH="$REVIEW/architecture"; RAW="$ARCH/raw"; CACHE="$ARCH/cache"
WAITPID="${1:-}"
if [ -n "$WAITPID" ]; then while kill -0 "$WAITPID" 2>/dev/null; do sleep 15; done; fi
USERTMP="$(getconf DARWIN_USER_TEMP_DIR)"
OUT="$RAW/phase4.tsv"
printf 'label\trep\tpolicy\trc\treal_s\tuser_s\tsys_s\tmax_rss_bytes\tload1\tperf\n' >"$OUT"
for rep in 1 2; do
  for pair in "codex:$HOME/.codex" "claude:$HOME/.claude" "project:$PROJECT_ROOT"; do
    lbl="${pair%%:*}"; rt="${pair#*:}"
    for variant in "off-nogitignore-tree:--cache off --no-gitignore --view tree --depth 1 --limit 1" "off-nogitignore-summary:--cache off --no-gitignore --view summary"; do
      pol="${variant%%:*}"; flags="${variant#*:}"
      f="$RAW/phase4-$lbl-rep$rep-$pol.txt"
      load="$(sysctl -n vm.loadavg | tr -d '{}' | awk '{print $1}')"
      # shellcheck disable=SC2086
      timing-lock /usr/bin/time -l "${FDU:-fdu}" "$rt" $flags --color never --progress never --cache-dir "$CACHE" >"$f" 2>&1
      rc=$?
      real="$(awk '/ real /{print $1}' "$f")"; user="$(awk '/ real /{print $3}' "$f")"; sys="$(awk '/ real /{print $5}' "$f")"
      rss="$(awk '/maximum resident set size/{print $1}' "$f")"; perf="$(grep -m1 '^perf:' "$f" | sed 's/^perf: //')"
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$lbl" "$rep" "$pol" "$rc" "$real" "$user" "$sys" "$rss" "$load" "$perf" >>"$OUT"
    done
  done
done
echo "phase4 done $(date)" >>"$RAW/run-all.log"
