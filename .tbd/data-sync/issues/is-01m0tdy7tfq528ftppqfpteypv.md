---
type: is
id: is-01m0tdy7tfq528ftppqfpteypv
title: Resume cursor can skip deltas and cannot reject another session
kind: bug
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0tdy8b6h17fqk7mqge56svh
  - type: blocks
    target: is-01m0tdy8swsdre8d15s96wx4km
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:52.910Z
updated_at: 2026-08-24T17:44:16.238Z
---
At PR 47 head e658915, IndexHandle::since captures deltas under one guard but PyIndex.since samples the returned clock under a later guard. A commit between those calls returns clock N+1 with operations only through N, so resuming at N+1 permanently skips the change. Clock also has no opened-session identity, so a Last-Event-ID from a prior process or root can be greater than the current clock and is accepted as an empty nontruncated replay. Fix: Since carries the terminal clock captured with its journal slice; define a cursor as opened-session identity plus sequence and reset or reject mismatched and future cursors. Test the forced interleaving and process or root replacement. fdu-jxs0 remains the separate trust-transition gap and fdu-4o0m remains the no-gap session handoff. Review finding FDU47-R3.
