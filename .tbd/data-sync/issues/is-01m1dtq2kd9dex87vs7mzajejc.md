---
type: is
id: is-01m1dtq2kd9dex87vs7mzajejc
title: "Spec: streaming performance parity without one-shot overhead"
kind: epic
status: in_progress
priority: 0
version: 26
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
updated_at: 2026-09-07T08:48:14.746Z
---
Restore detached one-shot performance to the pre-rewrite main control while preserving exact opened-root and public mutation semantics. Correctness fixes precede profiling and lifecycle specialization. The linked plan is the design and acceptance authority.

## Notes

Head afbb2ee (engine ad52469) is pushed in formal draft stack #53 (#48, #50, #51, #52), with a clean primary worktree, full isolated make check, cross-lint and all 19 CI checks passing. Review fixes cover the actual opened-state oracle, improvement-friendly allocation ceilings, per-artifact provenance, capability-enabled probes, default CLI scope, profile classification and architecture/spec reconciliation. Exp-102 records the profile-confirmed private public-mutation lookup change, with large/repeated-batch wall improvements of 49.78%/39.75% and exact oracles in an uncontrolled exploratory screen; allocated bytes rise 2.39%/4.95%, and no final-parity claim is made. Final allocation/oracle checks on both nominated real trees pass. fdu-0q6w and fdu-lj4h await quiet-host confirmation and unchanged final historical/structural/opened gates; latest preflight refused at 26.9% busy and a later host snapshot was 39.86%. fdu-rx0d remains blocked on those gates; Linux H86 stays separate. Cleanup fdu-iyg0 is complete: 6.1 GiB of previously staged disposable build output plus 88 KiB stale absent-checkout metadata were verified in Trash, not emptied. No active worktree, source branch/document, agent/Codex log or unique evidence was removed. Final audit found about 16 GiB physical free; docs/evidence total only 4.9 MiB and remain needed.
