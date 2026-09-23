---
type: is
id: is-01m2gy0csfh8tpe8m3dw279g2g
title: "Opened-root golden coverage: one key covers three refusal reasons, and the continuation_record_limit key was removed"
kind: task
status: in_progress
priority: 3
version: 3
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T21:43:40.069Z
updated_at: 2026-09-23T02:48:15.451Z
---
Guideline conformance review of PR #57, finding 8 (https://github.com/jlevy/fdu/pull/57#pullrequestreview-5203155952). golden_support.rs:532 at 30c3895. The coverage map records a single refusal key for NotADirectory, ContinuationRecordLimit and ContinuationUnavailable, and error.continuation_record_limit was removed with no per-projection replacement. A golden can therefore stop exercising a specific refusal unnoticed. Give each refusal reason its own coverage key, and make the lint require each one.

## Notes

Implemented on codex/alpha-opened-golden-fix: separate required projection refusal keys for NotADirectory, ContinuationRecordLimit and ContinuationUnavailable; real oversized continuation mixed read recorded beside successful lookup; Node corpus lint and omission mutations require all three independently. Normalize only observed_at_ns Some timestamp values, preserving None and equal stable counters. Updated named coherent-projections golden for typed Report. Fresh core golden/normalization5 tests, Node6 tests, and corpus audit5 sessions166records pass. Await independent review and final stack makecheck before closure.
