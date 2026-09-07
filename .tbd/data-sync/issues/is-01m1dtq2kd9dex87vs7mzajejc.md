---
type: is
id: is-01m1dtq2kd9dex87vs7mzajejc
title: "Spec: streaming performance parity without one-shot overhead"
kind: epic
status: in_progress
priority: 0
version: 24
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
updated_at: 2026-09-07T07:18:00.700Z
---
Restore detached one-shot performance to the pre-rewrite main control while preserving exact opened-root and public mutation semantics. Correctness fixes precede profiling and lifecycle specialization. The linked plan is the design and acceptance authority.

## Notes

Final review at 5d7b86f: formal draft PR #52 remains in GitHub stack #53. Exploratory evidence is current through exp-101: compact detached topology with inline entries improves default-tree wall 7.70% and RSS 37.79%, cold wall 5.87% and RSS 45.03% versus its immediate layout control. These are not historical parity or final-binary claims. The old exp-099 17-22% RSS checkpoint is superseded for representation decisions. Review reproduced an opened probe oracle bug and an allocation guard that rejects improvements; fdu-9o4u and fdu-dtb6 now have red-green fixes. fdu-b49n reconciles the final +3% paired CI upper bound and architecture. fdu-qoro incorporates PR #48 paging through the formal stack; fdu-lj4h then measures exact final binaries on both real subjects; fdu-rx0d owns final gates and CI. Linux H86 remains separate. Disk audit fdu-iyg0 is closed: documents/evidence retained, no active worktrees or logs trashed, one shared task build target used.
