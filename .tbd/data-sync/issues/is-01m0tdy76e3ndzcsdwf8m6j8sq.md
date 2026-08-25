---
type: is
id: is-01m0tdy76e3ndzcsdwf8m6j8sq
title: Watch updates a cloned index instead of the opened handle
kind: bug
status: closed
priority: 0
version: 5
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
updated_at: 2026-08-24T21:39:56.849Z
closed_at: 2026-08-24T21:39:56.848Z
close_reason: |
  Shipped. `make check` green.

  THE DEFECT. `PyIndex.watch` built `IndexHandle::new(self.inner.snapshot()?)` -- a deep
  clone into a second, private index. The watcher then mutated that copy, so the object the
  caller held went stale at the first event and only `refresh()` brought it back. A server
  holding that index would serve numbers that had stopped being true, with nothing in the
  answer saying so. It also cost O(entries) in time and memory at watch start.
  `Session::report` deep-cloned a second time, per repaint -- the same
  `snapshot()`-is-not-a-read regression the read path had already been fixed for, sitting
  in `watch_session.rs`.

  THE FIX, exactly as the review proposed. `self.inner.clone()`: an `IndexHandle` is an
  `Arc<RwLock<Index>>` deriving `Clone`, so a clone is the same authority, and
  `Session::report` goes through `with_index`. The comment defending the deep clone
  ("closing the feed cannot disturb the caller's index") described a fear rather than a
  risk: dropping a handle drops a reference. `Session::index_snapshot` keeps `snapshot()`,
  correctly -- a save needs a consistent copy without holding the lock across the write.

  A DOC THAT WAS TRUE AND STOPPED BEING TRUE. `PyWatch.report` carried a paragraph telling
  callers to prefer the session's own index over the one it was opened from, citing
  `fdu-m66a`. That advice existed because of the split brain; with one authority the
  distinction is gone, and the comment now says so rather than sending readers to a
  difference that no longer exists.

  TESTS.
  - Rust: `a_session_mutates_the_handle_it_was_opened_from` holds the handle the session was
    built from, applies a filesystem mutation, waits for the change, and asserts both the
    clock and the totals moved on the handle the caller kept -- then drops the session and
    asserts the index is still usable.
  - Python smoke: after consuming a mutation on the feed, `Index.read` sees a moved clock
    and a higher file count with no `refresh()`; after `feed.close()` the index still reads.

  MUTATION-CHECKED. Restoring `IndexHandle::new(self.inner.snapshot()?)` fails the smoke
  suite -- and at an earlier line than expected, which is itself the clearest statement of
  the bug: the very first consumed event never reached the caller's index, so its clock was
  still zero.
resolution: null
duplicate_of: null
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
