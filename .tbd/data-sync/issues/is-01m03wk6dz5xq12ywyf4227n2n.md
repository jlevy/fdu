---
type: is
id: is-01m03wk6dz5xq12ywyf4227n2n
title: Warm default metadata run loses to a cold scan of the same view
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-15-fdu-cache-layers-and-defaults.md
labels:
  - performance
  - cli
dependencies: []
parent_id: is-01m03bjey08898z8t9a2vhakm1
created_at: 2026-08-15T23:37:27.733Z
updated_at: 2026-08-16T00:03:43.580Z
closed_at: 2026-08-16T00:03:43.579Z
close_reason: "Fixed by the snapshot-read cost gate: repeat fdu . fell 4808->3829 ms on the 494k-entry field-report tree, from behind dust to level with it; warm-loses-to-cold defect eliminated. Golden contracts updated deliberately; open()/watch keep the warm path."
---
Field report reproduced on macOS/APFS, /Users/levy/wrk/aisw/trading (494,031 entries,
127,915 dirs, complete, no errors). Interleaved paired medians, uncontrolled host:

  fdu default WARM (load+walk+reconcile)   4808 ms   <- what repeat `fdu .` users see
  dust 1.2.4 default                       4400 ms
  fdu default COLD (walk+build+write)      3888 ms
  fdu --cache refresh (walk+build+write)   3597 ms
  fdu --cache off (walk+build)             3547 ms
  fdu --cache only (load+query)             366 ms
  snapshot write cost (refresh - off)       ~50 ms
  snapshot read+reconcile cost (warm - off) ~850-1260 ms

FDU_COUNTERS proves warm does identical filesystem work to cold (same 127,915 dir opens,
494,032 stats) plus ~8k extra syscalls reading the 38 MB snapshot. The read+reconcile
buys nothing: revalidation walks everything anyway, and the write it could save costs
~50 ms and is skipped-on-unchanged regardless.

This is the design-principles defect clause verbatim: "A warm path that loses to a cold
scan of the same view is a defect, not a trade-off." It is also the mechanism behind the
original field report (fdu slower than dust): only the WARM default loses to dust; cold
fdu and every matched-work arm beat it.

Fix direction, per the cache-layers spec cost model: extend Phase 1's cost rule from plan
selection to snapshot READ participation. A one-shot FullIndex metadata query (no
analysis) under auto/read-only should not load the snapshot: scan fresh, persist
write-behind as today (auto), skip the write (read-only). only/refresh keep their
contracts; watch/open() keep their index promise; analysis keeps loading because the
content sidecar pays (~3.7x measured).

Contract changes this implies, to make deliberately: cli-axes "The Next Run Revalidates
It" golden flips to cold_scan for the tree view; watch_persistence warm-start priming
must move off one-shot CLI metadata runs.

## Notes

FIXED by the read gate (commit dc99bbb on PR #31's branch). plan_report now derives
read_snapshot alongside retained_state; one-shot metadata queries under auto/read-only
scan fresh (auto still persists write-behind); only/analysis still read; open() and watch
keep the warm path.

Post-fix matrix, same tree (494k entries), 7 interleaved trials, uncontrolled host:

  [tree class]                          [scalar class]
  fdu . cold        3761 ms  fastest    fdu --view summary  3145 ms  fastest
  fdu . repeat      3829 ms  (+2%)      dumac               3250 ms  (+3%)
  dust              3884 ms  (+3%)      diskus              3620 ms  (+15%)
  dua               4033 ms  (+7%)      du -sk (BSD)       12653 ms  (+302%)
                                        gdu -sb (GNU du)   15373 ms  (+389%)

Repeat `fdu .` fell 4808 -> 3829 ms (-20%), from behind dust to level/ahead. The 1-3%
margins over dust/dumac are ties on an uncontrolled host, not claims (fdu-ow8y
discipline); the structural defect -- warm default slower than cold scan of the same
view -- is gone. Raw trials in session scratchpad matrix-trading.json /
results-rustup.json / results-appsupport.json.

Harness gap: no existing perf job covers the default one-shot CLI plan, so this is
evidenced by interleaved paired runs rather than a ledger artifact -- filed fdu-ao6p.
