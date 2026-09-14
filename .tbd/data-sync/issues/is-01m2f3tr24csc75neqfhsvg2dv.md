---
type: is
id: is-01m2f3tr24csc75neqfhsvg2dv
title: An ignore pattern a/\/b ignores a/b, where git matches nothing
kind: bug
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:46:57.603Z
updated_at: 2026-09-14T04:46:57.603Z
---
PR #48 delta review PR48-CTRL-1 (https://github.com/jlevy/fdu/pull/48#pullrequestreview-5194007815). crates/fdu-core/src/control/gitignore.rs:124-125 and 183-187 at d48b8f8, from 777dc6f. The parser drops the empty segment in a/\/b, so the line ignores a/b. git's wildmatch fails there and the line matches nothing. The same empty-segment filter already gives the wrong answer for a//b and //foo on main. Fix: keep empty segments so they match nothing, the way git does. Record the expected answers from git check-ignore in the existing table test.
