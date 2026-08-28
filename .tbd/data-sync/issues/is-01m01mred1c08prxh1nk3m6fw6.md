---
type: is
id: is-01m01mred1c08prxh1nk3m6fw6
title: Probe --no-oracle mode and engine-scoped counters
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-15T02:42:02.273Z
updated_at: 2026-08-28T15:35:48.237Z
---
Phase 0 instrument from the campaign-2 plan (cited there as fdu-9ydj, which was a duplicate and is closed). A --no-oracle probe mode and engine-phase counter scoping so attribution runs stop counting the harness: the oracle is ~39% of probe instructions and 46% of its allocation events. Platform-neutral; runnable on macOS.

## Notes

2026-08-28 (Linux session, from the fdu-33ri floor scoreboard): the oracle's share is larger than recorded, on the index tier specifically.

This bead records the oracle at ~39% of probe instructions and 46% of its allocation events. Measured as wall time on a 4 vCPU Linux container over /usr (75,976 entries), 30 trials interleaved: the cold-scan-index job's spawn wall exceeds its own component_ns by 149 ms against a 110 ms component. So 57% of a spawn-timed index-tier number is not engine work -- the probe computes its engine-digest oracle over the whole retained index before exiting.

The aggregate tier does not have this problem: its harness overhead is 2.1 ms against a 62 ms component (3%), because the tallies oracle is five integers rather than a multiset hash over every entry.

That asymmetry is the argument for --no-oracle being a correctness item rather than a tidiness one: any index-tier comparison timed around a process rather than inside one is measuring the oracle about as much as the engine, and the two tiers are not comparably contaminated, so a cross-tier reading is skewed as well.

floor.py records harness_overhead_ns (spawn wall minus the instrument's own timer) per instrument for exactly this reason, and uses the internal timer as its primary metric.

Corroborated by callgrind the same session (instruction counts, so immune to the container's noise):
valgrind --tool=callgrind on perf_probe scan-index --root /usr/include, profiling build, 90,639,628 Ir total.

  FLAT view:                                    CALLER TREE (--tree=caller):
    16.20% Sha256::compress (perf_probe.rs)       43.17% < Sha256::digest (9,332x)
    13.18% Sha256::compress (core/intrinsics)
     9.51% Sha256::compress (core/num)
     3.13% Sha256::compress (core/cmp)

The flat profile splits one cause across four source attributions, none alarming alone. The caller tree collapses them into a single caller at 43.17% of the whole profile. That is the probe's engine-digest oracle, not the engine.

Two independent instruments now agree: 57% of spawn wall and 43% of instructions on the index tier are the oracle. The 31.9% in the loop guide underestimates this tier.

The aggregate tier is NOT comparably affected (2.1 ms overhead against a 62 ms component, ~3%) because the tallies oracle is five integers rather than a multiset hash over every entry. The two tiers are unequally contaminated, so a cross-tier reading from spawn-timed numbers is skewed rather than uniformly inflated -- which is an argument for --no-oracle beyond just making index numbers cleaner.
