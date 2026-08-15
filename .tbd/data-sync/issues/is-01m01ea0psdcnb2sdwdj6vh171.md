---
type: is
id: is-01m01ea0psdcnb2sdwdj6vh171
title: Close the adaptive-worker evidence gap and restore macOS performance confidence
kind: epic
status: open
priority: 1
version: 21
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - macos
  - adaptive-workers
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
child_order_hints:
  - is-01m01eaac9f07exaqb7erjzf0y
  - is-01m01eahm9z4y9w8a36423xrt1
  - is-01m01eb1b1pkyywa9v6mzsar85
  - is-01m01easxwr1504mqmnha3brzh
  - is-01m01egyp43zd6yj43cjf1ge1d
  - is-01m01ecbhsetn1rmvfn8m26w7e
  - is-01m01ec396v5crqyg5sfasfehr
  - is-01m01eg0efe53jc3smgaza7wk7
  - is-01kzy1w2vbam0mr1z5we4y6fy0
  - is-01m01ebsw9cyhe8thve19grn1w
  - is-01m01cm1sb8xyw9ag3pabb5s3h
  - is-01m01eb8bdvte030yrhmng830e
  - is-01m01ebgg70g940yrq758647t0
  - is-01m01ed61j7yty2bqp0zw8v0xc
  - is-01m01edfz3bd7x2w91bh4qft2m
  - is-01m01edt62s6s8mfeyqgykasxq
created_at: 2026-08-15T00:49:18.040Z
updated_at: 2026-08-15T00:53:34.362Z
---
Own the complete response to the heterogeneous-macOS worker-policy failure exposed by a natural Application Support scan. This epic covers immediate correctness and observability defects, pre-registered research into safer controller signals and hardware/backend regimes, implementation of the measured winner, release-CLI comparison with dust, installed-binary provenance, performance-loop integration, and a final report.

Exit criteria: automatic scheduling is directly observable in raw artifacts; deterministic mixed-phase/topology and partial-result fixtures exist; completion-order bias has regression coverage; current and candidate policies are evaluated paired/interleaved across quiet and interactive macOS regimes; the selected policy passes exactness and resource gates and stays within the pre-registered 3% non-inferiority margins; the actual installed release CLI is verified as the only PATH-resolved fdu and does not show a significant representative-tree loss to dust; errors remain explicit; every experiment, including rejected directions, is recorded; platform-tuning guidance, the performance loop, ledger, and a complete gap-closure report agree.

## Notes

Execution map:

Foundation and immediate evidence defects:
- fdu-z17z: policy/backend telemetry in raw artifacts.
- fdu-w3ra: deterministic mixed-phase/topology and partial fixtures.
- fdu-7y4v: completion-order scheduler model and regression tests.
- fdu-8slr: fail-closed stability, best-fixed, resource, and bimodality gates (blocked by telemetry and corpora).
- fdu-ileg: profile the reproduced failure and dust before controller work (blocked by telemetry and corpora).

Pre-registered research loop:
- fdu-qzfi / H88: bulk versus fallback and topology signals.
- fdu-9pg6 / H87: CPU/P/E bounds and quiet versus interactive host pressure.
- fdu-7ur3 / H89: warm-steady, purge-diagnostic, and dedicated-APFS cold states.
- fdu-druf / H70: existing shared directory-opener experiment, moved here because it consumes the same total concurrency budget.
- fdu-9x4o / H86: synthesize the preceding evidence and compare continuous-window, staged, backlog/gradient, and reversible-parking controllers. It selects but does not ship a design.

Fix and product behavior:
- fdu-8evu: implement the measured winner and prove exact/liveness/platform behavior; blocked by all research tracks.
- fdu-yz68: qualify TCC/permission partial scans without weakening error semantics.
- fdu-o3s4: installed-binary provenance and single PATH resolution release smoke.

Qualification and durable completion:
- fdu-j062: actual installed release CLI versus pinned dust across the representative matrix with a 3% paired non-inferiority gate.
- fdu-zafc: integrate all new requirements into the performance loop, plans, schemas, tuning guide, and commands.
- fdu-0dd5: publish the complete audit, experiment history, selected fix, matrix, residual limits, and future research; it is the epic exit artifact.

Related work remains in its existing ownership: fdu-wfvx handles the broader controlled-host matrix; fdu-oqoy owns general CLI human-output polish; fdu-fnay owns scheduler orientation/granularity; fdu-aky1 owns the broader work-stealing feature. This epic coordinates with them where results overlap but does not duplicate or silently absorb their unrelated scope.
