---
type: is
id: is-01m2ew7g4hvrgk341tvpay3epj
title: gitignore matcher splits at an escaped slash, where git matches a literal separator
kind: bug
status: open
priority: 4
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T02:34:06.864Z
updated_at: 2026-09-14T02:34:06.864Z
---
Found while fixing fdu-s0xg (PR #48 review CLASS-5) at 154e4d7. Not a bracket form, so left out of that fix.

`crates/fdu-core/src/control/gitignore.rs:158` (split_segments, at 154e4d7) keeps the matcher's old rule that every `/` splits a pattern into segments, escaped or not, and leaves the backslash on the end of the previous segment. Git reads `\/` as a literal `/`, which in a pathname match only a separator can match. Verdicts from `git -c core.ignorecase=false check-ignore --no-index` (git 2.50.1):

| Pattern | Path | git | fdu at 154e4d7 |
| --- | --- | --- | --- |
| `a\/b` | `a/b` | ignored | not ignored |
| `a\/b` | `a\` then `b` (unix) | not ignored | ignored |
| `\/foo` | `foo` | not ignored | not ignored |
| `\/foo` | `\` then `foo` (unix) | not ignored | ignored |
| `x/**\/y` | `x/y` | not ignored | ignored |
| `x/**\/y` | `x/q/y` | ignored | ignored |

The rows for fdu are from reading the code; confirm them with a test first. Also check the trailing forms: `:91` requires an unescaped trailing `/` for a directory-only pattern, but git's `parse_path_pattern` strips a trailing `/` whether or not it is escaped.

Fix: make an escaped `/` a separator that keeps git's two exceptions. A leading `\/` is not an anchor, so the line can never match. `**\/` matches one or more directories, never zero. Add these rows to the recorded-verdict table in the module's tests (`BRACKET_CASES` style, with the live oracle). Low priority: nobody writes `\/` on purpose.
