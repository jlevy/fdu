---
type: is
id: is-01m01ea0psdcnb2sdwdj6vh171
title: Close the adaptive-worker evidence gap on Apple Silicon/APFS
kind: epic
status: closed
priority: 1
version: 29
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
updated_at: 2026-08-15T11:06:58.934Z
closed_at: 2026-08-15T11:06:45.660Z
close_reason: Epic complete with no production controller change. All P1 evidence, profiling, model, corpus, decision-contract, provenance, adapter, installed-surface, release-matrix, documentation, and report work is closed; P2 opener and cold-cache work is reparented and deferred. Full handoff gates remain the branch-level final check.
---
Own the evidence-led response to completion-order-sensitive automatic worker scaling on the measured Apple Silicon and local APFS regime. The epic covers bounded policy/backend telemetry, profiling and falsification, deterministic controller modeling, topology stress corpora, pre-registered statistical gates, signal and controller research, conditional implementation, claim-grade dust and installed-command provenance, release qualification, durable documentation, and a final report. It does not claim Intel Mac or non-APFS coverage without new measurements and does not require a production change when the evidence supports retaining the current policy.

Exit criteria: raw artifacts make each policy history and backend observable or explicitly unavailable; one frozen reproduction is profiled before controller work; real-tree phase claims are verified from telemetry rather than assumed from filesystem order; controller screening and held-out confirmation use pre-registered samples, stopping rules, resource thresholds, and 3% paired noninferiority/non-regression decisions; exactness, liveness, partial-result, and explicit-error behavior remain intact; any implementation is the confirmed winner, while “no acceptable winner/no production change” is a valid recorded result; the intended clean installed command and validated dust adapter are proven before release comparisons; the final report states the Apple Silicon/APFS evidence boundary, links the ledger, and reconciles all source-of-truth guidance.

## Notes

Completed 2026-08-15 on Apple M1 Pro/local APFS. Bounded traces, profiles, deterministic controller models, phase-checked corpora, fail-closed statistics, clean provenance, native and wheel installation attestations, the pinned dust adapter, partial-result tests, experiment records exp-056 through exp-059, and the final gap-closure report are complete. Discovery rejected repeated windows (+58.49% wall), staged expansion (+60.73%), and fixed counts above six; no candidate survived, so fdu-8evu closed with no production behavior change and no held-out controller confirmation was warranted. A controlled-interactive replication was invalidated by unrelated host pressure. The exact quiet native release cell was also inconclusive after two pressure invalidations; uncontrolled native and wheel diagnostics were exact and favored fdu but are not promoted to confirmation. fdu-druf and fdu-7ur3 were reparented to fdu-d5e1 and deferred as nonblocking P2 work.
