---
type: is
id: is-01kzxqxeqjqqzhr1m50xb0x2mk
title: Audit warm versus cold filesystem-cache benchmarking
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - performance
  - methodology
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T14:20:14.445Z
updated_at: 2026-08-23T02:11:33.033Z
closed_at: 2026-08-13T14:34:04.670Z
close_reason: Warm/cold cache-state audit completed; harness evidence and claim boundary are tested, all governing docs and the definitive manifest are synchronized, and distinct Linux and macOS controlled-cold follow-ups remain open.
---
Verify whether current FDU product and experiment comparisons intentionally measure warm-steady operating-system filesystem cache; evaluate Andrew Healey/dumac and diskus arguments from primary sources; distinguish FDU persisted cache from OS page/metadata cache; decide which warm and cold regimes answer which product questions; and update the benchmark protocol, research/spec/report, and harness checks as warranted.

## Notes

Audited the definitive 901,963-entry M1/APFS comparison and primary sources. The run performs one complete independent fingerprint plus three full-tree warmups per tool before 12 adjacent timed pairs. This supports a repeated-workload warm-steady claim, not full metadata residency; the host kern.maxvnodes target (263,168) is below the entry count and is diagnostic rather than a hit-ratio measure. Healey's warm result is valid as labeled, but no published cold samples establish his correlation claim or a cold effect size. Diskus's Linux example retains the winner while changing the relative gap from 10.18x cold to 2.20x warm. The comparator now rejects zero-warmup warm-steady runs, records explicit cache evidence, renders the claim boundary, and records Darwin max_vnodes. README, guide, manifest, report, white paper, research note, and spec are synchronized. Linux dual-regime work remains fdu-nffc; dedicated macOS protocol is fdu-rjqx.
