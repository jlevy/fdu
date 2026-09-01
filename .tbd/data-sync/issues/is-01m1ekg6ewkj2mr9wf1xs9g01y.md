---
type: is
id: is-01m1ekg6ewkj2mr9wf1xs9g01y
title: Differentially profile residual one-shot baseline mutation work
kind: task
status: closed
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - profiling
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
hold: null
hold_until: null
created_at: 2026-09-01T13:45:52.859Z
updated_at: 2026-09-01T14:10:14.681Z
started_at: 2026-09-01T13:45:55.744Z
closed_at: 2026-09-01T14:10:14.679Z
close_reason: "H106 ruled out revision bookkeeping by source comparison and causal publication by a 12-pair producer-only diagnostic: component +0.68% (95% CI -1.04% to +2.13%). Raw counter-disabled profiles also showed the standard profile command is now distorted by per-batch elapsed timers, so the next work is reproducible counter-disabled attribution before any larger structural experiment."
resolution: canceled
duplicate_of: null
---
After H104 and H105 ruled out prepared-batch construction, control projection, causal publication frequency, and reducer-call count as primary wall costs, compare current and pre-rewrite profiles and instrument corpus-scale per-entry mutation mechanisms. In particular, measure revision-clock and parent children-revision bookkeeping before preregistering any specialization. Do not change semantics until evidence identifies a mechanism capable of explaining at least 3% default-tree wall time.

## Notes

H106: source comparison ruled out revision counters because both b75bf85 and current bump identical revisions. Matched raw 8-second profiles with FDU_COUNTERS=0 sampled the current scan_into_index consumer 1,182 times versus 822 pre-rewrite; 345 current samples were scanner preparation. H104 already removed preparation without moving wall, so the bounded next diagnostic suppresses only the pre-claim causal flush in producer-only public scan, retains exact validation on the causal one-shot path, and proceeds to an unordered private builder only if cold-scan-producer component improves >=3% with CI below zero and batch count approaches the configured minimum.
