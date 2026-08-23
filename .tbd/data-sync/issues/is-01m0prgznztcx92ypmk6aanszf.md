---
type: is
id: is-01m0prgznztcx92ypmk6aanszf
title: "Python Index.refresh(path): scoped reconciliation as hint ingestion"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:55.198Z
updated_at: 2026-08-23T18:18:37.152Z
closed_at: 2026-08-23T18:18:37.152Z
close_reason: "Implemented and gated. Index.refresh(path) scopes the sweep to one subtree via reconcile_subtree_handle, which already existed and already takes the IndexHandle that fdu-gav9 moved PyIndex onto — the two beads composed better than the map predicted. This is the hint-ingestion primitive fdu-p02b left open: a caller running its own watcher for a filesystem this build's backends cannot serve pushes hints through the engine's one delta contract rather than a second path into the index. Tested in both directions: a change inside the scope is observed and reaches the same state a whole-tree refresh would, and a change outside it is not observed by a refresh scoped elsewhere but is still found by the whole-tree sweep — a scoped refresh that quietly walked everything would pass the first assertion alone. make check green, parity holds."
resolution: null
duplicate_of: null
---
Expose the engine's subtree reconciliation through Index.refresh(path=...). This is the foreign-watcher hint primitive for filesystems fdu's backends cannot serve (NFS, FUSE): every mutation still flows through the one delta contract. Asserted equivalent to full refresh on the touched subtree. Resolves half of fdu-p02b's open watcher question.

## Notes

scan::reconcile_subtree (scan.rs:2851) already exists and already takes the subtree path. The work is the Python signature and the equivalence test: a scoped refresh over the touched subtree must equal a full refresh restricted to it. Composes with fdu-gav9, which moves PyIndex onto IndexHandle and switches refresh to reconcile_handle (scan.rs:2861).
