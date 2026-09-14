---
type: is
id: is-01m0rw7cddvwh9vetyxkmgrvsm
title: "Handle lifecycle: prioritize() and close() on the opened root"
kind: feature
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T03:15:03.725Z
updated_at: 2026-09-14T01:49:35.782Z
---
MetaBrowser's InventoryHandle has five operations. fdu answers three and not two:

  open(root, config)  -> fdu.open / fdu.scan                      BUILT
  read(request)       -> Index.read, one guard, version + cursor
                         + state + work                           BUILT (fdu-2ivi, qgl9)
  refresh(request)    -> Index.refresh(path)                      BUILT (fdu-fh0k)
  prioritize(request) -> nothing                                  MISSING
  close()             -> only on Watch, not on Index              MISSING

prioritize(request) "changes discovery order without changing semantics" and accepts
paths with a positive depth. That is scheduling only: the same tree, the same totals, a
different order of arrival. fdu already has the machinery -- ScanOrder is exposed and the
walk has a work queue -- so this is a hint that reorders pending work, not a new traversal
mode. The semantics-preserving property is the thing to test: the same tree prioritized
differently must produce identical final totals.

close() "cancels and joins provider work before it returns". PyWatch::close exists;
PyIndex has none, so an embedder holding a handle over a live watch has no single call
that stops everything and joins. Note the semantics depend on what background work a
handle can own, which is the session -- so this is blocked by fdu-4o0m rather than
guessable.

Both are small in code and easy to get wrong in contract. prioritize must not be able to
change an answer, and close must be idempotent and safe to call from a thread that is not
the one draining the watch.

## Notes

2026-09-13 (PR #48 review 5192314101, prior findings): fixed in its OpenedIndex form, since prioritize() and close() exist on the opened root. The review noted the caveats of LIFE-4 and LIFE-5, both since fixed on #48 (fdu-6tqw, fdu-q8p7). Candidate to close once someone confirms nothing else in this bead's text is still owed.
