---
type: is
id: is-01m18sm4xhbs9y0cgb1wpakr95
title: fdu peak memory runs ~1.5x dust on every macOS tree measured
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - performance
  - macos
  - memory
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:37:27.215Z
updated_at: 2026-08-30T07:37:27.215Z
---
Ad-hoc warm measurements on this host, fdu from main vs dust 1.2.4:

| Tree | fdu time | dust time | fdu peak RSS | dust peak RSS |
|---|---|---|---|---|
| ~/wrk | 21.2 s | 27.5 s | 1.41 GiB | 887 MiB |
| ~ | 37.8 s | 87.3 s | 2.39 GiB | 1.59 GiB |
| ~/Library | 35.4 s | 29.7 s | 359 MiB | 218 MiB |

Wall time is NOT the deficit: warm, fdu is faster on both large trees. Memory is, consistently, by 1.5-1.6x. That is the plausible mechanism behind the field SIGKILL on ~/Library (fdu-6o5o): the reporter's host was at 95-99% disk and under pressure, and the process with the larger peak dies first.

Caveats to settle before this is claim-grade:
- These are single runs, not paired/interleaved, and were taken with ad-hoc /usr/bin/time rather than the project harness.
- The dust runs included its progress spinner; the harness invokes dust with --no-progress, so these dust times likely OVERSTATE dust's cost. Memory is less affected but re-measure anyway.
- Re-run through 'make perf-compare-tools' and record via 'make perf-record' before any figure is published.

Some of the gap is structural and expected: fdu retains a full incremental index built for delta, refresh, watch, and snapshot; dust builds a throwaway tree of path/size/children and exits. The question this bead answers is how much of the 1.5x is that contract and how much is unbudgeted retention that a roll-up never uses - which overlaps fdu-etfj, since control state is currently built for every scan.

Acceptance: peak RSS on nominated macOS real trees is measured through the harness, paired and interleaved; the portion attributable to the retained index is separated from the portion that is incidental; any reducible part has a named owner or a recorded decision to keep it.
