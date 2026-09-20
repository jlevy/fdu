---
type: is
id: is-01m2f3tr24csc75neqfhsvg2dv
title: An ignore pattern a/\/b ignores a/b, where git matches nothing
kind: bug
status: in_progress
priority: 3
version: 4
delegate: codex@spud10
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-14T04:46:57.603Z
updated_at: 2026-09-20T05:30:29.109Z
started_at: 2026-09-20T05:23:44.094Z
---
PR #48 delta review PR48-CTRL-1 (https://github.com/jlevy/fdu/pull/48#pullrequestreview-5194007815). crates/fdu-core/src/control/gitignore.rs:124-125 and 183-187 at d48b8f8, from 777dc6f. The parser drops the empty segment in a/\/b, so the line ignores a/b. git's wildmatch fails there and the line matches nothing. The same empty-segment filter already gives the wrong answer for a//b and //foo on main. Fix: keep empty segments so they match nothing, the way git does. Record the expected answers from git check-ignore in the existing table test.

## Notes

2026-09-20 correctness review: independently confirmed against the PR91 head 870bdcfb binary. Patterns a//b and a/\\/b both cause --only-ignored --view files to emit a/b, while an isolated git check-ignore --no-index oracle says not ignored. Same root cause as fdu-mn69; retain both until one coherent parser fix and shared recorded-oracle regressions are reviewed.
