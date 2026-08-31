---
type: is
id: is-01m18sm4xhbs9y0cgb1wpakr95
title: fdu peak memory runs ~1.5x dust on every macOS tree measured
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - performance
  - macos
  - memory
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:37:27.215Z
updated_at: 2026-08-31T04:15:22.118Z
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

## Notes

ATTRIBUTED. The memory gap is the retained index forced by the default VIEW. It is not the cache, and it is not the snapshot write.

Measured peak RSS, rustup-toolchains, 3 runs each (indicative, hand-run):

  cache off + --view summary + json : 13-17 MiB
  cache off + --view summary (text) : 13-14 MiB
  cache off, DEFAULT tree view      : 63-68 MiB   <-- the jump is here
  bare default (fdu ROOT)           : 75-77 MiB

Rows 2 and 3 share --cache off and differ only in the view: 14 MiB -> 66 MiB. The cache policy adds ~9 MiB on top of that (66 -> 75), roughly an eighth of the gap.

crates/fdu-core/src/execution.rs plan_report() says why:

  summary_is_sufficient = views == [Summary] && selection.is_unfiltered()
  policy_requires_index = Only => true; Refresh => cache_path.is_some();
                          Off | Auto | ReadOnly => false
  retained_state = if !policy_requires_index && !analysis_requested && summary_is_sufficient
                   { Summary } else { FullIndex }

--cache off does not avoid the index; it only PERMITS the Summary tier. What forces FullIndex is any view other than a bare unfiltered summary. The default tree view therefore retains one node per entry: ~60 MiB over the summary tier for 119,368 entries, about 550 bytes per retained entry.

Consequence for the fix: reducing this is about the tree view's retention, not about cache policy. Either the depth-2 default tree is served from a bounded structure rather than a full index, or per-entry retained size comes down. Control state built for every scan (fdu-etfj) sits inside that per-entry cost and is the cheapest part to remove first.

Wall time is NOT part of this deficit - see the correction in fdu-zibs. fdu default is faster than dust on this subject.
