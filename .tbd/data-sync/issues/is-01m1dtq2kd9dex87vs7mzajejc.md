---
type: is
id: is-01m1dtq2kd9dex87vs7mzajejc
title: "Spec: streaming performance parity without one-shot overhead"
kind: epic
status: in_progress
priority: 0
version: 25
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - correctness
dependencies: []
child_order_hints:
  - is-01m1dtqb9q9fnaqpwr5cw90j0m
  - is-01m1dtqbkxhdqfhczrbdctcaxq
  - is-01m1dtqkbyrydtq60902w1sgkr
  - is-01m1dtqrgwd4fn6ekn7dq8a6tg
  - is-01m1dtqxh815zb3zz6m3g11cx6
  - is-01m1dtr3hap1kqbkfcap66paq8
  - is-01m1dtr903vj783j9ajaxfnczf
  - is-01m1wxpmpmqycpjvgcav4daxp2
  - is-01m1wy9gswnqnwnh145m40jnfa
  - is-01m1wy9h39n1yem588vxwxanh4
  - is-01m1wykhs9r93rjprxm1q49hyj
  - is-01m1x4444ma3vw81wx5bm4k0g4
  - is-01m1x444e4rnksjs8v8p37padv
  - is-01m1x444q4jz0680n8a057r5z8
  - is-01m1x5t3acttkxkwfybncp9ssb
  - is-01m1x8a1w2qf59hps348kc3kvj
  - is-01m1xawdxr2v87f7km4jabmd8x
  - is-01m1xbp9qy40ymd7wvyaw0ckp8
created_at: 2026-09-01T06:32:43.884Z
updated_at: 2026-09-07T07:46:37.835Z
---
Restore detached one-shot performance to the pre-rewrite main control while preserving exact opened-root and public mutation semantics. Correctness fixes precede profiling and lifecycle specialization. The linked plan is the design and acceptance authority.

## Notes

Final review fixes are pushed through formal draft stack #53 (#48, #50, #51, #52). Head 64c6e61 passes full isolated make check, cross-lint and all 19 CI checks. Closed review beads cover the actual opened-state oracle, improvement-friendly allocation ceilings, architecture/spec reconciliation, stack refresh, per-artifact provenance, capability-enabled probes, default CLI scope and current profile classification. Exploratory evidence through exp-101 is retained, but historical final-binary timing remains unproved. Corrected scoped allocation checks on both nominated real trees match exact summaries/digests and stay below all allocation/reallocation/byte ceilings. fdu-0q6w now tests the profile-confirmed public ancestry preflight hotspot, with independent-model and control-pruning tests; fdu-lj4h then owns unchanged final quiet-host parity and fdu-rx0d owns final handoff. Linux H86 remains separate. Cleanup fdu-iyg0 retained source docs, evidence, active worktrees and agent logs; only disposable task-owned cache and a stale absent-checkout registration were staged in Trash. Builds now use per-checkout targets to avoid stale shared-target artifacts.
