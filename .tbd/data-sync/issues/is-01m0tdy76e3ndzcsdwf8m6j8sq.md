---
type: is
id: is-01m0tdy76e3ndzcsdwf8m6j8sq
title: Watch updates a cloned index instead of the opened handle
kind: bug
status: open
priority: 0
version: 3
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
updated_at: 2026-08-24T17:44:16.233Z
---
At PR 47 head e658915, PyIndex.watch creates IndexHandle::new from self.inner.snapshot. The watch updates a second full index while Index.read and rollup on the opened object remain stale, and watch open pays O(entries) copy time and memory. Session.report then snapshots that private index again for each repaint even though IndexHandle::with_index exists specifically to avoid that clone. This violates the single authoritative opened-root contract. Fix: Session receives self.inner.clone; closing the watcher drops only that Arc and the watcher registration. Report through with_index. Test that a mutation consumed from watch is visible from the original Index without refresh, close keeps the Index usable, and neither watch open nor report clones the full index. The capture-before-baseline handoff remains fdu-4o0m. Review finding FDU47-R2.
