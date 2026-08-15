---
type: is
id: is-01m01ea0psdcnb2sdwdj6vh171
title: Close the adaptive-worker evidence gap on Apple Silicon/APFS
kind: epic
status: open
priority: 1
version: 26
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
  - is-01m01egyp43zd6yj43cjf1ge1d
  - is-01m01eahm9z4y9w8a36423xrt1
  - is-01m01eb1b1pkyywa9v6mzsar85
  - is-01m01easxwr1504mqmnha3brzh
  - is-01m01fpdn3qbdv0y2458brk4mw
  - is-01m01ecbhsetn1rmvfn8m26w7e
  - is-01m01ec396v5crqyg5sfasfehr
  - is-01m01ebsw9cyhe8thve19grn1w
  - is-01m01cm1sb8xyw9ag3pabb5s3h
  - is-01m01eb8bdvte030yrhmng830e
  - is-01m01ebgg70g940yrq758647t0
  - is-01m01ed61j7yty2bqp0zw8v0xc
  - is-01m01edfz3bd7x2w91bh4qft2m
  - is-01m01edt62s6s8mfeyqgykasxq
  - is-01m01eg0efe53jc3smgaza7wk7
  - is-01kzy1w2vbam0mr1z5we4y6fy0
created_at: 2026-08-15T00:49:18.040Z
updated_at: 2026-08-15T01:18:40.047Z
---
Own the evidence-led response to completion-order-sensitive automatic worker scaling on the measured Apple Silicon and local APFS regime. The epic covers bounded policy/backend telemetry, profiling and falsification, deterministic controller modeling, topology stress corpora, pre-registered statistical gates, signal and controller research, conditional implementation, claim-grade dust and installed-command provenance, release qualification, durable documentation, and a final report. It does not claim Intel Mac or non-APFS coverage without new measurements and does not require a production change when the evidence supports retaining the current policy.

Exit criteria: raw artifacts make each policy history and backend observable or explicitly unavailable; one frozen reproduction is profiled before controller work; real-tree phase claims are verified from telemetry rather than assumed from filesystem order; controller screening and held-out confirmation use pre-registered samples, stopping rules, resource thresholds, and 3% paired noninferiority/non-regression decisions; exactness, liveness, partial-result, and explicit-error behavior remain intact; any implementation is the confirmed winner, while “no acceptable winner/no production change” is a valid recorded result; the intended clean installed command and validated dust adapter are proven before release comparisons; the final report states the Apple Silicon/APFS evidence boundary, links the ledger, and reconciles all source-of-truth guidance.

## Notes

Release-blocking path:
- fdu-z17z -> fdu-ileg establishes bounded policy/backend telemetry and profiles the frozen natural-shape reproduction.
- fdu-w3ra, fdu-7y4v, and fdu-8slr turn the profile into topology stress coverage, a deterministic completion-order model, and fail-closed decision contracts.
- fdu-qzfi and fdu-9pg6 study backend/topology signals and Apple Silicon host/hardware bounds; fdu-9x4o screens candidates and confirms a winner or records that none qualifies.
- fdu-8evu conditionally implements only a confirmed winner; resolving it with no code is valid when no candidate qualifies.
- fdu-b722 and fdu-o3s4 consume fdu-849g to validate dust and the installed fdu command; with fdu-yz68 they feed fdu-j062.
- fdu-j062 -> fdu-zafc -> fdu-0dd5 qualifies the supported release surface, updates the durable loop, and publishes the scoped conclusion.

Side qualifications and related ownership:
- fdu-7ur3 consumes the cold-cache protocol owned by fdu-rjqx after controller selection; it is not a release blocker unless its evidence opens a new explicitly gated defect.
- fdu-druf revisits shared directory openers only after the controller outcome and within one total concurrency budget; it does not block the adaptive-policy fix.
- fdu-9pg6 owns this epic’s Apple Silicon cells, reuses fdu-wfvx’s controlled-host contract, and publishes compatible results back without waiting for or cloning that bead’s Linux work.

Decision rules: for paired percent difference Delta, where positive means the candidate is slower, pass a +3% noninferiority/non-regression margin only when the confidence interval upper bound is at most +3%; establish inferiority when the lower bound is above +3%; otherwise report inconclusive. Never select and confirm a winning arm on the same samples. Missing metrics are null plus a reason, never zero. Resource overrides require pre-registered thresholds or a Pareto rule. Claim-grade evidence requires a clean source state; dirty builds remain exploratory.
