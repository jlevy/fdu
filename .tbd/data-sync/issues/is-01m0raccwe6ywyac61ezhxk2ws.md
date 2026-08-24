---
type: is
id: is-01m0raccwe6ywyac61ezhxk2ws
title: Per-result work counters on every query result
kind: feature
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T22:03:13.678Z
updated_at: 2026-08-23T23:45:49.019Z
closed_at: 2026-08-23T23:45:49.019Z
close_reason: |-
  Every bundled read now carries a Work record beside its answer.

  FIELDS: entries_visited, dirs_visited, rows, tally_rows, name_bytes, lock_wait_ns,
  wall_ns. On ReadBundle (and Python Bundle.work) rather than on each projection, because
  the projections shared one guard and one wall clock -- attributing a lock wait to one of
  three things that waited together would be inventing a number. It is also why measurement
  rides with the bundled read rather than with every accessor: the bundled read is what an
  interactive client serves from.

  entries_visited IS A MEASUREMENT, NOT A RESTATEMENT. Index::lookup gained a sink-typed
  sibling, lookup_visiting<V: Visits>, and the read paths pass Work while everything else
  passes a zero-sized Uncounted whose methods are empty. One body serves both, so the
  counted and uncounted walks cannot drift, and the apply path -- where lookup is hot --
  keeps no counter after monomorphisation. That was the point: an unconditional counter on
  lookup would have been a throughput change to justify by measurement rather than an
  instrument.

  tally_rows reports what a roll-up EXAMINED, not what it returned. Bounding extension rows
  still ranks every tally to decide which survive, so a result that looks perfectly bounded
  can be doing work that is not -- and a counter reporting the bound back would have hidden
  exactly that. There is a test for it.

  TWO FIELDS DELIBERATELY ABSENT, and the doc says why rather than leaving a gap:
  - CPU time. A read on a maintained index does no I/O, so wall time is CPU plus guard
    wait, and lock_wait_ns already separates them. Sampling a thread clock means a
    platform-gated syscall per read to restate what these two carry.
  - Bytes across the binding. The engine cannot see a binding and a binding can only
    estimate its own serialisation. name_bytes is the one term that grows without bound and
    it is exact; the rest is a fixed per-row schema that rows and tally_rows multiply.

  TESTS. Four in the engine plus assertions in both Python smoke suites. The load-bearing
  one builds 200 files under a three-deep path and pins that a roll-up read visits 5
  entries: mutation-checked by making the read walk its children, which takes it to 205
  while the answer stays byte-identical -- the regression no assertion on the result can
  catch. The smoke tests pin the exact decomposition (listing 3 + totals 1 + rollup 2 +
  absent path 1) rather than a range, so an arithmetic change has to be looked at.

  Deliberately NOT on Report: the CLI's report path already has a golden-visible cost
  oracle through FDU_COUNTERS, and putting wall times in report output would put
  nondeterminism in goldens.

  make check passes.
resolution: null
duplicate_of: null
---
Entries visited, directories visited, rows returned, lock wait, query wall and CPU, and bytes copied across the binding, reported beside each result as execution telemetry rather than semantic payload. Converts 'no hidden O(index) pass' from a review principle into an assertable contract: a frequent read must be proportional to its output or to a maintained index, and a counter makes a regression visible. Feeds the client's own serving benchmark.
