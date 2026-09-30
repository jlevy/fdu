---
type: is
id: is-01m2f0g2ecz6wyen1jzxy966tg
title: Ignore matcher drops empty segments, so a//b matches a/b where git matches nothing
kind: bug
status: closed
priority: 4
version: 6
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m3r273jb24qc4hp7ak005jfm
hold: null
hold_until: null
created_at: 2026-09-14T03:48:42.060Z
updated_at: 2026-09-30T03:48:51.257Z
started_at: 2026-09-20T05:23:44.114Z
closed_at: 2026-09-30T03:48:51.257Z
close_reason: "Already fixed at b1376507 (landed in 430abbe3, before 0.1.0): an empty segment between separators makes the pattern match nothing, as git's wildmatch does, while /// and a/**/ keep their recorded answers. PATH_SEGMENT_CASES records a//b, //foo, a\\//b, a/**// against the live git oracle; verified with the release binary against git 2.43 (a//b ignores nothing). Same root cause as fdu-c5kn."
resolution: null
duplicate_of: null
---
Found while fixing fdu-bqan (escaped slashes) at 777dc6f on codex/opened-root-inventory-rewrite. Not an escaped form, so left out of that fix.

`crates/fdu-core/src/control/gitignore.rs` (Pattern::parse, the `.filter(|(segment, _)| !segment.is_empty())` over `split_segments` at 777dc6f) drops every empty segment, so a doubled separator in a pattern reads as one. Git keeps it: wildmatch has to match `//` in the path, which a normalized path never has, so the line matches nothing.

Verdicts from `git -c core.ignorecase=false check-ignore --no-index -z --stdin` (git 2.50.1, user and system configuration isolated), and from the matcher at 777dc6f:

| Pattern | Path | git | fdu at 777dc6f |
| --- | --- | --- | --- |
| `a//b` | `a/b` | not ignored | ignored |

The same class reaches `a/\/b` (an escaped separator beside a plain one) and `a/**//` (a doubled trailing separator, which leaves `a/**/` directory-only where git needs a directory name ending in `/`).

The filter exists for real reasons: the leading anchor and the trailing `/` are stripped before splitting, and `///` must still drop to nothing (a recorded conformance case). Fix: treat an empty segment between two separators as one no component matches, so the line can never match, while keeping those cases. Add the rows to a recorded-verdict table with the live oracle. Low priority: nobody writes `a//b` on purpose.
