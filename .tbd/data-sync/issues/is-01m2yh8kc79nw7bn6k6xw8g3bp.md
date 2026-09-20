---
type: is
id: is-01m2yh8kc79nw7bn6k6xw8g3bp
title: Address remaining 0.1 correctness blockers after performance review
kind: task
status: in_progress
priority: 1
version: 10
delegate: codex@spud10
labels: []
dependencies: []
child_order_hints:
  - is-01m2yhjb6r67n23jera0t03ebv
  - is-01m2yhjbkazg1zds9x9janfd5m
  - is-01m2yhjbz0hfj930q39b01rmap
  - is-01m2yhjcawjwehkfdxrejrhe64
  - is-01m2yhtwjg6j0vjyjf1v9sehd5
  - is-01m2ymchsj7j6ayg8j46kc1v00
hold: null
hold_until: null
created_at: 2026-09-20T04:30:19.526Z
updated_at: 2026-09-20T05:24:54.704Z
started_at: 2026-09-20T04:31:31.377Z
---
Audit remaining release correctness against the current #92 stack, reconcile stale or already-fixed beads, implement confirmed defects in coherent slices with subagents, review changes, validate and publish PRs. Preserve unrelated directory-rollup work and existing core-model ownership. Final candidate verification remains distinct from publishing.

## Notes

Audit at PR92 head 937f9445: active delegated slices are output fidelity (aow4/3ex4/5at8/7bfu/joqd/lkuj/up8j), per-analyzer content and retry semantics (ogg0/xras/ibu1/am8r/ufjb/ky5m/ugom), provenance and watch lifecycle (c7gc/rjv3/wuip/8x1d/szll/tngk/0ywm/aach/jott). Remaining confirmed correctness queued: Windows validity fingerprint 6act; bounded content analysis memory b2qz; controls-off projection route consistency ay3c/xwmv/qxgh/brun/u767; Python raw-path decoding 8ihl; watch ignored flag refresh 4239. gija wider-sidecar mismatch and snv3 analyzed watch refusal appear addressed at this head, but retain closure until regression evidence and related model integration are reviewed. Final release-candidate verification tyvq remains distinct from these implementation fixes. No merge or release publishing authorized.

2026-09-20 implementation review: content-model commits 9449aef1 + 7711150f pass the full core all-features library suite outside the macOS sandbox (756 passed, 0 failed, 1 explicitly ignored manual performance test). Machine-output first slice 7bd3a965 passes 31 report tests, 6 emit/scalar tests, strict YAML 1.1/1.2 parsing and 21 cross-format cases plus the 121-string corpus. Integration/schema7/Python and final make check remain pending; these are implementation milestones, not release approval. Windows6act and controls-off projection ay3c/xwmv/qxgh/brun/u767 are assigned. Root review found and requested a second capture barrier for sticky overflow at watcher handoff and partial-acceptance enforcement after the mandatory reconciliation. Exact Markdown memory remains unresolved under b2qz.
