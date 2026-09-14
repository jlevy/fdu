---
type: is
id: is-01m2f0g2ecz6wyen1jzxy966tg
title: Ignore matcher drops empty segments, so a//b matches a/b where git matches nothing
kind: bug
status: open
priority: 4
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T03:48:42.060Z
updated_at: 2026-09-14T03:48:42.060Z
---
Found while fixing fdu-bqan (escaped slashes) at 777dc6f on codex/opened-root-inventory-rewrite. Not an escaped form, so left out of that fix.

`crates/fdu-core/src/control/gitignore.rs` (Pattern::parse, the `.filter(|(segment, _)| !segment.is_empty())` over `split_segments` at 777dc6f) drops every empty segment, so a doubled separator in a pattern reads as one. Git keeps it: wildmatch has to match `//` in the path, which a normalized path never has, so the line matches nothing.

Verdicts from `git -c core.ignorecase=false check-ignore --no-index -z --stdin` (git 2.50.1, user and system configuration isolated), and from the matcher at 777dc6f:

| Pattern | Path | git | fdu at 777dc6f |
| --- | --- | --- | --- |
| `a//b` | `a/b` | not ignored | ignored |

The same class reaches `a/\/b` (an escaped separator beside a plain one) and `a/**//` (a doubled trailing separator, which leaves `a/**/` directory-only where git needs a directory name ending in `/`).

The filter exists for real reasons: the leading anchor and the trailing `/` are stripped before splitting, and `///` must still drop to nothing (a recorded conformance case). Fix: treat an empty segment between two separators as one no component matches, so the line can never match, while keeping those cases. Add the rows to a recorded-verdict table with the live oracle. Low priority: nobody writes `a//b` on purpose.
