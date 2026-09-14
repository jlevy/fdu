---
type: is
id: is-01m2et31pte5v3c039ny9bjdjh
title: "PR #52 verification FIX52-2: admission checker stripper has no char-literal state, so an emission impl can hide"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-14T01:56:43.865Z
updated_at: 2026-09-14T01:56:43.865Z
---
Verification of #52 fix b1ae4e0 (PERF-4). scripts/check-admission-sites.mjs rustStructure (:55-140) tracks comments and strings but not char literals: a '"' opens string state until the next double quote and blanks real code. The emission audit (:210-235) enumerates 'impl WalkEmission for' over the stripped text, so an impl in the blanked span is neither audited nor reported -- the new 'every implementation is audited or fails closed' guarantee is fail-open there. REPRODUCED: appending 'fn is_quote(c: char) -> bool { c == '"' }' before an unaudited impl yields zero problems; without it, one. scan.rs:724 already contains the literal and resyncs only by luck. Fix: add a char-literal rule (escape, 3-char literal, else lifetime) and the probe as a test.
