---
type: is
id: is-01m2f0g6bsatn9gksxtt4zvytr
title: Ignore matcher reads *** between separators as *, where git reads it as **
kind: bug
status: in_progress
priority: 4
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
hold: null
hold_until: null
created_at: 2026-09-14T03:48:46.073Z
updated_at: 2026-09-20T05:23:44.135Z
started_at: 2026-09-20T05:23:44.135Z
---
Found while fixing fdu-bqan (escaped slashes) at 777dc6f on codex/opened-root-inventory-rewrite.

`crates/fdu-core/src/control/gitignore.rs` (Pattern::parse at 777dc6f) makes a segment a `**` only when it is exactly `**`. A run of three or more stars becomes `Segment::Glob`, which `normalize_glob` collapses to `*`, matching exactly one component. Git's wildmatch treats any run of two or more stars between separators as `**`: after the second star it skips the rest (`while (*++p == '*')`) before checking the separators around the run.

Verdicts from `git -c core.ignorecase=false check-ignore --no-index -z --stdin` (git 2.50.1, isolated configuration), and from the matcher at 777dc6f:

| Pattern | Path | git | fdu at 777dc6f |
| --- | --- | --- | --- |
| `a/***/b` | `a/b` | ignored | not ignored |
| `a/***/b` | `a/q/b` | ignored | ignored |
| `a/***/b` | `a/q/r/b` | ignored | not ignored |

Fix: treat a segment of two or more stars as `**` (and, before an escaped separator, as `DoubleStarOneOrMore`), and add the rows to a recorded-verdict table with the live oracle. Check the leading and trailing forms (`***/x`, `x/***`) against git too. Low priority.

## Notes

2026-09-20 correctness review: independently confirmed against PR91 head 870bdcfb. With a/b and a/q/r/b present, pattern a/***/b produces no --only-ignored file rows, while isolated git check-ignore marks both ignored. Oracle also confirms ***/x matches x and x/*** matches x/a/b. Include these leading/trailing forms in the recorded table for the eventual coherent matcher fix.
