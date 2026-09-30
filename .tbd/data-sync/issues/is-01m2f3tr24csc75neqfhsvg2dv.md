---
type: is
id: is-01m2f3tr24csc75neqfhsvg2dv
title: An ignore pattern a/\/b ignores a/b, where git matches nothing
kind: bug
status: closed
priority: 3
version: 7
delegate: codex
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m3r273jb24qc4hp7ak005jfm
hold: null
hold_until: null
created_at: 2026-09-14T04:46:57.603Z
updated_at: 2026-09-30T03:48:50.884Z
started_at: 2026-09-20T05:23:44.094Z
closed_at: 2026-09-30T03:48:50.884Z
close_reason: "Already fixed at b1376507 (landed in 430abbe3, before 0.1.0): Pattern::parse drops a pattern with an empty segment, so a/\\/b matches nothing, as git does. Verified with the release binary: --ignored=only --view files lists nothing for a/\\/b (and a//b) against a/b, a/q/b, matching git 2.43 check-ignore. Commit bbd8b48e adds the a/\\/b row to PATH_SEGMENT_CASES, which the live git oracle re-asks git about; test path_segment_edges_answer_as_git_check_ignore_does passes."
resolution: null
duplicate_of: null
---
PR #48 delta review PR48-CTRL-1 (https://github.com/jlevy/fdu/pull/48#pullrequestreview-5194007815). crates/fdu-core/src/control/gitignore.rs:124-125 and 183-187 at d48b8f8, from 777dc6f. The parser drops the empty segment in a/\/b, so the line ignores a/b. git's wildmatch fails there and the line matches nothing. The same empty-segment filter already gives the wrong answer for a//b and //foo on main. Fix: keep empty segments so they match nothing, the way git does. Record the expected answers from git check-ignore in the existing table test.

## Notes

2026-09-20 correctness review: independently confirmed against the PR91 head 870bdcfb binary. Patterns a//b and a/\\/b both cause --only-ignored --view files to emit a/b, while an isolated git check-ignore --no-index oracle says not ignored. Same root cause as fdu-mn69; retain both until one coherent parser fix and shared recorded-oracle regressions are reviewed.
