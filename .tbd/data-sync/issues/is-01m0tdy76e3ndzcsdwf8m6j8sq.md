---
type: is
id: is-01m0tdy76e3ndzcsdwf8m6j8sq
title: Watch updates a cloned index instead of the opened handle
kind: bug
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0rw7cddvwh9vetyxkmgrvsm
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:52.269Z
updated_at: 2026-08-24T20:45:43.655Z
---
At PR 47 head e658915, PyIndex.watch creates IndexHandle::new from self.inner.snapshot. The watch updates a second full index while Index.read and rollup on the opened object remain stale, and watch open pays O(entries) copy time and memory. Session.report then snapshots that private index again for each repaint even though IndexHandle::with_index exists specifically to avoid that clone. This violates the single authoritative opened-root contract. Fix: Session receives self.inner.clone; closing the watcher drops only that Arc and the watcher registration. Report through with_index. Test that a mutation consumed from watch is visible from the original Index without refresh, close keeps the Index usable, and neither watch open nor report clones the full index. The capture-before-baseline handoff remains fdu-4o0m. Review finding FDU47-R2.

## Notes

DESIGN SETTLED (2026-08-24 review). Verified: `PyIndex.watch` builds
`IndexHandle::new(self.inner.snapshot()?)` -- a deep clone into a second authority --
and `Session::report` deep-clones AGAIN per repaint (`self.index.snapshot()`), which is
the exact `snapshot()`-is-not-a-read regression the PR body itself warns about, sitting
in watch_session.rs.

THE FIX, two lines of intent: pass `self.inner.clone()` (an Arc to the same authority;
dropping the Session drops a reference, not the index -- the existing comment's fear is
unfounded); and `Session::report` goes through `with_index(|index| report(index, ...))`.
`Session::index_snapshot` (the persist path) legitimately keeps `snapshot()` -- a save
needs a consistent copy without holding the lock across the write.

TESTS (R2's list): consuming one watch mutation makes the original `Index.read` see it
without `refresh`; closing the watch leaves the caller's index usable; a structural
counter proving neither watch open nor repaint clones the tree. The
capture-before-baseline handoff stays fdu-4o0m's.
