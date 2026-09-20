---
type: is
id: is-01m2yh8kc79nw7bn6k6xw8g3bp
title: Address remaining 0.1 correctness blockers after performance review
kind: task
status: in_progress
priority: 1
version: 8
delegate: codex@spud10
labels: []
dependencies: []
child_order_hints:
  - is-01m2yhjb6r67n23jera0t03ebv
  - is-01m2yhjbkazg1zds9x9janfd5m
  - is-01m2yhjbz0hfj930q39b01rmap
  - is-01m2yhjcawjwehkfdxrejrhe64
  - is-01m2yhtwjg6j0vjyjf1v9sehd5
hold: null
hold_until: null
created_at: 2026-09-20T04:30:19.526Z
updated_at: 2026-09-20T04:42:11.792Z
started_at: 2026-09-20T04:31:31.377Z
---
Audit remaining release correctness against the current #92 stack, reconcile stale or already-fixed beads, implement confirmed defects in coherent slices with subagents, review changes, validate and publish PRs. Preserve unrelated directory-rollup work and existing core-model ownership. Final candidate verification remains distinct from publishing.

## Notes

Audit at PR92 head 937f9445: active delegated slices are output fidelity (aow4/3ex4/5at8/7bfu/joqd/lkuj/up8j), per-analyzer content and retry semantics (ogg0/xras/ibu1/am8r/ufjb/ky5m/ugom), provenance and watch lifecycle (c7gc/rjv3/wuip/8x1d/szll/tngk/0ywm/aach/jott). Remaining confirmed correctness queued: Windows validity fingerprint 6act; bounded content analysis memory b2qz; controls-off projection route consistency ay3c/xwmv/qxgh/brun/u767; Python raw-path decoding 8ihl; watch ignored flag refresh 4239. gija wider-sidecar mismatch and snv3 analyzed watch refusal appear addressed at this head, but retain closure until regression evidence and related model integration are reviewed. Final release-candidate verification tyvq remains distinct from these implementation fixes. No merge or release publishing authorized.
