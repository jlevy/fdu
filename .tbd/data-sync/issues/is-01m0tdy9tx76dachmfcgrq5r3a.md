---
type: is
id: is-01m0tdy9tx76dachmfcgrq5r3a
title: Reject a zero child-page limit
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:54.973Z
updated_at: 2026-08-24T23:08:45.214Z
closed_at: 2026-08-24T23:08:45.213Z
close_reason: |
  Shipped with `fdu-g0n4` in one commit. `make check` green, parity holds.

  MECHANISM, verified in `child_page`. With `Bound::Limit(0)` the loop breaks on the first
  child, so `more` is true and a remainder is produced -- and `next` is
  `more.then_some(last).flatten()`, where `last` is the last *emitted* name and nothing was
  emitted. The page is truncated and terminal at once: the caller is told there is more and
  handed no cursor to ask with.

  REFUSED, not clamped. A clamp to one answers a question the caller did not ask.
  `ChildPageRequest::validate` returns `Error::InvalidValue { kind: "page limit", .. }` and
  runs on both public entries -- `children_page` and the bundled `read` -- with a
  `debug_assert` inside `child_page` so a future third caller fails loudly rather than
  producing the stranded page again.

  DELIBERATELY STILL LEGAL: a zero *extension* bound (an extension map with everything in
  the remainder is a real answer) and a depth of zero (the root by itself). The rule is
  about *paged* bounds, where truncation has to imply resumability, and the doc comment says
  so where the next bound gets added.

  One existing test swept `for limit in 0..=7`; it starts at one now, with a comment
  pointing at the new test for the other end.

  TWO THINGS WORTH RECORDING.
  - My first insertion of the new test landed between `#[cfg(windows)]` and its `#[test]`,
    so the Windows-only test lost its gate and ran on Linux (where `C:\escape` is an
    ordinary filename and is not rejected) while the new test silently became Windows-only.
    Both symptoms -- one unexplained failure and one test that never ran -- came from that.
    Caught because the full suite was run rather than a filtered one; the filtered runs had
    been reporting "0 passed" and I had read that as "fine".
  - `npx tryscript run --update` outside the harness has no `fdu` on PATH, so it recorded
    "command not found" into every session in the file. Reverted, then used
    `make golden-update`. It also wrote `total 1.1 ms` and literal `/` separators where
    `[PERF_TIME]` and `[SEP]` belong -- exactly the trap AGENTS.md warns about -- so the
    recorded session was hand-corrected before the portability check ran.
resolution: null
duplicate_of: null
---
At PR 47 head e658915, ChildPageRequest accepts Bound::Limit(0). child_page then returns no rows, a nonempty remainder, and next=None because there is no last emitted name. The result says truncated but has_next false and gives the caller no cursor, so a nonempty directory cannot be paged. Fix: require a positive page limit at every public boundary, matching the MetaBrowser query contract, and test zero on the Rust and Python surfaces. Review finding FDU47-R7.

## Notes

DESIGN CONFIRMED (2026-08-24 review). Mechanism verified in `child_page`: with
Limit(0), `admits(0)` is false on the first row, `more = true`, `last = None`, so
`next: more.then_some(last).flatten()` is None -- remainder present (truncated) AND
next absent (terminal). Zero rows, no continuation.

Reject zero where a continuation is the contract: `ChildPageRequest` (Rust: structured
error from the request boundary, not a clamp -- a clamp answers a question the caller
did not ask) and the Python `limit=` (ValueError naming the rule). MetaBrowser's
contract already requires every page/row bound positive, so the adapter never sends it.

Deliberately NOT rejected: `extensions=0` (an extension map with everything in `rest`
is representable and has no cursor to strand) and `depth 0` (root-only is meaningful).
The trap is specifically a paged bound, where truncation must imply resumability.
Record that distinction in the doc comment so the next bound added lands on the right
side of it.
