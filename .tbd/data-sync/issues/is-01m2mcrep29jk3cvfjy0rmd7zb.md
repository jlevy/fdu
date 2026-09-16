---
type: is
id: is-01m2mcrep29jk3cvfjy0rmd7zb
title: "PR #63 delta review PR63D-TEST-1: control counter test asserts absolute equality on process-global counters"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-16T05:59:11.809Z
updated_at: 2026-09-16T06:19:11.150Z
closed_at: 2026-09-16T06:19:11.150Z
close_reason: "c9317c1: the control-counter test snapshots before and after the scan and asserts the three deltas with >= instead of resetting the process totals and asserting equality, the shape a_walk_moves_every_counter_it_should uses. Byte-identical to #65's fix at 7c311d8 so the branches merge without a conflict. Run 9 times against the whole fdu-core library suite: 669 passed, 0 failed, every time."
resolution: null
duplicate_of: null
---
Delta review 5218970886 at 9105768: crates/fdu-core/src/scan.rs:5876-5898. control_counters_attribute_reads_refusals_and_sharing calls counters::reset() then asserts control_reads == 3, control_refused == 1, control_sources_shared == 1. ENABLED is process-wide (counters.rs:727) and any other test thread that reads a .gitignore folds its counts into GLOBAL on exit (counters.rs:639-643), so the equalities can flake. test_serial only serialises tests that take it. Sibling tests scan.rs:5455-5480 and :7797-7815 take before/after deltas and assert with >=; follow that pattern. Run the fixed test at least 8 times alongside the fdu-core suite.
