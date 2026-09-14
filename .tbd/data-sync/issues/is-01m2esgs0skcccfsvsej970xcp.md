---
type: is
id: is-01m2esgs0skcccfsvsej970xcp
title: "parfloor.c: count directories it fails to open and add a DT_UNKNOWN fallback (FLOOR-12 C side)"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-14T01:46:45.144Z
updated_at: 2026-09-14T01:46:45.144Z
---
C-side remainder of PR #49 review FLOOR-12 (fdu-qcq5, closed as fixed on the harness side only). Recorded by the fixer.

**Defects in `explorations/benchmarks/spikes/parfloor.c`** (review cites `:116-121, 133-141, 221` at 1fa2309):
1. A directory that fails to open (EACCES) is skipped without `dirs++`, while fdu and `arena_spike` count it. On any subject with one permission-denied directory, `parfloor`'s tallies therefore disagree with every other instrument.
2. The `enum` variant has no `DT_UNKNOWN` fallback. On filesystems that do not fill `d_type` (some XFS, NFS, and FUSE configurations), entries are misclassified or uncounted.
3. A root-open failure prints `dirs = 2^64-1`, an unsigned underflow.

**What the harness fix did.** In `floor.py` (bce36f2, 5d71916):
- a disagreeing `parfloor-enum` reference row is dropped with its reason instead of vetoing the subject;
- reference rows never seed the oracle;
- an unreadable subject root is refused before anything runs;
- `parfloor stat`'s unreadable-subdirectory gap is documented as a scoreboard limitation.

`parfloor.c` itself is unchanged. It is Linux-only (`SYS_getdents64`, `statx`) and was never compiled on the macOS host that did the fix.

**Consequence left open.** `parfloor stat` is the denominator of every ×floor figure. On a subject with an unreadable subdirectory its tallies still disagree with fdu's, so the subject is unusable rather than measured.

**Fix.** On a Linux host:
- count a directory `parfloor` failed to open, matching fdu and `arena_spike`;
- add a `DT_UNKNOWN` → `statx` fallback to `enum`;
- make a root-open failure exit non-zero with a message.

Compile and run it against a fixture with a mode-000 subdirectory, and on a filesystem without `d_type`. Then decide whether `floor.py`'s documented limitation can be removed.

Review: https://github.com/jlevy/fdu/pull/49#pullrequestreview-5192251516
