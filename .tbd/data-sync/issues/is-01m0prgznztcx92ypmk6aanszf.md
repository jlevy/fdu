---
type: is
id: is-01m0prgznztcx92ypmk6aanszf
title: "Python Index.refresh(path): scoped reconciliation as hint ingestion"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:55.198Z
updated_at: 2026-08-23T17:01:31.707Z
---
Expose the engine's subtree reconciliation through Index.refresh(path=...). This is the foreign-watcher hint primitive for filesystems fdu's backends cannot serve (NFS, FUSE): every mutation still flows through the one delta contract. Asserted equivalent to full refresh on the touched subtree. Resolves half of fdu-p02b's open watcher question.

## Notes

scan::reconcile_subtree (scan.rs:2851) already exists and already takes the subtree path. The work is the Python signature and the equivalence test: a scoped refresh over the touched subtree must equal a full refresh restricted to it. Composes with fdu-gav9, which moves PyIndex onto IndexHandle and switches refresh to reconcile_handle (scan.rs:2861).
