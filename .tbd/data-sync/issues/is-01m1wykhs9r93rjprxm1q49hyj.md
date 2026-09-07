---
type: is
id: is-01m1wykhs9r93rjprxm1q49hyj
title: Reconcile parity criteria and final architecture documentation
kind: task
status: closed
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - review
  - documentation
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
  - type: blocks
    target: is-01m1x444e4rnksjs8v8p37padv
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T03:29:19.143Z
updated_at: 2026-09-07T05:48:55.154Z
closed_at: 2026-09-07T05:48:55.154Z
close_reason: Implemented in 1aa9ba5; red-green regressions, full isolated make check and cross-lint passed. All 19 CI checks passed in run 34087670872, including macOS/Linux/Windows probe tests. Spec and PR body reconciled; final performance proof remains separate.
resolution: null
duplicate_of: null
---
Final PR 52 review R3: the acceptance section requires median at most 1.03 and CI upper at most +5%, while later checkpoints and PR body treat +3% CI as the final gate. Pick one documented contract before running the final-binary historical parity comparison on both real subjects. Update the PR body and epic checkpoint from exp099 to exp101, clearly separating exploratory improvement versus an intermediate control from historical parity. Update the durable engine architecture description of shared commit paths to explicitly describe private historyless detached initialization and promotion while preserving the public exact mutation boundary. Avoid prescribing a particular storage representation or introducing backend indirection without a concrete second implementation.

## Notes

The existing spec now maps review fixes, formal stack refresh, final paired comparisons, and the handoff gate. One-shot wall and component medians and paired 95% upper bounds use +3%, consistent with the prior strict checkpoints and performance-loop noninferiority rule; this is fixed before new measurements. Durable architecture explicitly separates private historyless initialization from exact observable mutations and describes local child-container promotion as replaceable private storage. Existing source documents are reused; no new durable document added. New provenance blocker fdu-jsbz must also precede final measurement. Full gate/PR update/CI pending.
