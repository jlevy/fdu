---
type: is
id: is-01m0tdy7tfq528ftppqfpteypv
title: Resume cursor can skip deltas and cannot reject another session
kind: bug
status: closed
priority: 0
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
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
updated_at: 2026-08-26T07:01:50.826Z
closed_at: 2026-08-24T22:04:05.358Z
close_reason: |
  Shipped. `make check` green. This is the third and last of the P0 authority defects.

  TWO DEFECTS, one type. `PyIndex.since` took the journal slice under one guard and then
  sampled `self.inner.clock()` under a second, so a commit landing between them made the
  reported position one ahead of the deltas returned -- and resuming from it skipped that
  commit permanently, silently, only under concurrency. And the cursor was a bare integer: a
  `Last-Event-ID` from a prior process or a prior open compares as an ordinary position in
  the new index, so `since` answered with an empty, untruncated set. "Nothing changed", about
  a place this index has never been.

  `Cursor { session: SessionId, clock: Clock }` fixes both, and it belongs in core rather
  than the binding because the atomicity half does. `Since` now carries its terminal cursor,
  captured in the same expression as the slice, so a two-call interleaving is not merely
  unlikely but unrepresentable. `SessionId::mint()` folds wall-clock nanoseconds and a
  process-global counter through the crate's own FNV-1a -- no new dependency, no randomness
  API. Zero is reserved and never minted, so `Cursor::default()` matches nothing and is
  refused rather than read as the start of whichever index it reaches. This is the same
  shape MetaBrowser's provider contract already specifies for its own cursors.

  `Index::since` now returns `Result`: a foreign session and a future clock both raise
  `Error::CursorNotOfThisSession`, carrying the token, the serving session, and how far it
  has advanced. Refusing is the whole point -- an empty answer and a refusal are different
  facts, and only one of them is safe to believe.

  SURFACES. `Index.cursor()` and `Index.since(cursor)` on both the native and typed Python
  layers; a `Cursor` dataclass exported from `fdu`; `ChangeSet.clock` becomes
  `ChangeSet.cursor`. A bare integer is rejected with a message saying why, rather than
  being read as this session -- accepting it would put callers back where the session field
  was added to rescue them.

  THE SSE EXAMPLE is the consumer this was written for, so it moved with the type: the
  event id is now `session-clock`, `parse_last_event_id` rejects session 0, and an
  unrecognized token yields a `resync` frame with reason `unknown-session`. That branch did
  not exist before because the failure it handles was invisible.

  TESTS.
  - `a_resume_position_never_runs_ahead_of_the_deltas_it_was_captured_with`: the cursor
    equals the last delta's clock, and resuming from it returns nothing.
  - `a_cursor_from_another_session_or_from_the_future_is_refused`: two independently opened
    indexes mint different identities; both bad shapes and the default token are refused.
  - Python: both smoke suites assert the cursor comes back with the ops and that a foreign
    session raises; the SSE test covers the new id format and the session-0 rejection.
resolution: null
duplicate_of: null
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
