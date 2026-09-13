---
type: is
id: is-01m2eegfhjtav56tk3amg3bp4q
title: "PR #52 review PERF-8: the opened noninferiority gate names no metric and wall includes probe work"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:34:21.105Z
updated_at: 2026-09-13T23:10:24.155Z
closed_at: 2026-09-13T23:10:20.412Z
close_reason: "Fixed in the PR #52 plan commit (see notes): the parity plan's structural verdict and campaign-2's H86 acceptance list state opened-discovery noninferiority on paired component_ns, with opened wall time recorded but not gated, before any final timing sample."
resolution: null
duplicate_of: null
---
PR #52 review PERF-8 (Low). crates/fdu-core/examples/perf_probe.rs:979, 1011 and the parity plan at afbb2ee. The probe's commit summary and paged oracle run inside the timed process; the committed opened-discovery profile attributes 7.07% of samples to them even with the oracle disabled, diluting the paired wall percentage toward zero. The plan's opened +3% noninferiority gate names no metric, and exp-101 quotes wall time. Related to fdu-lj4h's final gates. Fix: state the opened gate on component_ns, as the plan already does for the one-shot jobs.

## Notes

Fixed in 9a536fe on PR #52.
