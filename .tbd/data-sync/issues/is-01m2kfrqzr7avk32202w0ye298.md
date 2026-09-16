---
type: is
id: is-01m2kfrqzr7avk32202w0ye298
title: "PR #63 review PR63-PERF-1: warm revalidate of an over-budget tree re-upserts refused files and clones the table per batch"
kind: task
status: in_progress
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-15T21:32:32.629Z
updated_at: 2026-09-16T00:11:08.898Z
---
PR #63 review PR63-PERF-1 (PLAUSIBLE) at 1fd71a9: scan.rs:4844-4854, index.rs:3706. Add control counters (control_reads, control_refused, control_sources_shared) under fdu_core::counters; make a cheap clearly-correct fix with a test if one exists, else file a stack-followup bead.
