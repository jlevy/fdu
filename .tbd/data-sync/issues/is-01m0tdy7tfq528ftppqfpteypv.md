---
type: is
id: is-01m0tdy7tfq528ftppqfpteypv
title: Resume cursor can skip deltas and cannot reject another session
kind: bug
status: open
priority: 0
version: 5
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
updated_at: 2026-08-24T20:45:44.003Z
---
At PR 47 head e658915, IndexHandle::since captures deltas under one guard but PyIndex.since samples the returned clock under a later guard. A commit between those calls returns clock N+1 with operations only through N, so resuming at N+1 permanently skips the change. Clock also has no opened-session identity, so a Last-Event-ID from a prior process or root can be greater than the current clock and is accepted as an empty nontruncated replay. Fix: Since carries the terminal clock captured with its journal slice; define a cursor as opened-session identity plus sequence and reset or reject mismatched and future cursors. Test the forced interleaving and process or root replacement. fdu-jxs0 remains the separate trust-transition gap and fdu-4o0m remains the no-gap session handoff. Review finding FDU47-R3.

## Notes

DESIGN SETTLED (2026-08-24 review). Verified: `PyIndex.since` takes the journal slice
under one guard (line ~1001) and samples `self.inner.clock()` under a second (~1029);
core `Since { deltas, truncated }` carries no terminal clock, so the atomicity fix
belongs in core, not in the binding. And the cursor is a bare u64: a Last-Event-ID from
a prior process compares numerically against an unrelated clock, and a future cursor
returns empty/non-truncated -- an unrelated position reads as current.

THE TYPE. `Cursor { session: SessionId, clock: Clock }` in fdu-core. `SessionId` is a
u64 minted per Index construction (SystemTime nanos mixed with a process-global counter
through the existing FNV -- no new dependency, no randomness API), identifying the
opened in-memory session; a snapshot reload is a NEW session by definition. This is the
same shape MetaBrowser's contract already specifies: "A ChangeCursor contains the same
session and sequence at the read boundary."

CORE CHANGES. `Since` gains `cursor: Cursor` captured under the SAME guard as the slice;
`Index::since(cursor)` rejects a foreign session (structured error naming both) and a
future clock within the right session (corruption or a rolled-back file: error, never
silence); truncation stays what it is. `IndexHandle::read`'s returned clock becomes the
same Cursor type -- this is the token fdu-91ru pins reads with, so land this bead first
and 91ru reuses the type.

BINDING. `since` returns {ops, cursor: {session, clock}, truncated}; the SSE example
threads the whole token through Last-Event-ID.

TESTS. A forced interleaving (commit between slice and clock -- impossible after, prove
with a paired-guard structural assertion); reopen/process-replacement returning a
session-mismatch error; future-cursor error; the empty index round-trips its own cursor.
