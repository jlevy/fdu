---
type: is
id: is-01m0raccjvpde63hx884rkmq5d
title: Scalar paged child rows with remainder, no extension-map copy
kind: feature
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T22:03:13.370Z
updated_at: 2026-08-26T07:01:50.826Z
closed_at: 2026-08-24T00:49:20.033Z
close_reason: |
  The listing and the breakdown are separate questions and now cost separately.

  WHAT WAS TRUE: IndexHandle::children built an owned RollUp per directory child, so a
  directory of a thousand children cloned a thousand BTreeMaps to render a thousand size
  columns, and there was no way to ask for fewer rows than the directory held.

  WHAT LANDS:
  - ChildSnapshot.rollup: Option<RollUp> becomes ChildSnapshot.totals:
    Option<RollUpScalars>. RollUpScalars was already there as a crate-internal view; it is
    now public, Copy, and allocation-free, so a row physically cannot carry a map.
  - IndexHandle::children_page(path, &ChildPageRequest) -> Option<ChildPage>, with an
    explicit row Bound and an `after` cursor. children() is the unbounded case of it.
  - ChildPage carries rows, a remainder, and a next cursor.
  - The extension breakdown is rollup_bounded() for the one directory being inspected,
    which is where a consumer actually wants it.
  - ReadRequest gains children_page, so the bundled read pages too and a listing plus its
    header is still one crossing and one guard.
  - Python: Index.children(path, after=, limit=) and Index.read(..., after=, limit=)
    return a typed ChildPage with .truncated and .has_next.

  THE CURSOR IS A NAME, NOT AN OFFSET. Children are name-ordered, and a directory that
  gains or loses an entry between pages shifts every offset after it: an offset cursor
  silently repeats or skips rows. Resuming from a name is a BTreeMap range seek.

  WORK PROPORTIONAL TO OUTPUT, ON EVERY PAGE. The remainder is the directory's own roll-up
  minus what the page emitted, and the withheld row count is the directory's width minus
  the rows returned -- both read off maintained state, so no withheld child is touched. It
  is the complement of THIS page rather than of everything delivered so far, which is what
  keeps it exact without a stateful cursor, and "showing 50 of 812" is the sentence a
  listing wants. Whether more pages remain is `next`, decided in O(1) by the loop stopping
  on the first child past the bound.

  TESTS. Six in the engine and one in the Python smoke suite. The load-bearing one replays
  the partition property at every bound from 0 to 7: rows plus remainder equal the
  directory exactly. Mutation-checked -- dropping the +1 from a directory's own dirs
  contribution, making the cursor inclusive, and an off-by-one in the withheld count are
  each caught. One test pins that the last page reports a remainder and no cursor, which
  is where a consumer that conflated the two would loop forever.

  Closes the spec's open question "does children() need its own bound" as yes; the spec is
  updated.

  make check passes.

  NOT EVERYTHING IN THE BEAD LANDED. The description asks rows to carry "scalar directory
  facts, classification identity, tags, and provenance". Tags did not land and cannot yet:
  there is no tag plane to read one from until fdu-mvt3 builds it, and that is blocked on
  metabrowser confirming it wants the hidden plane plus the ignore crate clearing the
  14-day cool-off. Everything else in the description is built. When the planes land,
  adding a tag field to ChildSnapshot is a small follow-on inside fdu-7rwf, which already
  owns the per-plane Child values -- so this is recorded rather than re-opened.
resolution: null
duplicate_of: null
---
children() clones every child and each directory child's complete by_ext map (verified: IndexHandle::children builds an owned RollUp per directory child over an unbounded BTreeMap). Split the listing from the breakdown: child rows carry scalar directory facts, classification identity, tags, and provenance, with an explicit row bound, a page cursor, and a stated remainder. The extension breakdown becomes its own bounded rollup projection requested only for the directory being inspected. Closes the spec's open question 'does children() need its own bound' as yes. Work proportional to visible output, never one FFI call per child nor an unbounded clone.
